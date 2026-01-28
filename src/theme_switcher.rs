use crate::{
    app_context::AppContext,
    app_error::AppError,
    configs::{
        config_file::ConfigFile,
        custom::{palette_custom::PaletteConfig, theme_custom::ThemeConfig},
        raw::{palette_raw::PaletteRaw, theme_raw::ThemeRaw},
    },
};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

/// Discover available theme files dynamically from the themes directory.
///
/// This function searches for theme files in the config directory hierarchy,
/// looking for `.toml` files in the `themes/` subdirectory.
///
/// **Search order:**
/// 1. `TGT_CONFIG_DIR/themes/` (if `TGT_CONFIG_DIR` is set)
/// 2. `./config/themes/` (debug mode only)
/// 3. `~/.config/tgt/config/themes/` (if exists)
/// 4. `~/.tgt/config/themes/` (if exists)
///
/// **Returns:**
/// A vector of theme names (without `.toml` extension), sorted alphabetically
/// with "theme" first if it exists (as it's the default theme).
///
/// **Example:**
/// If themes directory contains: `theme.toml`, `monokai.toml`, `nord.toml`
/// Returns: `["theme", "monokai", "nord"]`
pub fn discover_available_themes() -> Vec<String> {
    use crate::utils::{TGT, TGT_CONFIG_DIR};
    use std::env;

    let mut theme_names = Vec::new();

    // Search in config directory hierarchy (same order as CONFIG_DIR_HIERARCHY)
    let search_dirs = if let Ok(config_dir) = env::var(TGT_CONFIG_DIR) {
        vec![PathBuf::from(config_dir)]
    } else {
        let mut dirs = Vec::new();

        // Debug mode: check current directory's config folder
        if cfg!(debug_assertions) {
            if let Ok(current_dir) = std::env::current_dir() {
                let config_dir = current_dir.join("config");
                if config_dir.is_dir() {
                    dirs.push(config_dir);
                }
            }
        }

        // Standard user config directories
        if let Some(user_config_dir) = if cfg!(target_os = "macos") {
            dirs::home_dir().map(|h| h.join(".config"))
        } else {
            dirs::config_dir()
        } {
            let tgt_config = user_config_dir.join(TGT).join("config");
            if tgt_config.is_dir() {
                dirs.push(tgt_config);
            }
        }

        // Also check ~/.tgt/config (where build.rs copies files in release mode)
        if let Some(home) = dirs::home_dir() {
            let tgt_config = home.join(format!(".{}", TGT)).join("config");
            if tgt_config.is_dir() {
                dirs.push(tgt_config);
            }
        }

        dirs
    };

    // Search for theme files in each directory
    for config_dir in search_dirs {
        let themes_dir = config_dir.join("themes");
        if let Ok(entries) = fs::read_dir(&themes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                        let theme_name = file_name.to_string();
                        if !theme_names.contains(&theme_name) {
                            theme_names.push(theme_name);
                        }
                    }
                }
            }
        }
    }

    // Sort themes, but ensure "theme" comes first (default theme)
    theme_names.sort();
    if let Some(pos) = theme_names.iter().position(|t| t == "theme") {
        let default_theme = theme_names.remove(pos);
        theme_names.insert(0, default_theme);
    }

    theme_names
}

/// Get the list of available theme names.
///
/// This function discovers theme files dynamically from the themes directory.
/// The themes are sorted alphabetically with "theme" (the default) first.
///
/// **Note:** This is called lazily - themes are discovered when first accessed.
/// To ensure themes are discovered at compile time, use `discover_available_themes()` directly.
pub fn available_themes() -> Vec<String> {
    discover_available_themes()
}

/// Theme switcher that manages theme changes
pub struct ThemeSwitcher {
    /// Current theme index in the available themes list
    current_theme_index: usize,
    /// Available theme names
    themes: Vec<String>,
}

