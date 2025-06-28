use glob::glob;
use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, instrument};

use crate::config::WorkspaceConfig;
use crate::errors::project::ProjectUsecaseError;
use crate::errors::GfcResult;
use crate::models::docker_compose::{Container, ContainerState};
use crate::models::project::{
    Project, ProjectFile, ProjectFileLocations, ProjectName, ProjectStatus,
};
use crate::repositories::compose_client::ComposeClient;
use crate::repositories::git::GitClient;

#[derive(Debug, Clone)]
pub struct ProjectUsecase<C, G>
where
    C: ComposeClient + Send + Sync + 'static,
    G: GitClient + Send + Sync + 'static,
{
    pub compose_client: Arc<C>,
    pub git_client: Arc<G>,
    pub workspace_config: WorkspaceConfig,
}

impl<C, G> ProjectUsecase<C, G>
where
    C: ComposeClient + Send + Sync,
    G: GitClient + Send + Sync,
{
    pub fn new(
        compose_client: Arc<C>,
        git_client: Arc<G>,
        workspace_config: WorkspaceConfig,
    ) -> Self {
        Self {
            compose_client,
            git_client,
            workspace_config,
        }
    }

    #[instrument(skip(self), name = "project_usecase::create_project", fields(project.name = %project_file.name, git.url = %project_file.source.url, git.branch = %project_file.source.branch))]
    pub fn create_project(&self, project_file: ProjectFile) -> GfcResult<()> {
        validate_create_project_params(&project_file)?;

        let project_file_locations = self.get_project_file_locations(&project_file)?;

        validate_and_create_required_directories(
            &project_file_locations.manifest_folder,
            &project_file_locations.repository_folder,
        )
        .map_err(|e| ProjectUsecaseError::CreateProjectFailed {
            project_name: project_file_locations
                .manifest_folder
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            reason: e.to_string(),
        })?;

        write_project_definition_file(&project_file, &project_file_locations.manifest_file)
            .map_err(|e| ProjectUsecaseError::CreateProjectFailed {
                project_name: project_file.name.clone(),
                reason: e.to_string(),
            })?;

        let git_client = Arc::clone(&self.git_client);
        let compose_client = Arc::clone(&self.compose_client);

        debug!(
            project.path = %project_file_locations.manifest_folder.display(),
            repository.path = %project_file_locations.repository_folder.display(),
            compose.file = %project_file_locations.compose_file,
            "Starting async project setup"
        );

        tokio::task::spawn_blocking(move || {
            let _ = git_client.clone_repository(
                &project_file.source,
                &project_file_locations.repository_folder,
            );
            let _ = compose_client.up(&project_file_locations.compose_file);
        });

        Ok(())
    }

    #[instrument(skip(self), name = "project_usecase::list_projects")]
    pub fn list_projects(&self) -> GfcResult<Vec<Project>> {
        let project_workspace = Path::new(&self.workspace_config.manifests_root);

        discover_all_project_files_in(project_workspace)
            .and_then(|project_files| self.build_projects_from(project_files))
    }

    fn get_project_file_locations(
        &self,
        project_file: &ProjectFile,
    ) -> GfcResult<ProjectFileLocations> {
        let project_name = &project_file.name;
        let project_dir = Path::new(&self.workspace_config.manifests_root).join(project_name);
        let project_file_path = project_dir.join("project.yaml");
        let repository_dir = Path::new(&self.workspace_config.repositories_root).join(project_name);

        let compose_file_path = repository_dir
            .join(&project_file.source.path)
            .to_str()
            .ok_or_else(|| ProjectUsecaseError::InvalidPath {
                reason: format!(
                    "Invalid compose file path for project '{}': cannot convert path to string",
                    project_name
                ),
            })?
            .to_string();

        Ok(ProjectFileLocations {
            manifest_file: project_file_path,
            manifest_folder: project_dir,
            repository_folder: repository_dir,
            compose_file: compose_file_path,
        })
    }

    fn build_projects_from(&self, project_files: Vec<ProjectFile>) -> GfcResult<Vec<Project>> {
        project_files
            .into_iter()
            .map(|project_file| self.build_project_from(project_file))
            .collect::<Result<Vec<_>, _>>()
            .inspect(|projects| {
                info!(
                    project.count = projects.len(),
                    project.workspace_dir = %self.workspace_config.manifests_root,
                    "Successfully enriched all projects with runtime status"
                );
            })
    }

    fn build_project_from(&self, project_file: ProjectFile) -> GfcResult<Project> {
        let project_name = project_file.name.clone();

        self.determine_current_project_status(&project_file)
            .and_then(|status| {
                self.get_last_repository_update_timestamp(&project_file)
                    .map(|last_updated| (status, last_updated))
            })
            .and_then(|(status, last_updated_at)| {
                ProjectName::new(project_name)
                    .map_err(|e| ProjectUsecaseError::InvalidPath { reason: e }.into())
                    .map(|name| Project {
                        name,
                        source: project_file.source,
                        status,
                        last_updated_at,
                    })
            })
    }

    fn determine_current_project_status(
        &self,
        project_file: &ProjectFile,
    ) -> GfcResult<ProjectStatus> {
        let project_file_locations = self.get_project_file_locations(project_file)?;
        self.container_status_for(&project_file_locations.compose_file)
    }

    fn get_last_repository_update_timestamp(
        &self,
        project_file: &ProjectFile,
    ) -> GfcResult<chrono::DateTime<chrono::Utc>> {
        let repository_folder =
            Path::new(&self.workspace_config.repositories_root).join(&project_file.name);
        self.git_client
            .get_last_commit_timestamp(&repository_folder)
            .map_err(|e| {
                ProjectUsecaseError::ProjectNotFound {
                    project_name: project_file.name.clone(),
                    reason: format!("Git repository information unavailable: {}", e),
                }
                .into()
            })
    }

    fn container_status_for(&self, compose_file_path: &str) -> GfcResult<ProjectStatus> {
        let project_name = Path::new(compose_file_path)
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        let containers = self
            .compose_client
            .list_containers(compose_file_path)
            .map_err(|e| ProjectUsecaseError::ContainerStatusCheckFailed {
                project_name: project_name.to_string(),
                reason: e.to_string(),
            })?;

        Ok(determine_project_status_from(&containers))
    }
}

