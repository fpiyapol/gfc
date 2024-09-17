use anyhow::Result;
use std::fs::File;

use models::docker_compose::DockerCompose;
use repositories::container_client::ContainerClient;
use repositories::docker_client::DockerClient;

pub mod models;
pub mod repositories;

pub async fn init_docker() -> Result<()> {
    let docker_client = DockerClient::new()?;

    // let name = "for-test";
    // let image = "hello-world:latest";
    // docker_client.create_image(image).await?;
    // let _created_container = docker_client.create_container(name, image).await?;
    // docker_client.start_container(name).await?;
    // docker_client.stop_container(name).await?;
    // docker_client.remove_container(name).await?;
    // let containers = docker_client.list_containers().await?;
    // println!("{:?}", containers);

    let path = "resources/docker-compose.yaml";
    let compose = load_docker_compose(path)?;

    println!("{:?}", compose);

    Ok(())
}

fn load_docker_compose(path: &str) -> Result<DockerCompose> {
    let file = File::open(path)?;
    let compose: DockerCompose = serde_yaml::from_reader(file)?;
    Ok(compose)
}
