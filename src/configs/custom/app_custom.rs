use crate::{
    app_error::AppError,
    configs::{self, config_file::ConfigFile, config_type::ConfigType, raw::app_raw::AppRaw},
};
use std::{fs, path::Path, path::PathBuf};

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
    /// This function saves the configuration to the user's config directory only,
    /// skipping any repo config directories (like ./config/ in debug mode).
    /// This ensures that theme switches during development don't override the repo's
    /// default config files.
    ///
    /// Precedence order for saving:
    /// 1. User config directory (~/.config/tgt/config/app.toml or ~/.tgt/config/app.toml)
    /// 2. If no user config exists, create it in the user's config directory
    ///
    /// # Returns
    /// `Ok(())` if the configuration was saved successfully, or an error if it failed.
    pub fn save(&self) -> Result<(), AppError<()>> {
        use crate::utils::{TGT, TGT_CONFIG_DIR};
        use std::env;

        // Find user config directory (skip repo config directories)
        // Priority: TGT_CONFIG_DIR > ~/.config/tgt/config > ~/.tgt/config
        let user_config_path = if let Ok(config_dir_str) = env::var(TGT_CONFIG_DIR) {
            // Use TGT_CONFIG_DIR if set - this is the highest priority
            let config_dir = PathBuf::from(config_dir_str);
            // TGT_CONFIG_DIR can point to the config directory directly
            // or to a parent directory (e.g., ~/.config/tgt)
            if config_dir.join("app.toml").exists() {
                // Direct config directory with existing app.toml
                Some(config_dir.join("app.toml"))
            } else if config_dir.join("config").join("app.toml").exists() {
                // Parent directory with config subdirectory and existing app.toml
                Some(config_dir.join("config").join("app.toml"))
            } else if config_dir.is_dir() {
                // Directory exists - use it as config directory (will create app.toml)
                Some(config_dir.join("app.toml"))
            } else {
                // Directory doesn't exist - try to use it anyway (will be created)
                Some(config_dir.join("app.toml"))
            }
        } else {
            // Use standard user config directory
            if let Some(user_config_dir) = if cfg!(target_os = "macos") {
                dirs::home_dir().map(|h| h.join(".config"))
            } else {
                dirs::config_dir()
            } {
                // Try ~/.config/tgt/config/app.toml first
                let tgt_config = user_config_dir.join(TGT).join("config");
                if tgt_config.is_dir() || tgt_config.join("app.toml").exists() {
                    Some(tgt_config.join("app.toml"))
                } else {
                    // Fallback to ~/.tgt/config/app.toml
                    if let Some(home) = dirs::home_dir() {
                        let tgt_dir = home.join(format!(".{}", TGT));
                        if tgt_dir.join("config").is_dir()
                            || tgt_dir.join("config").join("app.toml").exists()
                        {
                            Some(tgt_dir.join("config").join("app.toml"))
                        } else {
                            // Create ~/.config/tgt/config/app.toml as default
                            Some(tgt_config.join("app.toml"))
                        }
                    } else {
                        None
                    }
                }
            } else {
                None
            }
        };

        let config_path = user_config_path.ok_or_else(|| {
            AppError::InvalidAction("Failed to determine user config directory".to_string())
        })?;

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
    use std::sync::Mutex;

    // Mutex to serialize tests that modify TGT_CONFIG_DIR environment variable
    // This prevents race conditions when tests run in parallel
    static ENV_LOCK: Mutex<()> = Mutex::new(());

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

    #[test]
    fn test_save_creates_user_config_directory() {
        use std::env;
        use tempfile::TempDir;

        // Acquire lock to prevent other tests from modifying TGT_CONFIG_DIR simultaneously
        let _guard = ENV_LOCK.lock().unwrap();

        // Save original value to restore later
        let original_value = env::var("TGT_CONFIG_DIR").ok();

        // Create a temporary directory to simulate user config directory
        let temp_dir = TempDir::new().unwrap();
        let temp_config_dir = temp_dir.path().join("tgt").join("config");

        // Set TGT_CONFIG_DIR to point to the config directory itself
        env::set_var("TGT_CONFIG_DIR", temp_config_dir.to_string_lossy().as_ref());

        let app_config = AppConfig::default();

        // Save should succeed
        let result = app_config.save();
        assert!(result.is_ok(), "Save should succeed: {:?}", result);

        // Verify file was created in user config directory (not repo config)
        let expected_path = temp_config_dir.join("app.toml");
        assert!(
            expected_path.exists(),
            "Config file should be created in user config directory"
        );

        // Clean up - restore original value
        match original_value {
            Some(val) => env::set_var("TGT_CONFIG_DIR", val),
            None => env::remove_var("TGT_CONFIG_DIR"),
        }
    }

    #[test]
    fn test_save_skips_repo_config_directory() {
        use std::env;
        use std::fs;
        use tempfile::TempDir;

        // Acquire lock to prevent other tests from modifying TGT_CONFIG_DIR simultaneously
        let _guard = ENV_LOCK.lock().unwrap();

        // Save original value to restore later
        let original_value = env::var("TGT_CONFIG_DIR").ok();

        // Create temp directories for both repo and user config
        let temp_base = TempDir::new().unwrap();
        let repo_config_dir = temp_base.path().join("repo_config");
        let user_config_dir = temp_base
            .path()
            .join("user_config")
            .join("tgt")
            .join("config");

        // Create repo config directory with an existing app.toml
        fs::create_dir_all(&repo_config_dir).unwrap();
        fs::write(
            repo_config_dir.join("app.toml"),
            "theme_filename = \"themes/repo_theme.toml\"\n",
        )
        .unwrap();

        // Set TGT_CONFIG_DIR to point to user config directory directly
        env::set_var("TGT_CONFIG_DIR", user_config_dir.to_string_lossy().as_ref());

        let app_config = AppConfig {
            theme_filename: "themes/user_theme.toml".to_string(),
            ..AppConfig::default()
        };

        // Save should write to user config directory, not repo config directory
        let result = app_config.save();
        assert!(result.is_ok(), "Save should succeed: {:?}", result);

        // Verify file was created in user config directory
        let user_config_file = user_config_dir.join("app.toml");
        assert!(
            user_config_file.exists(),
            "Config should be saved to user config directory"
        );

        // Verify repo config was not modified
        let repo_config_content = fs::read_to_string(repo_config_dir.join("app.toml")).unwrap();
        assert!(
            repo_config_content.contains("repo_theme"),
            "Repo config should not be modified"
        );

        // Verify user config has correct content
        let user_config_content = fs::read_to_string(&user_config_file).unwrap();
        assert!(
            user_config_content.contains("user_theme"),
            "User config should have correct theme"
        );

        // Clean up - restore original value
        match original_value {
            Some(val) => env::set_var("TGT_CONFIG_DIR", val),
            None => env::remove_var("TGT_CONFIG_DIR"),
        }
    }

    #[test]
    fn test_save_serializes_all_fields() {
        use std::env;
        use std::fs;
        use tempfile::TempDir;

        // Acquire lock to prevent other tests from modifying TGT_CONFIG_DIR simultaneously
        let _guard = ENV_LOCK.lock().unwrap();

        // Save original value to restore later
        let original_value = env::var("TGT_CONFIG_DIR").ok();

        let temp_dir = TempDir::new().unwrap();
        let temp_config_dir = temp_dir.path().join("tgt").join("config");
        env::set_var("TGT_CONFIG_DIR", temp_config_dir.to_string_lossy().as_ref());

        let app_config = AppConfig {
            mouse_support: false,
            paste_support: false,
            frame_rate: 30.0,
            show_status_bar: false,
            show_title_bar: false,
            theme_enable: false,
            theme_filename: "themes/test.toml".to_string(),
            take_api_id_from_telegram_config: false,
            take_api_hash_from_telegram_config: false,
        };

        let result = app_config.save();
        assert!(result.is_ok(), "Save should succeed");

        // Read back and verify all fields are present
        let saved_content = fs::read_to_string(temp_config_dir.join("app.toml")).unwrap();
        assert!(saved_content.contains("mouse_support"));
        assert!(saved_content.contains("paste_support"));
        assert!(saved_content.contains("frame_rate"));
        assert!(saved_content.contains("show_status_bar"));
        assert!(saved_content.contains("show_title_bar"));
        assert!(saved_content.contains("theme_enable"));
        assert!(saved_content.contains("theme_filename"));
        assert!(saved_content.contains("take_api_id_from_telegram_config"));
        assert!(saved_content.contains("take_api_hash_from_telegram_config"));

        // Clean up - restore original value
        match original_value {
            Some(val) => env::set_var("TGT_CONFIG_DIR", val),
            None => env::remove_var("TGT_CONFIG_DIR"),
        }
    }

    #[test]
    fn test_save_handles_missing_directory() {
        use std::env;
        use tempfile::TempDir;

        // Acquire lock to prevent other tests from modifying TGT_CONFIG_DIR simultaneously
        let _guard = ENV_LOCK.lock().unwrap();

        // Save original value to restore later
        let original_value = env::var("TGT_CONFIG_DIR").ok();

        let temp_dir = TempDir::new().unwrap();
        let temp_config_dir = temp_dir.path().join("nonexistent");
        env::set_var("TGT_CONFIG_DIR", temp_config_dir.to_string_lossy().as_ref());

        let app_config = AppConfig::default();

        // Save should create the directory structure
        let result = app_config.save();
        assert!(
            result.is_ok(),
            "Save should create missing directories: {:?}",
            result
        );
        assert!(
            temp_config_dir.exists(),
            "Config directory should be created"
        );
        assert!(
            temp_config_dir.join("app.toml").exists(),
            "Config file should be created"
        );

        // Clean up - restore original value
        match original_value {
            Some(val) => env::set_var("TGT_CONFIG_DIR", val),
            None => env::remove_var("TGT_CONFIG_DIR"),
        }
    }
}
