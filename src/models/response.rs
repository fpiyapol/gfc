use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GenericResponse<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub results: Option<Vec<T>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ResponseStatus {
    Success,
    Error(String),
}

impl<T> GenericResponse<T> {
    pub fn result(result: T) -> Self {
        GenericResponse {
            result: Some(result),
            results: None,
            error: None,
        }
    }

    pub fn results(results: Vec<T>) -> Self {
        GenericResponse {
            result: None,
            results: Some(results),
            error: None,
        }
    }

    pub fn error(error: String) -> Self {
        GenericResponse {
            result: None,
            results: None,
            error: Some(error),
        }
    }
}

impl<T> IntoResponse for GenericResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
