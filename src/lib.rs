pub mod docker_client;

use crate::docker_client::DockerClient;
use anyhow::Result;

pub async fn init_docker() -> Result<()> {
    let _docker_client = DockerClient::new()?;
    Ok(())
}
