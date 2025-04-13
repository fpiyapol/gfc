#[derive(Debug)]
pub struct ContainerCreateResponse {
    pub id: String,
}

#[derive(Debug)]
pub struct ContainerInfo {
    pub id: String,
    pub names: Vec<String>,
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
