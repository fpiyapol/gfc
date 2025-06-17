use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComposeError {
    #[error("Failed to start services in {path}: {reason}")]
    UpFailed { path: String, reason: String },

    #[error("Failed to stop services in {path}: {reason}")]
    DownFailed { path: String, reason: String },

    #[error("Failed to list containers in {path}: {reason}")]
    ListContainersFailed { path: String, reason: String },

    #[error("Compose file not found in {path}")]
    ComposeFileNotFound { path: String },
}

impl ComposeError {
    pub fn error_code(&self) -> &'static str {
        use crate::errors::codes::ErrorCode;

        match self {
            ComposeError::UpFailed { .. } => ErrorCode::COMPOSE_UP_FAILED,
            ComposeError::DownFailed { .. } => ErrorCode::COMPOSE_DOWN_FAILED,
            ComposeError::ListContainersFailed { .. } => ErrorCode::COMPOSE_LIST_CONTAINERS_FAILED,
            ComposeError::ComposeFileNotFound { .. } => ErrorCode::COMPOSE_FILE_NOT_FOUND,
        }
    }
}
