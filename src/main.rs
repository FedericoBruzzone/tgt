use std::io;

pub mod action;
pub mod app;
pub mod tui;
pub mod utils;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
  let mut app = app::App::new();
  app.run().await?;

  Ok(())
}
