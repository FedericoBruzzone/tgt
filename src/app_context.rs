use crate::{
    action::Action,
    configs::custom::{
        app_custom::AppConfig, keymap_custom::KeymapConfig, palette_custom::PaletteConfig,
        theme_custom::ThemeConfig,
    },
    tg::tg_backend::TgContext,
};
use ratatui::style::Style;
use std::sync::{atomic::AtomicBool, Arc, Mutex, MutexGuard};
use std::{io, sync::atomic::Ordering};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// Generate a function that returns a style based on the theme configuration.
/// This macro generates a function that returns a style based on the theme
/// configuration. The function takes the lock on the theme configuration and
/// returns the style for the specified attribute.
/// The macro takes the function name, the map name, and the attribute name as
/// arguments.
/// The function checks if the theme is enabled in the application configuration.
/// If the theme is enabled, it returns the style for the specified attribute in
/// the specified map in the theme configuration. If the theme is disabled, it
/// returns the default style.
///
/// # Arguments
/// * `fn_name` - The name of the function.
/// * `map` - The name of the map in the theme configuration.
/// * `attr_name` - The name of the attribute in the map.
///
/// # Example
/// ```rust
/// theme_style_generate!(style_border_component_focused, common, border_component_focused);
/// ```
/// This will generate a function called `style_border_component_focused` that
/// returns a style based on the `border_component_focused` attribute in the
/// `common` map in the theme configuration.
macro_rules! theme_style_generate {
    ($fn_name: ident, $map: ident, $attr_name: ident) => {
        #[inline]
        pub fn $fn_name(&self) -> Style {
            if self.app_config().theme_enable {
                self.theme_config()
                    .$map
                    .get(stringify!($attr_name))
                    .unwrap()
                    .as_style()
            } else {
                Style::default()
            }
        }
    };
}

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
    /// An unbounded receiver that receives action for processing.
    /// This is used to send actions from the main loop to the main loop.
    /// A copy of this receiver is passed to all components.
    action_tx: Mutex<UnboundedSender<Action>>,
    /// An unbounded sender that send action for processing.
    /// This is used to receive events from the action queue for processing.
    /// The main loop consumes actions from this receiver.
    action_rx: Mutex<UnboundedReceiver<Action>>,
    /// A boolean flag that represents whether the application should quit or
    /// not.
    quit: AtomicBool,
    /// The Telegram context.
    tg_context: Arc<TgContext>,
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
    /// * `tg_context` - The Telegram context.
    ///
    /// # Returns
    /// * `Result<Self, io::Error>` - An Ok result containing the new instance
    ///   of the `App` struct or an error.
    pub fn new(
        app_config: AppConfig,
        keymap_config: KeymapConfig,
        theme_config: ThemeConfig,
        palette_config: PaletteConfig,
        tg_context: TgContext,
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
            tg_context: Arc::new(tg_context),
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
    pub fn quit_acquire(&self) -> bool {
        self.quit.load(Ordering::Acquire)
    }
    /// Set the quit flag.
    /// This function sets the value of the quit flag.
    /// The quit flag is a shared resource and is protected by an atomic boolean.
    pub fn quit_store(&self, value: bool) {
        self.quit.store(value, Ordering::Release);
    }
    /// Get the Telegram context.
    /// This function returns the Telegram context.
    /// The Telegram context is a shared resource and the contained variables are
    /// protected by a Mutex.
    pub fn tg_context(&self) -> Arc<TgContext> {
        Arc::clone(&self.tg_context)
    }

    // ===== COMMON ======
    theme_style_generate!(
        style_border_component_focused,
        common,
        border_component_focused
    );

    theme_style_generate!(style_item_selected, common, item_selected);

    // ===== CHAT LIST =====
    theme_style_generate!(style_chat_list, chat_list, self);

    // ===== CHAT =====
    theme_style_generate!(style_chat, chat, self);
    theme_style_generate!(style_chat_message_myself, chat, message_myself);
    theme_style_generate!(style_chat_message_other, chat, message_other);

    // ===== PROMPT =====
    theme_style_generate!(style_prompt, prompt, self);
    theme_style_generate!(
        style_prompt_message_preview_text,
        prompt,
        message_preview_text
    );

    // ===== STATUS BAR =====
    theme_style_generate!(style_status_bar, status_bar, self);
    theme_style_generate!(style_status_bar_size_info_text, status_bar, size_info_text);
    theme_style_generate!(
        style_status_bar_size_info_numbers,
        status_bar,
        size_info_numbers
    );
    theme_style_generate!(style_status_bar_press_key_text, status_bar, press_key_text);
    theme_style_generate!(style_status_bar_press_key_key, status_bar, press_key_key);
    theme_style_generate!(
        style_status_bar_message_quit_text,
        status_bar,
        message_quit_text
    );
    theme_style_generate!(
        style_status_bar_message_quit_key,
        status_bar,
        message_quit_key
    );

    // ===== TITLE BAR =====
    theme_style_generate!(style_title_bar, title_bar, self);
    theme_style_generate!(style_title_bar_title1, title_bar, title1);
    theme_style_generate!(style_title_bar_title2, title_bar, title2);
    theme_style_generate!(style_title_bar_title3, title_bar, title3);
}
