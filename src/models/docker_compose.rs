use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ComposeProject {
    pub name: String,
    pub path: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct ServiceStatus {
    pub name: String,
    pub state: ServiceState,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum ServiceState {
    Created,
    Dead,
    Exited,
    Paused,
    Removing,
    Restarting,
    Running,
}

impl ServiceState {
    pub fn to_string(&self) -> &str {
        match self {
            ServiceState::Created => "created",
            ServiceState::Dead => "dead",
            ServiceState::Exited => "exited",
            ServiceState::Paused => "paused",
            ServiceState::Removing => "removing",
            ServiceState::Restarting => "restarting",
            ServiceState::Running => "running",
        }
    }
}
