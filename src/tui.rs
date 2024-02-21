use crossterm::{
  cursor,
  event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event as CrosstermEvent,
    EventStream, KeyCode, KeyEvent, KeyEventKind,
  },
  terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
  io::{self, Stderr},
  time,
};
use tokio::{
  sync::mpsc::{self, error::SendError, UnboundedReceiver, UnboundedSender},
  task::JoinHandle,
};

#[derive(Clone)]
pub enum Event {
  Init,
  Quit,
  Render,
  Key(KeyEvent),
}

pub struct Tui {
  pub terminal: Terminal<CrosstermBackend<Stderr>>,
  pub task: JoinHandle<Result<(), SendError<Event>>>,
  pub event_rx: UnboundedReceiver<Event>,
  pub event_tx: UnboundedSender<Event>,
  pub frame_rate: f64,
  pub mouse: bool,
  pub paste: bool,
}

impl Tui {
  pub fn new() -> Result<Self, io::Error> {
    let terminal = ratatui::Terminal::new(CrosstermBackend::new(io::stderr()))?;
    let task: JoinHandle<Result<(), SendError<Event>>> = tokio::spawn(async { Err(SendError(Event::Init)) });
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let frame_rate = 60.0;
    let mouse = false;
    let paste = false;
    Ok(Self {
      terminal,
      task,
      event_rx,
      event_tx,
      frame_rate,
      mouse,
      paste,
    })
  }

  pub fn enter(&mut self) -> Result<(), io::Error> {
    terminal::enable_raw_mode()?;
    crossterm::execute!(io::stderr(), EnterAlternateScreen, cursor::Hide)?;
    if self.mouse {
      crossterm::execute!(io::stderr(), EnableMouseCapture)?;
    }
    if self.paste {
      crossterm::execute!(io::stderr(), EnableBracketedPaste)?;
    }
    self.start();
    Ok(())
  }

  pub fn exit(&self) -> Result<(), io::Error> {
    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stderr(), LeaveAlternateScreen, cursor::Show)?;
    if self.mouse {
      crossterm::execute!(io::stderr(), DisableMouseCapture)?;
    }
    if self.paste {
      crossterm::execute!(io::stderr(), DisableBracketedPaste)?;
    }
    Ok(())
  }

  pub fn suspend(&mut self) -> Result<(), io::Error> {
    self.exit()?;
    #[cfg(not(windows))]
    signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
    Ok(())
  }

  pub fn resume(&mut self) -> Result<(), io::Error> {
    self.enter()?;
    Ok(())
  }

  pub fn frame_rate(mut self, frame_rate: f64) -> Self {
    self.frame_rate = frame_rate;
    self
  }

  pub fn mouse(mut self, mouse: bool) -> Self {
    self.mouse = mouse;
    self
  }

  pub fn paste(mut self, paste: bool) -> Self {
    self.paste = paste;
    self
  }

  pub async fn next(&mut self) -> Option<Event> {
    self.event_rx.recv().await
  }

  // ==============================
  // Private functions
  // ==============================

  fn start(&mut self) {
    let _event_tx = self.event_tx.clone();
    let render_delay = time::Duration::from_secs_f64(1.0 / self.frame_rate);

    self.task = tokio::spawn(async move {
      let mut reader = EventStream::new();
      let mut render_interval = tokio::time::interval(render_delay);

      _event_tx.send(Event::Init)?;
      loop {
        let crossterm_event = reader.next().fuse();
        let render_tick = render_interval.tick();

        tokio::select! {
          maybe_event = crossterm_event => {
            match maybe_event {
              Some(Ok(event)) => {
                match event {
                  CrosstermEvent::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                      if key.code == KeyCode::Char('q') {
                        _event_tx.send(Event::Quit)?;
                      } else {
                        _event_tx.send(Event::Key(key))?;
                      }
                    }
                  },
                  _ => unimplemented!()
                }
              },
              _ => unimplemented!()
            }
          },
          _ = render_tick => {
            _event_tx.send(Event::Render)?;
          }
        }
      }
    });
  }
}
