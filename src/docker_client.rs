use anyhow::Result;
use bollard::container::{Config, CreateContainerOptions, ListContainersOptions};
use bollard::image::CreateImageOptions;
use bollard::models::{ContainerCreateResponse, ContainerSummary};
use bollard::Docker;
use futures_util::stream::TryStreamExt;

pub struct DockerClient {
    docker: Docker,
}

impl DockerClient {
    pub fn new() -> Result<DockerClient> {
        println!("Creating Docker client");
        let docker = Docker::connect_with_local_defaults()?;
        Ok(Self { docker })
    }

    pub async fn create_container(
        &self,
        name: &str,
        image: &str,
    ) -> Result<ContainerCreateResponse> {
        println!("Creating container: {}", name);
        let options = Some(CreateContainerOptions {
            name,
            platform: None,
        });

        let config = Config {
            image: Some(image),
            ..Default::default()
        };

        let created_container = self.docker.create_container(options, config).await?;

        Ok(created_container)
    }

    pub async fn create_image(&self, image: &str) -> Result<()> {
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

    pub async fn remove_container(&self, name: &str) -> Result<()> {
        println!("Removing container: {}", name);
        Ok(self.docker.remove_container(name, None).await?)
    }

    pub async fn list_containers(&self) -> Result<Vec<ContainerSummary>> {
        println!("Listing containers");
        let options = Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        });

        let containers = self.docker.list_containers(options).await?;

        Ok(containers)
    }
}
