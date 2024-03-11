pub mod app;
pub mod app_error;
pub mod logger;
pub mod tui;
pub mod tui_backend;
pub mod utils;

pub mod components;
pub mod configs;
pub mod enums;
pub mod traits;

use lazy_static::lazy_static;

use crate::{
        app_error::AppError,
        configs::{config_dir_hierarchy::ConfigFile, custom::logger_custom::LoggerConfig},
        logger::Logger,
};

lazy_static! {
        pub static ref LOGGER_CONFIG: LoggerConfig = LoggerConfig::get_config();
}

/// The main entry point for the application.
/// This function initializes the application and runs the main event loop.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
async fn tokio_main() -> Result<(), AppError> {
        let logger = Logger::from_config(LOGGER_CONFIG.clone());
        logger.init();
        println!("{:?}", logger);
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
