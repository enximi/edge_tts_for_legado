use tokio::signal;

use crate::{app, config::Config, logging, state::AppState};

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = Config::load()?;
    let _log_guard = logging::init(&config.log)?;
    let bind_addr = config.bind_addr();
    let state = AppState::new(config)?;
    let app = app::router(state);
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    tracing::info!(address = %listener.local_addr()?, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    match signal::ctrl_c().await {
        Ok(()) => tracing::info!("shutdown signal received"),
        Err(error) => tracing::error!("failed to install Ctrl+C handler: {error}"),
    }
}
