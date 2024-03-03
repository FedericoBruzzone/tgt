use {
  crate::{
    app_error::AppError,
    enums::{action::Action, event::Event},
    tui::Tui,
    tui_backend::TuiBackend,
  },
  ratatui::layout::Rect,
  std::io,
  tokio::sync::mpsc,
};

pub struct App {
  tui: Tui,
  tui_backend: TuiBackend,
  frame_rate: f64,
  quit: bool,
}

impl App {
  pub fn new() -> Result<Self, io::Error> {
    let tui = Tui::new();
    let tui_backend = TuiBackend::new()?
      .with_frame_rate(60.0)
      .with_mouse(true)
      .with_paste(true);
    let frame_rate = 60.0;
    let quit = false;
    Ok(Self {
      tui,
      tui_backend,
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
    self.tui_backend.enter()?;

    // self.tui.init(self.tui_backend.terminal.size()?)?;
    self.tui.register_action_handler(action_tx.clone())?;

    loop {
      if self.quit {
        // TODO: tui.stop()?
        break;
      }
      self.handle_tui_backend_events(&mut action_tx).await?;

      while let Ok(action) = action_rx.try_recv() {
        match action {
          Action::Render => {
            self.tui_backend.terminal.draw(|f| {
              self.tui.draw(f, f.size()).unwrap(); // TODO: handle with AppError
            })?;
          }
          Action::Resize(width, height) => {
            self.tui_backend.terminal.resize(Rect::new(0, 0, width, height))?;
            self.tui_backend.terminal.draw(|f| {
              self.tui.draw(f, f.size()).unwrap(); // TODO: handle with AppError
            })?;
          }
          Action::Quit => {
            self.quit = true;
          }
          Action::Mouse(_mouse) => {} // TODO: handle mouse events
          _ => {}
        }

        if let Some(action) = self.tui.update(action.clone())? {
          action_tx.send(action)?;
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

    self.tui_backend.exit()?;

    Ok(())
  }

  // ==============================
  // Private functions
  // ==============================

  async fn handle_tui_backend_events(&mut self, action_tx: &mut mpsc::UnboundedSender<Action>) -> Result<(), AppError> {
    if let Some(event) = self.tui_backend.next().await {
      match event {
        Event::Quit => action_tx.send(Action::Quit)?,
        Event::Key(key) => action_tx.send(Action::Key(key))?,
        Event::Render => action_tx.send(Action::Render)?,
        Event::Mouse(mouse) => action_tx.send(Action::Mouse(mouse))?,
        Event::Resize(width, height) => action_tx.send(Action::Resize(width, height))?,
        _ =>
          /* Event::Init */
          {}
      }
      if let Some(action) = self.tui.handle_events(Some(event.clone()))? {
        action_tx.send(action)?;
      }
    }
    Ok(())
  }
}
