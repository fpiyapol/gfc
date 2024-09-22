use crate::models::container_client::{CreateContainerConfig, PortMapping};
use crate::models::docker_compose::DockerCompose;
use crate::repositories::container_client::ContainerClient;
use anyhow::Result;
use std::fs::File;

pub fn load_docker_compose(path: &str) -> Result<DockerCompose> {
    let file = File::open(path)?;
    let compose: DockerCompose = serde_yaml::from_reader(file)?;
    Ok(compose)
}

pub async fn up<C>(client: C, path: &str) -> Result<()>
where
    C: ContainerClient,
{
    let docker_compose = load_docker_compose(path)?;
    for (name, service) in docker_compose.services {
        let ports = service.ports.and_then(get_ports);
        let config = CreateContainerConfig {
            command: service.command,
            environment: service.environment,
            image: service.image.unwrap_or_default(),
            labels: None,
            name,
            ports,
            volumes: None,
        };

        client.create_container(config).await?;
    }

    Ok(())
}

fn get_ports(ports: Vec<String>) -> Option<Vec<PortMapping>> {
    Some(
        ports
            .iter()
            .map(|port| {
                let parts = port.split("/").collect::<Vec<&str>>();
                let protocol = if parts.len() > 1 { parts[1] } else { "tcp" };
                let port_parts = parts[0].split(":").collect::<Vec<&str>>();
                let (host_port, container_port) = (port_parts[0], port_parts[1]);

                PortMapping {
                    container_port: container_port.to_string(),
                    host_port: host_port.to_string(),
                    protocol: protocol.to_string(),
                }
            })
            .collect(),
    )
}

pub fn down() -> Result<()> {
    todo!()
}
