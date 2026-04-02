use axum::{
    Router,
    routing::{get, post},
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::{
    routes::{
        config::config_endpoint,
        tts::tts_endpoint,
        web::{favicon, index},
    },
    state::AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/config", get(config_endpoint))
        .route("/tts", post(tts_endpoint))
        .route("/favicon.ico", get(favicon).head(favicon))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
                .on_response(DefaultOnResponse::new().level(Level::DEBUG))
                .on_failure(DefaultOnFailure::new().level(Level::WARN)),
        )
        .with_state(state)
}
