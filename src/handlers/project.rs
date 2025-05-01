use axum::{extract::State, Json};

use crate::models::project::{Project, ProjectFile};
use crate::repositories::compose_client::ComposeClient;
use crate::usecases::project::ProjectUsecase;

// TODO: implement error handling

pub async fn get_projects<C>(State(usecase): State<ProjectUsecase<C>>) -> Json<Vec<Project>>
where
    C: ComposeClient,
{
    match usecase.list_projects() {
        Ok(projects) => Json(projects),
        Err(_) => Json(vec![]),
    }
}

pub async fn create_project<C>(
    State(usecase): State<ProjectUsecase<C>>,
    Json(project_file): Json<ProjectFile>,
) -> Json<Project>
where
    C: ComposeClient,
{
    match usecase.create_project(project_file) {
        Ok(project) => Json(project),
        Err(_) => Json(Project {
            name: "".to_string(),
            path: "".to_string(),
            status: "".to_string(),
        }),
    }
}
