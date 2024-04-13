use std::{collections::HashMap, path::Path};

use crate::APP_CONFIG;

use {
    crate::{
        app_error::AppError,
        configs::{
            self, config_file::ConfigFile, config_theme::ThemeStyle, config_type::ConfigType,
            raw::palette_raw::PaletteRaw,
        },
    },
    ratatui::style::Color,
};

#[derive(Clone, Debug)]
/// The palette configuration.
pub struct PaletteConfig {
    /// The palette.
    pub palette: HashMap<String, Color>,
}
/// The palette configuration implementation.
impl PaletteConfig {
    /// Get the default palette configuration.
    ///
    /// # Returns
    /// * `Result<Self>` - The default palette configuration.
    pub fn default_result() -> Result<Self, AppError> {
        configs::deserialize_to_config_into::<PaletteRaw, Self>(Path::new(
            &configs::custom::default_config_palette_file_path()?,
        ))
    }
}
/// The implementation of the configuration file for the palette.
impl ConfigFile for PaletteConfig {
    type Raw = PaletteRaw;

    fn get_type() -> ConfigType {
        ConfigType::Palette
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
                tracing::info!("Merging palette config");
                if let Some(palette) = other.palette {
                    palette.into_iter().for_each(|(k, v)| {
                        self.palette
                            .insert(k, ThemeStyle::str_to_color(&v).unwrap());
                    });
                }
                self.clone()
            }
        }
    }
}
/// The default implementation for the palette configuration.
impl Default for PaletteConfig {
    fn default() -> Self {
        Self::default_result().unwrap()
    }
}
/// The conversion from the raw palette configuration to the palette
/// configuration.
impl From<PaletteRaw> for PaletteConfig {
    fn from(raw: PaletteRaw) -> Self {
        let palette = raw
            .palette
            .unwrap()
            .into_iter()
            .map(|(k, v)| {
                (
                    k,
                    match ThemeStyle::str_to_color(&v) {
                        Ok(color) => color,
                        Err(e) => {
                            eprintln!("In the palette config: {}", e);
                            std::process::exit(1);
                        }
                    },
                )
            })
            .collect();
        Self { palette }
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::configs::{
            config_file::ConfigFile, config_type::ConfigType,
            custom::palette_custom::PaletteConfig, raw::palette_raw::PaletteRaw,
        },
        std::collections::HashMap,
    };

    #[test]
    fn test_palette_config_default() {
        let palette_config = crate::configs::custom::palette_custom::PaletteConfig::default();
        assert_eq!(palette_config.palette.len(), 16);
    }

    #[test]
    fn test_palette_config_from_raw_empty() {
        let raw = crate::configs::raw::palette_raw::PaletteRaw {
            palette: Some(HashMap::new()),
        };
        let palette_config = PaletteConfig::from(raw);
        assert_eq!(palette_config.palette.len(), 0);
    }

    #[test]
    fn test_palette_config_from_raw() {
        let mut palette = HashMap::new();
        palette.insert("black".to_string(), "#000000".parse().unwrap());
        palette.insert("white".to_string(), "#ffffff".parse().unwrap());
        let raw = PaletteRaw {
            palette: Some(palette),
        };
        let palette_config = PaletteConfig::from(raw);
        assert_eq!(palette_config.palette.len(), 2);
        assert_eq!(
            palette_config.palette.get("black").unwrap(),
            &"#000000".parse().unwrap()
        );
        assert_eq!(
            palette_config.palette.get("white").unwrap(),
            &"#ffffff".parse().unwrap()
        );
    }

    #[test]
    fn test_palette_config_merge() {
        let mut palette = HashMap::new();
        palette.insert("black".to_string(), "#000000".parse().unwrap());
        palette.insert("white".to_string(), "#ffffff".parse().unwrap());
        let raw = PaletteRaw {
            palette: Some(palette),
        };
        let palette_config = crate::configs::custom::palette_custom::PaletteConfig::from(raw);
        assert_eq!(palette_config.palette.len(), 2);
        assert_eq!(
            palette_config.palette.get("black").unwrap(),
            &"#000000".parse().unwrap()
        );
        assert_eq!(
            palette_config.palette.get("white").unwrap(),
            &"#ffffff".parse().unwrap()
        );
    }

    #[test]
    fn test_override_fields() {
        assert!(PaletteConfig::override_fields());
    }

    #[test]
    fn test_get_type() {
        assert_eq!(PaletteConfig::get_type(), ConfigType::Palette);
    }
}
