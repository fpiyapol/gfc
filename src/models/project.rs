use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::git::GitSource;

/// Represents the current operational status of a project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectStatus {
    /// All containers are running successfully
    Running {
        active_containers: usize,
        total_containers: usize,
    },
    /// No containers are currently running
    Stopped,
    /// Some but not all containers are running
    PartiallyRunning {
        active_containers: usize,
        total_containers: usize,
    },
    /// Project creation/deployment is in progress
    CreationInProgress,
    /// Project deployment failed
    DeploymentFailed { reason: String },
    /// Status cannot be determined
    Unknown,
}

impl ProjectStatus {
    /// Creates a ProjectStatus from container counts
    pub fn from_container_counts(active: usize, total: usize) -> Self {
        match (active, total) {
            (0, 0) => ProjectStatus::Unknown,
            (0, _) => ProjectStatus::Stopped,
            (active, total) if active == total => ProjectStatus::Running {
                active_containers: active,
                total_containers: total,
            },
            (active, total) => ProjectStatus::PartiallyRunning {
                active_containers: active,
                total_containers: total,
            },
        }
    }

    /// Returns a human-readable status string for display
    pub fn display_string(&self) -> String {
        match self {
            ProjectStatus::Running {
                active_containers,
                total_containers,
            } => {
                format!("Running ({}/{})", active_containers, total_containers)
            }
            ProjectStatus::Stopped => "Exited".to_string(),
            ProjectStatus::PartiallyRunning {
                active_containers,
                total_containers,
            } => {
                format!(
                    "Partially Running ({}/{})",
                    active_containers, total_containers
                )
            }
            ProjectStatus::CreationInProgress => "Creating...".to_string(),
            ProjectStatus::DeploymentFailed { reason } => format!("Failed: {}", reason),
            ProjectStatus::Unknown => "Unknown".to_string(),
        }
    }
}

/// A validated project name wrapper type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectName(String);

impl ProjectName {
    /// Creates a new ProjectName after validation
    pub fn new(name: String) -> Result<Self, String> {
        if name.trim().is_empty() {
            return Err("Project name cannot be empty".to_string());
        }

        if name.len() > 100 {
            return Err("Project name cannot exceed 100 characters".to_string());
        }

        // Check for invalid characters (basic validation)
        if name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
            return Err("Project name contains invalid characters".to_string());
        }

        Ok(ProjectName(name))
    }

    /// Returns the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the ProjectName and returns the inner String
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl std::fmt::Display for ProjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectFile {
    pub name: String,
    pub source: GitSource,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Project {
    pub name: ProjectName,
    pub source: GitSource,
    pub status: ProjectStatus,
    pub last_updated_at: DateTime<Utc>,
}
