use crossterm::{
  cursor,
  event::{DisableMouseCapture, EnableMouseCapture},
  terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stderr};
// use tokio::{
//     sync::{mpsc, Mutex},
//     task::JoinHandle,
// };

use crate::utils;

// pub type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stderr>>;

pub struct Tui {
  pub terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>,
  pub mouse: bool,
  pub paste: bool,
}

impl Tui {
  pub fn new() -> Result<Self, io::Error> {
    let terminal = ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stderr()))
      .unwrap_or_else(|e| utils::fail_with("Failed to create terminal", e));
    let mouse = false;
    let paste = false;
    Ok(Self { terminal, mouse, paste })
  }

  pub fn enter(&self) -> Result<(), io::Error> {
    crossterm::terminal::enable_raw_mode().unwrap_or_else(|e| utils::fail_with("Failed to enable raw mode", e));
    crossterm::execute!(std::io::stderr(), EnterAlternateScreen, cursor::Hide)
      .unwrap_or_else(|e| utils::fail_with("Failed to enter alternate screen", e));
    if self.mouse {
      crossterm::execute!(std::io::stderr(), EnableMouseCapture)
        .unwrap_or_else(|e| utils::fail_with("Failed to enable mouse capture", e));
    }
    if self.paste {
      crossterm::execute!(std::io::stderr(), crossterm::event::EnableBracketedPaste)
        .unwrap_or_else(|e| utils::fail_with("Failed to enable paste", e));
    }
    Ok(())
  }

  pub fn exit(&self) -> Result<(), io::Error> {
    crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, DisableMouseCapture, cursor::Show)
      .unwrap_or_else(|e| utils::fail_with("Failed to leave alternate screen", e));
    crossterm::terminal::disable_raw_mode().unwrap_or_else(|e| utils::fail_with("Failed to disable raw mode", e));
    Ok(())
  }

  pub fn suspend(&self) -> Result<(), io::Error> {
    self.exit()?;
    #[cfg(not(windows))]
    signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)
      .unwrap_or_else(|e| utils::fail_with("Failed to suspend", e));
    Ok(())
  }

  pub fn resume(&self) -> Result<(), io::Error> {
    self.enter()?;
    Ok(())
  }

  pub fn mouse(mut self, mouse: bool) -> Self {
    self.mouse = mouse;
    self
  }

  pub fn paste(mut self, paste: bool) -> Self {
    self.paste = paste;
    self
  }
}
