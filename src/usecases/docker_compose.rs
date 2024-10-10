use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use thiserror::Error;

use crate::models::container_client::{CreateContainerConfig, PortMapping};
use crate::models::docker_compose::{DockerComposeFile, Service};
use crate::repositories::container_client::ContainerClient;
use crate::repositories::docker_client::DockerClient;

#[derive(Debug, Error, PartialEq)]
pub enum DockerComposeError {
    #[error("Invalid port format: {0}")]
    InvalidPort(String),
}

pub struct DockerCompose {
    client: DockerClient,
    path: String,
    project_name: String,
}

impl DockerCompose {
    pub fn new(client: DockerClient, project_name: String, path: String) -> Self {
        Self {
            client,
            project_name,
            path,
        }
    }

    pub async fn up(&self) -> Result<()> {
        let docker_compose = load_docker_compose(&self.path)?;

        for (service_name, service) in docker_compose.services {
            let service_name = format!("{}-{}", &self.project_name, service_name);
            let config = create_container_config_from(
                &self.path,
                &self.project_name,
                &service_name,
                &service,
            )?;

            self.client.create_container(config).await?;
            self.client.start_container(&service_name).await?
        }

        Ok(())
    }

    pub async fn down(&self) -> Result<()> {
        let docker_compose = load_docker_compose(&self.path)?;

        for (service_name, _) in docker_compose.services {
            let service_name = format!("{}-{}", &self.project_name, service_name);

            self.client.stop_container(&service_name).await?;
            self.client.remove_container(&service_name).await?;
        }

        Ok(())
    }
}

fn load_docker_compose(path: &str) -> Result<DockerComposeFile> {
    let file = File::open(path)?;
    let compose: DockerComposeFile = serde_yaml::from_reader(file)?;
    Ok(compose)
}

fn create_container_config_from(
    path: &str,
    project_name: &str,
    service_name: &str,
    service: &Service,
) -> Result<CreateContainerConfig> {
    let labels = prepare_labels(path, project_name, service_name);
    let ports = prepare_ports(service)?;

    let config = CreateContainerConfig {
        command: service.command.clone(),
        environment: service.environment.clone(),
        image: service.image.clone().unwrap_or_default(),
        labels: Some(labels),
        name: service_name.to_string(),
        ports,
    };

    Ok(config)
}

fn prepare_labels(path: &str, project_name: &str, service_name: &str) -> HashMap<String, String> {
    HashMap::from([
        (
            "com.docker.compose.project".to_string(),
            project_name.to_string(),
        ),
        (
            "com.docker.compose.service".to_string(),
            service_name.to_string(),
        ),
        (
            "com.docker.compose.project.config_files".to_string(),
            path.to_string(),
        ),
        (
            "com.docker.compose.project.working_dir".to_string(),
            get_working_dir(path).to_string(),
        ),
    ])
}

fn prepare_ports(service: &Service) -> Result<Option<Vec<PortMapping>>, DockerComposeError> {
    service
        .ports
        .as_ref()
        .map(|ports| extract_port_mappings(ports))
        .transpose()
}

fn extract_port_mappings(ports: &[String]) -> Result<Vec<PortMapping>, DockerComposeError> {
    // TODO: Support port mapping with the host IP
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

fn get_working_dir(path: &str) -> &str {
    Path::new(path).parent().unwrap().to_str().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_port_mappings_given_valid_input_then_return_correct_mappings() {
        let input = vec!["8080:80/tcp".to_string(), "443:443/udp".to_string()];
        let result = extract_port_mappings(&input).unwrap();

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
    fn extract_port_mappings_given_valid_ports_without_protocol_then_use_default_tcp() {
        let input = vec!["8080:80".to_string(), "3000:3000".to_string()];
        let result = extract_port_mappings(&input).unwrap();

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
    fn extract_port_mappings_given_empty_input_then_return_empty_list() {
        let input: Vec<String> = vec![];
        let result = extract_port_mappings(&input).unwrap();
        let expected: Vec<PortMapping> = vec![];
        assert_eq!(expected, result);
    }

    #[test]
    fn extract_port_mappings_given_single_port_then_use_as_both_host_and_container() {
        let input = vec!["80/tcp".to_string()];
        let result = extract_port_mappings(&input).unwrap();

        let expected = vec![PortMapping {
            host_port: "80".to_string(),
            container_port: "80".to_string(),
            protocol: "tcp".to_string(),
        }];

        assert_eq!(expected, result);
    }
}
