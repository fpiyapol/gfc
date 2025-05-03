use anyhow::Result;
use glob::glob;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

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
    #[error("Invalid project path")]
    InvalidProjectPath,
    #[error("Invalid project name")]
    InvalidProjectName,

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),
    // #[error(transparent)]
    // Glob(#[from] glob::GlobError),
}

#[derive(Debug, Clone)]
pub struct ProjectUsecase<C, G>
where
    C: ComposeClient + Send + Sync + 'static,
    G: GitClient + Send + Sync + 'static,
{
    pub compose_client: Arc<C>,
    pub git_client: Arc<G>,
}

impl<C, G> ProjectUsecase<C, G>
where
    C: ComposeClient + Send + Sync,
    G: GitClient + Send + Sync,
{
    pub fn new(compose_client: Arc<C>, git_client: Arc<G>) -> Self {
        Self {
            compose_client,
            git_client,
        }
    }

    pub fn create_project(
        &self,
        project_file: ProjectFile,
    ) -> Result<GenericResponse<ResponseStatus>, ProjectUsecaseError> {
        println!("Creating project: {}", project_file.name);

        let git_client = Arc::clone(&self.git_client);
        let compose_client = Arc::clone(&self.compose_client);

        let (project_path, project_file_path, working_dir) = build_paths(&project_file.name);

        prepare_project_files(
            &project_file,
            &project_path,
            &project_file_path,
            &working_dir,
        )?;

        let source = project_file.source.clone();
        let working_dir = working_dir.clone();

        tokio::task::spawn_blocking(move || {
            let _ = git_client.clone_repository(&source, &working_dir);
            let _ = compose_client.up(working_dir.to_str().unwrap());
        });

        Ok(GenericResponse::result(ResponseStatus::Success))
    }

    pub fn list_projects(&self) -> Result<GenericResponse<Project>, ProjectUsecaseError> {
        let root_project_path = Path::new("resources/projects");
        let project_files = find_all_project_files(root_project_path)?;

        println!("Project files: {:#?}", project_files);

        let projects = project_files
            .into_iter()
            .map(|path| self.to_project(&path))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(GenericResponse::results(projects))
    }

    fn to_project(&self, project_file: &ProjectFile) -> Result<Project, ProjectUsecaseError> {
        let name = project_file.name.clone();
        let source = project_file.source.clone();
        let status = self.container_status_for(&name)?;
        let last_updated_at = "".to_string();

        Ok(Project {
            name,
            source,
            status,
            last_updated_at,
        })
    }

    fn container_status_for(&self, project_name: &str) -> Result<String, ProjectUsecaseError> {
        let working_dir = Path::new("resources/repositories").join(project_name);
        let containers = self
            .compose_client
            .list_containers(working_dir.to_str().unwrap())
            .map_err(|e| ProjectUsecaseError::ListProjectsFailed(e.to_string()))?;

        Ok(build_container_status_string(&containers))
    }
}

fn prepare_project_files(
    project_file: &ProjectFile,
    project_path: &Path,
    project_file_path: &Path,
    working_dir: &Path,
) -> Result<(), ProjectUsecaseError> {
    let content = serde_yaml::to_string(project_file)?;
    fs::create_dir(project_path)?;
    fs::write(project_file_path, content)?;
    fs::create_dir(working_dir)?;
    Ok(())
}

fn build_paths(project_name: &str) -> (PathBuf, PathBuf, PathBuf) {
    let project_path = Path::new("resources/projects").join(project_name);
    let project_file_path = project_path.join("project.yaml");
    let working_dir = Path::new("resources/repositories").join(project_name);
    (project_path, project_file_path, working_dir)
}

fn find_all_project_files(root_path: &Path) -> Result<Vec<ProjectFile>, ProjectUsecaseError> {
    let patterns = [
        format!("{}/**/*.yml", root_path.display()),
        format!("{}/**/*.yaml", root_path.display()),
    ];

    let files = patterns
        .iter()
        .flat_map(|pattern| glob(pattern).into_iter().flatten())
        .collect::<Result<Vec<PathBuf>, _>>()
        .map_err(|e| ProjectUsecaseError::ListProjectsFailed(e.to_string()))?;

    let contents = files
        .iter()
        .map(fs::read_to_string)
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| ProjectUsecaseError::ListProjectsFailed(e.to_string()))?;

    let projects = contents
        .iter()
        .map(|content| serde_yaml::from_str(content))
        .collect::<Result<Vec<ProjectFile>, _>>()
        .map_err(|e| ProjectUsecaseError::ListProjectsFailed(e.to_string()))?;

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

    #[test]
    fn given_two_running_containers_when_build_container_status_string_then_return_running_two_out_of_two(
    ) {
        let containers = vec![
            Container {
                name: "service1".to_string(),
                state: ContainerState::Running,
            },
            Container {
                name: "service2".to_string(),
                state: ContainerState::Running,
            },
        ];

        let actual = build_container_status_string(&containers);

        let expected = "Running (2/2)";

        assert_eq!(expected, actual);
    }

    #[test]
    fn given_one_running_and_one_exited_container_when_build_container_status_string_then_return_running_one_out_of_two(
    ) {
        let containers = vec![
            Container {
                name: "service1".to_string(),
                state: ContainerState::Running,
            },
            Container {
                name: "service2".to_string(),
                state: ContainerState::Exited,
            },
        ];

        let actual = build_container_status_string(&containers);

        let expected = "Running (1/2)";

        assert_eq!(expected, actual);
    }

    #[test]
    fn given_exited_containers_when_build_container_status_string_then_return_exited() {
        let containers = vec![Container {
            name: "service1".to_string(),
            state: ContainerState::Exited,
        }];

        let actual = build_container_status_string(&containers);

        let expected = "Exited";

        assert_eq!(expected, actual);
    }
}
