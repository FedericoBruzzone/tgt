pub mod app;
pub mod app_error;
pub mod tui;
pub mod tui_backend;
pub mod utils;

pub mod components;
pub mod enums;
pub mod traits;

use crate::app_error::AppError;

#[tokio::main]
/// The main entry point for the application.
/// This function initializes the application and runs the main event loop.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
async fn main() -> Result<(), AppError> {
  let mut app = app::App::new()?; //.with_frame_rate(60.0);
  app.run().await?;

  Ok(())
}
