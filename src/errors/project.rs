use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProjectUsecaseError {
    #[error("Failed to create project '{project_name}': {reason}")]
    CreateProjectFailed {
        project_name: String,
        reason: String,
    },

    #[error("Failed to list projects: {reason}")]
    ListProjectsFailed { reason: String },

    #[error("Invalid path: {reason}")]
    InvalidPath { reason: String },

    #[error("Project file '{project_name}' not found: {reason}")]
    ProjectNotFound {
        project_name: String,
        reason: String,
    },

    #[error("Failed to read project file '{project_name}': {reason}")]
    ProjectFileReadFailed {
        project_name: String,
        reason: String,
    },
}

impl ProjectUsecaseError {
    pub fn error_code(&self) -> &'static str {
        use crate::errors::codes::ErrorCode;

        match self {
            ProjectUsecaseError::CreateProjectFailed { .. } => ErrorCode::PROJECT_CREATE_FAILED,
            ProjectUsecaseError::ListProjectsFailed { .. } => ErrorCode::PROJECT_LIST_FAILED,
            ProjectUsecaseError::InvalidPath { .. } => ErrorCode::PROJECT_INVALID_PATH,
            ProjectUsecaseError::ProjectNotFound { .. } => ErrorCode::PROJECT_NOT_FOUND,
            ProjectUsecaseError::ProjectFileReadFailed { .. } => {
                ErrorCode::PROJECT_FILE_READ_FAILED
            }
        }
    }
}
