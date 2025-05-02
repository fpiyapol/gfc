use anyhow::Result;

use crate::models::docker_compose::Container;

pub trait ComposeClient {
    type Error;

    fn list_containers(&self, path: &str) -> Result<Vec<Container>, Self::Error>;
    fn up(&self, path: &str) -> Result<(), Self::Error>;
    fn down(&self, path: &str) -> Result<(), Self::Error>;
}
