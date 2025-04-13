pub mod models;
pub mod repositories;
pub mod usecases;

use anyhow::Result;
use models::container_client::ContainerEvent;
use tokio::sync::mpsc;

use crate::repositories::docker_client::DockerClient;
use crate::usecases::container_watcher::ContainerWatcher;

pub async fn start_service() -> Result<()> {
    let docker = DockerClient::new()?;
    let watcher = ContainerWatcher::new(docker);

    let (tx, mut rx) = mpsc::channel::<ContainerEvent>(100);

    tokio::spawn(async move {
        watcher.run(tx).await;
    });

    // Listen for incoming events
    while let Some(event) = rx.recv().await {
        println!("Received event: {:#?}", event);
    }

    Ok(())
}
