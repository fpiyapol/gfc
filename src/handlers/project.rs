use axum::{extract::State, Json};

use crate::models::project::Project;
use crate::repositories::compose_client::ComposeClient;
use crate::usecases::project::ProjectUsecase;

pub async fn get_projects<C>(State(usecase): State<ProjectUsecase<C>>) -> Json<Vec<Project>>
where
    C: ComposeClient,
{
    match usecase.list_projects() {
        Ok(projects) => Json(projects),
        Err(_) => Json(vec![]),
    }
}
