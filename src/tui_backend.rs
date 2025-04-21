use {
    crate::{app_context::AppContext, event::Event},
    crossterm::{
        cursor,
        event::{
            DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
            Event as CrosstermEvent, EventStream, KeyEventKind,
        },
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    },
    futures::{future::Fuse, stream::Next, FutureExt, StreamExt},
    ratatui::{backend::CrosstermBackend, Terminal},
    std::{
        io::{self, Stderr},
        rc::Rc,
        time::Duration,
    },
    tokio::{
        sync::mpsc::{error::SendError, UnboundedReceiver, UnboundedSender},
        task::JoinHandle,
    },
};

/// `TuiBackend` is a struct that represents the backend for the user interface.
/// It is responsible for managing the terminal and buffering events for
/// processing.
pub struct TuiBackend {
    /// A terminal instance that is used to render the user interface.
    pub terminal: Terminal<CrosstermBackend<Stderr>>,
    /// A join handle that represents the task for processing events.
    pub task: JoinHandle<Result<(), SendError<Event>>>,
    /// An unbounded sender that can send events for processing.
    /// This is used to send events from the terminal to the event queue.
    pub event_tx: UnboundedSender<Event>,
    /// An unbounded receiver that can receive events for processing.
    /// This is used to receive events from the event queue for processing.
    /// The main loop consumes events from this queue calling the next method.
    pub event_rx: UnboundedReceiver<Event>,
    /// The frame rate at which the user interface should be rendered.
    pub frame_rate: f64,
    /// A boolean flag that represents whether the mouse is enabled or not.
    pub mouse: bool,
    /// A boolean flag that represents whether the paste mode is enabled or
    /// not.
    pub paste: bool,
}

