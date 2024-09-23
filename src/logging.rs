use std::path::PathBuf;

use color_eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, prelude::*};

pub fn init() -> Result<()> {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    std::fs::create_dir_all(directory.clone())?;
    let log_path = directory.join("ratatui.log");
    let log_file = std::fs::File::create(log_path)?;
    // If the `RUST_LOG` environment variable is set, use that as the default, otherwise use the
    // value of the `LOG_ENV` environment variable. If the `LOG_ENV` environment variable contains
    // errors, then this will return an error.

    let file_subscriber = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false);
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .try_init()?;
    Ok(())
}
