use crate::utils::unwrap_or_fail;
use crossterm::{cursor, event, terminal};
use futures::{FutureExt, StreamExt};
use ratatui::backend;
use std::io;
use tokio::{sync::mpsc, task};

pub enum Event {
  Init,
  Quit,
  Key(event::KeyEvent),
}

pub struct Tui {
  pub terminal: ratatui::Terminal<backend::CrosstermBackend<std::io::Stderr>>,
  pub task: task::JoinHandle<()>,
  pub event_rx: mpsc::UnboundedReceiver<Event>,
  pub event_tx: mpsc::UnboundedSender<Event>,
  pub mouse: bool,
  pub paste: bool,
}

impl Tui {
  pub fn new() -> Result<Self, io::Error> {
    let terminal =
      unwrap_or_fail(ratatui::Terminal::new(backend::CrosstermBackend::new(io::stderr())), "Failed to create terminal");
    let task = tokio::spawn(async {});
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let mouse = false;
    let paste = false;
    Ok(Self { terminal, task, event_rx, event_tx, mouse, paste })
  }

  pub fn enter(&mut self) -> Result<(), io::Error> {
    unwrap_or_fail(terminal::enable_raw_mode(), "Failed to enable raw mode");
    unwrap_or_fail(
      crossterm::execute!(io::stderr(), terminal::EnterAlternateScreen, cursor::Hide),
      "Failed to hide cursor",
    );
    if self.mouse {
      unwrap_or_fail(crossterm::execute!(io::stderr(), event::EnableMouseCapture), "Failed to enable mouse capture");
    }
    if self.paste {
      unwrap_or_fail(crossterm::execute!(io::stderr(), event::EnableBracketedPaste), "Failed to enable paste");
    }
    self.start();
    Ok(())
  }

  pub fn exit(&self) -> Result<(), io::Error> {
    unwrap_or_fail(
      crossterm::execute!(io::stderr(), terminal::LeaveAlternateScreen, cursor::Show),
      "Failed to show cursor",
    );
    unwrap_or_fail(terminal::disable_raw_mode(), "Failed to disable raw mode");
    if self.mouse {
      unwrap_or_fail(crossterm::execute!(io::stderr(), event::DisableMouseCapture), "Failed to disable mouse capture");
    }
    if self.paste {
      unwrap_or_fail(crossterm::execute!(io::stderr(), event::DisableBracketedPaste), "Failed to disable paste");
    }
    Ok(())
  }

  pub fn suspend(&mut self) -> Result<(), io::Error> {
    self.exit()?;
    #[cfg(not(windows))]
    unwrap_or_fail(signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP), "Failed to raise SIGTSTP");
    Ok(())
  }

  pub fn resume(&mut self) -> Result<(), io::Error> {
    self.enter()?;
    Ok(())
  }

  pub async fn next(&mut self) -> Option<Event> {
    self.event_rx.recv().await
  }

  pub fn mouse(mut self, mouse: bool) -> Self {
    self.mouse = mouse;
    self
  }

  pub fn paste(mut self, paste: bool) -> Self {
    self.paste = paste;
    self
  }

  // ==============================
  // Private functions
  // ==============================

  fn start(&mut self) {
    let _event_tx = self.event_tx.clone();
    self.task = tokio::spawn(async move {
      let mut reader = event::EventStream::new();
      unwrap_or_fail(_event_tx.send(Event::Init), "Failed to send init event");
      loop {
        let crossterm_event = reader.next().fuse();

        tokio::select! {
              maybe_event = crossterm_event => {
                  match maybe_event {
                      Some(Ok(event)) => {
                          match event {
                              event::Event::Key(key) => {
                                  if key.kind == event::KeyEventKind::Press {
                                      unwrap_or_fail(_event_tx.send(Event::Key(key)),
                                                     format!("Failed to send key event: {:?}", key).as_str());
                                  }
                              },
                              _ => unimplemented!()
                          }
                      },
                      _ => unimplemented!()
                  }
              }
        }
      }
    });
  }
}
