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

use crate::{
    app_context::AppContext,
    app_error::AppError,
    configs::{
        config_file::ConfigFile,
        custom::{
            app_custom::AppConfig, keymap_custom::KeymapConfig, logger_custom::LoggerConfig,
            palette_custom::PaletteConfig, theme_custom::ThemeConfig,
        },
    },
    logger::Logger,
    tui::Tui,
    tui_backend::TuiBackend,
};
use lazy_static::lazy_static;
use std::{
    panic::{set_hook, take_hook},
    sync::Arc,
};

lazy_static! {
    pub static ref LOGGER_CONFIG: LoggerConfig = LoggerConfig::get_config();
    pub static ref KEYMAP_CONFIG: KeymapConfig = KeymapConfig::get_config();
    pub static ref APP_CONFIG: AppConfig = AppConfig::get_config();
    pub static ref PALETTE_CONFIG: PaletteConfig = PaletteConfig::get_config();
    pub static ref THEME_CONFIG: ThemeConfig = ThemeConfig::get_config();
}

/// The main entry point for the application.
/// This function initializes the application and runs the main event loop.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
async fn tokio_main() -> Result<(), AppError> {
    tracing::info!("Starting tokio main");

    lazy_static::initialize(&LOGGER_CONFIG);
    lazy_static::initialize(&KEYMAP_CONFIG);
    lazy_static::initialize(&APP_CONFIG);
    lazy_static::initialize(&PALETTE_CONFIG);
    lazy_static::initialize(&THEME_CONFIG);

    let logger = Logger::from_config(LOGGER_CONFIG.clone());
    logger.init();
    tracing::info!("Logger initialized with config: {:#?}", logger);

    let keymap_config = KEYMAP_CONFIG.clone();
    tracing::info!("Keymap config: {:#?}", keymap_config);

    let app_config = APP_CONFIG.clone();
    tracing::info!("App config: {:#?}", app_config);

    let palette_config = PALETTE_CONFIG.clone();
    tracing::info!("Palette config: {:#?}", palette_config);

    let theme_config = THEME_CONFIG.clone();
    tracing::info!("Theme config: {:#?}", theme_config);

    let app_context = Arc::new(AppContext::new(
        app_config,
        keymap_config,
        theme_config,
        palette_config,
    )?);
    let mut tui_backend = TuiBackend::new(app_context.clone())?;
    let mut tui = Tui::new(app_context.clone());
    init_panic_hook(tui_backend.mouse, tui_backend.paste);

    run::run_app(app_context.clone(), &mut tui, &mut tui_backend).await?;
    Ok(())
}

/// Initialize the panic hook to exit the `TuiBackend` and log the panic stack
/// backtrace.
///
/// # Arguments
/// * `mouse` - A boolean flag that represents whether the mouse was enabled
///   during the execution and need to be disabled.
/// * `paste` - A boolean flag that represents whether the paste mode was
///   enabled during the execution and need to be disabled.
fn init_panic_hook(mouse: bool, paste: bool) {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        TuiBackend::force_exit(mouse, paste).unwrap();
        let backtrace = std::backtrace::Backtrace::capture();
        tracing::error!("{}\nstack backtrace:\n{}", panic_info, backtrace);
        original_hook(panic_info); // comment to hide the stacktrace in stdout
    }));
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
