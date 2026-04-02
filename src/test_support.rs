use crate::{
    config::{AuthConfig, Config, LogConfig, ServerConfig, TtsConfig, TtsRetryConfig},
    state::AppState,
};

pub fn test_config() -> Config {
    Config {
        server: ServerConfig {
            host: "127.0.0.1".parse().expect("invalid test host"),
            port: 8000,
        },
        auth: AuthConfig {
            token: "secret".to_owned(),
        },
        log: LogConfig {
            directory: "logs".to_owned(),
            file_name: "app.log".to_owned(),
            max_file_size_mb: 50,
            max_keep_files: 10,
            stdout: true,
        },
        tts: TtsConfig {
            voice: "zh-CN-XiaoxiaoNeural".to_owned(),
            retry: TtsRetryConfig {
                max_attempts: 3,
                initial_backoff_ms: 1000,
            },
            request_timeout_secs: 30,
        },
    }
}

pub fn test_state() -> AppState {
    AppState::new(test_config()).expect("failed to create app state")
}
