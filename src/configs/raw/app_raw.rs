use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
/// The raw application configuration.
pub struct AppRaw {
    /// A boolean flag that represents whether the mouse is enabled or not.
    pub mouse_support: Option<bool>,
    /// A boolean flag that represents whether the clipboard is enabled or not.
    pub paste_support: Option<bool>,
    /// The frame rate at which the user interface should be rendered.
    pub frame_rate: Option<f64>,
    /// A boolean flag that represents whether the status bar should be shown
    /// or not.
    pub show_status_bar: Option<bool>,
    /// A boolean flag that represents whether the title bar should be shown or
    /// not.
    pub show_title_bar: Option<bool>,
    /// A boolean flag that represents whether the theme should be enabled or
    /// not.
    pub theme_enable: Option<bool>,
    /// The name of the theme file that should be used.
    pub theme_filename: Option<String>,
    /// A boolean flag that represents whether the API_ID should be taken from
    /// the Telegram configuration or from environment variable `API_ID`.
    pub take_api_id_from_telegram_config: Option<bool>,
    /// A boolean flag that represents whether the API_HASH should be taken from
    /// the Telegram configuration or from environment variables `API_HASH`.
    pub take_api_hash_from_telegram_config: Option<bool>,
}
