use axum::{
    Json,
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::Response,
};
use tracing::debug;

use crate::{
    error::AppError, http::auth::verify_header_token, legado::TtsRequest, state::AppState,
};

pub async fn tts_endpoint(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TtsRequest>,
) -> Result<Response, AppError> {
    verify_header_token(state.config.as_ref(), &headers)?;
    debug!(
        rate = payload.rate,
        text_len = payload.text.chars().count(),
        "accepted tts request"
    );
    let audio = state.tts_service.synthesize(payload).await?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "audio/mpeg")
        .body(Body::from(audio))
        .map_err(|error| AppError::Upstream(format!("Failed to build audio response: {error}")))
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::*;
    use crate::test_support::test_state;

    #[tokio::test]
    async fn rejects_empty_text_before_upstream_call() {
        let state = test_state();
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer secret"));

        let result = tts_endpoint(
            State(state),
            headers,
            Json(TtsRequest {
                text: "   ".to_owned(),
                rate: crate::legado::DEFAULT_LEGADO_RATE,
            }),
        )
        .await;

        assert!(matches!(result, Err(AppError::BadRequest("Text is empty"))));
    }

    #[tokio::test]
    async fn rejects_invalid_bearer_token() {
        let state = test_state();
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer nope"));

        let result = tts_endpoint(
            State(state),
            headers,
            Json(TtsRequest {
                text: "hello".to_owned(),
                rate: crate::legado::DEFAULT_LEGADO_RATE,
            }),
        )
        .await;

        assert!(matches!(
            result,
            Err(AppError::Unauthorized("Invalid Token"))
        ));
    }
}
