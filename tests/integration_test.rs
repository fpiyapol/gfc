use anyhow::Result;

use gfc::load_docker_compose;
use gfc::repositories::container_client::ContainerClient;
use gfc::repositories::docker_client::DockerClient;

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
    let created_container = docker_client.create_container(name, image).await;
    let removed_container = docker_client.remove_container(name).await;

    assert!(created_image.is_ok());
    assert!(created_container.is_ok());
    assert!(removed_container.is_ok());

    Ok(())
}

#[test]
fn load_docker_compose_file() -> Result<()> {
    let path = "resources/docker-compose.yaml";
    let docker_compose = load_docker_compose(path);
    assert!(docker_compose.is_ok());
    Ok(())
}
