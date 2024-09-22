use anyhow::Result;

use gfc::models::container_client::CreateContainerConfig;
use gfc::repositories::container_client::ContainerClient;
use gfc::repositories::docker_client::DockerClient;
use gfc::usecases::docker_compose;

#[test]
fn create_docker_client() {
    let docker_client = DockerClient::new();
    assert!(docker_client.is_ok());
}

#[tokio::test]
async fn create_and_remove_container() -> Result<()> {
    let docker_client = DockerClient::new()?;

    let name = "for-test";
    let image = "hello-world:latest";

    let created_image = docker_client.create_image(image).await;
    let config = CreateContainerConfig {
        image: image.to_string(),
        name: name.to_string(),
        ..Default::default()
    };
    let created_container = docker_client.create_container(config).await;
    let removed_container = docker_client.remove_container(name).await;

    assert!(created_image.is_ok());
    assert!(created_container.is_ok());
    assert!(removed_container.is_ok());

    Ok(())
}

#[tokio::test]
async fn docker_compose_up() -> Result<()> {
    let docker_client = DockerClient::new()?;
    let path = "resources/docker-compose.yaml";
    let up_result = docker_compose::up(docker_client, path).await;

    assert!(up_result.is_ok());
    Ok(())
}
