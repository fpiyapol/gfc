use serde::Deserialize;

#[derive(PartialEq, Eq, Debug, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub state: ServiceState,
}

#[derive(PartialEq, Eq, Debug, Deserialize)]
pub enum ServiceState {
    Paused,
    Restarting,
    Removing,
    Running,
    Dead,
    Created,
    Exited,
}

impl ServiceState {
    pub fn to_string(&self) -> &str {
        match self {
            ServiceState::Paused => "paused",
            ServiceState::Restarting => "restarting",
            ServiceState::Removing => "removing",
            ServiceState::Running => "running",
            ServiceState::Dead => "dead",
            ServiceState::Created => "created",
            ServiceState::Exited => "exited",
        }
    }
}
