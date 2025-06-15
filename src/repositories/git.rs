use chrono::prelude::*;
use std::path::Path;
use std::process::Command;
use std::process::Output;

use crate::errors::git::GitError;
use crate::models::git::GitSource;

pub trait GitClient {
    fn clone_repository(&self, source: &GitSource, working_dir: &Path) -> Result<(), GitError>;
    fn pull_repository(&self, source: &GitSource, working_dir: &Path) -> Result<(), GitError>;
    fn get_last_commit_timestamp(&self, working_dir: &Path) -> Result<DateTime<Utc>, GitError>;
}

#[derive(Debug, Clone)]
pub struct GitClientImpl;

impl GitClientImpl {
    fn execute_git_command(
        &self,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<Output, std::io::Error> {
        let mut command = Command::new("git");

        command.args(args);

        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }

        command.output()
    }
}

impl GitClient for GitClientImpl {
    fn clone_repository(&self, source: &GitSource, working_dir: &Path) -> Result<(), GitError> {
        println!(
            "Attempting to clone {} (branch: {}) to {}",
            source.url,
            source.branch,
            working_dir.display()
        );

        let args = [
            "clone",
            "--branch",
            &source.branch,
            &source.url,
            &working_dir.to_string_lossy(),
        ];

        let output = self.execute_git_command(&args, None).map_err(|e| {
            println!("Git clone command error: {}", e);
            GitError::CloneFailed {
                url: source.url.clone(),
                reason: format!("Failed to execute git clone command: {}", e),
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Git clone failed: {}", stderr);

            return Err(GitError::CloneFailed {
                url: source.url.clone(),
                reason: format!("Git clone failed: {}", stderr),
            });
        }

        Ok(())
    }

    fn pull_repository(&self, source: &GitSource, working_dir: &Path) -> Result<(), GitError> {
        if !working_dir.exists() {
            println!("Working directory does not exist, cloning instead of pulling");
            return self.clone_repository(source, working_dir);
        }

        println!(
            "Attempting to pull latest changes in {}",
            working_dir.display()
        );

        let args = ["pull"];

        let output = self
            .execute_git_command(&args, Some(working_dir))
            .map_err(|e| {
                println!("Git pull command error: {}", e);
                GitError::PullFailed {
                    path: working_dir.to_path_buf(),
                    reason: format!("Failed to execute git pull command: {}", e),
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Git pull failed: {}", stderr);

            return Err(GitError::PullFailed {
                path: working_dir.to_path_buf(),
                reason: format!("Git pull failed: {}", stderr),
            });
        }

        Ok(())
    }

    fn get_last_commit_timestamp(&self, working_dir: &Path) -> Result<DateTime<Utc>, GitError> {
        println!(
            "Retrieving last commit timestamp from {}",
            working_dir.display()
        );

        let args = ["log", "-1", "--format=%ct"];

        let output = self
            .execute_git_command(&args, Some(working_dir))
            .map_err(|e| {
                println!("Failed to execute git log command: {}", e);
                GitError::GetLastCommitTimestampFailed {
                    path: working_dir.to_path_buf(),
                    reason: format!("Failed to execute git log command: {}", e),
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            println!("Git log command failed: {}", stderr);

            return Err(GitError::GetLastCommitTimestampFailed {
                path: working_dir.to_path_buf(),
                reason: format!("Git log command failed: {}", stderr),
            });
        }

        let timestamp_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

        let epoch = timestamp_str.parse::<i64>().map_err(|e| {
            println!("Failed to parse timestamp '{}': {}", timestamp_str, e);
            GitError::GetLastCommitTimestampFailed {
                path: working_dir.to_path_buf(),
                reason: format!("Failed to parse timestamp '{}': {}", timestamp_str, e),
            }
        })?;

        Utc.timestamp_opt(epoch, 0).single().ok_or_else(|| {
            println!("Invalid timestamp value: {}", epoch);
            GitError::GetLastCommitTimestampFailed {
                path: working_dir.to_path_buf(),
                reason: format!("Invalid timestamp value: {}", epoch),
            }
        })
    }
}
