use axum::{extract::State, Json};

use crate::models::docker_compose::ComposeProject;
use crate::repositories::compose_client::ComposeClient;
use crate::usecases::compose::ComposeUsecase;

pub async fn get_projects<C>(State(usecase): State<ComposeUsecase<C>>) -> Json<Vec<ComposeProject>>
where
    C: ComposeClient,
{
    match usecase.list_compose_projects().await {
        Ok(projects) => Json(projects),
        Err(_) => Json(vec![]),
    }
}
