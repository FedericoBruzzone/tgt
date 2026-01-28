use crate::{
    app_error::AppError,
    configs::{self, config_file::ConfigFile, config_type::ConfigType, raw::app_raw::AppRaw},
};
use std::{fs, path::Path};

#[derive(Clone, Debug)]
/// The application configuration.
pub struct AppConfig {
    /// The mouse support.
    pub mouse_support: bool,
    /// The paste support.
    pub paste_support: bool,
    /// The frame rate.
    pub frame_rate: f64,
    /// The status bar visibility.
    pub show_status_bar: bool,
    /// The title bar visibility.
    pub show_title_bar: bool,
    /// Enable the theme.
    pub theme_enable: bool,
    /// The theme filename.
    pub theme_filename: String,
    /// Take the API ID from the Telegram configuration.
    pub take_api_id_from_telegram_config: bool,
    /// Take the API HASH from the Telegram configuration.
    pub take_api_hash_from_telegram_config: bool,
}
/// The application configuration implementation.
impl AppConfig {
    /// Get the default application configuration.
    ///
    /// # Returns
    /// The default application configuration.
    pub fn default_result() -> Result<Self, AppError<()>> {
        configs::deserialize_to_config_into::<AppRaw, Self>(Path::new(
            &configs::custom::default_config_app_file_path()?,
        ))
    }

    /// Save the application configuration to disk.
    /// This function searches for the config file location (using the same logic as loading)
    /// and saves the configuration there. If no existing config file is found, it saves to
    /// the default location in the user's config directory.
    ///
    /// # Returns
    /// `Ok(())` if the configuration was saved successfully, or an error if it failed.
    pub fn save(&self) -> Result<(), AppError<()>> {
        use crate::configs::config_file::ConfigFile;

        // Try to find existing config file location
        let config_path = if let Some(existing_path) = Self::search_config_file("app.toml") {
            existing_path
        } else {
            // If no existing config found, use the default location
            // This will be in the user's config directory (e.g., ~/.config/tgt/config/app.toml)
            let default_path = configs::custom::default_config_app_file_path().map_err(|e| {
                AppError::InvalidAction(format!("Failed to get config path: {}", e))
            })?;
            Path::new(&default_path).to_path_buf()
        };

        // Ensure the directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::InvalidAction(format!("Failed to create config directory: {}", e))
            })?;
        }

        // Convert AppConfig to AppRaw for serialization
        let raw: AppRaw = self.clone().into();

        // Serialize to TOML
        let toml_string = toml::to_string_pretty(&raw).map_err(|e| {
            AppError::InvalidAction(format!("Failed to serialize config to TOML: {}", e))
        })?;

        // Write to file
        fs::write(&config_path, toml_string)
            .map_err(|e| AppError::InvalidAction(format!("Failed to write config file: {}", e)))?;

        tracing::info!("Saved app config to {}", config_path.display());
        Ok(())
    }
}
/// The implementation of the configuration file for the application.
impl ConfigFile for AppConfig {
    type Raw = AppRaw;

    fn get_type() -> ConfigType {
        ConfigType::App
    }

    fn override_fields() -> bool {
        true
    }

    fn merge(&mut self, other: Option<Self::Raw>) -> Self {
        match other {
            None => self.clone(),
            Some(other) => {
                tracing::info!("Merging app config");
                if let Some(mouse_support) = other.mouse_support {
                    self.mouse_support = mouse_support;
                }
                if let Some(paste_support) = other.paste_support {
                    self.paste_support = paste_support;
                }
                if let Some(frame_rate) = other.frame_rate {
                    self.frame_rate = frame_rate;
                }
                if let Some(show_status_bar) = other.show_status_bar {
                    self.show_status_bar = show_status_bar;
                }
                if let Some(show_title_bar) = other.show_title_bar {
                    self.show_title_bar = show_title_bar;
                }
                if let Some(theme_enable) = other.theme_enable {
                    self.theme_enable = theme_enable;
                }
                if let Some(theme_filename) = other.theme_filename {
                    self.theme_filename = theme_filename;
                }
                if let Some(take_api_id_from_telegram_config) =
                    other.take_api_id_from_telegram_config
                {
                    self.take_api_id_from_telegram_config = take_api_id_from_telegram_config;
                }
                if let Some(take_api_hash_from_telegram_config) =
                    other.take_api_hash_from_telegram_config
                {
                    self.take_api_hash_from_telegram_config = take_api_hash_from_telegram_config;
                }
                self.clone()
            }
        }
    }
}
/// The default application configuration.
impl Default for AppConfig {
    fn default() -> Self {
        Self::default_result().unwrap()
    }
}
/// The conversion from the raw logger configuration to the logger
/// configuration.
impl From<AppRaw> for AppConfig {
    fn from(raw: AppRaw) -> Self {
        Self {
            mouse_support: raw.mouse_support.unwrap(),
            paste_support: raw.paste_support.unwrap(),
            frame_rate: raw.frame_rate.unwrap(),
            show_status_bar: raw.show_status_bar.unwrap(),
            show_title_bar: raw.show_title_bar.unwrap(),
            theme_enable: raw.theme_enable.unwrap(),
            theme_filename: raw.theme_filename.unwrap(),
            take_api_id_from_telegram_config: raw.take_api_id_from_telegram_config.unwrap(),
            take_api_hash_from_telegram_config: raw.take_api_hash_from_telegram_config.unwrap(),
        }
    }
}

