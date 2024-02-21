use crate::{
  action::Action,
  components::{home::Home, Component},
  tui::{Event, Tui},
};
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
    let home = Home::new();

    let components: Vec<Box<dyn Component>> = vec![Box::new(home)];
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
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();
    let mut tui = Tui::new()?.frame_rate(60.0).mouse(false).paste(true);
    tui.enter()?;

    for component in self.components.iter_mut() {
      component.register_action_handler(action_tx.clone())?;
      component.init(tui.terminal.size()?)?;
    }

    loop {
      if let Some(event) = tui.next().await {
        match event {
          Event::Quit => action_tx.send(Action::Quit)?,
          Event::Key(key) => action_tx.send(Action::Key(key))?,
          Event::Render => action_tx.send(Action::Render)?,
          _ => {}
        }
        for component in self.components.iter_mut() {
          if let Some(action) = component.handle_events(Some(event.clone()))? {
            action_tx.send(action)?;
          }
        }
      }

      while let Ok(action) = action_rx.try_recv() {
        match action {
          Action::Render => {
            tui.terminal.draw(|f| {
              for component in self.components.iter_mut() {
                component.draw(f, f.size()).unwrap(); // TODO: handle with AppError
              }
            })?;
          }
          Action::Quit => {
            self.quit = true;
          }
          _ => {}
        }

        for component in self.components.iter_mut() {
          if let Some(action) = component.update(action.clone())? {
            action_tx.send(action)?;
          }
        }
      }
      if self.quit {
        // TODO: tui.stop()?
        break;
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

  // fn handle_events_size() -> io::Result<i16> {
  //   if event::poll(std::time::Duration::from_millis(50))? {
  //     if let event::Event::Key(key) = event::read()? {
  //       if key.kind == event::KeyEventKind::Press {
  //         match key.code {
  //           event::KeyCode::Char('1') => return Ok(1),
  //           event::KeyCode::Char('2') => return Ok(-1),
  //           _ => {}
  //         }
  //       }
  //     }
  //   }
  //   Ok(0)
  // }
  //
  // fn handle_events_quit() -> io::Result<bool> {
  //   if event::poll(time::Duration::from_millis(50))? {
  //     if let event::Event::Key(key) = event::read()? {
  //       if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Char('q') {
  //         return Ok(true);
  //       }
  //     }
  //   }
  //   Ok(false)
  // }
  //
  // fn ui(size: u16, frame: &mut ratatui::Frame) {
  //   let main_layout = layout::Layout::new(
  //     layout::Direction::Vertical,
  //     [
  //       layout::Constraint::Length(1),
  //       layout::Constraint::Min(0),
  //       layout::Constraint::Length(1),
  //     ],
  //   )
  //   .split(frame.size());
  //   frame.render_widget(
  //     block::Block::new().borders(widgets::Borders::TOP).title("Title Bar"),
  //     main_layout[0],
  //   );
  //   frame.render_widget(
  //     block::Block::new()
  //       .borders(widgets::Borders::BOTTOM)
  //       .title(title::Title::from("Status Bar").position(title::Position::Bottom)),
  //     main_layout[2],
  //   );
  //
  //   let layout = layout::Layout::default()
  //     .direction(layout::Direction::Horizontal)
  //     .constraints([
  //       layout::Constraint::Percentage(size),
  //       layout::Constraint::Percentage(100 - size),
  //     ])
  //     .split(main_layout[1]);
  //
  //   frame.render_widget(
  //     block::Block::new()
  //       .border_set(border::PLAIN)
  //       .borders(widgets::Borders::TOP | widgets::Borders::LEFT | widgets::Borders::BOTTOM)
  //       .title("Left Block"),
  //     layout[0],
  //   );
  //
  //   let top_right_border_set = border::Set {
  //     top_left: line::NORMAL.horizontal_down,
  //     bottom_left: line::NORMAL.horizontal_up,
  //     ..border::PLAIN
  //   };
  //   frame.render_widget(
  //     block::Block::new()
  //       .border_set(top_right_border_set)
  //       .borders(widgets::Borders::ALL)
  //       .title("Top Right Block"),
  //     layout[1],
  //   );
  // }
}
