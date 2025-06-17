use axum::{extract::State, Json};

use crate::errors::GfcError;
use crate::models::project::{Project, ProjectFile};
use crate::repositories::compose_client::ComposeClient;
use crate::repositories::git::GitClient;
use crate::usecases::project::ProjectUsecase;

pub async fn get_projects<C, G>(
    State(usecase): State<ProjectUsecase<C, G>>,
) -> Result<Json<Vec<Project>>, GfcError>
where
    C: ComposeClient + Send + Sync,
    G: GitClient + Send + Sync,
{
    let projects = usecase.list_projects()?;
    Ok(Json(projects))
}

pub async fn create_project<C, G>(
    State(usecase): State<ProjectUsecase<C, G>>,
    Json(project_file): Json<ProjectFile>,
) -> Result<axum::http::StatusCode, GfcError>
where
    C: ComposeClient + Send + Sync,
    G: GitClient + Send + Sync,
{
    usecase.create_project(project_file)?;
    Ok(axum::http::StatusCode::CREATED)
}
