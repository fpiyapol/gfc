use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Failed to clone repository from {url}: {reason}")]
    CloneFailed { url: String, reason: String },

    #[error("Failed to pull latest changes for {path}: {reason}")]
    PullFailed { path: PathBuf, reason: String },

    #[error("Failed to get last commit timestamp from {path}: {reason}")]
    GetLastCommitTimestampFailed { path: PathBuf, reason: String },
}

impl GitError {
    pub fn error_code(&self) -> &'static str {
        use crate::errors::codes::ErrorCode;

        match self {
            GitError::CloneFailed { .. } => ErrorCode::GIT_CLONE_FAILED,
            GitError::PullFailed { .. } => ErrorCode::GIT_PULL_FAILED,
            GitError::GetLastCommitTimestampFailed { .. } => {
                ErrorCode::GIT_GET_LAST_COMMIT_TIMESTAMP_FAILED
            }
        }
    }
}