/// The conversion from the application configuration to the raw application
/// configuration. This is used when saving the configuration to disk.
impl From<AppConfig> for AppRaw {
    fn from(config: AppConfig) -> Self {
        Self {
            mouse_support: Some(config.mouse_support),
            paste_support: Some(config.paste_support),
            frame_rate: Some(config.frame_rate),
            show_status_bar: Some(config.show_status_bar),
            show_title_bar: Some(config.show_title_bar),
            theme_enable: Some(config.theme_enable),
            theme_filename: Some(config.theme_filename),
            take_api_id_from_telegram_config: Some(config.take_api_id_from_telegram_config),
            take_api_hash_from_telegram_config: Some(config.take_api_hash_from_telegram_config),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::{
        config_file::ConfigFile, custom::app_custom::AppConfig, raw::app_raw::AppRaw,
    };

    #[test]
    fn test_app_config_default() {
        let app_config = AppConfig::default();
        assert!(app_config.mouse_support);
        assert!(app_config.paste_support);
        assert_eq!(app_config.frame_rate, 60.0);
        assert!(app_config.show_status_bar);
        assert!(app_config.show_title_bar);
        assert!(app_config.theme_enable);
        // theme_filename comes from the config file, which may vary
        assert!(!app_config.theme_filename.is_empty());
        assert!(app_config.take_api_id_from_telegram_config);
        assert!(app_config.take_api_hash_from_telegram_config);
    }

    #[test]
    fn test_app_config_from_raw() {
        let app_raw = AppRaw {
            mouse_support: Some(true),
            paste_support: Some(true),
            frame_rate: Some(30.0),
            show_status_bar: Some(true),
            show_title_bar: Some(true),
            theme_enable: Some(true),
            theme_filename: Some("test".to_string()),
            take_api_id_from_telegram_config: Some(true),
            take_api_hash_from_telegram_config: Some(true),
        };
        let app_config = AppConfig::from(app_raw);
        assert!(app_config.mouse_support);
        assert!(app_config.paste_support);
        assert_eq!(app_config.frame_rate, 30.0);
        assert!(app_config.show_status_bar);
        assert!(app_config.show_title_bar);
        assert!(app_config.theme_enable);
        assert_eq!(app_config.theme_filename, "test");
    }

    #[test]
    fn test_app_config_merge() {
        let mut app_config = AppConfig::from(AppRaw {
            mouse_support: Some(true),
            paste_support: Some(true),
            frame_rate: Some(60.0),
            show_status_bar: Some(true),
            show_title_bar: Some(true),
            theme_enable: Some(true),
            theme_filename: Some("test".to_string()),
            take_api_id_from_telegram_config: Some(true),
            take_api_hash_from_telegram_config: Some(true),
        });
        let app_raw = AppRaw {
            mouse_support: Some(false),
            paste_support: Some(false),
            frame_rate: None,
            show_status_bar: None,
            show_title_bar: None,
            theme_enable: None,
            theme_filename: None,
            take_api_id_from_telegram_config: None,
            take_api_hash_from_telegram_config: None,
        };
        app_config = app_config.merge(Some(app_raw));
        assert!(!app_config.mouse_support);
        assert!(!app_config.paste_support);
        assert_eq!(app_config.frame_rate, 60.0);
        assert!(app_config.show_status_bar);
        assert!(app_config.show_title_bar);
        assert!(app_config.theme_enable);
        assert_eq!(app_config.theme_filename, "test");
    }

    #[test]
    fn test_app_config_override_fields() {
        assert!(AppConfig::override_fields());
    }

    #[test]
    fn test_merge_all_fields() {
        let mut app_config = AppConfig::default();
        let app_raw = AppRaw {
            mouse_support: None,
            paste_support: None,
            frame_rate: None,
            show_status_bar: None,
            show_title_bar: None,
            theme_enable: None,
            theme_filename: None,
            take_api_id_from_telegram_config: None,
            take_api_hash_from_telegram_config: None,
        };
        app_config = app_config.merge(Some(app_raw));
        assert!(app_config.mouse_support);
        assert!(app_config.paste_support);
        assert_eq!(app_config.frame_rate, 60.0);
        assert!(app_config.show_status_bar);
        assert!(app_config.show_title_bar);
        assert!(app_config.theme_enable);
        // theme_filename comes from the config file, which may vary
        assert!(!app_config.theme_filename.is_empty());
        assert!(app_config.take_api_id_from_telegram_config);
        assert!(app_config.take_api_hash_from_telegram_config);
    }

    #[test]
    fn test_get_type() {
        assert_eq!(
            AppConfig::get_type(),
            crate::configs::config_type::ConfigType::App
        );
    }
}
