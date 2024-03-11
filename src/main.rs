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

use crate::app_error::AppError;

/// The main entry point for the application.
/// This function initializes the application and runs the main event loop.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
async fn tokio_main() -> Result<(), AppError> {
  // let tmp = LoggerConfig::get_config();
  logger::initialize_logging()?;
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
