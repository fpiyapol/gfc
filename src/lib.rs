use anyhow::Result;

use repositories::container_client::ContainerClient;
use repositories::docker_client::DockerClient;

pub mod models;
pub mod repositories;

pub async fn init_docker() -> Result<()> {
    let docker_client = DockerClient::new()?;

    let name = "for-test";
    let image = "hello-world:latest";
    docker_client.create_image(image).await?;
    let _created_container = docker_client.create_container(name, image).await?;
    docker_client.start_container(name).await?;
    docker_client.stop_container(name).await?;
    docker_client.remove_container(name).await?;
    let containers = docker_client.list_containers().await?;
    println!("{:?}", containers);

    Ok(())
}
