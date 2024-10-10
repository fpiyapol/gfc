use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct DockerComposeFile {
    pub services: HashMap<String, Service>,
}

#[derive(Debug, Deserialize)]
pub struct Service {
    pub command: Option<Vec<String>>,
    pub container_name: Option<String>,
    pub environment: Option<Vec<String>>,
    pub image: Option<String>,
    pub ports: Option<Vec<String>>,
}
