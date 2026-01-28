use crate::{
    app_context::AppContext,
    app_error::AppError,
    configs::{
        config_file::ConfigFile,
        custom::{palette_custom::PaletteConfig, theme_custom::ThemeConfig},
        raw::{palette_raw::PaletteRaw, theme_raw::ThemeRaw},
    },
};
use std::sync::Arc;

/// List of available VSCode themes
pub const AVAILABLE_THEMES: &[&str] = &[
    "theme", // Default theme
    "monokai",
    "catppuccin",
    "github",
    "tokyo",
    "nord",
    "gruvbox",
    "onedark",
];

/// Theme switcher that manages theme changes
pub struct ThemeSwitcher {
    /// Current theme index in the available themes list
    current_theme_index: usize,
    /// Available theme names
    themes: Vec<String>,
}

impl ThemeSwitcher {
    /// Create a new theme switcher with the default themes
    pub fn new() -> Self {
        Self {
            current_theme_index: 0,
            themes: AVAILABLE_THEMES.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Create a new theme switcher with custom themes
    pub fn with_themes(themes: Vec<String>) -> Self {
        Self {
            current_theme_index: 0,
            themes,
        }
    }

    /// Get the current theme name
    pub fn current_theme(&self) -> Option<&str> {
        self.themes
            .get(self.current_theme_index)
            .map(|s| s.as_str())
    }

    /// Get all available themes
    pub fn available_themes(&self) -> &[String] {
        &self.themes
    }

    /// Switch to the next theme in the list
    pub fn next_theme(&mut self) -> Option<&str> {
        if self.themes.is_empty() {
            return None;
        }
        self.current_theme_index = (self.current_theme_index + 1) % self.themes.len();
        self.current_theme()
    }

    /// Switch to the previous theme in the list
    pub fn previous_theme(&mut self) -> Option<&str> {
        if self.themes.is_empty() {
            return None;
        }
        if self.current_theme_index == 0 {
            self.current_theme_index = self.themes.len() - 1;
        } else {
            self.current_theme_index -= 1;
        }
        self.current_theme()
    }

    /// Switch to a specific theme by name
    pub fn switch_to_theme(&mut self, theme_name: &str) -> Result<(), AppError<()>> {
        if let Some(index) = self.themes.iter().position(|t| t == theme_name) {
            self.current_theme_index = index;
            Ok(())
        } else {
            Err(AppError::InvalidAction(format!(
                "Theme '{}' not found",
                theme_name
            )))
        }
    }

    /// Apply the current theme to the app context
    pub fn apply_current_theme(&self, app_context: &Arc<AppContext>) -> Result<(), AppError<()>> {
        let theme_name = self
            .current_theme()
            .ok_or_else(|| AppError::InvalidAction("No theme available".to_string()))?;
        Self::apply_theme(app_context, theme_name)
    }

    /// Apply a specific theme to the app context
    pub fn apply_theme(
        app_context: &Arc<AppContext>,
        theme_name: &str,
    ) -> Result<(), AppError<()>> {
        // Update the theme filename in app config
        // Themes are stored in the themes/ subdirectory
        let theme_filename = format!("themes/{}.toml", theme_name);
        let app_config_to_save = {
            let mut app_config = app_context.app_config();
            app_config.theme_filename = theme_filename.clone();
            // Clone the config so we can save it after releasing the lock
            app_config.clone()
        };

        // Save the updated app config to disk so the theme persists across restarts
        if let Err(e) = app_config_to_save.save() {
            tracing::warn!("Failed to save app config after theme switch: {}", e);
            // Don't fail the theme switch if saving fails, just log a warning
        } else {
            tracing::info!("Saved app config with theme_filename: {}", theme_filename);
        }

        // Reload theme and palette from the new file
        // IMPORTANT: Load palette FIRST, then convert theme using that palette
        // This ensures theme styles resolve palette colors correctly
        let palette_for_conversion =
            match PaletteConfig::deserialize_custom_config::<PaletteRaw>(&theme_filename) {
                Some(raw) => {
                    // File found, convert from raw to config
                    let palette_config: PaletteConfig = raw.into();
                    let palette_clone = palette_config.palette.clone();
                    {
                        let mut palette_config_mut = app_context.palette_config();
                        *palette_config_mut = palette_config;
                    }
                    tracing::info!(
                        "Successfully loaded palette from {} ({} colors)",
                        theme_filename,
                        palette_clone.len()
                    );
                    palette_clone
                }
                None => {
                    tracing::warn!(
                        "Palette not found in theme file '{}', using current palette",
                        theme_filename
                    );
                    // Use current palette from AppContext
                    app_context.palette_config().palette.clone()
                }
            };

        // Now load and convert the theme using the palette we just loaded/obtained
        tracing::info!("Attempting to load theme from: {}", theme_filename);
        match ThemeConfig::deserialize_custom_config::<ThemeRaw>(&theme_filename) {
            Some(raw) => {
                // File found, convert from raw to config using the palette we just loaded
                tracing::info!("Theme file found, converting to ThemeConfig");
                let new_theme_config: ThemeConfig =
                    ThemeConfig::from_raw_with_palette(raw, &palette_for_conversion);
                let common_keys_count = new_theme_config.common.len();
                {
                    let mut theme_config = app_context.theme_config();
                    tracing::info!("Updating theme config (common keys: {})", common_keys_count);
                    *theme_config = new_theme_config.clone();
                }
                // Verify the update worked by reading back
                {
                    let theme_config = app_context.theme_config();
                    let updated_keys_count = theme_config.common.len();
                    tracing::info!(
                        "Theme config updated successfully (common keys before: {}, after: {})",
                        common_keys_count,
                        updated_keys_count
                    );
                    if updated_keys_count != common_keys_count {
                        tracing::error!("Theme config update failed! Key count mismatch.");
                    }
                }
                tracing::info!(
                    "Successfully loaded and applied theme from {}",
                    theme_filename
                );
            }
            None => {
                tracing::warn!(
                    "Theme file '{}' not found, keeping current theme",
                    theme_filename
                );
                return Err(AppError::InvalidAction(format!(
                    "Theme file '{}' not found",
                    theme_filename
                )));
            }
        }

        tracing::info!("Switched to theme: {}", theme_name);

        // Force a final read to ensure the update is visible
        {
            let _theme_config = app_context.theme_config();
            tracing::debug!("Theme config lock acquired and released to ensure update is visible");
        }

        Ok(())
    }
}

impl Default for ThemeSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_switcher_new() {
        let switcher = ThemeSwitcher::new();
        assert_eq!(switcher.themes.len(), AVAILABLE_THEMES.len());
        assert_eq!(switcher.current_theme_index, 0);
        assert_eq!(switcher.current_theme(), Some("theme"));
    }

    #[test]
    fn test_theme_switcher_with_themes() {
        let themes = vec![
            "theme1".to_string(),
            "theme2".to_string(),
            "theme3".to_string(),
        ];
        let switcher = ThemeSwitcher::with_themes(themes.clone());
        assert_eq!(switcher.themes, themes);
        assert_eq!(switcher.current_theme_index, 0);
        assert_eq!(switcher.current_theme(), Some("theme1"));
    }

    #[test]
    fn test_current_theme() {
        let switcher = ThemeSwitcher::new();
        assert_eq!(switcher.current_theme(), Some("theme"));
    }

    #[test]
    fn test_current_theme_empty() {
        let switcher = ThemeSwitcher::with_themes(vec![]);
        assert_eq!(switcher.current_theme(), None);
    }

    #[test]
    fn test_available_themes() {
        let switcher = ThemeSwitcher::new();
        let themes = switcher.available_themes();
        assert_eq!(themes.len(), AVAILABLE_THEMES.len());
        assert_eq!(themes[0], "theme");
    }

    #[test]
    fn test_next_theme() {
        let mut switcher = ThemeSwitcher::new();
        assert_eq!(switcher.current_theme(), Some("theme"));
        assert_eq!(switcher.next_theme(), Some("monokai"));
        assert_eq!(switcher.current_theme(), Some("monokai"));
    }

    #[test]
    fn test_next_theme_wraps_around() {
        let mut switcher = ThemeSwitcher::new();
        // Go to the last theme
        for _ in 0..(AVAILABLE_THEMES.len() - 1) {
            switcher.next_theme();
        }
        assert_eq!(switcher.current_theme(), Some("onedark"));
        // Next should wrap to first
        assert_eq!(switcher.next_theme(), Some("theme"));
    }

    #[test]
    fn test_previous_theme() {
        let mut switcher = ThemeSwitcher::new();
        assert_eq!(switcher.current_theme(), Some("theme"));
        // Previous from first should wrap to last
        assert_eq!(switcher.previous_theme(), Some("onedark"));
        assert_eq!(switcher.current_theme(), Some("onedark"));
    }

    #[test]
    fn test_previous_theme_wraps_around() {
        let mut switcher = ThemeSwitcher::new();
        switcher.current_theme_index = 1; // Set to second theme (monokai)
        assert_eq!(switcher.current_theme(), Some("monokai"));
        assert_eq!(switcher.previous_theme(), Some("theme"));
    }

    #[test]
    fn test_next_theme_empty() {
        let mut switcher = ThemeSwitcher::with_themes(vec![]);
        assert_eq!(switcher.next_theme(), None);
    }

    #[test]
    fn test_previous_theme_empty() {
        let mut switcher = ThemeSwitcher::with_themes(vec![]);
        assert_eq!(switcher.previous_theme(), None);
    }

    #[test]
    fn test_switch_to_theme() {
        let mut switcher = ThemeSwitcher::new();
        assert!(switcher.switch_to_theme("nord").is_ok());
        assert_eq!(switcher.current_theme(), Some("nord"));
    }

    #[test]
    fn test_switch_to_theme_invalid() {
        let mut switcher = ThemeSwitcher::new();
        assert!(switcher.switch_to_theme("nonexistent").is_err());
        // Should still be on the original theme
        assert_eq!(switcher.current_theme(), Some("theme"));
    }

    #[test]
    fn test_switch_to_theme_case_sensitive() {
        let mut switcher = ThemeSwitcher::new();
        assert!(switcher.switch_to_theme("Nord").is_err()); // Case sensitive
        assert!(switcher.switch_to_theme("nord").is_ok());
    }

    #[test]
    fn test_cycle_through_all_themes() {
        let mut switcher = ThemeSwitcher::new();
        let initial_theme = switcher.current_theme().unwrap().to_string();

        // Cycle through all themes
        for _ in 0..AVAILABLE_THEMES.len() {
            switcher.next_theme();
        }

        // Should be back to initial theme
        assert_eq!(switcher.current_theme(), Some(initial_theme.as_str()));
    }

    #[test]
    fn test_dynamic_theme_addition() {
        let mut switcher = ThemeSwitcher::with_themes(vec!["theme1".to_string()]);
        assert_eq!(switcher.current_theme(), Some("theme1"));

        // Add a new theme dynamically
        switcher.themes.push("theme2".to_string());
        assert_eq!(switcher.available_themes().len(), 2);
        assert_eq!(switcher.current_theme(), Some("theme1"));

        // Switch to new theme
        assert!(switcher.switch_to_theme("theme2").is_ok());
        assert_eq!(switcher.current_theme(), Some("theme2"));
    }
}
