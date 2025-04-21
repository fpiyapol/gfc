use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ComposeProject {
    pub name: String,
    pub path: String,
    pub status: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct Container {
    pub name: String,
    pub state: ContainerState,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub enum ContainerState {
    Created,
    Dead,
    Exited,
    Paused,
    Removing,
    Restarting,
    Running,
}

impl ContainerState {
    pub fn to_string(&self) -> &str {
        match self {
            ContainerState::Created => "created",
            ContainerState::Dead => "dead",
            ContainerState::Exited => "exited",
            ContainerState::Paused => "paused",
            ContainerState::Removing => "removing",
            ContainerState::Restarting => "restarting",
            ContainerState::Running => "running",
        }
    }
}
