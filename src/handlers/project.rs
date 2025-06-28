use axum::{extract::State, Json};
use tracing::{error, instrument};

use crate::errors::GfcError;
use crate::models::project::{Project, ProjectFile};
use crate::repositories::compose_client::ComposeClient;
use crate::repositories::git::GitClient;
use crate::usecases::project::ProjectUsecase;

#[instrument(skip(usecase), name = "get_projects")]
pub async fn get_projects<C, G>(
    State(usecase): State<ProjectUsecase<C, G>>,
) -> Result<Json<Vec<Project>>, GfcError>
where
    C: ComposeClient + Send + Sync,
    G: GitClient + Send + Sync,
{
    usecase
        .list_projects()
        .inspect_err(|e| error!("Project listing failed: {}", e))
        .map(Json)
}

#[instrument(skip(usecase), name = "create_project")]
pub async fn create_project<C, G>(
    State(usecase): State<ProjectUsecase<C, G>>,
    Json(project_file): Json<ProjectFile>,
) -> Result<axum::http::StatusCode, GfcError>
where
    C: ComposeClient + Send + Sync,
    G: GitClient + Send + Sync,
{
    usecase
        .create_project(project_file)
        .inspect_err(|e| error!("Project creation failed: {}", e))
        .map(|_| axum::http::StatusCode::CREATED)
}
