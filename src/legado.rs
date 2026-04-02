use serde::{Deserialize, Serialize};

pub const DEFAULT_LEGADO_RATE: i32 = 10;

#[derive(Debug, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    #[serde(default = "default_legado_rate")]
    pub rate: i32,
}

#[derive(Debug, Deserialize)]
pub struct ConfigQuery {
    pub token: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LegadoConfig {
    #[serde(rename = "concurrentRate")]
    pub concurrent_rate: &'static str,
    #[serde(rename = "contentType")]
    pub content_type: &'static str,
    pub header: String,
    pub id: u64,
    #[serde(rename = "loginCheckJs")]
    pub login_check_js: &'static str,
    #[serde(rename = "loginUi")]
    pub login_ui: &'static str,
    #[serde(rename = "loginUrl")]
    pub login_url: &'static str,
    pub name: &'static str,
    pub url: String,
}

pub fn default_legado_rate() -> i32 {
    DEFAULT_LEGADO_RATE
}

pub fn legado_rate_to_percent(rate: i32) -> i32 {
    (rate - DEFAULT_LEGADO_RATE) * 10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_legado_rate_to_edge_percent() {
        assert_eq!(legado_rate_to_percent(10), 0);
        assert_eq!(legado_rate_to_percent(5), -50);
        assert_eq!(legado_rate_to_percent(15), 50);
    }

    #[test]
    fn request_defaults_to_legado_baseline_rate() {
        let payload: TtsRequest =
            serde_json::from_str(r#"{"text":"hello"}"#).expect("failed to deserialize request");

        assert_eq!(payload.rate, DEFAULT_LEGADO_RATE);
    }
}
