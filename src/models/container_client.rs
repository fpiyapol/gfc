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
    pub environment: Option<HashMap<String, String>>,
    pub image: String,
    pub labels: Option<HashMap<String, String>>,
    pub name: String,
    pub network_mode: Option<Option<String>>,
    pub ports: Option<Vec<PortMapping>>,
    pub restart_policy: Option<Option<String>>,
    pub volumes: Option<Vec<VolumeBinding>>,
}

#[derive(Debug, Clone)]
pub struct PortMapping {
    pub host_port: u64,
    pub container_port: u64,
}

#[derive(Debug, Clone)]
pub struct VolumeBinding {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

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
