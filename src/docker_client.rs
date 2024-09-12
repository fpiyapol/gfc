use anyhow::Result;
use bollard::container::{Config, CreateContainerOptions};
use bollard::image::CreateImageOptions;
use bollard::models::ContainerCreateResponse;
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
        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };

        let _ = self
            .docker
            .create_image(Some(options), None, None)
            .try_collect::<Vec<_>>()
            .await?;

        Ok(())
    }
}
