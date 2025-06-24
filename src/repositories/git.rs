use chrono::prelude::*;
use std::path::Path;
use std::process::Command;
use std::process::Output;
use tracing::{debug, instrument};

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
    #[instrument(skip(self), name = "git_repository::clone_repository", fields(git.url = %source.url, git.branch = %source.branch, working_dir = %working_dir.display()))]
    fn clone_repository(&self, source: &GitSource, working_dir: &Path) -> Result<(), GitError> {
        debug!(
            git.command = "clone",
            git.args = ?["clone", "--branch", &source.branch, &source.url, &working_dir.to_string_lossy()],
            "Executing git clone command"
        );

        let args = [
            "clone",
            "--branch",
            &source.branch,
            &source.url,
            &working_dir.to_string_lossy(),
        ];

        let output = self.execute_git_command(&args, None).map_err(|e| {
            GitError::CloneFailed {
                url: source.url.clone(),
                reason: format!("Failed to execute git clone command: {}", e),
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!(
                git.exit_code = ?output.status.code(),
                git.stderr = %stderr,
                "Git clone command failed"
            );
            return Err(GitError::CloneFailed {
                url: source.url.clone(),
                reason: format!("Git clone failed: {}", stderr),
            });
        }

        debug!("Git clone completed successfully");
        Ok(())
    }

    #[instrument(skip(self), name = "git_repository::pull_repository", fields(git.url = %source.url, working_dir = %working_dir.display()))]
    fn pull_repository(&self, source: &GitSource, working_dir: &Path) -> Result<(), GitError> {
        if !working_dir.exists() {
            debug!(
                working_dir.exists = false,
                "Working directory does not exist, falling back to clone"
            );
            return self.clone_repository(source, working_dir);
        }

        debug!(
            git.command = "pull",
            git.args = ?["pull"],
            "Executing git pull command"
        );

        let args = ["pull"];

        let output = self
            .execute_git_command(&args, Some(working_dir))
            .map_err(|e| {
                GitError::PullFailed {
                    path: working_dir.to_path_buf(),
                    reason: format!("Failed to execute git pull command: {}", e),
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!(
                git.exit_code = ?output.status.code(),
                git.stderr = %stderr,
                "Git pull command failed"
            );
            return Err(GitError::PullFailed {
                path: working_dir.to_path_buf(),
                reason: format!("Git pull failed: {}", stderr),
            });
        }

        debug!("Git pull completed successfully");
        Ok(())
    }

    #[instrument(skip(self), name = "git_repository::get_last_commit_timestamp", fields(working_dir = %working_dir.display()))]
    fn get_last_commit_timestamp(&self, working_dir: &Path) -> Result<DateTime<Utc>, GitError> {
        debug!(
            git.command = "log",
            git.args = ?["log", "-1", "--format=%ct"],
            "Executing git log command to get last commit timestamp"
        );

        let args = ["log", "-1", "--format=%ct"];

        let output = self
            .execute_git_command(&args, Some(working_dir))
            .map_err(|e| {
                GitError::GetLastCommitTimestampFailed {
                    path: working_dir.to_path_buf(),
                    reason: format!("Failed to execute git log command: {}", e),
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!(
                git.exit_code = ?output.status.code(),
                git.stderr = %stderr,
                "Git log command failed"
            );
            return Err(GitError::GetLastCommitTimestampFailed {
                path: working_dir.to_path_buf(),
                reason: format!("Git log command failed: {}", stderr),
            });
        }

        let timestamp_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!(
            git.timestamp_raw = %timestamp_str,
            "Received timestamp from git log"
        );

        let epoch = timestamp_str.parse::<i64>().map_err(|e| {
            GitError::GetLastCommitTimestampFailed {
                path: working_dir.to_path_buf(),
                reason: format!("Failed to parse timestamp '{}': {}", timestamp_str, e),
            }
        })?;

        let timestamp = Utc.timestamp_opt(epoch, 0).single().ok_or_else(|| {
            GitError::GetLastCommitTimestampFailed {
                path: working_dir.to_path_buf(),
                reason: format!("Invalid timestamp value: {}", epoch),
            }
        })?;

        debug!(
            git.timestamp_parsed = %timestamp,
            git.epoch = epoch,
            "Successfully parsed commit timestamp"
        );

        Ok(timestamp)
    }
}
