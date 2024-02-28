use app::AppError;

pub mod action;
pub mod app;
pub mod components;
pub mod traits;
pub mod tui;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), AppError> {
  let mut app = app::App::new()?.frame_rate(60.0);
  app.run().await?;

  Ok(())
}
