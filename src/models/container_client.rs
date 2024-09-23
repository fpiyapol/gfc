use bollard::models::{PortBinding, PortMap};
use std::collections::HashMap;

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
    pub volumes: Option<Vec<VolumeMapping>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortMapping {
    pub container_port: String,
    pub host_port: String,
    pub protocol: String,
}

#[derive(Debug, Clone)]
pub struct VolumeMapping {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

type ExposedPort = HashMap<String, HashMap<(), ()>>;

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

impl From<CreateContainerConfig> for bollard::container::Config<String> {
    fn from(value: CreateContainerConfig) -> Self {
        let (exposed_ports, port_bindings) = match extract_ports(&value) {
            Some((exposed, bindings)) => (Some(exposed), Some(bindings)),
            None => (None, None),
        };

        let host_config = bollard::models::HostConfig {
            port_bindings,
            // binds: Some(binds),
            ..Default::default()
        };

        Self {
            env: value.environment,
            cmd: value.command,
            image: Some(value.image),
            labels: value.labels,
            exposed_ports,
            host_config: Some(host_config),
            ..Default::default()
        }
    }
}

fn extract_ports(config: &CreateContainerConfig) -> Option<(ExposedPort, PortMap)> {
    config.ports.as_ref().map(|port_mappings| {
        port_mappings
            .iter()
            .map(|port_mapping| {
                let port_key = format!("{}/{}", port_mapping.container_port, port_mapping.protocol);
                let exposed_port = (port_key.clone(), HashMap::<(), ()>::new());
                let port_binding = (
                    port_key,
                    Some(vec![PortBinding {
                        host_port: Some(port_mapping.host_port.clone()),
                        ..Default::default()
                    }]),
                );

                (exposed_port, port_binding)
            })
            .unzip()
    })
}
