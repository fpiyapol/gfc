use thiserror::Error;

#[derive(Debug, Error)]
pub enum DockerComposeError {
    #[error("Failed to start services in {path}: {reason}")]
    UpFailed { path: String, reason: String },

    #[error("Failed to stop services in {path}: {reason}")]
    DownFailed { path: String, reason: String },

    #[error("Failed to list containers in {path}: {reason}")]
    ListContainersFailed { path: String, reason: String },

    #[error("Docker Compose file not found in {path}")]
    ComposeFileNotFound { path: String },
}

impl DockerComposeError {
    pub fn error_code(&self) -> &'static str {
        use crate::errors::codes::ErrorCode;

        match self {
            DockerComposeError::UpFailed { .. } => ErrorCode::DOCKER_COMPOSE_UP_FAILED,
            DockerComposeError::DownFailed { .. } => ErrorCode::DOCKER_COMPOSE_DOWN_FAILED,
            DockerComposeError::ListContainersFailed { .. } => {
                ErrorCode::DOCKER_COMPOSE_LIST_CONTAINERS_FAILED
            }
            DockerComposeError::ComposeFileNotFound { .. } => {
                ErrorCode::DOCKER_COMPOSE_FILE_NOT_FOUND
            }
        }
    }
}
