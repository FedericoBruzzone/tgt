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
}
/// The theme configuration implementation.
impl ThemeConfig {
    /// Get the default theme configuration.
    ///
    /// # Returns
    /// * `Result<Self>` - The default theme configuration.
    pub fn default_result() -> Result<Self, AppError<()>> {
        configs::deserialize_to_config_into::<ThemeRaw, Self>(Path::new(
            &configs::custom::default_config_theme_file_path()?,
        ))
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

    // We need to override the default implementation of the get_config function
    // to use the theme_filename from the app config.
    // The default value of theme_filename is "theme.toml".
    fn get_config() -> Self {
        if Self::override_fields() {
            let mut default = Self::default();
            default.merge(Self::deserialize_custom_config::<Self::Raw>(
                // Self::get_type().as_default_filename().as_str(),
                &APP_CONFIG.theme_filename,
            ))
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

        Self {
            common,
            chat_list,
            chat,
            prompt,
            status_bar,
            title_bar,
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
        assert_eq!(theme_config.common.len(), 2);
        assert_eq!(theme_config.chat_list.len(), 5);
        assert_eq!(theme_config.chat.len(), 3);
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
        };
        let theme_config = ThemeConfig::from(theme_raw);
        assert_eq!(theme_config.common.len(), 2);
        assert_eq!(theme_config.chat_list.len(), 0);
        assert_eq!(theme_config.chat.len(), 0);
        assert_eq!(theme_config.prompt.len(), 0);
        assert_eq!(theme_config.status_bar.len(), 0);
        assert_eq!(theme_config.title_bar.len(), 0);
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
        };
        let theme_config = theme_config.merge(Some(theme_raw));
        assert_eq!(theme_config.common.len(), 2);
        assert_eq!(theme_config.chat_list.len(), 0);
        assert_eq!(theme_config.chat.len(), 0);
        assert_eq!(theme_config.prompt.len(), 0);
        assert_eq!(theme_config.status_bar.len(), 0);
        assert_eq!(theme_config.title_bar.len(), 0);
        assert_eq!(
            theme_config.common.get("default").unwrap().fg,
            Some(Color::Blue)
        );
        assert_eq!(
            theme_config.common.get("default").unwrap().bg,
            Some(Color::Rgb(255, 255, 255))
        );
        assert_eq!(
            theme_config.common.get("selected").unwrap().fg,
            Some(Color::Rgb(0, 0, 0))
        );
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
        };
        theme_config = theme_config.merge(Some(theme_raw));
        assert_eq!(theme_config.common.len(), 2);
        assert_eq!(theme_config.chat_list.len(), 5);
        assert_eq!(theme_config.chat.len(), 3);
        assert_eq!(theme_config.prompt.len(), 4);
        assert_eq!(theme_config.status_bar.len(), 9);
        assert_eq!(theme_config.title_bar.len(), 4);
    }

    #[test]
    fn test_get_type() {
        assert_eq!(ThemeConfig::get_type(), ConfigType::Theme);
    }
}
