use {
    crate::{
        configs::custom::keymap_custom::KeymapConfig, enums::action::Action,
        tui::Tui,
    },
    tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

/// `App` is a struct that represents the main application.
/// It is responsible for managing the user interface and the backend.
pub struct AppContext {
    /// The user interface for the application.
    tui: Tui,
    /// The keymap configuration.
    keymap_config: KeymapConfig,
    /// An unbounded sender that send action for processing.
    action_rx: UnboundedReceiver<Action>,
    /// An unbounded receiver that receives action for processing.
    action_tx: UnboundedSender<Action>,
    /// The frame rate at which the user interface should be rendered.
    pub frame_rate: f64,
    /// A boolean flag that represents whether the application should quit or
    /// not.
    pub quit: bool,
    /// A boolean flag that represents whether the mouse is enabled or not.
    pub mouse: bool,
    /// A boolean flag that represents whether the clipboard is enabled or not.
    pub paste: bool,
}

impl AppContext {
    /// Create a new instance of the `App` struct.
    ///
    /// # Returns
    /// * `Result<Self, io::Error>` - An Ok result containing the new instance
    ///   of the `App` struct or an error.
    pub fn new(keymap_config: KeymapConfig) -> Result<Self, std::io::Error> {
        let frame_rate = 60.0;
        let tui = Tui::new().with_keymap_config(keymap_config.clone());
        let (action_tx, action_rx) =
            tokio::sync::mpsc::unbounded_channel::<Action>();
        let quit = false;
        let mouse = false;
        let paste = false;
        Ok(Self {
            tui,
            keymap_config,
            action_rx,
            action_tx,
            frame_rate,
            quit,
            mouse,
            paste,
        })
    }

    pub fn with_frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }

    pub fn with_mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }

    pub fn with_paste(mut self, paste: bool) -> Self {
        self.paste = paste;
        self
    }

    pub fn tui_ref(&self) -> &Tui {
        &self.tui
    }

    pub fn tui_mut_ref(&mut self) -> &mut Tui {
        &mut self.tui
    }

    pub fn keymap_config_ref(&self) -> &KeymapConfig {
        &self.keymap_config
    }

    pub fn keymap_config_mut_ref(&mut self) -> &mut KeymapConfig {
        &mut self.keymap_config
    }

    pub fn action_rx_ref(&self) -> &UnboundedReceiver<Action> {
        &self.action_rx
    }

    pub fn action_rx_mut_ref(&mut self) -> &mut UnboundedReceiver<Action> {
        &mut self.action_rx
    }

    pub fn action_tx_ref(&self) -> &UnboundedSender<Action> {
        &self.action_tx
    }

    pub fn action_tx_mut_ref(&mut self) -> &mut UnboundedSender<Action> {
        &mut self.action_tx
    }

    pub fn action_tx_clone(&self) -> UnboundedSender<Action> {
        self.action_tx.clone()
    }
}