impl ThemeSwitcher {
    /// Create a new theme switcher with themes discovered dynamically from the themes directory.
    ///
    /// Themes are discovered by scanning for `.toml` files in the `themes/` subdirectory
    /// of each config directory in the hierarchy. The themes are sorted alphabetically
    /// with "theme" (the default) first.
    pub fn new() -> Self {
        Self {
            current_theme_index: 0,
            themes: discover_available_themes(),
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
        // Should have at least one theme (the default "theme")
        assert!(
            !switcher.themes.is_empty(),
            "Should discover at least one theme"
        );
        assert_eq!(switcher.current_theme_index, 0);
        // First theme should be "theme" (default theme comes first)
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
        // Should have at least one theme
        assert!(!themes.is_empty(), "Should discover at least one theme");
        // First theme should be "theme" (default theme comes first)
        assert_eq!(themes[0], "theme");
    }

    #[test]
    fn test_next_theme() {
        let mut switcher = ThemeSwitcher::new();
        assert_eq!(switcher.current_theme(), Some("theme"));
        // Get the second theme (after "theme" which is first)
        let second_theme = switcher.themes.get(1).cloned();
        assert!(second_theme.is_some(), "Should have at least 2 themes");
        assert_eq!(switcher.next_theme(), second_theme.as_deref());
        assert_eq!(switcher.current_theme(), second_theme.as_deref());
    }

    #[test]
    fn test_next_theme_wraps_around() {
        let mut switcher = ThemeSwitcher::new();
        let theme_count = switcher.themes.len();
        // Go to the last theme
        for _ in 0..(theme_count - 1) {
            switcher.next_theme();
        }
        // Next should wrap to first
        assert_eq!(switcher.next_theme(), Some("theme"));
    }

    #[test]
    fn test_previous_theme() {
        let mut switcher = ThemeSwitcher::new();
        assert_eq!(switcher.current_theme(), Some("theme"));
        // Previous from first should wrap to last
        let last_theme = switcher.themes.last().unwrap().clone();
        assert_eq!(switcher.previous_theme(), Some(last_theme.as_str()));
        assert_eq!(switcher.current_theme(), Some(last_theme.as_str()));
    }

    #[test]
    fn test_previous_theme_wraps_around() {
        let mut switcher = ThemeSwitcher::new();
        assert!(switcher.themes.len() >= 2, "Should have at least 2 themes");
        switcher.current_theme_index = 1; // Set to second theme
        assert_eq!(switcher.previous_theme(), Some("theme"));
        assert_eq!(switcher.current_theme(), Some("theme"));
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
        let theme_count = switcher.themes.len();

        // Cycle through all themes
        for _ in 0..theme_count {
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

    #[test]
    fn test_discover_available_themes_count() {
        let themes = discover_available_themes();
        // Should discover at least the default theme
        assert!(!themes.is_empty(), "Should discover at least one theme");

        // Count actual theme files in config/themes/ directory
        let expected_count = if let Ok(current_dir) = std::env::current_dir() {
            let themes_dir = current_dir.join("config").join("themes");
            if themes_dir.is_dir() {
                fs::read_dir(&themes_dir)
                    .ok()
                    .map(|entries| {
                        entries
                            .flatten()
                            .filter(|e| {
                                e.path().is_file()
                                    && e.path().extension().and_then(|s| s.to_str()) == Some("toml")
                            })
                            .count()
                    })
                    .unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        // Should discover all theme files (at least the expected count)
        if expected_count > 0 {
            assert_eq!(
                themes.len(),
                expected_count,
                "Should discover all {} theme files from config/themes/",
                expected_count
            );
        } else {
            // If we can't count files (e.g., in CI), just verify we have themes
            assert!(themes.len() >= 1, "Should discover at least one theme");
        }
    }

    #[test]
    fn test_discover_available_themes_filenames() {
        let themes = discover_available_themes();

        // Should include the default theme
        assert!(
            themes.contains(&"theme".to_string()),
            "Should include default 'theme'"
        );

        // Get actual theme filenames from config/themes/ directory
        let actual_theme_files: Vec<String> = if let Ok(current_dir) = std::env::current_dir() {
            let themes_dir = current_dir.join("config").join("themes");
            if themes_dir.is_dir() {
                fs::read_dir(&themes_dir)
                    .ok()
                    .map(|entries| {
                        entries
                            .flatten()
                            .filter_map(|e| {
                                let path = e.path();
                                if path.is_file()
                                    && path.extension().and_then(|s| s.to_str()) == Some("toml")
                                {
                                    path.file_stem()
                                        .and_then(|s| s.to_str())
                                        .map(|s| s.to_string())
                                } else {
                                    None
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Verify all discovered themes match actual files
        if !actual_theme_files.is_empty() {
            for theme in &themes {
                assert!(
                    actual_theme_files.contains(theme),
                    "Discovered theme '{}' should exist in config/themes/",
                    theme
                );
            }

            // Verify all actual files are discovered (allowing for some files to be skipped if not accessible)
            for file_theme in &actual_theme_files {
                assert!(
                    themes.contains(file_theme),
                    "Theme file '{}' from config/themes/ should be discovered",
                    file_theme
                );
            }
        }
    }

    #[test]
    fn test_discover_available_themes_ordering() {
        let themes = discover_available_themes();

        // "theme" should be first (default theme)
        if themes.contains(&"theme".to_string()) {
            assert_eq!(themes[0], "theme", "Default 'theme' should be first");
        }

        // Rest should be sorted alphabetically
        if themes.len() > 1 {
            let rest = &themes[1..];
            let mut sorted_rest = rest.to_vec();
            sorted_rest.sort();
            assert_eq!(
                rest,
                sorted_rest.as_slice(),
                "Themes after 'theme' should be sorted alphabetically"
            );
        }
    }

    #[test]
    fn test_discover_available_themes_no_duplicates() {
        let themes = discover_available_themes();
        let mut seen = std::collections::HashSet::new();

        for theme in &themes {
            assert!(
                !seen.contains(theme),
                "Theme '{}' should not appear twice",
                theme
            );
            seen.insert(theme.clone());
        }
    }

    #[test]
    fn test_discover_available_themes_only_toml_files() {
        let themes = discover_available_themes();

        // All theme names should be valid (no .toml extension, no path separators)
        for theme in &themes {
            assert!(
                !theme.contains('.'),
                "Theme name '{}' should not contain '.'",
                theme
            );
            assert!(
                !theme.contains('/'),
                "Theme name '{}' should not contain '/'",
                theme
            );
            assert!(
                !theme.contains('\\'),
                "Theme name '{}' should not contain '\\'",
                theme
            );
            assert!(!theme.is_empty(), "Theme name should not be empty");
        }
    }

    #[test]
    fn test_available_themes_function() {
        let themes = available_themes();
        // Should return the same as discover_available_themes()
        let discovered = discover_available_themes();
        assert_eq!(themes, discovered);
    }
}
