use std::{collections::HashMap, path::Path};

use {
    crate::{
        app_error::AppError,
        configs::{
            self, config_file::ConfigFile, config_theme::ThemeStyle,
            config_type::ConfigType, raw::palette_raw::PaletteRaw,
        },
    },
    ratatui::style::Color,
};

#[derive(Clone, Debug)]
pub struct PaletteConfig {
    pub palette: HashMap<String, Color>,
}

impl PaletteConfig {
    pub fn default_result() -> Result<Self, AppError> {
        configs::deserialize_to_config_into::<PaletteRaw, Self>(Path::new(
            &configs::custom::default_config_palette_file_path()?,
        ))
    }
}

impl Default for PaletteConfig {
    fn default() -> Self {
        Self::default_result().unwrap()
    }
}

impl ConfigFile for PaletteConfig {
    type Raw = PaletteRaw;

    fn get_type() -> ConfigType {
        ConfigType::Palette
    }

    fn override_fields() -> bool {
        true
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
