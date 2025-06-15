use glob::glob;
use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::WorkspaceConfig;
use crate::errors::project::ProjectUsecaseError;
use crate::errors::GfcResult;
use crate::models::docker_compose::{Container, ContainerState};
use crate::models::git::GitSource;
use crate::models::project::{Project, ProjectFile};
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

    pub fn create_project(&self, project_file: ProjectFile) -> GfcResult<()> {
        println!("Creating project: {}", project_file.name);

        let git_client = Arc::clone(&self.git_client);
        let compose_client = Arc::clone(&self.compose_client);

        let (project_path, project_file_path, repository_path) =
            get_project_and_repository_workspace_paths(&self.workspace_config, &project_file.name);

        setup_project_workspace(
            &project_file,
            &project_path,
            &project_file_path,
            &repository_path,
        )
        .map_err(|e| ProjectUsecaseError::CreateProjectFailed {
            project_name: project_file.name.clone(),
            reason: e.to_string(),
        })?;

        let git_source = project_file.source.clone();
        let repository_dir = repository_path.clone();
        let project_name = project_file.name.clone();
        let compose_file_path = self.get_compose_file_path(&project_name, &git_source)?;

        tokio::task::spawn_blocking(move || {
            let _ = git_client.clone_repository(&git_source, &repository_dir);
            let _ = compose_client.up(&compose_file_path);
            println!("Project '{}' creation completed successfully", project_name);
        });

        Ok(())
    }

    pub fn list_projects(&self) -> GfcResult<Vec<Project>> {
        let root_project_path = Path::new(&self.workspace_config.projects_dir);
        let project_files = find_all_project_files(root_project_path).map_err(|e| {
            ProjectUsecaseError::ListProjectsFailed {
                reason: format!("Failed to find project files: {}", e),
            }
        })?;

        let projects = project_files
            .into_iter()
            .map(|project_file| self.to_project(&project_file))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ProjectUsecaseError::ListProjectsFailed {
                reason: format!("Failed to process project files: {}", e),
            })?;

        Ok(projects)
    }

    fn get_compose_file_path(
        &self,
        project_name: &str,
        git_source: &GitSource,
    ) -> GfcResult<String> {
        let repository_dir = Path::new(&self.workspace_config.repositories_dir).join(project_name);
        let compose_file_path = repository_dir.join(&git_source.path);

        compose_file_path
            .to_str()
            .ok_or_else(|| {
                let error_msg = format!(
                    "Invalid path to compose file for project '{}': cannot convert path to string",
                    project_name
                );
                println!("Project operation failed: {}", error_msg);
                ProjectUsecaseError::InvalidPath { reason: error_msg }.into()
            })
            .map(String::from)
    }

    fn to_project(&self, project_file: &ProjectFile) -> GfcResult<Project> {
        let name = project_file.name.clone();
        let source = project_file.source.clone();

        // Get compose file path with proper error handling
        let compose_file = self.get_compose_file_path(&name, &source)?;

        // Get container status with proper error handling
        let status = self.container_status_for(&compose_file)?;

        // Get git commit timestamp with proper error handling
        let repository_dir = Path::new(&self.workspace_config.repositories_dir).join(&name);
        let last_updated_at = self
            .git_client
            .get_last_commit_timestamp(&repository_dir)
            .map_err(|e| {
                let error_msg = format!(
                    "Failed to get last commit timestamp for project '{}': {}",
                    name, e
                );
                println!("Project operation failed: {}", error_msg);
                ProjectUsecaseError::ProjectNotFound {
                    project_name: name.clone(),
                    reason: format!("Git repository information unavailable: {}", e),
                }
            })?
            .to_string();

        Ok(Project {
            name,
            source,
            status,
            last_updated_at,
        })
    }

    fn container_status_for(&self, compose_file_path: &str) -> GfcResult<String> {
        // Extract project name from the path for better error messages
        let project_name = Path::new(compose_file_path)
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        let containers = self
            .compose_client
            .list_containers(compose_file_path)
            .map_err(|e| {
                let error_msg = format!(
                    "Failed to list containers for project '{}' using compose file '{}': {}",
                    project_name, compose_file_path, e
                );
                // Print technical message for logging
                println!("Project operation failed: {}", error_msg);

                // Return structured error with fields
                ProjectUsecaseError::ListProjectsFailed {
                    reason: format!("Container listing failed: {}", e),
                }
            })?;

        Ok(build_container_status_string(&containers))
    }
}

/// Ensure all workspace directories exist and project definition file is written.
fn setup_project_workspace(
    project_file: &ProjectFile,
    project_path: &Path,
    project_file_path: &Path,
    repository_path: &Path,
) -> std::io::Result<()> {
    ensure_workspace_dirs(project_path, repository_path)?;
    write_project_definition_file(project_file, project_file_path)
}

