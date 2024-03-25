use {
    crate::{
        configs::custom::{app_custom::AppConfig, keymap_custom::KeymapConfig},
        enums::action::Action,
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
    /// The application configuration.
    app_config: AppConfig,
    /// An unbounded sender that send action for processing.
    action_rx: UnboundedReceiver<Action>,
    /// An unbounded receiver that receives action for processing.
    action_tx: UnboundedSender<Action>,
    /// A boolean flag that represents whether the application should quit or
    /// not.
    pub quit: bool,
}
/// Implementation of the `AppContext` struct.
impl AppContext {
    /// Create a new instance of the `App` struct.
    ///
    /// # Arguments
    /// * `app_config` - The application configuration.
    /// * `keymap_config` - The keymap configuration.
    ///
    /// # Returns
    /// * `Result<Self, io::Error>` - An Ok result containing the new instance
    ///   of the `App` struct or an error.
    pub fn new(
        app_config: AppConfig,
        keymap_config: KeymapConfig,
    ) -> Result<Self, std::io::Error> {
        let tui = Tui::new(app_config.clone(), keymap_config.clone());
        let (action_tx, action_rx) =
            tokio::sync::mpsc::unbounded_channel::<Action>();
        let quit = false;
        Ok(Self {
            tui,
            keymap_config,
            app_config,
            action_rx,
            action_tx,
            quit,
        })
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
    /// Get the application configuration.
    ///
    /// # Returns
    /// * `&AppConfig` - A reference to the application configuration.
    pub fn app_config_ref(&self) -> &AppConfig {
        &self.app_config
    }
    /// Get the application configuration.
    ///
    /// # Returns
    /// * `&mut AppConfig` - A mutable reference to the application
    ///   configuration.
    pub fn app_config_mut_ref(&mut self) -> &mut AppConfig {
        &mut self.app_config
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
    /// * `&mut UnboundedReceiver<Action>` - A mutable reference to the
    ///   unbounded receiver that receives action for processing.
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