impl TuiBackend {
    /// Create a new instance of the `TuiBackend` struct.
    /// # Arguments
    /// * `app_context` - An Arc wrapped AppContext struct.
    ///
    /// # Returns
    /// * `Result<Self, io::Error>` - An Ok result containing the new instance
    ///   of the `TuiBackend` struct or an error.
    pub fn new(app_context: Rc<AppContext>) -> Result<Self, std::io::Error> {
        tracing::info!("Creating TuiBackend");
        let frame_rate = app_context.app_config().frame_rate;
        let mouse = app_context.app_config().mouse_support;
        let paste = app_context.app_config().paste_support;
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
        let task: JoinHandle<Result<(), SendError<Event>>> =
            tokio::spawn(async { Err(SendError(Event::Init)) });
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
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
    /// Enter the user interface and start processing events.
    /// This will enable the raw mode for the terminal and switch to the
    /// alternate screen.
    ///
    /// # Returns
    /// * `Result<(), io::Error>` - An Ok result or an error.
    pub fn enter(&mut self) -> Result<(), io::Error> {
        match crossterm::terminal::enable_raw_mode() {
            Ok(_) => tracing::info!("Raw mode enabled"),
            Err(e) => tracing::error!("Error enabling raw mode: {}", e),
        }
        match crossterm::execute!(
            std::io::stderr(),
            EnterAlternateScreen,
            // cursor::Hide
        ) {
            Ok(_) => tracing::info!("Alternate screen enabled"),
            Err(e) => tracing::error!("Error enabling alternate screen: {}", e),
        };
        if self.mouse {
            crossterm::execute!(std::io::stderr(), EnableMouseCapture)?;
        }
        if self.paste {
            crossterm::execute!(std::io::stderr(), EnableBracketedPaste)?;
        }
        self.start();
        Ok(())
    }
    /// Exit the user interface and stop processing events.
    /// This will disable the raw mode for the terminal and switch back to the
    /// main screen.
    ///
    /// # Arguments
    /// * `mouse` - A boolean flag that represents whether the mouse was enabled
    ///   during the execution and need to be disabled.
    /// * `paste` - A boolean flag that represents whether the paste mode was
    ///   enabled during the execution and need to be disabled.
    ///
    /// # Returns
    /// * `Result<(), io::Error>` - An Ok result or an error.
    pub fn force_exit(mouse: bool, paste: bool) -> Result<(), std::io::Error> {
        crossterm::terminal::disable_raw_mode()?;
        tracing::info!("Raw mode disabled");
        crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show)?;
        tracing::info!("Alternate screen disabled");
        if mouse {
            crossterm::execute!(std::io::stderr(), DisableMouseCapture)?;
            tracing::info!("Mouse disabled");
        }
        if paste {
            crossterm::execute!(std::io::stderr(), DisableBracketedPaste)?;
            tracing::info!("Paste disabled");
        }
        Ok(())
    }
    /// Exit the user interface and stop processing events.
    /// This will disable the raw mode for the terminal and switch back to the
    /// main screen.
    ///
    /// # Returns
    /// * `Result<(), io::Error>` - An Ok result or an error.
    pub fn exit(&self) {
        match TuiBackend::force_exit(self.mouse, self.paste) {
            Ok(_) => tracing::info!("Tui backend exited"),
            Err(e) => tracing::error!("Error exiting tui backend: {}", e),
        }
    }
    /// Suspend the user interface and stop processing events.
    /// This will disable the raw mode for the terminal and switch back to the
    /// main screen.
    ///
    /// # Returns
    /// * `Result<(), io::Error>` - An Ok result or an error.
    pub fn suspend(&mut self) -> Result<(), std::io::Error> {
        tracing::info!("Suspending TuiBackend");
        self.exit();
        #[cfg(not(windows))]
        signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }
    /// Resume the user interface and start processing events.
    ///
    /// # Returns
    /// * `Result<(), io::Error>` - An Ok result or an error.
    pub fn resume(&mut self) -> Result<(), std::io::Error> {
        tracing::info!("Resuming TuiBackend");
        self.enter()?;
        Ok(())
    }
    /// Set the frame rate at which the user interface should be rendered.
    /// The frame rate is specified in frames per second (FPS).
    /// The default frame rate is 60 FPS.
    ///
    /// # Arguments
    /// * `frame_rate` - The frame rate at which the user interface should be
    ///   rendered.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `TuiBackend` struct.
    pub fn with_frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }
    /// Enable or disable the mouse for the user interface.
    /// By default, the mouse is disabled.
    ///
    /// # Arguments
    /// * `mouse` - A boolean flag that represents whether the mouse is enabled
    ///   or not.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `TuiBackend` struct.
    pub fn with_mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }
    /// Enable or disable the paste mode for the user interface.
    /// By default, the paste mode is disabled.
    ///
    /// # Arguments
    /// * `paste` - A boolean flag that represents whether the paste mode is
    ///   enabled or not.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `TuiBackend` struct.
    pub fn with_paste(mut self, paste: bool) -> Self {
        self.paste = paste;
        self
    }
    /// Send an event asynchronously for processing.
    /// This will pop from the event queue the first event that is ready and
    /// return it. If no event is available, this will sleep until an event
    /// is available.
    ///
    /// # Returns
    /// * `Option<Event>` - An optional event that is ready for processing.
    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
    /// Start processing events asynchronously.
    /// This will spawn a new task that will process events.
    /// The task will listen for events from the terminal and send them to the
    /// event queue for processing.
    fn start(&mut self) {
        let event_tx = self.event_tx.clone();
        let render_delay = Duration::from_secs_f64(1.0 / self.frame_rate);

        self.task = tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut render_interval = tokio::time::interval(render_delay);

            event_tx.send(Event::Init)?;
            loop {
                let crossterm_event: Fuse<Next<'_, EventStream>> = reader.next().fuse();
                let render_tick = render_interval.tick();

                tokio::select! {
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(event)) => {
                                match event {
                                    CrosstermEvent::Key(key) => {
                                        // Needed for Windows because without it the keys is sent twice.
                                        if key.kind == KeyEventKind::Press {
                                            event_tx.send(Event::Key(key.code, key.modifiers))?;
                                        }
                                    },
                                    CrosstermEvent::Mouse(mouse) => {
                                        event_tx.send(Event::Mouse(mouse))?;
                                    },
                                    CrosstermEvent::Resize(width, height) => {
                                        event_tx.send(Event::Resize(width, height))?;
                                    },
                                    CrosstermEvent::FocusLost => {
                                        event_tx.send(Event::FocusLost)?;
                                    }
                                    CrosstermEvent::FocusGained => {
                                        event_tx.send(Event::FocusGained)?;
                                    }
                                    CrosstermEvent::Paste(text) => {
                                        event_tx.send(Event::Paste(text))?;
                                    },
                                }
                          },
                          _ => unimplemented!()
                        }
                    },
                    _ = render_tick => {
                        event_tx.send(Event::Render)?;
                    }
                }
            }
        });
    }
}
