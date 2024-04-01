pub mod app_context;
pub mod app_error;
pub mod logger;
pub mod tui;
pub mod tui_backend;
pub mod utils;

pub mod components;
pub mod configs;
pub mod enums;
pub mod run;

use lazy_static::lazy_static;

use crate::{
    app_context::AppContext,
    app_error::AppError,
    configs::{
        config_file::ConfigFile,
        custom::{
            app_custom::AppConfig, keymap_custom::KeymapConfig,
            logger_custom::LoggerConfig, theme_custom::ThemeConfig,
        },
    },
    logger::Logger,
    tui_backend::TuiBackend,
};

lazy_static! {
    pub static ref LOGGER_CONFIG: LoggerConfig = LoggerConfig::get_config();
    pub static ref KEYMAP_CONFIG: KeymapConfig = KeymapConfig::get_config();
    pub static ref APP_CONFIG: AppConfig = AppConfig::get_config();
    pub static ref THEME_CONFIG: ThemeConfig = ThemeConfig::get_config();
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

    let app_config = APP_CONFIG.clone();
    tracing::info!("App config: {:#?}", app_config);
    println!("{:#?}", app_config);

    let theme_config = THEME_CONFIG.clone();
    tracing::info!("Theme config: {:#?}", theme_config);
    println!("{:#?}", theme_config);

    let mut app_context = AppContext::new(app_config, keymap_config)?;
    let mut tui_backend = TuiBackend::new(
        app_context.app_config_ref().frame_rate,
        app_context.app_config_ref().mouse_support,
        app_context.app_config_ref().paste_support,
    )?;

    tracing::info!("Starting main");
    run::run_app(&mut app_context, &mut tui_backend).await?;
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
