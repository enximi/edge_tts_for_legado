use axum::http::HeaderMap;

use crate::{config::Config, http::origin::request_origin, legado::LegadoConfig};

const LEGADO_CONFIG_ID: u64 = 1735914000000;
const LEGADO_CONFIG_NAME: &str = "EdgeTTS for Legado";
const LEGADO_REQUEST_BODY_TEMPLATE: &str =
    r#"{\"text\": \"{{speakText}}\", \"rate\": {{speakSpeed}}}"#;

pub fn build_legado_config(config: &Config, headers: &HeaderMap) -> LegadoConfig {
    let tts_url = format!("{}/tts", request_origin(headers));
    let request_options = build_request_options();
    let request_headers = build_request_headers(&config.auth.token);

    LegadoConfig {
        concurrent_rate: "1000",
        content_type: "audio/mpeg",
        header: request_headers,
        id: LEGADO_CONFIG_ID,
        login_check_js: "",
        login_ui: "",
        login_url: "",
        name: LEGADO_CONFIG_NAME,
        url: format!("{tts_url},{request_options}"),
    }
}

fn build_request_options() -> String {
    format!(
        "{{\"method\": \"POST\", \"body\": \"{}\"}}",
        LEGADO_REQUEST_BODY_TEMPLATE
    )
}

fn build_request_headers(token: &str) -> String {
    format!(
        "{{\n\"Content-Type\": \"application/json\",\n\"Authorization\": \"Bearer {token}\"\n}}"
    )
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::*;
    use crate::test_support::test_config;

    #[test]
    fn builds_legado_config_with_current_origin() {
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("example.com"));
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https"));

        let config = build_legado_config(&test_config(), &headers);

        assert_eq!(config.content_type, "audio/mpeg");
        assert!(config.url.starts_with("https://example.com/tts,"));
        assert!(config.header.contains("Bearer secret"));
        assert!(config.url.contains("\"method\": \"POST\""));
        assert_eq!(
            config.url,
            "https://example.com/tts,{\"method\": \"POST\", \"body\": \"{\\\"text\\\": \\\"{{speakText}}\\\", \\\"rate\\\": {{speakSpeed}}}\"}"
        );
        assert_eq!(
            config.header,
            "{\n\"Content-Type\": \"application/json\",\n\"Authorization\": \"Bearer secret\"\n}"
        );
    }
}
