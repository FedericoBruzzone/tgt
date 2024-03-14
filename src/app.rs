use {
    crate::{
        app_error::AppError,
        enums::{action::Action, event::Event},
        tui::Tui,
        tui_backend::TuiBackend,
    },
    ratatui::layout::Rect,
    tokio::sync::mpsc::UnboundedSender,
};

/// `App` is a struct that represents the main application.
/// It is responsible for managing the user interface and the backend.
pub struct App {
    /// The user interface for the application.
    tui: Tui,
    /// The backend for the user interface.
    tui_backend: TuiBackend,
    /// The frame rate at which the user interface should be rendered.
    frame_rate: f64,
    /// A boolean flag that represents whether the application should quit or not.
    quit: bool,
}

impl App {
    /// Create a new instance of the `App` struct.
    ///
    /// # Returns
    /// * `Result<Self, io::Error>` - An Ok result containing the new instance of the `App` struct or an error.
    pub fn new() -> Result<Self, std::io::Error> {
        let tui = Tui::new();
        let frame_rate = 60.0;
        let tui_backend = TuiBackend::new()?
            .with_frame_rate(frame_rate)
            .with_mouse(true)
            .with_paste(true);
        let quit = false;
        Ok(Self {
            tui,
            tui_backend,
            frame_rate,
            quit,
        })
    }
    /// Set the frame rate at which the user interface should be rendered.
    /// The frame rate is specified in frames per second (FPS).
    /// The default frame rate is 60 FPS.
    ///
    /// # Arguments
    /// * `frame_rate` - The frame rate at which the user interface should be rendered.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `TuiBackend` struct.
    pub fn with_frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self.tui_backend = self.tui_backend.with_frame_rate(frame_rate);
        self
    }
    /// Run the main event loop for the application.
    /// This function will process events and actions for the user interface and the backend.
    ///
    /// # Returns
    /// * `Result<(), AppError>` - An Ok result or an error.
    pub async fn run(&mut self) -> Result<(), AppError> {
        tracing::info!("Starting app");
        let (mut action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel::<Action>();
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
                            self.tui.draw(f, f.size()).unwrap();
                            // TODO: handle with AppError
                        })?;
                    }
                    Action::Resize(width, height) => {
                        self.tui_backend.terminal.resize(Rect::new(0, 0, width, height))?;
                        self.tui_backend.terminal.draw(|f| {
                            self.tui.draw(f, f.size()).unwrap();
                            // TODO: handle with AppError
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
    /// Handle incoming events from the TUI backend.
    /// This function will process events from the TUI backend and produce actions if necessary.
    ///
    /// # Arguments
    /// * `action_tx` - A mutable reference to the action sender.
    ///
    /// # Returns
    /// * `Result<(), AppError>` - An Ok result or an error.
    async fn handle_tui_backend_events(&mut self, action_tx: &mut UnboundedSender<Action>) -> Result<(), AppError> {
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
