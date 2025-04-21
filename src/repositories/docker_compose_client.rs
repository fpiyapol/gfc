use anyhow::{anyhow, Result};
use mockall::automock;
use mockall::predicate::*;
use std::process::Command;

use crate::models::docker_compose::{Container, ContainerState};
use crate::repositories::compose_client::ComposeClient;

#[derive(Debug, Clone)]
pub struct DockerComposeClient;

impl DockerComposeClient {
    pub fn new() -> Result<DockerComposeClient> {
        Ok(Self {})
    }

    fn run_cmd(args: &[&str], path: &str) -> Result<String> {
        let output = Command::new("docker")
            .args(args)
            .current_dir(path)
            .output()
            .map_err(|e| anyhow!("Failed to execute docker compose: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!(
                "docker compose failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

#[automock]
impl ComposeClient for DockerComposeClient {
    fn up(&self, path: &str) -> Result<()> {
        println!("Running docker compose up");
        Self::run_cmd(&["compose", "up", "-d"], path)?;
        Ok(())
    }

    fn down(&self, path: &str) -> Result<()> {
        println!("Running docker compose down");
        Self::run_cmd(&["compose", "down"], path)?;
        Ok(())
    }

    fn list_containers(&self, path: &str) -> Result<Vec<Container>> {
        println!("Running docker compose ps");
        let output = Self::run_cmd(&["compose", "ps", "--all", "--format", "json"], path)?;

        let status = output
            .lines()
            .filter_map(|line| {
                let service: serde_json::Value = serde_json::from_str(line).ok()?;
                Some(Container {
                    name: service.get("Name")?.as_str()?.to_string(),
                    state: match service.get("State")?.as_str()?.to_string().as_str() {
                        "paused" => ContainerState::Paused,
                        "restarting" => ContainerState::Restarting,
                        "removing" => ContainerState::Removing,
                        "running" => ContainerState::Running,
                        "dead" => ContainerState::Dead,
                        "created" => ContainerState::Created,
                        "exited" => ContainerState::Exited,
                        _ => return None,
                    },
                })
            })
            .collect();

        Ok(status)
    }
}
