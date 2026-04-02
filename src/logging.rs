use std::fs;

use logroller::{LogRollerBuilder, Rotation, RotationSize};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::config::LogConfig;

pub fn init(config: &LogConfig) -> Result<WorkerGuard, Box<dyn std::error::Error>> {
    fs::create_dir_all(&config.directory)?;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let file_appender = LogRollerBuilder::new(&config.directory, &config.file_name)
        .rotation(Rotation::SizeBased(RotationSize::MB(
            config.max_file_size_mb,
        )))
        .max_keep_files(config.max_keep_files)
        .build()?;
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(filter)
        .with(config.stdout.then(fmt::layer))
        .with(
            fmt::layer()
                .with_ansi(false)
                .with_writer(file_writer)
                .with_target(true),
        )
        .try_init()?;

    Ok(guard)
}
