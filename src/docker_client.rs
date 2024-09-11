use anyhow::Result;
use bollard::Docker;

pub struct DockerClient {
    docker: Docker,
}

impl DockerClient {
    pub fn new() -> Result<DockerClient> {
        let docker = Docker::connect_with_local_defaults()?;
        Ok(Self { docker })
    }
}
