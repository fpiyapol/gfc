use anyhow::Result;
use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

use crate::config::ResourcesConfig;
use crate::models::docker_compose::{Container, ContainerState};
use crate::models::project::{Project, ProjectFile};
use crate::models::response::{GenericResponse, ResponseStatus};
use crate::repositories::compose_client::ComposeClient;
use crate::repositories::git::GitClient;

#[derive(Debug, Error)]
pub enum ProjectUsecaseError {
    #[error("Failed to create project: {0}")]
    CreateProjectFailed(String),
    #[error("Failed to list projects: {0}")]
    ListProjectsFailed(String),
}

#[derive(Debug, Clone)]
pub struct ProjectUsecase<C, G>
where
    C: ComposeClient + Send + Sync + 'static,
    G: GitClient + Send + Sync + 'static,
{
    pub compose_client: Arc<C>,
    pub git_client: Arc<G>,
    pub resources_config: ResourcesConfig,
}

impl<C, G> ProjectUsecase<C, G>
where
    C: ComposeClient + Send + Sync,
    G: GitClient + Send + Sync,
{
    pub fn new(
        compose_client: Arc<C>,
        git_client: Arc<G>,
        resources_config: ResourcesConfig,
    ) -> Self {
        Self {
            compose_client,
            git_client,
            resources_config,
        }
    }

    pub fn create_project(
        &self,
        project_file: ProjectFile,
    ) -> Result<GenericResponse<ResponseStatus>, ProjectUsecaseError> {
        println!("Creating project: {}", project_file.name);

        let git_client = Arc::clone(&self.git_client);
        let compose_client = Arc::clone(&self.compose_client);

        let (project_path, project_file_path, repository_dir) =
            get_project_and_repository_paths(&self.resources_config, &project_file.name);

        setup_project_workspace(
            &project_file,
            &project_path,
            &project_file_path,
            &repository_dir,
        )
        .map_err(|e| ProjectUsecaseError::CreateProjectFailed(e.to_string()))?;

        let source = project_file.source.clone();
        let repository_dir = repository_dir.clone();

        tokio::task::spawn_blocking(move || {
            let _ = git_client.clone_repository(&source, &repository_dir);
            let _ = compose_client.up(repository_dir.to_str().unwrap());
        });

        Ok(GenericResponse::result(ResponseStatus::Success))
    }

    pub fn list_projects(&self) -> Result<GenericResponse<Project>, ProjectUsecaseError> {
        let root_project_path = Path::new(&self.resources_config.projects_dir);
        let project_files = find_all_project_files(root_project_path)
            .map_err(|e| ProjectUsecaseError::ListProjectsFailed(e.to_string()))?;

        let projects = project_files
            .into_iter()
            .map(|project_file| self.to_project(&project_file))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ProjectUsecaseError::ListProjectsFailed(e.to_string()))?;

        Ok(GenericResponse::results(projects))
    }

    fn to_project(&self, project_file: &ProjectFile) -> Result<Project> {
        let name = project_file.name.clone();
        let source = project_file.source.clone();
        let status = self.container_status_for(&name)?;
        let repository_dir = Path::new(&self.resources_config.repositories_dir).join(&name);
        let last_updated_at = self
            .git_client
            .get_last_commit_timestamp(&repository_dir)?
            .to_string();

        Ok(Project {
            name,
            source,
            status,
            last_updated_at,
        })
    }

    fn container_status_for(&self, project_name: &str) -> Result<String, ProjectUsecaseError> {
        let repository_dir = Path::new(&self.resources_config.repositories_dir).join(project_name);
        let containers = self
            .compose_client
            .list_containers(repository_dir.to_str().unwrap())
            .map_err(|e| ProjectUsecaseError::ListProjectsFailed(e.to_string()))?;

        Ok(build_container_status_string(&containers))
    }
}

/// Prepare the project and repository directories and write the project YAML file.
/// Creates all directories if they do not exist.
fn setup_project_workspace(
    project_file: &ProjectFile,
    project_path: &Path,
    project_file_path: &Path,
    repository_path: &Path,
) -> Result<()> {
    let content = serde_yaml::to_string(project_file)?;
    fs::create_dir_all(project_path)?;
    fs::write(project_file_path, content)?;
    fs::create_dir_all(repository_path)?;
    Ok(())
}

fn get_project_and_repository_paths(
    resources_config: &ResourcesConfig,
    project_name: &str,
) -> (PathBuf, PathBuf, PathBuf) {
    let project_path = Path::new(&resources_config.projects_dir).join(project_name);
    let project_file_path = project_path.join("project.yaml");
    let repository_path = Path::new(&resources_config.repositories_dir).join(project_name);
    (project_path, project_file_path, repository_path)
}

fn find_all_project_files(root_path: &Path) -> Result<Vec<ProjectFile>> {
    let patterns = [
        format!("{}/**/*.yml", root_path.display()),
        format!("{}/**/*.yaml", root_path.display()),
    ];

    let files = patterns
        .iter()
        .flat_map(|pattern| glob(pattern).into_iter().flatten())
        .collect::<Result<Vec<PathBuf>, _>>()?;

    let contents = files
        .iter()
        .map(fs::read_to_string)
        .collect::<Result<Vec<String>, _>>()?;

    let projects = contents
        .iter()
        .map(|content| serde_yaml::from_str(content))
        .collect::<Result<Vec<ProjectFile>, _>>()?;

    Ok(projects)
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
    use crate::models::docker_compose::{Container, ContainerState};
    use crate::usecases::project::build_container_status_string;

    fn make_container(name: &str, state: ContainerState) -> Container {
        Container {
            name: name.to_string(),
            state,
        }
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
}
