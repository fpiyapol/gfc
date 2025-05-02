use anyhow::{Error, Result};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::{extract::State, Json};

use crate::models::project::{Project, ProjectFile};
use crate::models::response::{GenericResponse, ResponseStatus};
use crate::repositories::compose_client::ComposeClient;
use crate::repositories::git::GitClient;
use crate::usecases::project::ProjectUsecase;

pub struct HandlerError(Error);

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        (
            StatusCode::OK,
            Json(GenericResponse::<String>::error(format!(
                "Something went wrong: {}",
                self.0
            ))),
        )
            .into_response()
    }
}

impl<E> From<E> for HandlerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub async fn get_projects<C, G>(
    State(usecase): State<ProjectUsecase<C, G>>,
) -> Result<Json<GenericResponse<Project>>, HandlerError>
where
    C: ComposeClient + Send + Sync,
    G: GitClient + Send + Sync,
{
    Ok(Json(usecase.list_projects()?))
}

pub async fn create_project<C, G>(
    State(usecase): State<ProjectUsecase<C, G>>,
    Json(project_file): Json<ProjectFile>,
) -> Result<Json<GenericResponse<ResponseStatus>>, HandlerError>
where
    C: ComposeClient + Send + Sync,
    G: GitClient + Send + Sync,
{
    Ok(Json(usecase.create_project(project_file)?))
}
