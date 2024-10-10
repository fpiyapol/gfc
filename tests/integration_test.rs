use anyhow::Result;

use gfc::models::container_client::CreateContainerConfig;
use gfc::repositories::container_client::ContainerClient;
use gfc::repositories::docker_client::DockerClient;
use gfc::usecases::docker_compose::DockerCompose;

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
async fn docker_compose_up_and_down() -> Result<()> {
    let docker_client = DockerClient::new()?;
    let path = "resources/docker-compose.yaml".to_string();
    let project_name = "int-test".to_string();
    let docker_compose = DockerCompose::new(docker_client.clone(), project_name, path);

    // let docker_compose_svc = D
    let up_result = docker_compose.up().await;
    let containers = docker_client.list_containers().await?;

    let actual_number_of_containers = containers.len();
    let mut actual_container_names = containers
        .iter()
        .flat_map(|con| con.names.clone())
        .map(|name| name.trim_start_matches('/').to_string())
        .collect::<Vec<String>>();

    let expected_number_of_containers = 2;
    let mut expected_container_names = vec![
        "int-test-for-test-a".to_string(),
        "int-test-for-test-b".to_string(),
    ];

    actual_container_names.sort();
    expected_container_names.sort();

    assert!(up_result.is_ok());
    assert_eq!(expected_number_of_containers, actual_number_of_containers);
    assert_eq!(expected_container_names, actual_container_names);

    let down_result = docker_compose.down().await;
    let containers = docker_client.list_containers().await?;

    let actual_number_of_containers = containers.len();

    let expected_number_of_containers = 0;

    assert!(down_result.is_ok());
    assert_eq!(expected_number_of_containers, actual_number_of_containers);

    Ok(())
}
