use anyhow::{anyhow, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::models::docker_compose::ComposeProject;
use crate::models::docker_compose::ServiceState;
use crate::repositories::docker_compose_client::DockerComposeClient;

#[derive(Debug, Clone)]
pub struct ComposeUsecase {
    docker_compose_client: DockerComposeClient,
}

impl ComposeUsecase {
    pub fn new(docker_compose_client: DockerComposeClient) -> Self {
        Self {
            docker_compose_client,
        }
    }

    pub async fn list_compose_projects(&self) -> Result<Vec<ComposeProject>> {
        find_all_compose_projects(Path::new("resources"))
            .await?
            .into_iter()
            .map(|project| self.to_compose_project(&project))
            .collect()
    }

    fn to_compose_project(&self, project: &Path) -> Result<ComposeProject> {
        let name = project
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid project name"))?;

        let path = project
            .to_str()
            .ok_or_else(|| anyhow!("Invalid project path"))?;

        let status = self.get_container_status(project)?;

        Ok(ComposeProject {
            name: name.to_string(),
            path: path.to_string(),
            status,
        })
    }

    fn get_container_status(&self, project: &Path) -> Result<String> {
        let containers = self
            .docker_compose_client
            .list_containers(project.to_str().unwrap())?;
        let total = containers.len();
        let running = containers
            .iter()
            .filter(|c| c.state == ServiceState::Running)
            .count();

        Ok(format!("Running ({}/{})", running, total))
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
