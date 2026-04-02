use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    detail: String,
}

#[derive(Debug)]
pub enum AppError {
    Unauthorized(&'static str),
    BadRequest(&'static str),
    Upstream(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, detail) = match self {
            Self::Unauthorized(detail) => (StatusCode::UNAUTHORIZED, detail.to_owned()),
            Self::BadRequest(detail) => (StatusCode::BAD_REQUEST, detail.to_owned()),
            Self::Upstream(detail) => (StatusCode::BAD_GATEWAY, detail),
        };

        (status, Json(ErrorResponse { detail })).into_response()
    }
}
