use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::docker_compose::{Container, ContainerState};
use crate::models::project::Project;
use crate::repositories::compose_client::ComposeClient;

#[derive(Debug, Clone)]
pub struct ProjectUsecase<C>
where
    C: ComposeClient,
{
    pub compose_client: C,
}

impl<C: ComposeClient> ProjectUsecase<C> {
    pub fn new(compose_client: C) -> Self {
        Self { compose_client }
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let root_path = Path::new("resources");
        let project_paths = find_all_project_paths(root_path)?;

        project_paths
            .into_iter()
            .map(|path| self.to_project(&path))
            .collect()
    }

    fn to_project(&self, project_path: &Path) -> Result<Project> {
        let name = extract_project_name_from(project_path)?;
        let path = extract_project_path_from(project_path)?;
        let status = self.container_status_for(project_path)?;

        Ok(Project { name, path, status })
    }

    fn container_status_for(&self, project_path: &Path) -> Result<String> {
        let path_str = extract_project_path_from(project_path)?;
        let containers = self.compose_client.list_containers(&path_str)?;

        Ok(build_container_status_string(&containers))
    }
}

fn find_all_project_paths(root_path: &Path) -> Result<Vec<PathBuf>> {
    let mut projects_paths = Vec::new();

    for entry in fs::read_dir(root_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            projects_paths.push(path);
        }
    }

    Ok(projects_paths)
}

fn extract_project_name_from(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(str::to_string)
        .ok_or_else(|| anyhow!("Invalid project name"))
}

fn extract_project_path_from(path: &Path) -> Result<String> {
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| anyhow!("Invalid project path"))
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
    fn given_two_running_containers_when_get_container_status_then_return_running_two_out_of_two() {
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
    fn given_one_running_and_one_exited_container_when_get_container_status_then_return_running_one_out_of_two(
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
    fn given_exited_containers_when_get_container_status_then_return_exited() {
        let containers = vec![Container {
            name: "service1".to_string(),
            state: ContainerState::Exited,
        }];

        let actual = build_container_status_string(&containers);

        let expected = "Exited";

        assert_eq!(expected, actual);
    }
}
