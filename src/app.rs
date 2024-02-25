use crate::{
  action::Action,
  components::{home::Home, Component},
  tui::{Event, Tui},
};
use ratatui::layout::Rect;
use std::io;
use tokio::sync::mpsc::{self, error::SendError};

// ========== Error ==========
#[derive(Debug)]
pub enum AppError {
  Io(io::Error),
  Send(SendError<Action>),
}

impl From<io::Error> for AppError {
  fn from(error: io::Error) -> Self {
    Self::Io(error)
  }
}

impl From<SendError<Action>> for AppError {
  fn from(error: SendError<Action>) -> Self {
    Self::Send(error)
  }
}

impl std::fmt::Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::Io(error) => write!(f, "IO error: {}", error),
      Self::Send(error) => write!(f, "Send error: {}", error),
    }
  }
}
// impl std::error::Error for AppError {}

// ===========================

pub struct App {
  components: Vec<Box<dyn Component>>,
  frame_rate: f64,
  quit: bool,
}

impl App {
  pub fn new() -> Result<Self, io::Error> {
    let components: Vec<Box<dyn Component>> = vec![Box::new(Home::new())];
    let frame_rate = 60.0;
    let quit = false;
    Ok(Self {
      components,
      frame_rate,
      quit,
    })
  }

  pub fn frame_rate(mut self, frame_rate: f64) -> Self {
    self.frame_rate = frame_rate;
    self
  }

  pub async fn run(&mut self) -> Result<(), AppError> {
    let (mut action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();
    let mut tui = Tui::new()?.frame_rate(60.0).mouse(true).paste(true);
    tui.enter()?;

    for component in self.components.iter_mut() {
      component.register_action_handler(action_tx.clone())?;
      component.init(tui.terminal.size()?)?;
    }

    loop {
      if self.quit {
        // TODO: tui.stop()?
        break;
      }
      self.handle_tui_events(&mut tui, &mut action_tx).await?;

      while let Ok(action) = action_rx.try_recv() {
        match action {
          Action::Render => {
            tui.terminal.draw(|f| {
              for component in self.components.iter_mut() {
                component.draw(f, f.size()).unwrap(); // TODO: handle with AppError
              }
            })?;
          }
          Action::Resize(width, height) => {
            tui.terminal.resize(Rect::new(0, 0, width, height))?;
            tui.terminal.draw(|f| {
              for component in self.components.iter_mut() {
                component.draw(f, f.size()).unwrap(); // TODO: handle with AppError
              }
            })?;
          }
          Action::Quit => {
            self.quit = true;
          }
          Action::Mouse(_mouse) => {} // TODO: handle mouse events
          _ => {}
        }

        for component in self.components.iter_mut() {
          if let Some(action) = component.update(action.clone())? {
            action_tx.send(action)?;
          }
        }
      }
    }

    // let mut size: u16 = 20;
    // let mut should_quit = false;
    // while !should_quit {
    //   tui.terminal.draw(|f| Self::ui(size, f))?;
    //   size = ((size as i16) + Self::handle_events_size()?) as u16;
    //   should_quit = Self::handle_events_quit()?;
    // }

    tui.exit()?;

    Ok(())
  }

  // ==============================
  // Private functions
  // ==============================

  async fn handle_tui_events(
    &mut self,
    tui: &mut Tui,
    action_tx: &mut mpsc::UnboundedSender<Action>,
  ) -> Result<(), AppError> {
    if let Some(event) = tui.next().await {
      match event {
        Event::Quit => action_tx.send(Action::Quit)?,
        Event::Key(key) => action_tx.send(Action::Key(key))?,
        Event::Render => action_tx.send(Action::Render)?,
        Event::Mouse(mouse) => action_tx.send(Action::Mouse(mouse))?,
        Event::Resize(width, height) => action_tx.send(Action::Resize(width, height))?,
        _ => {}
      }
      for component in self.components.iter_mut() {
        if let Some(action) = component.handle_events(Some(event.clone()))? {
          action_tx.send(action)?;
        }
      }
    }
    Ok(())
  }
}
