pub mod config;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod usecases;

use anyhow::Result;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

use crate::config::Config;
use crate::handlers::project::{create_project, get_projects};
use crate::repositories::docker_compose_client::DockerComposeClient;
use crate::repositories::git::GitClientImpl;
use crate::usecases::project::ProjectUsecase;

pub async fn init() -> Result<()> {
    let config = load_config("config/default.yaml")?;
    let project_usecase = create_project_usecase(&config)?;
    let app = build_app(project_usecase);

    let address = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("Server running at http://{}", address);

    axum::serve(listener, app).await?;
    Ok(())
}

fn load_config<P>(path: P) -> Result<Config>
where
    P: AsRef<std::path::Path>,
{
    let config = Config::from_file(path)?;
    Ok(config)
}

fn create_project_usecase(
    config: &Config,
) -> Result<ProjectUsecase<DockerComposeClient, GitClientImpl>> {
    let docker_compose_client = Arc::new(DockerComposeClient::new()?);
    let git_client = Arc::new(GitClientImpl);

    Ok(ProjectUsecase::new(
        docker_compose_client,
        git_client,
        config.resources.clone(),
    ))
}

fn build_app(project_usecase: ProjectUsecase<DockerComposeClient, GitClientImpl>) -> Router {
    Router::new()
        .route("/projects", get(get_projects))
        .route("/projects", post(create_project))
        .with_state(project_usecase)
}
