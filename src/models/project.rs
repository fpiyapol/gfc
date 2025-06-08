use serde::{Deserialize, Serialize};

use crate::models::git::GitSource;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectFile {
    pub name: String,
    pub source: GitSource,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    pub source: GitSource,
    pub status: String,
    pub last_updated_at: String,
}
