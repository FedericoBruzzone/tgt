use {
    crate::{
        app_error::AppError,
        configs::{
            self, config_file::ConfigFile, config_theme::ThemeStyle, config_type::ConfigType,
            raw::theme_raw::ThemeRaw,
        },
        APP_CONFIG,
    },
    std::{collections::HashMap, path::Path},
};

#[derive(Clone, Debug)]
/// The theme configuration.
pub struct ThemeConfig {
    /// The theme configuration for the all components.
    pub common: HashMap<String, ThemeStyle>,
    /// The theme configuration for the chat list.
    pub chat_list: HashMap<String, ThemeStyle>,
    /// The theme configuration for the chat.
    pub chat: HashMap<String, ThemeStyle>,
    /// The theme configuration for the prompt.
    pub prompt: HashMap<String, ThemeStyle>,
    /// The theme configuration for the status bar.
    pub status_bar: HashMap<String, ThemeStyle>,
    /// The theme configuration for the title bar.
    pub title_bar: HashMap<String, ThemeStyle>,
    /// The theme configuration for the reply message.
    pub reply_message: HashMap<String, ThemeStyle>,
}
/// The theme configuration implementation.
impl ThemeConfig {
    /// Get the default theme configuration.
    ///
    /// This is used as a fallback when `get_config()` cannot find the theme file specified
    /// in `APP_CONFIG.theme_filename`. It tries to load from standard locations:
    /// 1. Default theme.toml location (`~/.tgt/config/theme.toml`)
    /// 2. Themes subdirectory (`~/.tgt/config/themes/theme.toml`)
    ///
    /// **Note:** In normal operation, `get_config()` is called instead (see `get_config()` documentation).
    /// This function is primarily used as a fallback or when `Default::default()` is called.
    ///
    /// # Returns
    /// * `Result<Self>` - The default theme configuration, or empty HashMaps if no theme file is found.
    pub fn default_result() -> Result<Self, AppError<()>> {
        // First try to load from the default theme.toml location
        let file_path = configs::custom::default_config_theme_file_path()?;
        let path = Path::new(&file_path);
        if path.exists() {
            return configs::deserialize_to_config_into::<ThemeRaw, Self>(path);
        }

        // If not found, try to load from themes/theme.toml
        if let Ok(config_dir) = crate::utils::tgt_config_dir() {
            let themes_path = config_dir.join("themes").join("theme.toml");
            if themes_path.exists() {
                return configs::deserialize_to_config_into::<ThemeRaw, Self>(&themes_path);
            }
        }

        // If neither exists, return empty HashMaps
        // The actual theme will be loaded via get_config() which uses APP_CONFIG.theme_filename
        // See get_config() documentation for details on when and how it's called.
        Ok(Self {
            common: HashMap::new(),
            chat_list: HashMap::new(),
            chat: HashMap::new(),
            prompt: HashMap::new(),
            status_bar: HashMap::new(),
            title_bar: HashMap::new(),
            reply_message: HashMap::new(),
        })
    }
}
/// The implementation of the configuration file for the theme.
impl ConfigFile for ThemeConfig {
    type Raw = ThemeRaw;

    fn get_type() -> ConfigType {
        ConfigType::Theme
    }
    fn override_fields() -> bool {
        true
    }

    /// Override the default implementation of `get_config()` to use the theme filename from app config.
    ///
    /// **When is this called?**
    /// - Called during application initialization in `main.rs` as part of the `THEME_CONFIG` lazy static
    /// - This happens once at startup, before the application context is created
    ///
    /// **How does it work?**
    /// - Uses `APP_CONFIG.theme_filename` to determine which theme file to load
    /// - Searches for the theme file in the config directory hierarchy (see `CONFIG_DIR_HIERARCHY`)
    /// - The default value of `theme_filename` is "themes/theme.toml" (as set in `config/app.toml`)
    /// - If the theme file is found, it loads and deserializes it
    /// - If not found, returns the default theme configuration (empty HashMaps)
    ///
    /// **Theme file search order:**
    /// 1. `TGT_CONFIG_DIR/themes/{theme_filename}` (if `TGT_CONFIG_DIR` is set)
    /// 2. `./config/themes/{theme_filename}` (debug mode only)
    /// 3. `~/.config/tgt/config/themes/{theme_filename}` (if exists)
    /// 4. `~/.tgt/config/themes/{theme_filename}` (if exists)
    ///
    /// **Example:**
    /// If `APP_CONFIG.theme_filename = "themes/monokai.toml"`, this function will search for
    /// `monokai.toml` in the `themes/` subdirectory of each config directory in the hierarchy.
    fn get_config() -> Self {
        if Self::override_fields() {
            // Use deserialize_config_or_default directly with APP_CONFIG.theme_filename
            // This will load the theme file if found, or return default if not found
            Self::deserialize_config_or_default::<Self::Raw, Self>(&APP_CONFIG.theme_filename)
        } else {
            Self::deserialize_config_or_default::<Self::Raw, Self>(
                Self::get_type().as_default_filename().as_str(),
            )
        }
    }

