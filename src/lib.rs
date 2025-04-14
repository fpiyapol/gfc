pub mod handlers;
pub mod models;
pub mod repositories;
pub mod usecases;

use anyhow::Result;
use axum::{routing::get, Router};
use handlers::compose::get_projects;

use crate::repositories::docker_compose_client::DockerComposeClient;
use crate::usecases::compose::ComposeUsecase;

pub async fn start() -> Result<()> {
    let docker_compose_client = DockerComposeClient::new()?;
    let compose_usecase = ComposeUsecase::new(docker_compose_client);

    let app = Router::new()
        .route("/projects", get(get_projects))
        .with_state(compose_usecase);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
