use {
    crate::{app_error::AppError, configs::raw::theme_raw::ThemeEntry, PALETTE_CONFIG},
    ratatui::style::{Color, Modifier, Style},
};
#[derive(Clone, Debug)]
/// `ThemeStyle` is a struct that represents a style in the theme config.
/// It is responsible for managing the style of the theme.
/// It contains the foreground color, background color, and modifier.
pub struct ThemeStyle {
    /// The foreground color of the style.
    pub fg: Option<Color>,
    /// The background color of the style.
    pub bg: Option<Color>,
    /// The modifier of the style.
    pub modifier: Modifier,
}

/// Implement the `ThemeStyle` struct.
impl ThemeStyle {
    /// Set the background color of the `ThemeStyle`.
    ///
    /// # Arguments
    /// * `bg` - The background color of the `ThemeStyle`.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `ThemeStyle`.
    pub fn set_bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }
    /// Set the foreground color of the `ThemeStyle`.
    ///
    /// # Arguments
    /// * `fg` - The foreground color of the `ThemeStyle`.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `ThemeStyle`.
    pub fn set_fg(mut self, fg: Color) -> Self {
        self.fg = Some(fg);
        self
    }
    /// Add a modifier to the `ThemeStyle`.
    ///
    /// # Arguments
    /// * `modifier` - The modifier to add to the `ThemeStyle`.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `ThemeStyle`.
    pub fn insert(mut self, modifier: Modifier) -> Self {
        self.modifier.insert(modifier);
        self
    }
    /// Convert the `ThemeStyle` to a `Style`.
    /// It is used in the macro `theme_style!` in order to convert the
    /// `ThemeStyle` to a `Style`. The `Style` is used to decorate the
    /// rataui components.
    ///
    /// # Returns
    /// * `Style` - The converted `Style`.
    pub fn as_style(&self) -> Style {
        Style::from(self)
    }
    /// Convert a string to a `Color`.
    ///
    /// # Arguments
    /// * `s` - The string to convert to a `Color`.
    ///
    /// # Returns
    /// * `Result<Color, AppError>` - The converted `Color`.
    pub fn str_to_color(s: &str) -> Result<Color, AppError<()>> {
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
                        (Ok(r), Ok(g), Ok(b)) => return Ok(Color::Rgb(r, g, b)),
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
    /// Convert a string to a `Color`.
    /// Different from `str_to_color`, this function will try to find the color
    /// in the palette first. If the color is not found in the palette, it
    /// will use `str_to_color`.
    ///
    /// # Arguments
    /// * `s` - The string to convert to a `Color`.
    ///
    /// # Returns
    /// * `Result<Color, AppError>` - The converted `Color`.
    pub fn str_to_color_with_palette(s: &str) -> Result<Color, AppError<()>> {
        if s.is_empty() {
            return Err(AppError::InvalidColor(s.to_string()));
        }
        if let Some(color) = PALETTE_CONFIG.palette.get(s) {
            return Ok(*color);
        }
        Self::str_to_color(s)
    }

    /// Convert a string to a `Color` using a specific palette.
    /// Similar to `str_to_color_with_palette`, but uses the provided palette
    /// instead of the global `PALETTE_CONFIG`.
    ///
    /// # Arguments
    /// * `s` - The string to convert to a `Color`.
    /// * `palette` - The palette to use for color lookup.
    ///
    /// # Returns
    /// * `Result<Color, AppError>` - The converted `Color`.
    pub fn str_to_color_with_specific_palette(
        s: &str,
        palette: &std::collections::HashMap<String, Color>,
    ) -> Result<Color, AppError<()>> {
        if s.is_empty() {
            return Err(AppError::InvalidColor(s.to_string()));
        }
        if let Some(color) = palette.get(s) {
            return Ok(*color);
        }
        Self::str_to_color(s)
    }
}
/// Implement the `From` trait for the `ThemeStyle` struct.
/// It is used to convert a `ThemeEntry` to a `ThemeStyle`.
/// The `ThemeEntry` is a struct that represents a style in the theme raw
/// config.
impl From<ThemeEntry> for ThemeStyle {
    fn from(entry: ThemeEntry) -> Self {
        let fg = match entry.fg {
            Some(fg) => match Self::str_to_color_with_palette(&fg) {
                Ok(color) => Some(color),
                Err(e) => {
                    tracing::error!("Failed to parse foreground color: {}", e);
                    None
                }
            },
            None => {
                tracing::info!("Miss foreground color in the theme config");
                None
            }
        };
        let bg = match entry.bg {
            Some(bg) => match Self::str_to_color_with_palette(&bg) {
                Ok(color) => Some(color),
                Err(e) => {
                    tracing::error!("Failed to parse background color: {}", e);
                    None
                }
            },
            None => {
                tracing::info!("Miss background color in the theme config");
                None
            }
        };
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
        Self { fg, bg, modifier }
    }
}