fn validate_create_project_params(project_file: &ProjectFile) -> GfcResult<()> {
    ProjectName::new(project_file.name.clone()).map_err(|e| {
        ProjectUsecaseError::CreateProjectFailed {
            project_name: project_file.name.clone(),
            reason: e,
        }
    })?;

    if project_file.source.url.trim().is_empty() {
        return Err(ProjectUsecaseError::CreateProjectFailed {
            project_name: project_file.name.clone(),
            reason: "Git URL cannot be empty".to_string(),
        }
        .into());
    }

    if project_file.source.branch.trim().is_empty() {
        return Err(ProjectUsecaseError::CreateProjectFailed {
            project_name: project_file.name.clone(),
            reason: "Git branch cannot be empty".to_string(),
        }
        .into());
    }

    Ok(())
}

fn validate_and_create_required_directories(
    project_dir: &Path,
    repository_dir: &Path,
) -> std::io::Result<()> {
    fs::create_dir_all(project_dir)?;
    fs::create_dir_all(repository_dir)?;
    Ok(())
}

fn write_project_definition_file(project: &ProjectFile, dest: &Path) -> std::io::Result<()> {
    let yaml = serde_yaml::to_string(project).map_err(io::Error::other)?;
    fs::write(dest, yaml)
}

fn discover_all_project_files_in(workspace_root: &Path) -> GfcResult<Vec<ProjectFile>> {
    debug!(
        project.workspace_dir = %workspace_root.display(),
        "Scanning workspace for project files"
    );

    find_all_project_files_in(workspace_root)
        .map_err(|e| {
            ProjectUsecaseError::ListProjectsFailed {
                reason: e.to_string(),
            }
            .into()
        })
        .inspect(|files| {
            debug!(
                project.files_found = files.len(),
                "Found project definition files"
            );
        })
}

