use std::time::Duration;

use edge_tts_rust::{Boundary, EdgeTtsClient, SpeakOptions};
use tokio::time::{Instant, sleep, timeout};
use tracing::{error, info};

use crate::{
    config::{TtsConfig, TtsRetryConfig},
    error::AppError,
    legado::{TtsRequest, legado_rate_to_percent},
};

#[derive(Debug, Clone)]
pub struct TtsService {
    client: EdgeTtsClient,
    config: TtsConfig,
}

impl TtsService {
    pub fn new(config: TtsConfig) -> Result<Self, edge_tts_rust::Error> {
        Ok(Self {
            client: EdgeTtsClient::new()?,
            config,
        })
    }

    pub async fn synthesize(&self, payload: TtsRequest) -> Result<Vec<u8>, AppError> {
        Self::validate_request(&payload)?;

        let options = Self::build_speak_options(&self.config, payload.rate);
        let retry = self.config.retry.clone();
        let text = payload.text;

        self.synthesize_with_retry(text, options, retry).await
    }

    fn validate_request(payload: &TtsRequest) -> Result<(), AppError> {
        if payload.text.trim().is_empty() {
            Err(AppError::BadRequest("Text is empty"))
        } else {
            Ok(())
        }
    }

    fn build_speak_options(config: &TtsConfig, rate: i32) -> SpeakOptions {
        SpeakOptions {
            voice: config.voice.clone(),
            rate: format!("{:+}%", legado_rate_to_percent(rate)),
            volume: "+0%".to_owned(),
            pitch: "+0Hz".to_owned(),
            boundary: Boundary::Sentence,
        }
    }

    async fn synthesize_with_retry(
        &self,
        text: String,
        options: SpeakOptions,
        retry: TtsRetryConfig,
    ) -> Result<Vec<u8>, AppError> {
        let mut last_error = None::<String>;
        let total_timeout = Duration::from_secs(self.config.request_timeout_secs);
        let deadline = Instant::now() + total_timeout;

        for attempt in 1..=retry.max_attempts {
            let attempt_timeout = remaining_attempt_budget(deadline, &retry, attempt)
                .ok_or_else(|| upstream_timeout_error(self.config.request_timeout_secs))?;

            match timeout(
                attempt_timeout,
                self.client.synthesize(text.clone(), options.clone()),
            )
            .await
            {
                Ok(Ok(result)) => return Ok(result.audio),
                Ok(Err(error)) => {
                    info!(
                        "edge tts synthesize attempt {attempt}/{} failed: {error}",
                        retry.max_attempts
                    );
                    last_error = Some(error.to_string());
                }
                Err(_) => {
                    info!(
                        "edge tts synthesize attempt {attempt}/{} timed out after {:?}",
                        retry.max_attempts, attempt_timeout
                    );
                    last_error = Some(format!(
                        "attempt {attempt} timed out after {:?}",
                        attempt_timeout
                    ));
                }
            }

            if attempt < retry.max_attempts {
                let backoff = backoff_for_attempt(retry.initial_backoff_ms, attempt);
                let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                    error!(
                        timeout_secs = self.config.request_timeout_secs,
                        "edge tts synthesize exceeded total retry budget"
                    );
                    return Err(upstream_timeout_error(self.config.request_timeout_secs));
                };

                if backoff >= remaining {
                    error!(
                        timeout_secs = self.config.request_timeout_secs,
                        "edge tts synthesize ran out of time before next retry"
                    );
                    return Err(upstream_timeout_error(self.config.request_timeout_secs));
                }

                sleep(backoff).await;
            }
        }

        match last_error {
            Some(error) => {
                error!(
                    attempts = retry.max_attempts,
                    "edge tts synthesize failed after all retries: {error}"
                );
                Err(AppError::Upstream(format!(
                    "TTS upstream error after {} attempts: {error}",
                    retry.max_attempts
                )))
            }
            None => {
                error!("edge tts synthesize failed without an explicit upstream error");
                Err(AppError::Upstream(
                    "TTS upstream failed without returning an explicit error".to_owned(),
                ))
            }
        }
    }
}

fn remaining_attempt_budget(
    deadline: Instant,
    retry: &TtsRetryConfig,
    attempt: u32,
) -> Option<Duration> {
    let remaining = deadline.checked_duration_since(Instant::now())?;
    let attempts_left = retry.max_attempts.checked_sub(attempt)?.saturating_add(1);
    let reserved_backoff = total_future_backoff(retry.initial_backoff_ms, attempts_left - 1);
    let available = remaining.checked_sub(reserved_backoff)?;

    if available.is_zero() {
        None
    } else {
        Some(available / attempts_left)
    }
}

fn total_future_backoff(initial_backoff_ms: u64, retries_left_after_attempt: u32) -> Duration {
    let mut total = Duration::ZERO;

    for offset in 1..=retries_left_after_attempt {
        total = total.saturating_add(backoff_for_attempt(initial_backoff_ms, offset));
    }

    total
}

fn upstream_timeout_error(request_timeout_secs: u64) -> AppError {
    AppError::Upstream(format!(
        "TTS upstream timed out after {} seconds",
        request_timeout_secs
    ))
}

fn backoff_for_attempt(initial_backoff_ms: u64, attempt: u32) -> Duration {
    let multiplier = 1u64 << attempt.saturating_sub(1);
    Duration::from_millis(initial_backoff_ms.saturating_mul(multiplier))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{TtsConfig, TtsRetryConfig},
        legado::{DEFAULT_LEGADO_RATE, TtsRequest},
    };

    #[test]
    fn builds_expected_voice_and_rate() {
        let config = TtsConfig {
            voice: "zh-CN-XiaoxiaoNeural".to_owned(),
            retry: TtsRetryConfig {
                max_attempts: 3,
                initial_backoff_ms: 1000,
            },
            request_timeout_secs: 30,
        };
        let options = TtsService::build_speak_options(&config, DEFAULT_LEGADO_RATE + 5);

        assert_eq!(options.voice, "zh-CN-XiaoxiaoNeural");
        assert_eq!(options.rate, "+50%");
    }

    #[test]
    fn rejects_empty_text() {
        let result = TtsService::validate_request(&TtsRequest {
            text: "   ".to_owned(),
            rate: DEFAULT_LEGADO_RATE,
        });

        assert!(matches!(result, Err(AppError::BadRequest("Text is empty"))));
    }

    #[test]
    fn uses_exponential_backoff() {
        assert_eq!(backoff_for_attempt(1000, 1), Duration::from_secs(1));
        assert_eq!(backoff_for_attempt(1000, 2), Duration::from_secs(2));
        assert_eq!(backoff_for_attempt(1000, 3), Duration::from_secs(4));
    }

    #[test]
    fn sums_future_backoff() {
        assert_eq!(total_future_backoff(1000, 0), Duration::ZERO);
        assert_eq!(total_future_backoff(1000, 1), Duration::from_secs(1));
        assert_eq!(total_future_backoff(1000, 2), Duration::from_secs(3));
    }

    #[tokio::test]
    async fn allocates_attempt_budget_without_consuming_retry_backoff() {
        let retry = TtsRetryConfig {
            max_attempts: 3,
            initial_backoff_ms: 1000,
        };
        let deadline = Instant::now() + Duration::from_secs(30);

        let budget = remaining_attempt_budget(deadline, &retry, 1).expect("missing budget");

        assert!(budget >= Duration::from_secs(8));
        assert!(budget <= Duration::from_secs(9));
    }
}