/// Create workspace directories if they are missing.
fn ensure_workspace_dirs(project_dir: &Path, repository_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(project_dir)?;
    fs::create_dir_all(repository_dir)?;
    Ok(())
}

/// Serialise the `project.yaml` file beside the project directory.
fn write_project_definition_file(project: &ProjectFile, dest: &Path) -> std::io::Result<()> {
    let yaml = serde_yaml::to_string(project).map_err(io::Error::other)?;
    fs::write(dest, yaml)
}

fn get_project_and_repository_workspace_paths(
    workspace_config: &WorkspaceConfig,
    project_name: &str,
) -> (PathBuf, PathBuf, PathBuf) {
    let project_path = Path::new(&workspace_config.projects_dir).join(project_name);
    let project_file_path = project_path.join("project.yaml");
    let repository_path = Path::new(&workspace_config.repositories_dir).join(project_name);
    (project_path, project_file_path, repository_path)
}

fn find_all_project_files(workspace_root: &Path) -> std::io::Result<Vec<ProjectFile>> {
    let paths = glob_project_file_paths(workspace_root)?;

    let contents: Vec<String> = paths
        .iter()
        .map(fs::read_to_string)
        .collect::<Result<Vec<String>, _>>()?;

    contents
        .iter()
        .map(|yaml| serde_yaml::from_str::<ProjectFile>(yaml).map_err(io::Error::other))
        .collect()
}

/// Return all `*.yml` and `*.yaml` files under the projects workspace.
fn glob_project_file_paths(root: &Path) -> std::io::Result<Vec<PathBuf>> {
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

fn build_container_status_string(containers: &[Container]) -> String {
    let total = containers.len();
    let running = containers
        .iter()
        .filter(|c| c.state == ContainerState::Running)
        .count();

    match running {
        0 => "Exited".to_string(),
        _ => format!("Running ({}/{})", running, total),
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::path::Path;
    use tempfile::TempDir;

    use super::{find_all_project_files, glob_project_file_paths};
    use crate::models::docker_compose::{Container, ContainerState};
    use crate::usecases::project::build_container_status_string;

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

    #[test]
    fn given_two_running_containers_when_build_container_status_string_then_return_running_two_out_of_two(
    ) {
        let containers = vec![
            make_container("service1", ContainerState::Running),
            make_container("service2", ContainerState::Running),
        ];

        let actual = build_container_status_string(&containers);

        assert_eq!(actual, "Running (2/2)");
    }

    #[test]
    fn given_one_running_and_one_exited_container_when_build_container_status_string_then_return_running_one_out_of_two(
    ) {
        let containers = vec![
            make_container("service1", ContainerState::Running),
            make_container("service2", ContainerState::Exited),
        ];

        let actual = build_container_status_string(&containers);

        assert_eq!(actual, "Running (1/2)");
    }

    #[test]
    fn given_all_exited_containers_when_build_container_status_string_then_return_exited() {
        let containers = vec![
            make_container("service1", ContainerState::Exited),
            make_container("service2", ContainerState::Exited),
        ];

        let actual = build_container_status_string(&containers);

        assert_eq!(actual, "Exited");
    }

    #[test]
    fn given_empty_container_list_when_build_container_status_string_then_return_exited() {
        let containers: Vec<Container> = vec![];
        let actual = build_container_status_string(&containers);
        assert_eq!(actual, "Exited");
    }

    #[test]
    fn given_three_mixed_containers_when_build_container_status_string_then_return_running_two_out_of_three(
    ) {
        let containers = vec![
            make_container("service1", ContainerState::Running),
            make_container("service2", ContainerState::Exited),
            make_container("service3", ContainerState::Running),
        ];

        let actual = build_container_status_string(&containers);

        assert_eq!(actual, "Running (2/3)");
    }

    #[test]
    fn given_two_project_files_when_glob_then_return_two_paths() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        write_yaml(tmp.path(), "a.yml", "name: a\nsource:\n  url: https://example.com/a.git\n  branch: main\n  path: docker-compose.yml\n")?;
        write_yaml(tmp.path(), "b.yaml", "name: b\nsource:\n  url: https://example.com/b.git\n  branch: main\n  path: docker-compose.yml\n")?;

        let paths = glob_project_file_paths(tmp.path())?;

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

        let projects = find_all_project_files(tmp.path())?;

        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "foo");
        Ok(())
    }

    #[test]
    fn given_invalid_yaml_when_find_all_then_error() -> anyhow::Result<()> {
        let tmp = TempDir::new()?;
        write_yaml(tmp.path(), "bad.yml", "this: is: not: yaml")?;

        let result = find_all_project_files(tmp.path());

        assert!(result.is_err());
        Ok(())
    }
}
