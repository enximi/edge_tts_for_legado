use axum::http::HeaderMap;
use tracing::{info, warn};

use crate::{config::Config, error::AppError};

pub fn verify_query_token(config: &Config, token: Option<&str>) -> Result<(), AppError> {
    if token == Some(config.auth.token.as_str()) {
        return Ok(());
    }

    match token {
        Some(_) => warn!("rejected config request with invalid token"),
        None => info!("rejected config request with missing token"),
    }

    Err(AppError::Unauthorized("Invalid Token"))
}

pub fn verify_header_token(config: &Config, headers: &HeaderMap) -> Result<(), AppError> {
    let Some(auth_header) = header_value(headers, "authorization") else {
        info!("rejected tts request with missing authorization header");
        return Err(AppError::Unauthorized("Invalid Token"));
    };

    let mut parts = auth_header.splitn(2, ' ');
    let scheme = parts.next().unwrap_or_default();
    let token = parts.next().unwrap_or_default();

    if scheme.eq_ignore_ascii_case("bearer") && token == config.auth.token {
        Ok(())
    } else {
        warn!("rejected tts request with invalid bearer token");
        Err(AppError::Unauthorized("Invalid Token"))
    }
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::*;
    use crate::test_support::test_config;

    #[test]
    fn accepts_matching_query_token() {
        assert!(verify_query_token(&test_config(), Some("secret")).is_ok());
    }

    #[test]
    fn rejects_missing_query_token() {
        assert!(matches!(
            verify_query_token(&test_config(), None),
            Err(AppError::Unauthorized("Invalid Token"))
        ));
    }

    #[test]
    fn accepts_matching_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer secret"));

        assert!(verify_header_token(&test_config(), &headers).is_ok());
    }

    #[test]
    fn rejects_invalid_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer nope"));

        assert!(matches!(
            verify_header_token(&test_config(), &headers),
            Err(AppError::Unauthorized("Invalid Token"))
        ));
    }
}
