use anyhow::Result;
use bollard::models::{PortBinding, PortMap};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug)]
pub struct ContainerCreateResponse {
    pub id: String,
}

#[derive(Debug)]
pub struct ContainerInfo {
    pub id: String,
    pub names: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct CreateContainerConfig {
    pub command: Option<Vec<String>>,
    pub environment: Option<Vec<String>>,
    pub image: String,
    pub labels: Option<HashMap<String, String>>,
    pub name: String,
    pub ports: Option<Vec<PortMapping>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortMapping {
    pub container_port: String,
    pub host_port: String,
    pub protocol: String,
}

#[derive(Debug, Error)]
pub enum ContainerClientModelError {}

type ExposedPorts = HashMap<String, HashMap<(), ()>>;

impl From<bollard::models::ContainerCreateResponse> for ContainerCreateResponse {
    fn from(value: bollard::models::ContainerCreateResponse) -> Self {
        ContainerCreateResponse { id: value.id }
    }
}

impl From<bollard::models::ContainerSummary> for ContainerInfo {
    fn from(value: bollard::models::ContainerSummary) -> Self {
        ContainerInfo {
            id: value.id.unwrap_or("".to_string()),
            names: value.names.unwrap_or_default(),
        }
    }
}

impl TryFrom<CreateContainerConfig> for bollard::container::Config<String> {
    type Error = ContainerClientModelError;

    fn try_from(value: CreateContainerConfig) -> Result<Self, Self::Error> {
        let default_port_mappings = Vec::new();
        let port_mappings = value.ports.as_ref().unwrap_or(&default_port_mappings);
        let exposed_ports = get_exposed_ports(port_mappings)?;
        let port_bindings = get_port_bindings(port_mappings)?;

        let host_config = bollard::models::HostConfig {
            port_bindings: Some(port_bindings),
            ..Default::default()
        };

        Ok(Self {
            env: value.environment,
            cmd: value.command,
            image: Some(value.image),
            labels: value.labels,
            exposed_ports: Some(exposed_ports),
            host_config: Some(host_config),
            ..Default::default()
        })
    }
}

fn get_exposed_ports(
    port_mappings: &[PortMapping],
) -> Result<ExposedPorts, ContainerClientModelError> {
    let mut exposed_ports = HashMap::new();

    for port_mapping in port_mappings {
        let port_key = format!("{}/{}", port_mapping.container_port, port_mapping.protocol);
        exposed_ports.insert(port_key, HashMap::new());
    }

    Ok(exposed_ports)
}

fn get_port_bindings(port_mappings: &[PortMapping]) -> Result<PortMap, ContainerClientModelError> {
    let mut port_bindings = HashMap::new();

    for port_mapping in port_mappings {
        let port_key = format!("{}/{}", port_mapping.container_port, port_mapping.protocol);
        let port_binding = Some(vec![PortBinding {
            host_port: Some(port_mapping.host_port.clone()),
            ..Default::default()
        }]);

        port_bindings.insert(port_key, port_binding);
    }

    Ok(port_bindings)
}
