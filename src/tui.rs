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

use crate::utils::unwrap_or_fail;

pub struct Tui {
  pub terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>,
  pub mouse: bool,
  pub paste: bool,
}

impl Tui {
  pub fn new() -> Result<Self, io::Error> {
    let terminal = unwrap_or_fail(
      ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stderr())),
      "Failed to create terminal",
    );
    let mouse = false;
    let paste = false;
    Ok(Self { terminal, mouse, paste })
  }

  pub fn enter(&self) -> Result<(), io::Error> {
    unwrap_or_fail(crossterm::terminal::enable_raw_mode(), "Failed to enable raw mode");
    unwrap_or_fail(crossterm::execute!(std::io::stderr(), EnterAlternateScreen, cursor::Hide), "Failed to hide cursor");
    if self.mouse {
      unwrap_or_fail(crossterm::execute!(std::io::stderr(), EnableMouseCapture), "Failed to enable mouse capture");
    }
    if self.paste {
      unwrap_or_fail(
        crossterm::execute!(std::io::stderr(), crossterm::event::EnableBracketedPaste),
        "Failed to enable paste",
      );
    }
    Ok(())
  }

  pub fn exit(&self) -> Result<(), io::Error> {
    unwrap_or_fail(crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show), "Failed to show cursor");
    unwrap_or_fail(crossterm::terminal::disable_raw_mode(), "Failed to disable raw mode");
    if self.mouse {
      unwrap_or_fail(crossterm::execute!(std::io::stderr(), DisableMouseCapture), "Failed to disable mouse capture");
    }
    if self.paste {
      unwrap_or_fail(
        crossterm::execute!(std::io::stderr(), crossterm::event::DisableBracketedPaste),
        "Failed to disable paste",
      );
    }
    Ok(())
  }

  pub fn suspend(&self) -> Result<(), io::Error> {
    self.exit()?;
    #[cfg(not(windows))]
    unwrap_or_fail(
      signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP),
      "Failed to leave alternate screen",
    );
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
