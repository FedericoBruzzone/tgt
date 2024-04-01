use std::collections::HashMap;

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct ThemeEntry {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub italic: Option<bool>,
    pub bold: Option<bool>,
    pub underline: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
/// The raw theme configuration.
pub struct ThemeRaw {
    /// A boolean flag that represents whether the theme is enabled or not.
    pub theme_enable: Option<bool>,
    /// The background color.
    pub background_color: Option<String>,

    /// The theme for the default mode.
    // pub default: Option<ThemeEntry>,
    // Highlight component etc...

    /// The theme for the status bar.
    pub status_bar: Option<HashMap<String, ThemeEntry>>,
}
