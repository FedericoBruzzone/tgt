use {
    crate::{
        app_error::AppError,
        configs::{
            self,
            config_file::ConfigFile,
            config_type::ConfigType,
            raw::theme_raw::{ThemeEntry, ThemeRaw},
        },
    },
    ratatui::style::{Color, Modifier, Style},
    std::{collections::HashMap, path::Path},
};

#[derive(Clone, Debug)]
pub struct ThemeStyle {
    pub fg: Color,
    pub bg: Color,
    pub modifier: Modifier,
}

impl ThemeStyle {
    pub fn set_bg(mut self, bg: Color) -> Self {
        self.bg = bg;
        self
    }
    pub fn set_fg(mut self, fg: Color) -> Self {
        self.fg = fg;
        self
    }

    pub fn insert(mut self, modifier: Modifier) -> Self {
        self.modifier.insert(modifier);
        self
    }

    pub fn as_style(&self) -> Style {
        Style::from(self)
    }

    pub fn str_to_color(s: &str) -> Result<Color, AppError> {
        match s {
            "black" => Ok(Color::Black),
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "yellow" => Ok(Color::Yellow),
            "blue" => Ok(Color::Blue),
            "magenta" => Ok(Color::Magenta),
            "cyan" => Ok(Color::Cyan),
            "gray" => Ok(Color::Gray),
            "dark_gray" => Ok(Color::DarkGray),
            "light_red" => Ok(Color::LightRed),
            "light_green" => Ok(Color::LightGreen),
            "light_yellow" => Ok(Color::LightYellow),
            "light_blue" => Ok(Color::LightBlue),
            "light_magenta" => Ok(Color::LightMagenta),
            "light_cyan" => Ok(Color::LightCyan),
            "white" => Ok(Color::White),
            "reset" | "" => Ok(Color::Reset),
            s if s.starts_with('#') => {
                let hex = s.trim_start_matches('#');
                match hex.len() {
                    3 => {
                        let r = u8::from_str_radix(&hex[0..1], 16).unwrap();
                        let g = u8::from_str_radix(&hex[1..2], 16).unwrap();
                        let b = u8::from_str_radix(&hex[2..3], 16).unwrap();
                        Ok(Color::Rgb(r, g, b))
                    }
                    6 => {
                        let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
                        let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
                        let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
                        Ok(Color::Rgb(r, g, b))
                    }
                    _ => Err(AppError::InvalidColor(s.to_string())),
                }
            }
            s => {
                if let [r, g, b] = s
                    .split(',')
                    .map(|s| s.trim())
                    .collect::<Vec<&str>>()
                    .as_slice()
                {
                    match (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>()) {
                        (Ok(r), Ok(g), Ok(b)) => {
                            return Ok(Color::Rgb(r, g, b))
                        }
                        _ => return Err(AppError::InvalidColor(s.to_string())),
                    }
                }
                match s.parse::<u8>() {
                    Ok(n) => Ok(Color::Indexed(n)),
                    _ => Err(AppError::InvalidColor(s.to_string())),
                }
            }
        }
    }
}

impl Default for ThemeStyle {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            modifier: Modifier::empty(),
        }
    }
}

impl From<&ThemeStyle> for Style {
    fn from(style: &ThemeStyle) -> Self {
        Self::default()
            .fg(style.fg)
            .bg(style.bg)
            .add_modifier(style.modifier)
    }
}

impl From<ThemeEntry> for ThemeStyle {
    fn from(entry: ThemeEntry) -> Self {
        let fg = Self::str_to_color(&entry.fg.unwrap_or("reset".to_string()));
        let bg = Self::str_to_color(&entry.bg.unwrap_or("reset".to_string()));
        let mut modifier = Modifier::empty();
        modifier.insert(match entry.italic {
            Some(true) => Modifier::ITALIC,
            _ => Modifier::empty(),
        });
        modifier.insert(match entry.bold {
            Some(true) => Modifier::BOLD,
            _ => Modifier::empty(),
        });
        modifier.insert(match entry.underline {
            Some(true) => Modifier::UNDERLINED,
            _ => Modifier::empty(),
        });
        Self {
            fg: fg.unwrap(),
            bg: bg.unwrap(),
            modifier,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ThemeConfig {
    pub palette: HashMap<String, Color>,

    pub status_bar: HashMap<String, ThemeStyle>,
}

impl ThemeConfig {
    pub fn default_result() -> Result<Self, AppError> {
        configs::deserialize_to_config_into::<ThemeRaw, Self>(Path::new(
            &configs::custom::default_config_theme_file_path()?,
        ))
    }
}

impl ConfigFile for ThemeConfig {
    type Raw = ThemeRaw;

    fn get_type() -> ConfigType {
        ConfigType::Theme
    }
    fn override_fields() -> bool {
        true
    }

    fn merge(&mut self, other: Option<Self::Raw>) -> Self {
        match other {
            None => self.clone(),
            Some(other) => {
                tracing::info!("Merging theme config");
                if let Some(palette) = other.palette {
                    palette.into_iter().for_each(|(k, v)| {
                        self.palette
                            .insert(k, ThemeStyle::str_to_color(&v).unwrap());
                    });
                }
                if let Some(status_bar) = other.status_bar {
                    status_bar.into_iter().for_each(|(k, v)| {
                        self.status_bar.insert(k, ThemeStyle::from(v));
                    });
                }
                self.clone()
            }
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self::default_result().unwrap()
    }
}

impl From<ThemeRaw> for ThemeConfig {
    fn from(raw: ThemeRaw) -> Self {
        let status_bar = raw
            .status_bar
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::from(v)))
            .collect();
        let palette = raw
            .palette
            .unwrap()
            .into_iter()
            .map(|(k, v)| (k, ThemeStyle::str_to_color(&v).unwrap()))
            .collect();

        Self {
            palette,
            status_bar,
        }
    }
}
