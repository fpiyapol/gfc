use anyhow::Result;
use mockall::automock;
use mockall::predicate::*;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

use crate::models::docker_compose::{Container, ContainerState};
use crate::repositories::compose_client::ComposeClient;

const SUPPORTED_COMPOSE_FILES: &[&str] = &[
    "docker-compose.yml",
    "docker-compose.yaml",
    "compose.yml",
    "compose.yaml",
];

#[derive(Debug, Error)]
pub enum DockerComposeError {
    #[error("Directory not found")]
    DirectoryNotFound,
    #[error("Docker compose file does not exist")]
    DockerComposeFileDoesNotExist,
    #[error("Failed to execute docker compose: {0}")]
    DockerComposeExecutionFailed(#[from] std::io::Error),
    #[error("Failed to parse docker compose output")]
    DockerComposeOutputParseFailed(#[from] serde_json::Error),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("Unknown state: {0}")]
    UnknownState(String),
}

#[derive(Debug, Clone)]
pub struct DockerComposeClient;

impl DockerComposeClient {
    pub fn new() -> Result<DockerComposeClient> {
        Ok(Self {})
    }

    fn run_cmd(args: &[&str], path: &str) -> Result<String, DockerComposeError> {
        let output = Command::new("docker")
            .args(args)
            .current_dir(path)
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[automock]
impl ComposeClient for DockerComposeClient {
    type Error = DockerComposeError;

    fn up(&self, path: &str) -> Result<(), Self::Error> {
        println!("Running docker compose up");
        let compose_file_name = find_compose_file_name(Path::new(path))?;
        Self::run_cmd(&["compose", "-f", &compose_file_name, "up", "-d"], path).map(|_| ())
    }

    fn down(&self, path: &str) -> Result<(), Self::Error> {
        println!("Running docker compose down");
        let compose_file_name = find_compose_file_name(Path::new(path))?;
        Self::run_cmd(&["compose", "-f", &compose_file_name, "down"], path).map(|_| ())
    }

    fn list_containers(&self, path: &str) -> Result<Vec<Container>, Self::Error> {
        println!("Running docker compose ps");
        let compose_file_name = find_compose_file_name(Path::new(path))?;
        Self::run_cmd(
            &[
                "compose",
                "-f",
                &compose_file_name,
                "ps",
                "--all",
                "--format",
                "json",
            ],
            path,
        )?
        .lines()
        .map(|line| {
            let value: serde_json::Value = serde_json::from_str(line)?;

            let name = value
                .get("Name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DockerComposeError::MissingField("Name".into()))?
                .to_string();

            let state_str = value
                .get("State")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DockerComposeError::MissingField("State".into()))?;

            let state = match state_str {
                "paused" => ContainerState::Paused,
                "restarting" => ContainerState::Restarting,
                "removing" => ContainerState::Removing,
                "running" => ContainerState::Running,
                "dead" => ContainerState::Dead,
                "created" => ContainerState::Created,
                "exited" => ContainerState::Exited,
                other => return Err(DockerComposeError::UnknownState(other.into())),
            };

            Ok(Container { name, state })
        })
        .collect()
    }
}

fn find_compose_file_name(dir: &Path) -> Result<String, DockerComposeError> {
    println!("Finding compose file name in {}", dir.display());
    SUPPORTED_COMPOSE_FILES.iter().for_each(|name| {
        println!("Checking {}", name);
        println!("Path: {}", dir.join(name).display());
        println!("Exists: {}", dir.join(name).exists());
    });

    SUPPORTED_COMPOSE_FILES
        .iter()
        .find(|name| dir.join(name).exists())
        .map(|name| name.to_string())
        .ok_or(DockerComposeError::DockerComposeFileDoesNotExist)
}
