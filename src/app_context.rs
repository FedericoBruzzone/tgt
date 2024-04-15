use std::sync::{atomic::AtomicBool, Mutex, MutexGuard};

use crate::{
    configs::custom::{
        app_custom::AppConfig, keymap_custom::KeymapConfig, palette_custom::PaletteConfig,
        theme_custom::ThemeConfig,
    },
    enums::action::Action,
};
use std::io;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// `AppContext` is a struct that represents the main application.
/// It contains the application configuration, keymap configuration, theme
/// configuration, palette configuration, and an unbounded sender and receiver
/// for sending and receiving actions.
/// It also contains a boolean flag that represents whether the application
/// should quit or not.
/// The `AppContext` struct is used to pass the application context around the
/// application.
/// The `AppContext` struct is thread-safe and can be shared across threads.
pub struct AppContext {
    /// The application configuration.
    app_config: Mutex<AppConfig>,
    /// The keymap configuration.
    keymap_config: Mutex<KeymapConfig>,
    /// The theme configuration.
    theme_config: Mutex<ThemeConfig>,
    /// The palette configuration.
    palette_config: Mutex<PaletteConfig>,
    /// An unbounded sender that send action for processing.
    action_rx: Mutex<UnboundedReceiver<Action>>,
    /// An unbounded receiver that receives action for processing.
    action_tx: Mutex<UnboundedSender<Action>>,
    /// A boolean flag that represents whether the application should quit or
    /// not.
    pub quit: AtomicBool,
}
/// Implementation of the `AppContext` struct.
impl AppContext {
    /// Create a new instance of the `App` struct.
    ///
    /// # Arguments
    /// * `app_config` - The application configuration.
    /// * `keymap_config` - The keymap configuration.
    /// * `theme_config` - The theme configuration.
    /// * `palette_config` - The palette configuration.
    ///
    /// # Returns
    /// * `Result<Self, io::Error>` - An Ok result containing the new instance
    ///   of the `App` struct or an error.
    pub fn new(
        app_config: AppConfig,
        keymap_config: KeymapConfig,
        theme_config: ThemeConfig,
        palette_config: PaletteConfig,
    ) -> Result<Self, io::Error> {
        let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel::<Action>();
        let quit = false;
        Ok(Self {
            app_config: Mutex::new(app_config),
            keymap_config: Mutex::new(keymap_config),
            palette_config: Mutex::new(palette_config),
            theme_config: Mutex::new(theme_config),
            action_rx: Mutex::new(action_rx),
            action_tx: Mutex::new(action_tx),
            quit: AtomicBool::new(quit),
        })
    }
    /// Get the application configuration.
    /// This function takes the lock on the application configuration and returns
    /// the application configuration.
    /// The application configuration is a shared resource and is protected by a
    /// mutex.
    ///
    /// # Returns
    /// * `MutexGuard<'_, AppConfig>` - The application configuration.
    pub fn app_config(&self) -> MutexGuard<'_, AppConfig> {
        self.app_config.lock().unwrap()
    }
    /// Get the keymap configuration.
    /// This function takes the lock on the keymap configuration and returns the
    /// keymap configuration.
    /// The keymap configuration is a shared resource and is protected by a mutex.
    ///
    /// # Returns
    /// * `MutexGuard<'_, KeymapConfig>` - The keymap configuration.
    pub fn keymap_config(&self) -> MutexGuard<'_, KeymapConfig> {
        self.keymap_config.lock().unwrap()
    }
    /// Get the theme configuration.
    /// This function takes the lock on the theme configuration and returns the
    /// theme configuration.
    /// The theme configuration is a shared resource and is protected by a mutex.
    ///
    /// # Returns
    /// * `MutexGuard<'_, ThemeConfig>` - The theme configuration.
    pub fn theme_config(&self) -> MutexGuard<'_, ThemeConfig> {
        self.theme_config.lock().unwrap()
    }
    /// Get the palette configuration.
    /// This function takes the lock on the palette configuration and returns the
    /// palette configuration.
    /// The palette configuration is a shared resource and is protected by a mutex.
    ///
    /// # Returns
    /// * `MutexGuard<'_, PaletteConfig>` - The palette configuration.
    pub fn palette_config(&self) -> MutexGuard<'_, PaletteConfig> {
        self.palette_config.lock().unwrap()
    }
    /// Get the action receiver.
    /// This function takes the lock on the action receiver and returns the action
    /// receiver.
    /// The action receiver is a shared resource and is protected by a mutex.
    ///
    /// # Returns
    /// * `MutexGuard<'_, UnboundedReceiver<Action>>` - The action receiver.
    pub fn action_rx(&self) -> MutexGuard<'_, UnboundedReceiver<Action>> {
        self.action_rx.lock().unwrap()
    }
    /// Get the action sender.
    /// This function takes the lock on the action sender and returns the action
    /// sender.
    /// The action sender is a shared resource and is protected by a mutex.
    ///
    /// # Returns
    /// * `MutexGuard<'_, UnboundedSender<Action>>` - The action sender.
    pub fn action_tx(&self) -> MutexGuard<'_, UnboundedSender<Action>> {
        self.action_tx.lock().unwrap()
    }
    /// Get the quit flag.
    /// This function returns the value of the quit flag.
    /// The quit flag is a shared resource and is protected by an atomic boolean.
    ///
    /// # Returns
    /// * `bool` - The value of the quit flag.
    pub fn quit(&self) -> bool {
        self.quit.load(std::sync::atomic::Ordering::Acquire)
    }
}
