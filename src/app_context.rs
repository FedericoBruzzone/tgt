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
    /// Set the frame rate at which the user interface should be rendered.
    ///
    /// # Arguments
    /// * `frame_rate` - The frame rate at which the user interface should be
    ///
    /// # Returns
    /// * `Self` - The `App` struct.
    pub fn with_frame_rate(mut self, frame_rate: f64) -> Self {
        self.frame_rate = frame_rate;
        self
    }
    /// Set the boolean flag that represents whether the mouse is enabled or
    /// not.
    ///
    /// # Arguments
    /// * `mouse` - A boolean flag that represents whether the mouse is enabled
    ///  or not.
    ///
    ///  # Returns
    ///  * `Self` - The `App` struct.
    pub fn with_mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }
    /// Set the boolean flag that represents whether the clipboard is enabled
    /// or not.
    ///
    /// # Arguments
    /// * `paste` - A boolean flag that represents whether the clipboard is
    ///  enabled or not.
    ///
    ///  # Returns
    ///  * `Self` - The `App` struct.
    pub fn with_paste(mut self, paste: bool) -> Self {
        self.paste = paste;
        self
    }
    /// Get the user interface for the application.
    ///
    /// # Returns
    /// * `&Tui` - A reference to the user interface for the application.
    pub fn tui_ref(&self) -> &Tui {
        &self.tui
    }
    /// Get the user interface for the application.
    ///
    /// # Returns
    /// * `&mut Tui` - A mutable reference to the user interface for the
    ///  application.
    pub fn tui_mut_ref(&mut self) -> &mut Tui {
        &mut self.tui
    }
    /// Get the keymap configuration.
    ///
    /// # Returns
    /// * `&KeymapConfig` - A reference to the keymap configuration.
    pub fn keymap_config_ref(&self) -> &KeymapConfig {
        &self.keymap_config
    }
    /// Get the keymap configuration.
    ///
    /// # Returns
    /// * `&mut KeymapConfig` - A mutable reference to the keymap configuration.
    pub fn keymap_config_mut_ref(&mut self) -> &mut KeymapConfig {
        &mut self.keymap_config
    }
    /// Get the unbounded receiver that receives action for processing.
    ///
    /// # Returns
    /// * `&UnboundedReceiver<Action>` - A reference to the unbounded receiver
    /// that receives action for processing.
    pub fn action_rx_ref(&self) -> &UnboundedReceiver<Action> {
        &self.action_rx
    }
    /// Get the unbounded receiver that receives action for processing.
    ///
    /// # Returns
    /// * `&mut UnboundedReceiver<Action>` - A mutable reference to the unbounded receiver that receives action for processing.
    pub fn action_rx_mut_ref(&mut self) -> &mut UnboundedReceiver<Action> {
        &mut self.action_rx
    }
    /// Get the unbounded sender that send action for processing.
    ///
    /// # Returns
    /// * `&UnboundedSender<Action>` - A reference to the unbounded sender that
    pub fn action_tx_ref(&self) -> &UnboundedSender<Action> {
        &self.action_tx
    }
    /// Get the unbounded sender that send action for processing.
    ///
    /// # Returns
    /// * `&mut UnboundedSender<Action>` - A mutable reference to the unbounded
    pub fn action_tx_mut_ref(&mut self) -> &mut UnboundedSender<Action> {
        &mut self.action_tx
    }
    /// Get a clone of the unbounded sender that send action for processing.
    /// This is useful for sending action from other components.
    ///
    /// # Returns
    /// * `UnboundedSender<Action>` - A clone of the unbounded sender that send
    pub fn action_tx_clone(&self) -> UnboundedSender<Action> {
        self.action_tx.clone()
    }
}
