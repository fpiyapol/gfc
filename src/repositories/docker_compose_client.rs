use anyhow::{Context, Result};
use mockall::automock;
use mockall::predicate::*;
use std::path::Path;
use std::process::Command;
use std::process::Output;

use crate::errors::docker_compose::DockerComposeError;
use crate::models::docker_compose::{Container, ContainerState};
use crate::repositories::compose_client::ComposeClient;

#[derive(Debug, Clone)]
pub struct DockerComposeClient;

impl DockerComposeClient {
    pub fn new() -> Result<DockerComposeClient, DockerComposeError> {
        Ok(DockerComposeClient {})
    }

    fn execute_docker_command(
        &self,
        args: &[&str],
        working_dir: &str,
    ) -> Result<Output, std::io::Error> {
        Command::new("docker")
            .args(args)
            .current_dir(working_dir)
            .output()
    }

    fn validate_compose_file_exists_and_not_directory(
        &self,
        file_path: &str,
    ) -> Result<(), DockerComposeError> {
        let path = Path::new(file_path);

        if !path.exists() {
            return Err(DockerComposeError::ComposeFileNotFound {
                path: file_path.to_string(),
            });
        }

        if !path.is_file() {
            return Err(DockerComposeError::ComposeFileNotFound {
                path: file_path.to_string(),
            });
        }

        Ok(())
    }

    fn get_working_dir_and_file_name_from(
        &self,
        compose_file_path: &str,
    ) -> Result<(String, String), DockerComposeError> {
        let path = Path::new(compose_file_path);
        let working_dir = match path.parent().and_then(|p| p.to_str()) {
            Some(dir) => dir.to_string(),
            None => {
                println!("Could not extract parent directory from path, using current directory");
                ".".to_string()
            }
        };

        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                println!(
                    "Could not extract filename from path: {}",
                    compose_file_path
                );
                DockerComposeError::ComposeFileNotFound {
                    path: compose_file_path.to_string(),
                }
            })
            .map(|s| s.to_string())?;

        Ok((working_dir, filename))
    }

    fn parse_container_json(
        &self,
        json_line: &str,
        path: &str,
    ) -> Result<Container, DockerComposeError> {
        let value = serde_json::from_str(json_line)
            .with_context(|| format!("Parsing container JSON from file '{}'", path))
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

    fn up(&self, compose_file_path: &str) -> Result<(), Self::Error> {
        println!("Running docker compose up using {}", compose_file_path);

        self.validate_compose_file_exists_and_not_directory(compose_file_path)?;

        let (working_dir, filename) = self.get_working_dir_and_file_name_from(compose_file_path)?;

        let args = ["compose", "-f", &filename, "up", "-d"];

        let output = self
            .execute_docker_command(&args, &working_dir)
            .map_err(|e| {
                println!("Docker compose up command error: {}", e);
                DockerComposeError::UpFailed {
                    path: compose_file_path.to_string(),
                    reason: format!("Failed to execute docker compose up command: {}", e),
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Docker compose up failed: {}", stderr);

            return Err(DockerComposeError::UpFailed {
                path: compose_file_path.to_string(),
                reason: format!("Docker compose up failed: {}", stderr),
            });
        }

        Ok(())
    }

    fn down(&self, compose_file_path: &str) -> Result<(), Self::Error> {
        println!("Running docker compose down using {}", compose_file_path);

        self.validate_compose_file_exists_and_not_directory(compose_file_path)?;

        let (working_dir, filename) = self.get_working_dir_and_file_name_from(compose_file_path)?;

        let args = ["compose", "-f", &filename, "down"];

        let output = self
            .execute_docker_command(&args, &working_dir)
            .map_err(|e| {
                println!("Docker compose down command error: {}", e);
                DockerComposeError::DownFailed {
                    path: compose_file_path.to_string(),
                    reason: format!("Failed to execute docker compose down command: {}", e),
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Docker compose down failed: {}", stderr);

            return Err(DockerComposeError::DownFailed {
                path: compose_file_path.to_string(),
                reason: format!("Docker compose down failed: {}", stderr),
            });
        }

        Ok(())
    }

    fn list_containers(&self, compose_file_path: &str) -> Result<Vec<Container>, Self::Error> {
        println!("Listing containers using {}", compose_file_path);

        self.validate_compose_file_exists_and_not_directory(compose_file_path)?;
        let (working_dir, filename) = self.get_working_dir_and_file_name_from(compose_file_path)?;

        let args = [
            "compose", "-f", &filename, "ps", "--all", "--format", "json",
        ];

        let output = self
            .execute_docker_command(&args, &working_dir)
            .with_context(|| {
                format!(
                    "Executing 'docker compose ps' using '{}'",
                    compose_file_path
                )
            })
            .map_err(|e| {
                let reason = format!("Failed to execute docker compose ps command: {:#}", e);
                println!("Docker compose operation failed: {}", reason);
                DockerComposeError::ListContainersFailed {
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
            return Err(DockerComposeError::ListContainersFailed {
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
