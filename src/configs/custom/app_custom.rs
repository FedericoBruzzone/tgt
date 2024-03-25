use {
    crate::{
        app_error::AppError,
        configs::{
            self, config_file::ConfigFile, config_type::ConfigType,
            raw::app_raw::AppRaw,
        },
    },
    std::path::Path,
};

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
}
/// The application configuration implementation.
impl AppConfig {
    /// Get the default application configuration.
    ///
    /// # Returns
    /// The default application configuration.
    pub fn default_result() -> Result<Self, AppError> {
        configs::deserialize_to_config_into::<AppRaw, Self>(Path::new(
            &configs::custom::default_config_app_file_path()?,
        ))
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
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::{
        config_file::ConfigFile, custom::app_custom::AppConfig,
        raw::app_raw::AppRaw,
    };

    #[test]
    fn test_app_config_default() {
        let app_config = AppConfig::default();
        assert!(app_config.mouse_support);
        assert!(app_config.paste_support);
        assert_eq!(app_config.frame_rate, 60.0);
    }

    #[test]
    fn test_app_config_from_raw() {
        let app_raw = AppRaw {
            mouse_support: Some(true),
            paste_support: Some(true),
            frame_rate: Some(30.0),
            show_status_bar: Some(true),
            show_title_bar: Some(true),
        };
        let app_config = AppConfig::from(app_raw);
        assert!(app_config.mouse_support);
        assert!(app_config.paste_support);
        assert_eq!(app_config.frame_rate, 30.0);
    }

    #[test]
    fn test_app_config_merge() {
        let mut app_config = AppConfig::from(AppRaw {
            mouse_support: Some(true),
            paste_support: Some(true),
            frame_rate: Some(60.0),
            show_status_bar: Some(true),
            show_title_bar: Some(true),
        });
        let app_raw = AppRaw {
            mouse_support: Some(false),
            paste_support: Some(false),
            frame_rate: None,
            show_status_bar: None,
            show_title_bar: None,
        };
        app_config = app_config.merge(Some(app_raw));
        assert!(!app_config.mouse_support);
        assert!(!app_config.paste_support);
        assert_eq!(app_config.frame_rate, 60.0);
    }

    #[test]
    fn test_app_config_override_fields() {
        assert!(AppConfig::override_fields());
    }
}
