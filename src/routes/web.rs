use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

const INDEX_HTML: &str = include_str!("../../assets/index.html");

pub async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub async fn favicon() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}
