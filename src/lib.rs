use anyhow::Result;
use std::fs::File;

use models::docker_compose::DockerCompose;
use repositories::docker_client::DockerClient;

pub mod models;
pub mod repositories;

pub async fn init_docker() -> Result<()> {
    let _docker_client = DockerClient::new()?;
    Ok(())
}

pub fn load_docker_compose(path: &str) -> Result<DockerCompose> {
    let file = File::open(path)?;
    let compose: DockerCompose = serde_yaml::from_reader(file)?;
    Ok(compose)
}
