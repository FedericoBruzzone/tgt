use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::stderr;
use std::process::exit;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

use crate::utils;

// pub type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<std::io::Stderr>>;

pub struct Tui {
    pub terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>,
}

impl Tui {
    pub fn new() -> Result<Self, ()> {
        let terminal = ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stderr()))
            .unwrap_or_else(|e| utils::fail_with("Failed to create terminal", e));
        Ok(Self { terminal })
    }

    pub fn enter(&self) -> Result<(), ()> {
        crossterm::terminal::enable_raw_mode()
            .unwrap_or_else(|e| utils::fail_with("Failed to enable raw mode", e));
        crossterm::execute!(
            std::io::stderr(),
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )
        .unwrap_or_else(|e| utils::fail_with("Failed to enter alternate screen", e));
        Ok(())
    }

    pub fn exit(&self) -> Result<(), ()> {
        crossterm::execute!(
            std::io::stderr(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        )
        .unwrap_or_else(|e| utils::fail_with("Failed to leave alternate screen", e));
        crossterm::terminal::disable_raw_mode()
            .unwrap_or_else(|e| utils::fail_with("Failed to disable raw mode", e));
        Ok(())
    }

    pub fn suspend(&self) -> Result<(), ()> {
        self.exit()?;
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)
            .unwrap_or_else(|e| utils::fail_with("Failed to suspend", e));
        Ok(())
    }
}
