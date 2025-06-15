use anyhow::{Context, Result};
use mockall::automock;
use mockall::predicate::*;
use std::path::Path;
use std::process::Command;
use std::process::Output;

use crate::errors::compose::ComposeError;
use crate::models::docker_compose::{Container, ContainerState};
use crate::repositories::compose_client::ComposeClient;

#[derive(Debug, Clone)]
pub struct DockerComposeClient;

impl DockerComposeClient {
    pub fn new() -> Result<DockerComposeClient, ComposeError> {
        Ok(DockerComposeClient {})
    }

    fn execute(&self, args: &[&str]) -> Result<Output, std::io::Error> {
        Command::new("docker").arg("compose").args(args).output()
    }

    fn validate_compose_file_exists_and_not_directory(
        &self,
        compose_file_path: &str,
    ) -> Result<(), ComposeError> {
        let path = Path::new(compose_file_path);

        if !path.exists() {
            return Err(ComposeError::ComposeFileNotFound {
                path: compose_file_path.to_string(),
            });
        }

        if !path.is_file() {
            return Err(ComposeError::ComposeFileNotFound {
                path: compose_file_path.to_string(),
            });
        }

        Ok(())
    }

    fn parse_container_json(&self, json_line: &str, path: &str) -> Result<Container, ComposeError> {
        let value = serde_json::from_str(json_line)
            .with_context(|| format!("Parsing container JSON from file '{}'", path))
            .map_err(|e| {
                println!("Docker compose operation failed: {}", e);
                ComposeError::ListContainersFailed {
                    path: path.to_string(),
                    reason: format!("Failed to parse Docker compose output: {:#}", e),
                }
            })?;

        let name = self.extract_json_string_field(&value, "Name", path)?;
        let state = self
            .extract_json_string_field(&value, "State", path)
            .and_then(|state| self.parse_container_state(&state, path))?;

        Ok(Container { name, state })
    }

    fn extract_json_string_field(
        &self,
        json: &serde_json::Value,
        field: &str,
        path: &str,
    ) -> Result<String, ComposeError> {
        json.get(field)
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| {
                let reason = format!("Missing '{}' field in Docker compose output", field);
                println!("Docker compose operation failed: {}", reason);
                ComposeError::ListContainersFailed {
                    path: path.to_string(),
                    reason,
                }
            })
    }

    fn parse_container_state(
        &self,
        state_str: &str,
        path: &str,
    ) -> Result<ContainerState, ComposeError> {
        match state_str {
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
                Err(ComposeError::ListContainersFailed {
                    path: path.to_string(),
                    reason,
                })
            }
        }
    }
}

#[automock]
impl ComposeClient for DockerComposeClient {
    fn up(&self, compose_file_path: &str) -> Result<(), ComposeError> {
        println!("Running docker compose up using {}", compose_file_path);

        self.validate_compose_file_exists_and_not_directory(compose_file_path)?;

        let args = ["-f", compose_file_path, "up", "-d"];

        let output = self.execute(&args).map_err(|e| {
            println!("Docker compose up command error: {}", e);
            ComposeError::UpFailed {
                path: compose_file_path.to_string(),
                reason: format!("Failed to execute docker compose up command: {}", e),
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Docker compose up failed: {}", stderr);

            return Err(ComposeError::UpFailed {
                path: compose_file_path.to_string(),
                reason: format!("Docker compose up failed: {}", stderr),
            });
        }

        Ok(())
    }

    fn down(&self, compose_file_path: &str) -> Result<(), ComposeError> {
        println!("Running docker compose down using {}", compose_file_path);

        self.validate_compose_file_exists_and_not_directory(compose_file_path)?;

        let args = ["-f", compose_file_path, "down"];

        let output = self.execute(&args).map_err(|e| {
            println!("Docker compose down command error: {}", e);
            ComposeError::DownFailed {
                path: compose_file_path.to_string(),
                reason: format!("Failed to execute docker compose down command: {}", e),
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Docker compose down failed: {}", stderr);

            return Err(ComposeError::DownFailed {
                path: compose_file_path.to_string(),
                reason: format!("Docker compose down failed: {}", stderr),
            });
        }

        Ok(())
    }

    fn list_containers(&self, compose_file_path: &str) -> Result<Vec<Container>, ComposeError> {
        println!("Listing containers using {}", compose_file_path);

        self.validate_compose_file_exists_and_not_directory(compose_file_path)?;
        let args = ["-f", compose_file_path, "ps", "--all", "--format", "json"];

        let output = self
            .execute(&args)
            .with_context(|| {
                format!(
                    "Executing 'docker compose ps' using '{}'",
                    compose_file_path
                )
            })
            .map_err(|e| {
                let reason = format!("Failed to execute docker compose ps command: {:#}", e);
                println!("Docker compose operation failed: {}", reason);
                ComposeError::ListContainersFailed {
                    path: compose_file_path.to_string(),
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
            return Err(ComposeError::ListContainersFailed {
                path: compose_file_path.to_string(),
                reason,
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| self.parse_container_json(line, compose_file_path))
            .collect()
    }
}
