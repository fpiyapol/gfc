use anyhow::Result;

use gfc::errors::compose::ComposeError;
use gfc::models::docker_compose::ContainerState;
use gfc::repositories::compose_client::ComposeClient;
use gfc::repositories::container_client::ContainerClient;
use gfc::repositories::docker_client::DockerClient;
use gfc::repositories::docker_compose_client::DockerComposeClient;

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

#[tokio::test]
async fn docker_compose_up_and_down() -> Result<()> {
    let docker_compose_client = DockerComposeClient::new()?;
    let project = "resources/for-integration-test/for-test-a/docker-compose.yml";

    let up_result = docker_compose_client.up(&project);
    let status = docker_compose_client.list_containers(&project);

    println!("up_result: {:#?}", up_result);
    println!("status: {:#?}", status);

    assert!(up_result.is_ok());
    assert!(status.is_ok());
    assert!(status
        .unwrap()
        .iter()
        .all(|s| s.state == ContainerState::Running));

    let down_result = docker_compose_client.down(&project);
    assert!(down_result.is_ok());

    Ok(())
}

#[tokio::test]
async fn docker_compose_execute_error() -> Result<()> {
    let docker_compose_client = DockerComposeClient::new()?;
    let project = "resources/non-exist-project";

    let up_result = docker_compose_client.up(&project);
    let status = docker_compose_client.list_containers(&project);

    assert!(match up_result {
        Err(ComposeError::ComposeFileNotFound { .. }) => true,
        _ => false,
    });

    assert!(match status {
        Err(ComposeError::ComposeFileNotFound { .. }) => true,
        _ => false,
    });

    Ok(())
}
