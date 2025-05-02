use anyhow::{anyhow, Result};
use chrono::prelude::*;
use std::path::Path;
use std::process::Command;

use crate::models::git::GitSource;

pub trait GitClient {
    fn clone_repository(&self, source: &GitSource, working_dir: &Path) -> Result<()>;
    fn pull_repository(&self, source: &GitSource, working_dir: &Path) -> Result<()>;
    fn get_last_commit_timestamp(&self, working_dir: &Path) -> Result<DateTime<Utc>>;
}

#[derive(Debug, Clone)]
pub struct GitClientImpl;

impl GitClient for GitClientImpl {
    fn clone_repository(&self, source: &GitSource, working_dir: &Path) -> Result<()> {
        Command::new("git")
            .arg("clone")
            .arg("--branch")
            .arg(&source.branch)
            .arg(&source.url)
            .arg(working_dir)
            .status()?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow!("Failed to clone {}", source.url))
    }

    fn pull_repository(&self, source: &GitSource, working_dir: &Path) -> Result<()> {
        if !working_dir.exists() {
            return self.clone_repository(source, working_dir);
        }

        Command::new("git")
            .arg("pull")
            .current_dir(working_dir)
            .status()?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow!("Failed to pull {}", working_dir.display()))
    }

    fn get_last_commit_timestamp(&self, working_dir: &Path) -> Result<DateTime<Utc>> {
        let output = Command::new("git")
            .current_dir(working_dir)
            .args(["log", "-1", "--format=%ct"])
            .output()
            .map_err(|e| {
                anyhow!(
                    "Failed to get last commit timestamp from {:?}: {}",
                    working_dir,
                    e
                )
            })?;

        let timestamp = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<i64>()
            .map(|epoch| Utc.timestamp_opt(epoch, 0).unwrap())
            .map_err(|e| {
                anyhow!(
                    "Failed to parse epoch timestamp from {:?}: {}",
                    working_dir,
                    e
                )
            })?;

        Ok(timestamp)
    }
}
