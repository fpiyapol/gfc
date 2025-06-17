use axum::http::StatusCode;
use axum::{response::IntoResponse, Json};
use serde::Serialize;

use crate::errors::{GfcError, HasErrorCode};

#[derive(Serialize)]
struct Problem<'a> {
    title: &'a str,
    detail: String,
    code: &'a str,
}

fn map_error(err: &GfcError) -> StatusCode {
    use GfcError::*;
    match err {
        Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
        Git(_) => StatusCode::BAD_GATEWAY,
        Compose(_) => StatusCode::BAD_GATEWAY,
        Project(_) => StatusCode::UNPROCESSABLE_ENTITY,
        Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

impl IntoResponse for GfcError {
    fn into_response(self) -> axum::response::Response {
        let status = map_error(&self);
        let problem = Problem {
            title: status.canonical_reason().unwrap_or("error"),
            detail: self.to_string(),
            code: self.error_code(),
        };
        (status, Json(problem)).into_response()
    }
}