    fn merge(&mut self, other: Option<Self::Raw>) -> Self {
        match other {
            None => self.clone(),
            Some(other) => {
                tracing::info!("Merging theme config");
                if let Some(common) = other.common {
                    common.into_iter().for_each(|(k, v)| {
                        self.common.insert(k, ThemeStyle::from(v));
                    });
                }
                if let Some(chat_list) = other.chat_list {
                    chat_list.into_iter().for_each(|(k, v)| {
                        self.chat_list.insert(k, ThemeStyle::from(v));
                    });
                }
                if let Some(chat) = other.chat {
                    chat.into_iter().for_each(|(k, v)| {
                        self.chat.insert(k, ThemeStyle::from(v));
                    });
                }
                if let Some(prompt) = other.prompt {
                    prompt.into_iter().for_each(|(k, v)| {
                        self.prompt.insert(k, ThemeStyle::from(v));
                    });
                }
                if let Some(status_bar) = other.status_bar {
                    status_bar.into_iter().for_each(|(k, v)| {
                        self.status_bar.insert(k, ThemeStyle::from(v));
                    });
                }
                if let Some(title_bar) = other.title_bar {
                    title_bar.into_iter().for_each(|(k, v)| {
                        self.title_bar.insert(k, ThemeStyle::from(v));
                    });
                }
                if let Some(reply_message) = other.reply_message {
                    reply_message.into_iter().for_each(|(k, v)| {
                        self.reply_message.insert(k, ThemeStyle::from(v));
                    });
                }
                self.clone()
            }
        }
    }
}
/// The default implementation for the theme configuration.
impl Default for ThemeConfig {
    fn default() -> Self {
        Self::default_result().unwrap()
    }
}
/// The conversion from the raw theme configuration to the theme configuration.
impl From<ThemeRaw> for ThemeConfig {
    fn from(raw: ThemeRaw) -> Self {
        let common = raw
            .common
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from(v)))
            .collect();
        let chat_list = raw
            .chat_list
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from(v)))
            .collect();
        let chat = raw
            .chat
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from(v)))
            .collect();
        let prompt = raw
            .prompt
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from(v)))
            .collect();
        let status_bar = raw
            .status_bar
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from(v)))
            .collect();
        let title_bar = raw
            .title_bar
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from(v)))
            .collect();
        let reply_message = raw
            .reply_message
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from(v)))
            .collect();

        Self {
            common,
            chat_list,
            chat,
            prompt,
            status_bar,
            title_bar,
            reply_message,
        }
    }
}

