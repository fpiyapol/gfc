use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use thiserror::Error;

use crate::models::container_client::{CreateContainerConfig, PortMapping};
use crate::models::docker_compose::DockerCompose;
use crate::repositories::container_client::ContainerClient;

#[derive(Debug, Error, PartialEq)]
pub enum DockerComposeError {
    #[error("Invalid port format: {0}")]
    InvalidPort(String),
}

pub async fn up<C>(client: C, project_name: &str, path: &str) -> Result<()>
where
    C: ContainerClient,
{
    let docker_compose = load_docker_compose(path)?;
    for (name, service) in docker_compose.services {
        let service_name = format!("{}-{}", project_name, name);
        let labels = generate_labels(project_name, &name);
        let ports = service.ports.map(get_ports).transpose().unwrap();

        let config = CreateContainerConfig {
            command: service.command,
            environment: service.environment,
            image: service.image.unwrap_or_default(),
            labels: Some(labels),
            name: service_name,
            ports,
        };

        client.create_container(config).await?;
    }

    Ok(())
}

pub async fn down<C>(client: C, path: &str) -> Result<()>
where
    C: ContainerClient,
{
    todo!()
}

fn load_docker_compose(path: &str) -> Result<DockerCompose> {
    let file = File::open(path)?;
    let compose: DockerCompose = serde_yaml::from_reader(file)?;
    Ok(compose)
}

fn generate_labels(project_name: &str, service_name: &str) -> HashMap<String, String> {
    HashMap::from([
        (
            "com.docker.compose.project".to_string(),
            project_name.to_string(),
        ),
        (
            "com.docker.compose.service".to_string(),
            service_name.to_string(),
        ),
    ])
}

fn get_ports(ports: Vec<String>) -> Result<Vec<PortMapping>, DockerComposeError> {
    ports
        .iter()
        .map(|port| {
            let parts = port.split("/").collect::<Vec<&str>>();
            let protocol = if parts.len() > 1 { parts[1] } else { "tcp" };
            let port_parts = parts[0].split(":").collect::<Vec<&str>>();

            let (host_port, container_port) = match port_parts.as_slice() {
                [port] => (port, port),
                [host_port, container_port] => (host_port, container_port),
                _ => return Err(DockerComposeError::InvalidPort(port.clone())),
            };

            Ok(PortMapping {
                container_port: container_port.to_string(),
                host_port: host_port.to_string(),
                protocol: protocol.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_ports_given_valid_input_then_return_correct_mappings() {
        let input = vec!["8080:80/tcp".to_string(), "443:443/udp".to_string()];
        let result = get_ports(input).unwrap();

        let expected = vec![
            PortMapping {
                host_port: "8080".to_string(),
                container_port: "80".to_string(),
                protocol: "tcp".to_string(),
            },
            PortMapping {
                host_port: "443".to_string(),
                container_port: "443".to_string(),
                protocol: "udp".to_string(),
            },
        ];

        assert_eq!(expected, result);
    }

    #[test]
    fn get_ports_given_valid_ports_without_protocol_then_use_default_tcp() {
        let input = vec!["8080:80".to_string(), "3000:3000".to_string()];
        let result = get_ports(input).unwrap();

        let expected = vec![
            PortMapping {
                host_port: "8080".to_string(),
                container_port: "80".to_string(),
                protocol: "tcp".to_string(),
            },
            PortMapping {
                host_port: "3000".to_string(),
                container_port: "3000".to_string(),
                protocol: "tcp".to_string(),
            },
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn get_ports_given_empty_input_then_return_empty_list() {
        let input: Vec<String> = vec![];
        let result = get_ports(input).unwrap();
        let expected: Vec<PortMapping> = vec![];
        assert_eq!(expected, result);
    }

    #[test]
    fn get_ports_given_single_port_then_use_as_both_host_and_container() {
        let input = vec!["80/tcp".to_string()];
        let result = get_ports(input).unwrap();

        let expected = vec![PortMapping {
            host_port: "80".to_string(),
            container_port: "80".to_string(),
            protocol: "tcp".to_string(),
        }];

        assert_eq!(expected, result);
    }
}
