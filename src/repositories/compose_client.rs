use crate::errors::compose::ComposeError;
use crate::models::docker_compose::Container;

pub trait ComposeClient {
    fn list_containers(&self, path: &str) -> Result<Vec<Container>, ComposeError>;
    fn up(&self, path: &str) -> Result<(), ComposeError>;
    fn down(&self, path: &str) -> Result<(), ComposeError>;
}
