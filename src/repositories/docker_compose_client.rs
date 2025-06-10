use anyhow::{Context, Result};
use mockall::automock;
use mockall::predicate::*;
use std::path::Path;
use std::process::Command;
use std::process::Output;

use crate::errors::docker_compose::DockerComposeError;
use crate::models::docker_compose::{Container, ContainerState};
use crate::repositories::compose_client::ComposeClient;

const SUPPORTED_COMPOSE_FILES: &[&str] = &[
    "docker-compose.yml",
    "docker-compose.yaml",
    "compose.yml",
    "compose.yaml",
];

#[derive(Debug, Clone)]
pub struct DockerComposeClient;

impl DockerComposeClient {
    pub fn new() -> Result<DockerComposeClient, DockerComposeError> {
        Ok(DockerComposeClient {})
    }

    fn execute_docker_command(&self, args: &[&str], path: &str) -> Result<Output, std::io::Error> {
        Command::new("docker").args(args).current_dir(path).output()
    }

    fn find_compose_file(&self, path: &str) -> Result<String, DockerComposeError> {
        for file_name in SUPPORTED_COMPOSE_FILES.iter() {
            let file_path = Path::new(path).join(file_name);
            if file_path.exists() {
                return Ok(file_name.to_string());
            }
        }

        Err(DockerComposeError::ComposeFileNotFound {
            path: path.to_string(),
        })
    }

    fn parse_container_json(
        &self,
        json_line: &str,
        path: &str,
    ) -> Result<Container, DockerComposeError> {
        let value: serde_json::Value = serde_json::from_str(json_line)
            .with_context(|| format!("Parsing container JSON from path '{}'", path))
            .map_err(|e| {
                println!("Docker compose operation failed: {}", e);
                DockerComposeError::ListContainersFailed {
                    path: path.to_string(),
                    reason: format!("Failed to parse Docker compose output: {:#}", e),
                }
            })?;

        let name = self.extract_json_string_field(&value, "Name", path)?;
        let state_str = self.extract_json_string_field(&value, "State", path)?;
        let state = self.parse_container_state(state_str, path)?;

        Ok(Container { name, state })
    }

    fn extract_json_string_field(
        &self,
        json: &serde_json::Value,
        field: &str,
        path: &str,
    ) -> Result<String, DockerComposeError> {
        json.get(field)
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| {
                let reason = format!("Missing '{}' field in Docker compose output", field);
                println!("Docker compose operation failed: {}", reason);
                DockerComposeError::ListContainersFailed {
                    path: path.to_string(),
                    reason,
                }
            })
    }

    fn parse_container_state(
        &self,
        state_str: String,
        path: &str,
    ) -> Result<ContainerState, DockerComposeError> {
        match state_str.as_str() {
            "paused" => Ok(ContainerState::Paused),
            "restarting" => Ok(ContainerState::Restarting),
            "removing" => Ok(ContainerState::Removing),
            "running" => Ok(ContainerState::Running),
            "dead" => Ok(ContainerState::Dead),
            "created" => Ok(ContainerState::Created),
            "exited" => Ok(ContainerState::Exited),
            other => {
                let reason = format!("Unknown container state: {}", other);
                println!("Docker compose operation failed: {}", reason);
                Err(DockerComposeError::ListContainersFailed {
                    path: path.to_string(),
                    reason,
                })
            }
        }
    }
}

#[automock]
impl ComposeClient for DockerComposeClient {
    type Error = DockerComposeError;

    fn up(&self, path: &str) -> Result<(), Self::Error> {
        println!("Running docker compose up in {}", path);

        let compose_file_name = self.find_compose_file(path)?;
        let args = ["compose", "-f", &compose_file_name, "up", "-d"];

        let output = self.execute_docker_command(&args, path).map_err(|e| {
            println!("Docker compose up command error: {}", e);
            DockerComposeError::UpFailed {
                path: path.to_string(),
                reason: format!("Failed to execute docker compose up command: {}", e),
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Docker compose up failed: {}", stderr);

            return Err(DockerComposeError::UpFailed {
                path: path.to_string(),
                reason: format!("Docker compose up failed: {}", stderr),
            });
        }

        Ok(())
    }

    fn down(&self, path: &str) -> Result<(), Self::Error> {
        println!("Running docker compose down in {}", path);

        let compose_file_name = self.find_compose_file(path)?;
        let args = ["compose", "-f", &compose_file_name, "down"];

        let output = self.execute_docker_command(&args, path).map_err(|e| {
            println!("Docker compose down command error: {}", e);
            DockerComposeError::DownFailed {
                path: path.to_string(),
                reason: format!("Failed to execute docker compose down command: {}", e),
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Docker compose down failed: {}", stderr);

            return Err(DockerComposeError::DownFailed {
                path: path.to_string(),
                reason: format!("Docker compose down failed: {}", stderr),
            });
        }

        Ok(())
    }

    fn list_containers(&self, path: &str) -> Result<Vec<Container>, Self::Error> {
        println!("Listing containers in {}", path);

        let compose_file_name = self
            .find_compose_file(path)
            .with_context(|| format!("Finding docker-compose file in '{}'", path))
            .map_err(|e| DockerComposeError::ListContainersFailed {
                path: path.to_string(),
                reason: format!("Could not find compose file: {:#}", e),
            })?;

        let args = [
            "compose",
            "-f",
            &compose_file_name,
            "ps",
            "--all",
            "--format",
            "json",
        ];

        let output = self
            .execute_docker_command(&args, path)
            .with_context(|| format!("Executing 'docker compose ps' in '{}'", path))
            .map_err(|e| {
                let reason = format!("Failed to execute docker compose ps command: {:#}", e);
                println!("Docker compose operation failed: {}", reason);
                DockerComposeError::ListContainersFailed {
                    path: path.to_string(),
                    reason,
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let reason = format!(
                "Docker compose ps command returned non-zero exit status: {}",
                stderr
            );
            println!("Docker compose operation failed: {}", reason);
            return Err(DockerComposeError::ListContainersFailed {
                path: path.to_string(),
                reason,
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| self.parse_container_json(line, path))
            .collect()
    }
}