fn find_all_project_files_in(workspace_root: &Path) -> std::io::Result<Vec<ProjectFile>> {
    let paths = find_all_file_paths_in(workspace_root)?;

    let contents: Vec<String> = paths
        .iter()
        .map(fs::read_to_string)
        .collect::<Result<Vec<String>, _>>()?;

    contents
        .iter()
        .map(|yaml| serde_yaml::from_str::<ProjectFile>(yaml).map_err(io::Error::other))
        .collect()
}

fn find_all_file_paths_in(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let patterns = ["**/*.yml", "**/*.yaml"];
    patterns
        .iter()
        .map(|pattern| format!("{}/{}", root.display(), pattern))
        .map(|pattern| glob(&pattern).map_err(io::Error::other))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Result<Vec<PathBuf>, _>>()
        .map_err(io::Error::other)
}

fn determine_project_status_from(containers: &[Container]) -> ProjectStatus {
    let total = containers.len();
    let running = containers
        .iter()
        .filter(|c| c.state == ContainerState::Running)
        .count();

    ProjectStatus::from_container_counts(running, total)
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::TempDir;

    use crate::config::WorkspaceConfig;
    use crate::models::docker_compose::{Container, ContainerState};
    use crate::models::git::GitSource;
    use crate::models::project::{ProjectFile, ProjectStatus};
    use crate::repositories::{compose_client::MockComposeClient, git::MockGitClient};
    use crate::usecases::project::*;

    fn make_container(name: &str, state: ContainerState) -> Container {
        Container {
            name: name.to_string(),
            state,
        }
    }

    fn write_yaml(dir: &Path, name: &str, yaml: &str) -> std::io::Result<()> {
        let path = dir.join(name);
        File::create(&path)?;
        fs::write(path, yaml)
    }

    fn create_mock_usecase() -> ProjectUsecase<MockComposeClient, MockGitClient> {
        ProjectUsecase::new(
            Arc::new(MockComposeClient::new()),
            Arc::new(MockGitClient::new()),
            WorkspaceConfig {
                manifests_root: "/workspace/projects".to_string(),
                repositories_root: "/workspace/repos".to_string(),
            },
        )
    }

    #[test]
    fn given_two_running_containers_when_determine_project_status_then_return_running_two_out_of_two(
    ) {
        let containers = vec![
            make_container("service1", ContainerState::Running),
            make_container("service2", ContainerState::Running),
        ];

        let actual = determine_project_status_from(&containers);

        assert_eq!(
            actual,
            ProjectStatus::Running {
                active_containers: 2,
                total_containers: 2
            }
        );
    }

    #[test]
    fn given_one_running_and_one_exited_container_when_determine_project_status_then_return_partially_running(
    ) {
        let containers = vec![
            make_container("service1", ContainerState::Running),
            make_container("service2", ContainerState::Exited),
        ];

        let actual = determine_project_status_from(&containers);

        assert_eq!(
            actual,
            ProjectStatus::PartiallyRunning {
                active_containers: 1,
                total_containers: 2
            }
        );
    }

    #[test]
    fn given_all_exited_containers_when_determine_project_status_then_return_stopped() {
        let containers = vec![
            make_container("service1", ContainerState::Exited),
            make_container("service2", ContainerState::Exited),
        ];

        let actual = determine_project_status_from(&containers);

        assert_eq!(actual, ProjectStatus::Stopped);
    }

    #[test]
    fn given_empty_container_list_when_determine_project_status_then_return_unknown() {
        let containers: Vec<Container> = vec![];
        let actual = determine_project_status_from(&containers);
        assert_eq!(actual, ProjectStatus::Unknown);
    }

    #[test]
    fn given_three_mixed_containers_when_determine_project_status_then_return_partially_running_two_out_of_three(
    ) {
        let containers = vec![
            make_container("service1", ContainerState::Running),
            make_container("service2", ContainerState::Exited),
            make_container("service3", ContainerState::Running),
        ];

        let actual = determine_project_status_from(&containers);

        assert_eq!(
            actual,
            ProjectStatus::PartiallyRunning {
                active_containers: 2,
                total_containers: 3
            }
        );
    }

    #[test]
    fn given_two_project_files_when_discovering_then_return_two_paths() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        write_yaml(tmp.path(), "a.yml", "name: a\nsource:\n  url: https://example.com/a.git\n  branch: main\n  path: docker-compose.yml\n")?;
        write_yaml(tmp.path(), "b.yml", "name: b\nsource:\n  url: https://example.com/b.git\n  branch: main\n  path: docker-compose.yml\n")?;

        let paths = find_all_file_paths_in(tmp.path())?;

        assert_eq!(paths.len(), 2);
        Ok(())
    }

    #[test]
    fn given_valid_project_files_when_find_all_then_return_structs() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        write_yaml(
            tmp.path(),
            "foo.yml",
            "---\nname: foo\nsource:\n  url: https://example.com/foo.git\n  branch: main\n  path: docker-compose.yml\n",
        )?;

        let projects = find_all_project_files_in(tmp.path())?;

        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "foo");
        Ok(())
    }

    #[test]
    fn given_invalid_yaml_when_find_all_then_error() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        write_yaml(tmp.path(), "bad.yml", "this: is: not: yaml")?;

        let result = find_all_project_files_in(tmp.path());

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn given_standard_project_file_when_getting_locations_then_return_correct_paths() {
        let usecase = create_mock_usecase();
        let project_file = ProjectFile {
            name: "test-project".to_string(),
            source: GitSource {
                url: "https://github.com/example/repo.git".to_string(),
                branch: "main".to_string(),
                path: "docker-compose.yml".to_string(),
            },
        };
        let result = usecase.get_project_file_locations(&project_file).unwrap();

        assert_eq!(
            result.manifest_folder.to_str().unwrap(),
            "/workspace/projects/test-project"
        );
        assert_eq!(
            result.manifest_file.to_str().unwrap(),
            "/workspace/projects/test-project/project.yaml"
        );
        assert_eq!(
            result.repository_folder.to_str().unwrap(),
            "/workspace/repos/test-project"
        );
        assert_eq!(
            result.compose_file,
            "/workspace/repos/test-project/docker-compose.yml"
        );
    }

    #[test]
    fn given_project_with_custom_compose_path_when_getting_locations_then_return_correct_custom_paths(
    ) {
        let usecase = create_mock_usecase();
        let project_file = ProjectFile {
            name: "custom-project".to_string(),
            source: GitSource {
                url: "https://github.com/example/repo.git".to_string(),
                branch: "main".to_string(),
                path: "deploy/compose.yaml".to_string(),
            },
        };
        let result = usecase.get_project_file_locations(&project_file).unwrap();

        assert_eq!(
            result.manifest_folder.to_str().unwrap(),
            "/workspace/projects/custom-project"
        );
        assert_eq!(
            result.manifest_file.to_str().unwrap(),
            "/workspace/projects/custom-project/project.yaml"
        );
        assert_eq!(
            result.repository_folder.to_str().unwrap(),
            "/workspace/repos/custom-project"
        );
        assert_eq!(
            result.compose_file,
            "/workspace/repos/custom-project/deploy/compose.yaml"
        );
    }

    #[test]
    fn given_valid_project_params_when_validating_then_return_success() {
        let project_file = ProjectFile {
            name: "valid-project".to_string(),
            source: GitSource {
                url: "https://github.com/example/repo.git".to_string(),
                branch: "main".to_string(),
                path: "docker-compose.yml".to_string(),
            },
        };

        assert!(validate_create_project_params(&project_file).is_ok());
    }

    #[test]
    fn given_empty_git_url_when_validating_project_params_then_return_error() {
        let project_file = ProjectFile {
            name: "test-project".to_string(),
            source: GitSource {
                url: "".to_string(),
                branch: "main".to_string(),
                path: "docker-compose.yml".to_string(),
            },
        };

        assert!(validate_create_project_params(&project_file).is_err());
    }
}
