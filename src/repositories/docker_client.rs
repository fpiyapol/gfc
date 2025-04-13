use anyhow::Result;
use async_trait::async_trait;
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, StartContainerOptions,
    StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::Docker;
use futures_util::stream::TryStreamExt;

use crate::models::container_client::{ContainerCreateResponse, ContainerInfo};
use crate::repositories::container_client::ContainerClient;

#[derive(Debug, Clone)]
pub struct DockerClient {
    docker: Docker,
}

impl DockerClient {
    pub fn new() -> Result<DockerClient> {
        println!("Creating Docker client");
        let docker = Docker::connect_with_local_defaults()?;
        Ok(Self { docker })
    }
}

#[async_trait]
impl ContainerClient for DockerClient {
    async fn create_container(&self, name: &str, image: &str) -> Result<ContainerCreateResponse> {
        println!("Creating container: {}", name);
        let options = Some(CreateContainerOptions {
            name,
            platform: None,
        });

        let config = Config {
            image: Some(image),
            ..Default::default()
        };
        let created_container = self.docker.create_container(options, config).await?.into();

        Ok(created_container)
    }

    async fn create_image(&self, image: &str) -> Result<()> {
        println!("Creating image: {}", image);
        let options = Some(CreateImageOptions {
            from_image: image,
            ..Default::default()
        });

        let _ = self
            .docker
            .create_image(options, None, None)
            .try_collect::<Vec<_>>()
            .await?;

        Ok(())
    }

    async fn list_containers(&self) -> Result<Vec<ContainerInfo>> {
        println!("Listing containers");
        let options = Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        });

        let containers = self
            .docker
            .list_containers(options)
            .await?
            .into_iter()
            .map(ContainerInfo::from)
            .collect();

        Ok(containers)
    }

    async fn remove_container(&self, name: &str) -> Result<()> {
        println!("Removing container: {}", name);
        Ok(self.docker.remove_container(name, None).await?)
    }

    async fn start_container(&self, name: &str) -> Result<()> {
        println!("Starting container: {}", name);
        Ok(self
            .docker
            .start_container(name, None::<StartContainerOptions<String>>)
            .await?)
    }

    async fn stop_container(&self, name: &str) -> Result<()> {
        println!("Stopping container: {}", name);
        let timeout = 30;
        let options = Some(StopContainerOptions { t: timeout });

        Ok(self.docker.stop_container(name, options).await?)
    }
}
