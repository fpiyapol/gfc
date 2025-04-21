use anyhow::{anyhow, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::models::docker_compose::ContainerState;
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

    pub async fn list_compose_projects(&self) -> Result<Vec<Project>> {
        find_all_compose_projects(Path::new("resources"))
            .await?
            .into_iter()
            .map(|project| self.to_compose_project(&project))
            .collect()
    }

    fn to_compose_project(&self, project: &Path) -> Result<Project> {
        let name = project
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid project name"))?;

        let path = project
            .to_str()
            .ok_or_else(|| anyhow!("Invalid project path"))?;

        let status = self.get_container_status(project)?;

        Ok(Project {
            name: name.to_string(),
            path: path.to_string(),
            status,
        })
    }

    fn get_container_status(&self, project: &Path) -> Result<String> {
        let project_path_str = project
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert path to string: {}", project.display()))?;

        let containers = self.compose_client.list_containers(project_path_str)?;

        let total = containers.len();
        let running = containers
            .iter()
            .filter(|c| c.state == ContainerState::Running)
            .count();

        Ok(match running {
            0 => "Exited".to_string(),
            _ => format!("Running ({}/{})", running, total),
        })
    }
}

pub async fn find_all_compose_projects(path: &Path) -> Result<Vec<PathBuf>> {
    let mut projects = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            projects.push(path);
        }
    }

    Ok(projects)
}

#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use std::path::Path;

    use crate::models::docker_compose::{Container, ContainerState};
    use crate::repositories::docker_compose_client::MockDockerComposeClient;
    use crate::usecases::project::ProjectUsecase;

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

        let mut mock_docker_compose_client = MockDockerComposeClient::new();

        mock_docker_compose_client
            .expect_list_containers()
            .with(eq("/mock/path/to/project"))
            .return_once(|_| Ok(containers));

        let usecase = ProjectUsecase::new(mock_docker_compose_client);

        let actual = usecase
            .get_container_status(Path::new("/mock/path/to/project"))
            .unwrap();

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

        let mut mock_docker_compose_client = MockDockerComposeClient::new();

        mock_docker_compose_client
            .expect_list_containers()
            .with(eq("/mock/path/to/project"))
            .return_once(|_| Ok(containers));

        let usecase = ProjectUsecase::new(mock_docker_compose_client);

        let actual = usecase
            .get_container_status(Path::new("/mock/path/to/project"))
            .unwrap();

        let expected = "Running (1/2)";

        assert_eq!(expected, actual);
    }

    #[test]
    fn given_exited_containers_when_get_container_status_then_return_exited() {
        let containers = vec![Container {
            name: "service1".to_string(),
            state: ContainerState::Exited,
        }];

        let mut mock_docker_compose_client = MockDockerComposeClient::new();

        mock_docker_compose_client
            .expect_list_containers()
            .with(eq("/mock/path/to/project"))
            .return_once(|_| Ok(containers));

        let usecase = ProjectUsecase::new(mock_docker_compose_client);

        let actual = usecase
            .get_container_status(Path::new("/mock/path/to/project"))
            .unwrap();

        let expected = "Exited";

        assert_eq!(expected, actual);
    }
}
