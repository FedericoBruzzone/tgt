use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
/// The theme entry.
pub struct ThemeEntry {
    /// The foreground color.
    pub fg: Option<String>,
    /// The background color.
    pub bg: Option<String>,
    /// The italic option.
    pub italic: Option<bool>,
    /// The bold option.
    pub bold: Option<bool>,
    /// The underline option.
    pub underline: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
/// The raw theme configuration.
pub struct ThemeRaw {
    /// Customization for all components.
    pub common: Option<HashMap<String, ThemeEntry>>,
    /// The theme for the chat list.
    pub chat_list: Option<HashMap<String, ThemeEntry>>,
    /// The theme for the chat.
    pub chat: Option<HashMap<String, ThemeEntry>>,
    /// The theme for the prompt.
    pub prompt: Option<HashMap<String, ThemeEntry>>,
    /// The theme for the status bar.
    pub status_bar: Option<HashMap<String, ThemeEntry>>,
    /// The theme for the title bar.
    pub title_bar: Option<HashMap<String, ThemeEntry>>,
}
