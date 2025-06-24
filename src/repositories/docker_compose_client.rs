use anyhow::{Context, Result};
use mockall::automock;
use mockall::predicate::*;
use std::path::Path;
use std::process::Command;
use std::process::Output;
use tracing::{debug, instrument};

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
    #[instrument(skip(self), name = "compose_repository::up", fields(compose.file = %compose_file_path))]
    fn up(&self, compose_file_path: &str) -> Result<(), ComposeError> {

        debug!(
            compose.file_path = %compose_file_path,
            "Validating compose file exists"
        );
        self.validate_compose_file_exists_and_not_directory(compose_file_path)?;

        let args = ["-f", compose_file_path, "up", "-d"];
        debug!(
            docker.command = "compose",
            docker.args = ?args,
            "Executing docker compose up command"
        );

        let output = self.execute(&args).map_err(|e| {
            ComposeError::UpFailed {
                path: compose_file_path.to_string(),
                reason: format!("Failed to execute docker compose up command: {}", e),
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!(
                docker.exit_code = ?output.status.code(),
                docker.stderr = %stderr,
                "Docker compose up command failed"
            );
            return Err(ComposeError::UpFailed {
                path: compose_file_path.to_string(),
                reason: format!("Docker compose up failed: {}", stderr),
            });
        }

        debug!("Docker compose up completed successfully");
        Ok(())
    }

    #[instrument(skip(self), name = "compose_repository::down", fields(compose.file = %compose_file_path))]
    fn down(&self, compose_file_path: &str) -> Result<(), ComposeError> {

        debug!(
            compose.file_path = %compose_file_path,
            "Validating compose file exists for down operation"
        );
        self.validate_compose_file_exists_and_not_directory(compose_file_path)?;

        let args = ["-f", compose_file_path, "down"];
        debug!(
            docker.command = "compose",
            docker.args = ?args,
            "Executing docker compose down command"
        );

        let output = self.execute(&args).map_err(|e| {
            ComposeError::DownFailed {
                path: compose_file_path.to_string(),
                reason: format!("Failed to execute docker compose down command: {}", e),
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!(
                docker.exit_code = ?output.status.code(),
                docker.stderr = %stderr,
                "Docker compose down command failed"
            );
            return Err(ComposeError::DownFailed {
                path: compose_file_path.to_string(),
                reason: format!("Docker compose down failed: {}", stderr),
            });
        }

        debug!("Docker compose down completed successfully");
        Ok(())
    }

    #[instrument(skip(self), name = "compose_repository::list_containers", fields(compose.file = %compose_file_path))]
    fn list_containers(&self, compose_file_path: &str) -> Result<Vec<Container>, ComposeError> {

        debug!(
            compose.file_path = %compose_file_path,
            "Validating compose file exists for container listing"
        );
        self.validate_compose_file_exists_and_not_directory(compose_file_path)?;
        
        let args = ["-f", compose_file_path, "ps", "--all", "--format", "json"];
        debug!(
            docker.command = "compose",
            docker.args = ?args,
            "Executing docker compose ps command"
        );

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
                ComposeError::ListContainersFailed {
                    path: compose_file_path.to_string(),
                    reason,
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!(
                docker.exit_code = ?output.status.code(),
                docker.stderr = %stderr,
                "Docker compose ps command failed"
            );
            let reason = format!(
                "Docker compose ps command returned non-zero exit status: {}",
                stderr
            );
            return Err(ComposeError::ListContainersFailed {
                path: compose_file_path.to_string(),
                reason,
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let lines: Vec<&str> = stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect();
        
        debug!(
            docker.output_lines = lines.len(),
            "Processing docker compose ps output"
        );

        lines
            .iter()
            .map(|line| {
                debug!(
                    docker.container_json = %line,
                    "Parsing container JSON"
                );
                self.parse_container_json(line, compose_file_path)
            })
            .collect()
    }
}
