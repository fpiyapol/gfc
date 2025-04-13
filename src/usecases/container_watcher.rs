use std::collections::HashMap;

use futures_util::StreamExt;
use tokio::sync::mpsc;

use crate::models::container_client::{ContainerEvent, ContainerEventAction};
use crate::repositories::{container_client::ContainerClient, docker_client::DockerClient};

pub struct ContainerWatcher {
    pub docker: DockerClient,
}

impl ContainerWatcher {
    pub fn new(docker: DockerClient) -> ContainerWatcher {
        Self { docker }
    }

    /// Starts watching Docker events and sends filtered Compose service events
    pub async fn run(&self, tx: mpsc::Sender<ContainerEvent>) {
        let mut filters = HashMap::new();
        let project_name = "test";
        filters.insert(
            "label".to_string(),
            vec![format!("com.docker.compose.project={}", project_name)],
        );

        let mut stream = self.docker.watch_events();

        while let Some(Ok(event)) = stream.next().await {
            if let Some(container_event) = parse_event(event) {
                if let Err(err) = tx.send(container_event).await {
                    eprintln!("[ContainerWatcher] Failed to send event: {}", err);
                }
            }
        }
    }
}

fn parse_event(event: bollard::models::EventMessage) -> Option<ContainerEvent> {
    let actor = event.actor?;
    let container_id = actor.id?;

    let attributes = actor.attributes?;
    let container_name = attributes.get("name")?.clone();

    let action = event.action?;
    let action = match action.as_str() {
        "create" => ContainerEventAction::Create,
        "start" => ContainerEventAction::Start,
        "stop" => ContainerEventAction::Stop,
        "restart" => ContainerEventAction::Restart,
        "pause" => ContainerEventAction::Pause,
        "unpause" => ContainerEventAction::Unpause,
        "die" => ContainerEventAction::Die,
        "destroy" => ContainerEventAction::Destroy,
        "kill" => ContainerEventAction::Kill,
        "oom" => ContainerEventAction::Oom,
        "health_status: healthy" => ContainerEventAction::HealthStatus,
        "health_status: unhealthy" => ContainerEventAction::HealthStatus,
        _ => return None,
    };

    Some(ContainerEvent {
        container_id,
        container_name,
        action,
    })
}
