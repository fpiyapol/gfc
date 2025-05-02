use axum::{extract::State, Json};

use crate::models::project::{CreateProjectResponse, Project, ProjectFile};
use crate::repositories::compose_client::ComposeClient;
use crate::repositories::git::GitClient;
use crate::usecases::project::ProjectUsecase;

// TODO: implement error handling
pub async fn get_projects<C, G>(State(usecase): State<ProjectUsecase<C, G>>) -> Json<Vec<Project>>
where
    C: ComposeClient + Send + Sync,
    G: GitClient + Send + Sync,
{
    match usecase.list_projects() {
        Ok(projects) => Json(projects),
        Err(_) => Json(vec![]),
    }
}

// perform async process. so cant return the project but just response with acknowledgment, 200
pub async fn create_project<C, G>(
    State(usecase): State<ProjectUsecase<C, G>>,
    Json(project_file): Json<ProjectFile>,
) -> Json<CreateProjectResponse>
where
    C: ComposeClient + Send + Sync,
    G: GitClient + Send + Sync,
{
    match usecase.create_project(project_file) {
        Ok(_) => Json(CreateProjectResponse {
            status: "created".to_string(),
        }),
        Err(e) => Json(CreateProjectResponse {
            status: format!("error: {}", e),
        }),
    }
}
