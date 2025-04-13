use anyhow::Result;
use async_trait::async_trait;
use futures_util::Stream;

use crate::models::container_client::{ContainerCreateResponse, ContainerInfo};

#[async_trait]
pub trait ContainerClient {
    async fn create_container(&self, name: &str, image: &str) -> Result<ContainerCreateResponse>;
    async fn create_image(&self, image: &str) -> Result<()>;
    async fn list_containers(&self) -> Result<Vec<ContainerInfo>>;
    async fn remove_container(&self, name: &str) -> Result<()>;
    async fn start_container(&self, name: &str) -> Result<()>;
    async fn stop_container(&self, name: &str) -> Result<()>;
    fn watch_events(
        &self,
    ) -> impl Stream<Item = Result<bollard::models::EventMessage, bollard::errors::Error>>;
}
