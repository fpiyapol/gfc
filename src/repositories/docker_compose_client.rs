use crate::models::docker_compose::{ServiceState, ServiceStatus};
use anyhow::{anyhow, Result};
use std::process::Command;

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

    pub fn up(&self, path: &str) -> Result<()> {
        println!("Running docker compose up");
        Self::run_cmd(&["compose", "up", "-d"], path)?;
        Ok(())
    }

    pub fn down(&self, path: &str) -> Result<()> {
        println!("Running docker compose down");
        Self::run_cmd(&["compose", "down"], path)?;
        Ok(())
    }

    pub fn ps(&self, path: &str) -> Result<Vec<ServiceStatus>> {
        println!("Running docker compose ps");
        let output = Self::run_cmd(&["compose", "ps", "--format", "json"], path)?;

        let status = output
            .lines()
            .filter_map(|line| {
                let service: serde_json::Value = serde_json::from_str(line).ok()?;
                Some(ServiceStatus {
                    name: service.get("Name")?.as_str()?.to_string(),
                    state: match service.get("State")?.as_str()?.to_string().as_str() {
                        "paused" => ServiceState::Paused,
                        "restarting" => ServiceState::Restarting,
                        "removing" => ServiceState::Removing,
                        "running" => ServiceState::Running,
                        "dead" => ServiceState::Dead,
                        "created" => ServiceState::Created,
                        "exited" => ServiceState::Exited,
                        _ => return None,
                    },
                })
            })
            .collect();

        Ok(status)
    }
}
