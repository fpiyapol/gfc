pub mod codes;
pub mod compose;
pub mod git;
pub mod project;

use thiserror::Error;

use crate::config::ConfigError;
use crate::errors::compose::ComposeError;
use crate::errors::git::GitError;
use crate::errors::project::ProjectUsecaseError;

pub type GfcResult<T> = Result<T, GfcError>;

pub trait HasErrorCode {
    fn error_code(&self) -> &'static str;
}

#[derive(Debug, Error)]
pub enum GfcError {
    #[error(transparent)]
    Config(#[from] ConfigError),

    #[error(transparent)]
    Git(#[from] GitError),

    #[error(transparent)]
    Compose(#[from] ComposeError),

    #[error(transparent)]
    Project(#[from] ProjectUsecaseError),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl HasErrorCode for GfcError {
    fn error_code(&self) -> &'static str {
        match self {
            GfcError::Config(e) => e.error_code(),
            GfcError::Git(e) => e.error_code(),
            GfcError::Compose(e) => e.error_code(),
            GfcError::Project(e) => e.error_code(),
            GfcError::Internal(_) => "E000",
        }
    }
}
