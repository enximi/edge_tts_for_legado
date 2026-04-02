use std::sync::Arc;

use crate::{config::Config, services::tts::TtsService};

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub tts_service: Arc<TtsService>,
}

impl AppState {
    pub fn new(config: Config) -> Result<Self, edge_tts_rust::Error> {
        let tts_service = TtsService::new(config.tts.clone())?;

        Ok(Self {
            config: Arc::new(config),
            tts_service: Arc::new(tts_service),
        })
    }
}
