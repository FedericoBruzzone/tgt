pub mod app;
pub mod app_error;
pub mod logger;
pub mod tui;
pub mod tui_backend;
pub mod utils;

pub mod components;
pub mod configs;
pub mod enums;

use lazy_static::lazy_static;

use crate::{
    app_error::AppError,
    configs::{
        config_file::ConfigFile,
        custom::{keymap_custom::KeymapConfig, logger_custom::LoggerConfig},
    },
    logger::Logger,
};

lazy_static! {
    pub static ref LOGGER_CONFIG: LoggerConfig = LoggerConfig::get_config();
    pub static ref KEYMAP_CONFIG: KeymapConfig = KeymapConfig::get_config();
}

/// The main entry point for the application.
/// This function initializes the application and runs the main event loop.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
async fn tokio_main() -> Result<(), AppError> {
    let logger = Logger::from_config(LOGGER_CONFIG.clone());
    logger.init();
    tracing::info!("Logger initialized with config: {:#?}", logger);
    println!("{:#?}", logger);

    let keymap_config = KEYMAP_CONFIG.clone();
    tracing::info!("Keymap config: {:#?}", keymap_config);
    println!("{:#?}", keymap_config);

    let mut app = app::App::new()?; //.with_frame_rate(60.0);
    tracing::info!("Starting main");
    app.run().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    if let Err(e) = tokio_main().await {
        tracing::error!("Something went wrong: {}", e);
        Err(e)
    } else {
        Ok(())
    }
}
