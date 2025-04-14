use axum::{extract::State, Json};

use crate::models::docker_compose::ComposeProject;
use crate::usecases::compose::ComposeUsecase;

pub async fn get_projects(State(usecase): State<ComposeUsecase>) -> Json<Vec<ComposeProject>> {
    match usecase.list_compose_projects().await {
        Ok(projects) => Json(projects),
        Err(_) => Json(vec![]),
    }
}