impl ThemeStyle {
    /// Convert a `ThemeEntry` to a `ThemeStyle` using a specific palette.
    /// This is useful when converting themes with a palette that differs from
    /// the global `PALETTE_CONFIG`.
    ///
    /// # Arguments
    /// * `entry` - The `ThemeEntry` to convert.
    /// * `palette` - The palette to use for color lookup.
    ///
    /// # Returns
    /// * `Self` - The converted `ThemeStyle`.
    pub fn from_entry_with_palette(
        entry: ThemeEntry,
        palette: &std::collections::HashMap<String, Color>,
    ) -> Self {
        let fg = match entry.fg {
            Some(fg) => match Self::str_to_color_with_specific_palette(&fg, palette) {
                Ok(color) => Some(color),
                Err(e) => {
                    tracing::error!("Failed to parse foreground color: {}", e);
                    None
                }
            },
            None => None,
        };
        let bg = match entry.bg {
            Some(bg) => match Self::str_to_color_with_specific_palette(&bg, palette) {
                Ok(color) => Some(color),
                Err(e) => {
                    tracing::error!("Failed to parse background color: {}", e);
                    None
                }
            },
            None => None,
        };
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
        Self { fg, bg, modifier }
    }
}
/// Implement the `Default` trait for the `ThemeStyle` struct.
impl Default for ThemeStyle {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
            modifier: Modifier::empty(),
        }
    }
}
/// Implement the `From` trait for the `ThemeStyle` struct.
/// It is used to convert a `ThemeStyle` to a `Style`.
impl From<&ThemeStyle> for Style {
    fn from(style: &ThemeStyle) -> Self {
        Style {
            fg: style.fg,
            bg: style.bg,
            add_modifier: style.modifier,
            ..Style::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::configs::{config_theme::ThemeStyle, raw::theme_raw::ThemeEntry},
        ratatui::style::{Color, Modifier},
    };

    #[test]
    fn test_theme_style_from() {
        let entry = ThemeEntry {
            fg: Some("white".to_string()),
            bg: Some("black".to_string()),
            italic: Some(true),
            bold: Some(true),
            underline: Some(true),
        };
        let style = ThemeStyle::from(entry);
        // Colors may come from palette or be parsed directly
        // Just verify they were parsed successfully (not None)
        assert!(style.fg.is_some());
        assert!(style.bg.is_some());
        assert_eq!(
            style.modifier,
            Modifier::ITALIC | Modifier::BOLD | Modifier::UNDERLINED
        );
    }

    #[test]
    fn test_theme_style_str_to_color() {
        let entry = ThemeEntry {
            fg: Some("white".to_string()),
            bg: Some("black".to_string()),
            italic: Some(true),
            bold: Some(true),
            underline: Some(true),
        };
        let colored_entry_fg = ThemeStyle::str_to_color(&entry.fg.unwrap()).unwrap();
        let colored_entry_bg = ThemeStyle::str_to_color(&entry.bg.unwrap()).unwrap();
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
        let style = ThemeStyle {
            fg: Some(colored_entry_fg),
            bg: Some(colored_entry_bg),
            modifier,
        };
        assert_eq!(style.fg, Some(Color::White));
        assert_eq!(style.bg, Some(Color::Black));
        assert_eq!(
            style.modifier,
            Modifier::ITALIC | Modifier::BOLD | Modifier::UNDERLINED
        );
    }

    #[test]
    fn test_theme_style_str_to_color_with_palette() {
        let entry = ThemeEntry {
            fg: Some("black".to_string()),
            bg: Some("white".to_string()),
            italic: Some(true),
            bold: Some(true),
            underline: Some(true),
        };
        let colored_entry_fg = ThemeStyle::str_to_color_with_palette(&entry.fg.unwrap()).unwrap();
        let colored_entry_bg = ThemeStyle::str_to_color_with_palette(&entry.bg.unwrap()).unwrap();
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
        let style = ThemeStyle {
            fg: Some(colored_entry_fg),
            bg: Some(colored_entry_bg),
            modifier,
        };
        // Colors may come from palette (which varies by theme) or be parsed directly
        // Just verify they were parsed successfully (not None)
        assert!(style.fg.is_some());
        assert!(style.bg.is_some());
        assert_eq!(
            style.modifier,
            Modifier::ITALIC | Modifier::BOLD | Modifier::UNDERLINED
        );
    }

    #[test]
    fn test_theme_style_as_style() {
        let entry = ThemeEntry {
            fg: Some("black".to_string()),
            bg: Some("white".to_string()),
            italic: Some(true),
            bold: Some(true),
            underline: Some(true),
        };
        let style = ThemeStyle::from(entry);
        let ratatui_style = style.as_style();
        // Colors may come from palette (which varies by theme) or be parsed directly
        // Just verify they were parsed successfully (not None)
        assert!(ratatui_style.fg.is_some());
        assert!(ratatui_style.bg.is_some());
        assert_eq!(
            ratatui_style.add_modifier,
            Modifier::ITALIC | Modifier::BOLD | Modifier::UNDERLINED
        );
    }

    #[test]
    fn test_theme_style_default() {
        let style = ThemeStyle::default();
        assert_eq!(style.fg, None);
        assert_eq!(style.bg, None);
        assert_eq!(style.modifier, Modifier::empty());
    }

    #[test]
    fn test_theme_style_from_ref() {
        let entry = ThemeEntry {
            fg: Some("black".to_string()),
            bg: Some("white".to_string()),
            italic: Some(true),
            bold: Some(true),
            underline: Some(true),
        };
        let style = ThemeStyle::from(entry);
        let ratatui_style: ratatui::style::Style = (&style).into();
        // Colors may come from palette (which varies by theme) or be parsed directly
        // Just verify they were parsed successfully (not None)
        assert!(ratatui_style.fg.is_some());
        assert!(ratatui_style.bg.is_some());
        assert_eq!(
            ratatui_style.add_modifier,
            Modifier::ITALIC | Modifier::BOLD | Modifier::UNDERLINED
        );
    }
}