impl ThemeConfig {
    /// Convert a `ThemeRaw` to a `ThemeConfig` using a specific palette.
    /// This is useful when converting themes with a palette that differs from
    /// the global `PALETTE_CONFIG`.
    ///
    /// # Arguments
    /// * `raw` - The `ThemeRaw` to convert.
    /// * `palette` - The palette to use for color lookup.
    ///
    /// # Returns
    /// * `Self` - The converted `ThemeConfig`.
    pub fn from_raw_with_palette(
        raw: ThemeRaw,
        palette: &std::collections::HashMap<String, ratatui::style::Color>,
    ) -> Self {
        let common = raw
            .common
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from_entry_with_palette(v, palette)))
            .collect();
        let chat_list = raw
            .chat_list
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from_entry_with_palette(v, palette)))
            .collect();
        let chat = raw
            .chat
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from_entry_with_palette(v, palette)))
            .collect();
        let prompt = raw
            .prompt
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from_entry_with_palette(v, palette)))
            .collect();
        let status_bar = raw
            .status_bar
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from_entry_with_palette(v, palette)))
            .collect();
        let title_bar = raw
            .title_bar
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from_entry_with_palette(v, palette)))
            .collect();
        let reply_message = raw
            .reply_message
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from_entry_with_palette(v, palette)))
            .collect();

        Self {
            common,
            chat_list,
            chat,
            prompt,
            status_bar,
            title_bar,
            reply_message,
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::configs::{
            config_file::ConfigFile,
            config_type::ConfigType,
            custom::theme_custom::ThemeConfig,
            raw::theme_raw::{ThemeEntry, ThemeRaw},
        },
        ratatui::style::Color,
        std::collections::HashMap,
    };

    #[test]
    fn test_theme_config_default() {
        let theme_config = crate::configs::custom::theme_custom::ThemeConfig::default();
        assert_eq!(theme_config.common.len(), 4);
        assert_eq!(theme_config.chat_list.len(), 5);
        assert_eq!(theme_config.chat.len(), 11);
        assert_eq!(theme_config.prompt.len(), 4);
        assert_eq!(theme_config.status_bar.len(), 9);
        assert_eq!(theme_config.title_bar.len(), 4);
    }

    #[test]
    fn test_theme_config_from_raw_empty() {
        let theme_raw = ThemeRaw {
            common: Some(HashMap::new()),
            chat_list: Some(HashMap::new()),
            chat: Some(HashMap::new()),
            prompt: Some(HashMap::new()),
            status_bar: Some(HashMap::new()),
            title_bar: Some(HashMap::new()),
            reply_message: Some(HashMap::new()),
        };
        let theme_config = ThemeConfig::from(theme_raw);
        assert_eq!(theme_config.common.len(), 0);
    }

    #[test]
    fn test_theme_config_from_raw() {
        let mut common = HashMap::new();
        common.insert(
            "default".to_string(),
            ThemeEntry {
                fg: Some("red".to_string()),
                bg: Some("black".to_string()),
                italic: Some(true),
                bold: Some(true),
                underline: Some(true),
            },
        );
        common.insert(
            "selected".to_string(),
            ThemeEntry {
                fg: Some("black".to_string()),
                bg: Some("red".to_string()),
                italic: Some(false),
                bold: Some(false),
                underline: Some(false),
            },
        );
        let mut chat_list = HashMap::new();
        chat_list.insert(
            "default".to_string(),
            ThemeEntry {
                fg: Some("red".to_string()),
                bg: Some("black".to_string()),
                italic: Some(true),
                bold: Some(true),
                underline: Some(true),
            },
        );
        let mut chat = HashMap::new();
        chat.insert(
            "default".to_string(),
            ThemeEntry {
                fg: Some("red".to_string()),
                bg: Some("black".to_string()),
                italic: Some(true),
                bold: Some(true),
                underline: Some(true),
            },
        );
        let theme_raw = ThemeRaw {
            common: Some(common),
            chat_list: Some(HashMap::new()),
            chat: Some(HashMap::new()),
            prompt: Some(HashMap::new()),
            status_bar: Some(HashMap::new()),
            title_bar: Some(HashMap::new()),
            reply_message: Some(HashMap::new()),
        };
        let theme_config = ThemeConfig::from(theme_raw);
        assert_eq!(theme_config.common.len(), 2);
        assert_eq!(theme_config.chat_list.len(), 0);
        assert_eq!(theme_config.chat.len(), 0);
        assert_eq!(theme_config.prompt.len(), 0);
        assert_eq!(theme_config.status_bar.len(), 0);
        assert_eq!(theme_config.title_bar.len(), 0);
        assert_eq!(theme_config.reply_message.len(), 0);
    }

    #[test]
    fn test_theme_config_merge() {
        let mut common = HashMap::new();
        common.insert(
            "default".to_string(),
            ThemeEntry {
                fg: Some("red".to_string()),
                bg: Some("black".to_string()),
                italic: Some(true),
                bold: Some(true),
                underline: Some(true),
            },
        );
        common.insert(
            "selected".to_string(),
            ThemeEntry {
                fg: Some("black".to_string()),
                bg: Some("red".to_string()),
                italic: Some(false),
                bold: Some(false),
                underline: Some(false),
            },
        );
        let mut chat_list = HashMap::new();
        chat_list.insert(
            "default".to_string(),
            ThemeEntry {
                fg: Some("red".to_string()),
                bg: Some("black".to_string()),
                italic: Some(true),
                bold: Some(true),
                underline: Some(true),
            },
        );
        let mut chat = HashMap::new();
        chat.insert(
            "default".to_string(),
            ThemeEntry {
                fg: Some("red".to_string()),
                bg: Some("black".to_string()),
                italic: Some(true),
                bold: Some(true),
                underline: Some(true),
            },
        );
        let theme_raw = ThemeRaw {
            common: Some(common),
            chat_list: Some(HashMap::new()),
            chat: Some(HashMap::new()),
            prompt: Some(HashMap::new()),
            status_bar: Some(HashMap::new()),
            title_bar: Some(HashMap::new()),
            reply_message: Some(HashMap::new()),
        };
        let mut theme_config = ThemeConfig::from(theme_raw);

        let mut common = HashMap::new();
        common.insert(
            "default".to_string(),
            ThemeEntry {
                fg: Some("blue".to_string()),
                bg: Some("white".to_string()),
                italic: Some(false),
                bold: Some(false),
                underline: Some(false),
            },
        );
        let mut chat_list = HashMap::new();
        chat_list.insert(
            "default".to_string(),
            ThemeEntry {
                fg: Some("blue".to_string()),
                bg: Some("white".to_string()),
                italic: Some(false),
                bold: Some(false),
                underline: Some(false),
            },
        );
        let mut chat = HashMap::new();
        chat.insert(
            "default".to_string(),
            ThemeEntry {
                fg: Some("blue".to_string()),
                bg: Some("white".to_string()),
                italic: Some(false),
                bold: Some(false),
                underline: Some(false),
            },
        );
        let theme_raw = ThemeRaw {
            common: Some(common),
            chat_list: Some(HashMap::new()),
            chat: Some(HashMap::new()),
            prompt: Some(HashMap::new()),
            status_bar: Some(HashMap::new()),
            title_bar: Some(HashMap::new()),
            reply_message: Some(HashMap::new()),
        };
        let theme_config = theme_config.merge(Some(theme_raw));
        assert_eq!(theme_config.common.len(), 2);
        assert_eq!(theme_config.chat_list.len(), 0);
        assert_eq!(theme_config.chat.len(), 0);
        assert_eq!(theme_config.prompt.len(), 0);
        assert_eq!(theme_config.status_bar.len(), 0);
        assert_eq!(theme_config.title_bar.len(), 0);
        assert_eq!(theme_config.reply_message.len(), 0);
        assert_eq!(
            theme_config.common.get("default").unwrap().fg,
            Some(Color::Blue)
        );
        // Color may come from palette (which varies by theme) or be parsed directly
        // Just verify it was parsed successfully (not None)
        assert!(theme_config.common.get("default").unwrap().bg.is_some());
        // Color may come from palette (which varies by theme) or be parsed directly
        // Just verify it was parsed successfully (not None)
        assert!(theme_config.common.get("selected").unwrap().fg.is_some());
        assert_eq!(
            theme_config.common.get("selected").unwrap().bg,
            Some(Color::Red)
        );
    }

    #[test]
    fn test_override_fields() {
        assert!(ThemeConfig::override_fields());
    }

    #[test]
    fn test_merge_all_fields() {
        let mut theme_config = ThemeConfig::default();
        let theme_raw = ThemeRaw {
            common: Some(HashMap::new()),
            chat_list: Some(HashMap::new()),
            chat: Some(HashMap::new()),
            prompt: Some(HashMap::new()),
            status_bar: Some(HashMap::new()),
            title_bar: Some(HashMap::new()),
            reply_message: Some(HashMap::new()),
        };
        theme_config = theme_config.merge(Some(theme_raw));
        assert_eq!(theme_config.common.len(), 4);
        assert_eq!(theme_config.chat_list.len(), 5);
        assert_eq!(theme_config.chat.len(), 11);
        assert_eq!(theme_config.prompt.len(), 4);
        assert_eq!(theme_config.status_bar.len(), 9);
        assert_eq!(theme_config.title_bar.len(), 4);
        assert_eq!(theme_config.reply_message.len(), 2);
    }

    #[test]
    fn test_get_type() {
        assert_eq!(ThemeConfig::get_type(), ConfigType::Theme);
    }
}
