use anyhow::Result;

use crate::models::docker_compose::Container;

pub trait ComposeClient {
    fn list_containers(&self, path: &str) -> Result<Vec<Container>>;
    fn up(&self, path: &str) -> Result<()>;
    fn down(&self, path: &str) -> Result<()>;
}
