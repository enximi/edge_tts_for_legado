use std::net::{IpAddr, SocketAddr};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub log: LogConfig,
    pub tts: TtsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
    pub directory: String,
    pub file_name: String,
    pub max_file_size_mb: u64,
    pub max_keep_files: u64,
    pub stdout: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TtsConfig {
    pub voice: String,
    pub retry: TtsRetryConfig,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TtsRetryConfig {
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let settings = config::Config::builder()
            .set_default("server.host", "127.0.0.1")?
            .set_default("server.port", 8000)?
            .set_default("log.directory", "logs")?
            .set_default("log.file_name", "app.log")?
            .set_default("log.max_file_size_mb", 50)?
            .set_default("log.max_keep_files", 10)?
            .set_default("log.stdout", true)?
            .set_default("tts.voice", "zh-CN-XiaoxiaoNeural")?
            .set_default("tts.retry.max_attempts", 3)?
            .set_default("tts.retry.initial_backoff_ms", 1000)?
            .set_default("tts.request_timeout_secs", 30)?
            .add_source(config::File::new("config.toml", config::FileFormat::Toml).required(false))
            .add_source(
                config::Environment::with_prefix("APP")
                    .prefix_separator("__")
                    .separator("__"),
            )
            .build()?;

        let config: Self = settings.try_deserialize()?;
        config.validate()?;
        Ok(config)
    }

    pub fn bind_addr(&self) -> SocketAddr {
        SocketAddr::new(self.server.host, self.server.port)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        validate_not_blank("auth.token", &self.auth.token)?;
        validate_not_blank("log.directory", &self.log.directory)?;
        validate_not_blank("log.file_name", &self.log.file_name)?;
        validate_positive("log.max_file_size_mb", self.log.max_file_size_mb)?;
        validate_positive("log.max_keep_files", self.log.max_keep_files)?;
        validate_positive("tts.retry.max_attempts", self.tts.retry.max_attempts as u64)?;
        validate_positive("tts.request_timeout_secs", self.tts.request_timeout_secs)
    }
}

fn validate_not_blank(name: &str, value: &str) -> Result<(), ConfigError> {
    if value.trim().is_empty() {
        Err(ConfigError::Invalid(format!("{name} must not be empty")))
    } else {
        Ok(())
    }
}

fn validate_positive(name: &str, value: u64) -> Result<(), ConfigError> {
    if value == 0 {
        Err(ConfigError::Invalid(format!(
            "{name} must be greater than 0"
        )))
    } else {
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to build application config: {0}")]
    Build(#[from] config::ConfigError),
    #[error("invalid application config: {0}")]
    Invalid(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::test_config;

    #[test]
    fn rejects_zero_retry_attempts() {
        let mut config = test_config();
        config.tts.retry.max_attempts = 0;

        assert!(matches!(
            config.validate(),
            Err(ConfigError::Invalid(message))
            if message == "tts.retry.max_attempts must be greater than 0"
        ));
    }

    #[test]
    fn rejects_zero_request_timeout() {
        let mut config = test_config();
        config.tts.request_timeout_secs = 0;

        assert!(matches!(
            config.validate(),
            Err(ConfigError::Invalid(message))
            if message == "tts.request_timeout_secs must be greater than 0"
        ));
    }

    #[test]
    fn rejects_empty_log_directory() {
        let mut config = test_config();
        config.log.directory = " ".to_owned();

        assert!(matches!(
            config.validate(),
            Err(ConfigError::Invalid(message))
            if message == "log.directory must not be empty"
        ));
    }

    #[test]
    fn rejects_zero_log_file_size() {
        let mut config = test_config();
        config.log.max_file_size_mb = 0;

        assert!(matches!(
            config.validate(),
            Err(ConfigError::Invalid(message))
            if message == "log.max_file_size_mb must be greater than 0"
        ));
    }

    #[test]
    fn rejects_zero_log_keep_files() {
        let mut config = test_config();
        config.log.max_keep_files = 0;

        assert!(matches!(
            config.validate(),
            Err(ConfigError::Invalid(message))
            if message == "log.max_keep_files must be greater than 0"
        ));
    }

    #[test]
    fn rejects_empty_token() {
        let mut config = test_config();
        config.auth.token = " ".to_owned();

        assert!(matches!(
            config.validate(),
            Err(ConfigError::Invalid(message))
            if message == "auth.token must not be empty"
        ));
    }
}
