pub mod action;
pub mod app_context;
pub mod app_error;
pub mod cli;
pub mod component_name;
pub mod event;
pub mod logger;
pub mod tui;
pub mod tui_backend;
pub mod utils;

// Folders
pub mod components;
pub mod configs;
pub mod run;
pub mod tg;

use crate::app_context::AppContext;
use crate::app_error::AppError;
use crate::configs::{
    config_file::ConfigFile,
    custom::{
        app_custom::AppConfig, keymap_custom::KeymapConfig, logger_custom::LoggerConfig,
        palette_custom::PaletteConfig, theme_custom::ThemeConfig,
    },
};
use crate::logger::Logger;
use crate::tg::{tg_backend::TgBackend, tg_context::TgContext};
use crate::tui::Tui;
use crate::tui_backend::TuiBackend;
use clap::Parser;
use configs::custom::telegram_custom::TelegramConfig;
use lazy_static::lazy_static;
use std::panic::{set_hook, take_hook};
use std::sync::Arc;

lazy_static! {
    pub static ref LOGGER_CONFIG: LoggerConfig = LoggerConfig::get_config();
    pub static ref KEYMAP_CONFIG: KeymapConfig = KeymapConfig::get_config();
    pub static ref APP_CONFIG: AppConfig = AppConfig::get_config();
    pub static ref PALETTE_CONFIG: PaletteConfig = PaletteConfig::get_config();
    pub static ref THEME_CONFIG: ThemeConfig = ThemeConfig::get_config();
    pub static ref TELEGRAM_CONFIG: TelegramConfig = TelegramConfig::get_config();
}

/// The main entry point for the application.
/// This function initializes the application and runs the main event loop.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
async fn tokio_main() -> Result<(), AppError<()>> {
    tracing::info!("Starting tokio main");

    let cli_args = cli::CliArgs::parse();
    tracing::info!("Parsed CLI arguments: {:?}", cli_args);

    // Initialize the lazy static variables
    // This is done to ensure that the configuration files are read only once
    // and the values are shared across the application.
    lazy_static::initialize(&LOGGER_CONFIG);
    lazy_static::initialize(&KEYMAP_CONFIG);
    lazy_static::initialize(&APP_CONFIG);
    lazy_static::initialize(&PALETTE_CONFIG);
    lazy_static::initialize(&THEME_CONFIG);
    lazy_static::initialize(&TELEGRAM_CONFIG);

    let logger = Logger::from_config(LOGGER_CONFIG.clone());
    logger.init();
    tracing::info!("Logger initialized with config: {:?}", logger);

    let keymap_config = KEYMAP_CONFIG.clone();
    tracing::info!("Keymap config: {:?}", keymap_config);

    let app_config = APP_CONFIG.clone();
    tracing::info!("App config: {:?}", app_config);

    let palette_config = PALETTE_CONFIG.clone();
    tracing::info!("Palette config: {:?}", palette_config);

    let theme_config = THEME_CONFIG.clone();
    tracing::info!("Theme config: {:?}", theme_config);

    let mut telegram_config = TELEGRAM_CONFIG.clone();
    tracing::info!("Telegram config: {:?}", telegram_config);
    // This is used to disable the message database when running the application as a CLI.
    // This is done to avoid that deleting a message other application in
    // a chats causes the `--send-message` to resend the messages that were deleted.
    telegram_config.use_message_database = std::env::args().count() <= 1;

    let tg_context = TgContext::default();
    tracing::info!("Telegram context: {:?}", tg_context);
    let app_context = Arc::new(AppContext::new(
        app_config,
        keymap_config,
        theme_config,
        palette_config,
        telegram_config,
        tg_context,
        cli_args,
    )?);
    tracing::info!("App context: {:?}", app_context);

    let mut tui_backend = TuiBackend::new(Arc::clone(&app_context))?;
    tracing::info!("Tui backend initialized");
    init_panic_hook(tui_backend.mouse, tui_backend.paste);
    let mut tui = Tui::new(Arc::clone(&app_context));
    tracing::info!("Tui initialized");
    let mut tg_backend = TgBackend::new(Arc::clone(&app_context)).unwrap();
    tracing::info!("Telegram backend initialized");

    match run::run_app(
        Arc::clone(&app_context),
        &mut tui,
        &mut tui_backend,
        &mut tg_backend,
    )
    .await
    {
        Ok(_) => {
            tracing::info!("Application exited successfully");
            std::process::exit(0);
        }
        Err(e) => {
            tracing::error!("Application exited with error: {}", e);
            std::process::exit(1);
        }
    }
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
        // Intentionally ignore errors here since we're already in a panic
        TuiBackend::force_exit(mouse, paste).unwrap();
        let backtrace = std::backtrace::Backtrace::capture();
        tracing::error!("{}\nstack backtrace:\n{}", panic_info, backtrace);
        original_hook(panic_info); // comment to hide the stacktrace in stdout
    }));
}

#[tokio::main]
async fn main() -> Result<(), AppError<()>> {
    if let Err(e) = tokio_main().await {
        tracing::error!("Something went wrong: {}", e);
        Err(e)
    } else {
        Ok(())
    }
}
