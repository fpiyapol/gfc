pub mod handlers;
pub mod models;
pub mod repositories;
pub mod usecases;

use anyhow::Result;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

use crate::handlers::project::{create_project, get_projects};
use crate::repositories::docker_compose_client::DockerComposeClient;
use crate::repositories::git::GitClientImpl;
use crate::usecases::project::ProjectUsecase;

pub async fn init() -> Result<()> {
    let docker_compose_client = Arc::new(DockerComposeClient::new()?);
    let git_client = Arc::new(GitClientImpl);

    let project_usecase = ProjectUsecase::new(docker_compose_client, git_client);

    let app = Router::new()
        .route("/projects", get(get_projects))
        .route("/projects", post(create_project))
        .with_state(project_usecase);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
