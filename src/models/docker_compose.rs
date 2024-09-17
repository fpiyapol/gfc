use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct DockerCompose {
    version: Option<String>,
    services: HashMap<String, Service>,
}

#[derive(Debug, Deserialize)]
pub struct Service {
    command: Option<Vec<String>>,
    container_name: Option<String>,
    environment: Option<Vec<String>>,
    image: Option<String>,
    ports: Option<Vec<String>>,
}
