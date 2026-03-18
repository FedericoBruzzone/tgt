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

/// Theme post-processing to improve readability for VSCode-imported themes.
///
/// Many VSCode themes look great in an editor but can map poorly to a TUI where we rely heavily on
/// a small number of foreground colors over a mostly-flat background. This function enforces:
/// - minimum contrast for chat list title + message preview against the chat list background
/// - differentiation between chat name and message preview so they don't look identical
///
/// We intentionally **skip** the built-in `theme` and `first_theme` which are already tuned.
fn adapt_theme_for_readability(theme_name: &str, theme: &mut ThemeConfig) {
    if theme_name == "theme" || theme_name == "first_theme" {
        return;
    }

    // ----- Color utilities -----
    fn srgb_to_linear(c: f32) -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    fn rel_luminance(rgb: (u8, u8, u8)) -> f32 {
        let (r, g, b) = (rgb.0 as f32 / 255.0, rgb.1 as f32 / 255.0, rgb.2 as f32 / 255.0);
        let (r, g, b) = (srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b));
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }
    fn contrast_ratio(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> f32 {
        let (l1, l2) = {
            let a = rel_luminance(fg);
            let b = rel_luminance(bg);
            if a >= b { (a, b) } else { (b, a) }
        };
        (l1 + 0.05) / (l2 + 0.05)
    }
    fn rgb_distance(a: (u8, u8, u8), b: (u8, u8, u8)) -> u32 {
        let dr = a.0 as i32 - b.0 as i32;
        let dg = a.1 as i32 - b.1 as i32;
        let db = a.2 as i32 - b.2 as i32;
        (dr * dr + dg * dg + db * db) as u32
    }
    fn clamp_u8(v: f32) -> u8 {
        v.round().clamp(0.0, 255.0) as u8
    }
    fn lerp(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
        (
            clamp_u8(a.0 as f32 + (b.0 as f32 - a.0 as f32) * t),
            clamp_u8(a.1 as f32 + (b.1 as f32 - a.1 as f32) * t),
            clamp_u8(a.2 as f32 + (b.2 as f32 - a.2 as f32) * t),
        )
    }

    // Ratatui Color -> RGB (best-effort)
    fn xterm_index_to_rgb(idx: u8) -> (u8, u8, u8) {
        // 0-15: ANSI colors
        const ANSI16: [(u8, u8, u8); 16] = [
            (0, 0, 0),
            (128, 0, 0),
            (0, 128, 0),
            (128, 128, 0),
            (0, 0, 128),
            (128, 0, 128),
            (0, 128, 128),
            (192, 192, 192),
            (128, 128, 128),
            (255, 0, 0),
            (0, 255, 0),
            (255, 255, 0),
            (0, 0, 255),
            (255, 0, 255),
            (0, 255, 255),
            (255, 255, 255),
        ];
        if idx < 16 {
            return ANSI16[idx as usize];
        }
        if (16..=231).contains(&idx) {
            let i = idx - 16;
            let r = i / 36;
            let g = (i % 36) / 6;
            let b = i % 6;
            let conv = |c: u8| -> u8 {
                match c {
                    0 => 0,
                    1 => 95,
                    2 => 135,
                    3 => 175,
                    4 => 215,
                    _ => 255,
                }
            };
            return (conv(r), conv(g), conv(b));
        }
        // 232-255: grayscale ramp
        let gray = 8 + (idx - 232) * 10;
        (gray, gray, gray)
    }

    fn color_to_rgb(color: ratatui::style::Color) -> Option<(u8, u8, u8)> {
        use ratatui::style::Color::*;
        match color {
            Reset => None,
            Black => Some((0, 0, 0)),
            Red => Some((205, 49, 49)),
            Green => Some((13, 188, 121)),
            Yellow => Some((229, 229, 16)),
            Blue => Some((36, 114, 200)),
            Magenta => Some((188, 63, 188)),
            Cyan => Some((17, 168, 205)),
            Gray => Some((204, 204, 204)),
            DarkGray => Some((102, 102, 102)),
            LightRed => Some((241, 76, 76)),
            LightGreen => Some((35, 209, 139)),
            LightYellow => Some((245, 245, 67)),
            LightBlue => Some((59, 142, 234)),
            LightMagenta => Some((214, 112, 214)),
            LightCyan => Some((41, 184, 219)),
            White => Some((255, 255, 255)),
            Rgb(r, g, b) => Some((r, g, b)),
            Indexed(i) => Some(xterm_index_to_rgb(i)),
        }
    }

    // ----- Pull baseline background -----
    let bg = theme
        .chat_list
        .get("self")
        .and_then(|s| s.bg)
        .and_then(color_to_rgb);
    let Some(chat_list_bg) = bg else {
        return;
    };

    // Minimum contrast ratios.
    // - Chat name: primary label (higher)
    // - Message preview: secondary text (slightly lower but still readable)
    const MIN_CR_NAME: f32 = 5.0;
    const MIN_CR_MSG: f32 = 4.2;
    // Minimum separation between name and message colors (squared distance in RGB space).
    // 40^2 = 1600
    const MIN_NAME_MSG_DIST2: u32 = 1600;
    // How strongly we "mute" message preview towards background for a more professional hierarchy.
    // (We still enforce MIN_CR_MSG after muting.)
    const MSG_MUTE_TOWARDS_BG: f32 = 0.35;
    // Ensure a noticeable luminance difference between name and message (helps when hues are similar).
    const MIN_NAME_MSG_LUMA_SEP: f32 = 0.12;

    // Adjust a foreground color to satisfy contrast against bg by blending towards white/black.
    fn ensure_contrast(
        fg: (u8, u8, u8),
        bg: (u8, u8, u8),
        min_ratio: f32,
    ) -> (u8, u8, u8) {
        if contrast_ratio(fg, bg) >= min_ratio {
            return fg;
        }
        let bg_l = rel_luminance(bg);
        let target = if bg_l < 0.5 { (255, 255, 255) } else { (0, 0, 0) };
        // Binary-search-ish stepping.
        let mut best = fg;
        let mut lo = 0.0f32;
        let mut hi = 1.0f32;
        for _ in 0..12 {
            let t = (lo + hi) * 0.5;
            let candidate = lerp(fg, target, t);
            if contrast_ratio(candidate, bg) >= min_ratio {
                best = candidate;
                hi = t;
            } else {
                lo = t;
            }
        }
        best
    }

    // Helper to mutate a ThemeStyle fg when it is RGB/Indexed/basic.
    fn set_fg_rgb(style: &mut crate::configs::config_theme::ThemeStyle, rgb: (u8, u8, u8)) {
        style.fg = Some(ratatui::style::Color::Rgb(rgb.0, rgb.1, rgb.2));
    }

    // Grab chat list relevant colors
    let name_rgb = theme
        .chat_list
        .get("item_chat_name")
        .and_then(|s| s.fg)
        .and_then(color_to_rgb);
    let msg_rgb = theme
        .chat_list
        .get("item_message_content")
        .and_then(|s| s.fg)
        .and_then(color_to_rgb);

    // If missing, nothing to do.
    if name_rgb.is_none() || msg_rgb.is_none() {
        return;
    }
    let mut name_rgb_v = name_rgb.unwrap();
    let mut msg_rgb_v = msg_rgb.unwrap();

    // Ensure both are readable against background first.
    name_rgb_v = ensure_contrast(name_rgb_v, chat_list_bg, MIN_CR_NAME);
    msg_rgb_v = ensure_contrast(msg_rgb_v, chat_list_bg, MIN_CR_MSG);

    // Establish a professional visual hierarchy:
    // - keep chat name as the more prominent label
    // - mute message preview slightly towards background (secondary text), while keeping it readable
    {
        let muted = lerp(msg_rgb_v, chat_list_bg, MSG_MUTE_TOWARDS_BG);
        msg_rgb_v = ensure_contrast(muted, chat_list_bg, MIN_CR_MSG);
    }

    // Ensure name and message are clearly distinguishable (both by color distance and luminance).
    let mut name_l = rel_luminance(name_rgb_v);
    let mut msg_l = rel_luminance(msg_rgb_v);
    if rgb_distance(name_rgb_v, msg_rgb_v) < MIN_NAME_MSG_DIST2
        || (name_l - msg_l).abs() < MIN_NAME_MSG_LUMA_SEP
    {
        // Prefer keeping message as "secondary": try moving it closer to background first.
        let c_towards_bg = ensure_contrast(lerp(msg_rgb_v, chat_list_bg, 0.25), chat_list_bg, MIN_CR_MSG);

        // Alternative: move it away from background (if towards-bg doesn't separate enough).
        let bg_l = rel_luminance(chat_list_bg);
        let away_target = if bg_l < 0.5 { (255, 255, 255) } else { (0, 0, 0) };
        let c_away = ensure_contrast(lerp(msg_rgb_v, away_target, 0.25), chat_list_bg, MIN_CR_MSG);

        let score = |cand: (u8, u8, u8)| -> (u32, f32) {
            let dist = rgb_distance(name_rgb_v, cand);
            let l_sep = (rel_luminance(cand) - name_l).abs();
            (dist, l_sep)
        };
        let s1 = score(c_towards_bg);
        let s2 = score(c_away);

        msg_rgb_v = if (s2.0, (s2.1 * 1000.0) as u32) > (s1.0, (s1.1 * 1000.0) as u32) {
            c_away
        } else {
            c_towards_bg
        };

        // Final nudge: if luminance is still too close, push message slightly opposite direction.
        name_l = rel_luminance(name_rgb_v);
        msg_l = rel_luminance(msg_rgb_v);
        if (name_l - msg_l).abs() < MIN_NAME_MSG_LUMA_SEP {
            let target = if msg_l >= name_l {
                // message too bright vs name → darken a bit (towards bg if bg is dark; towards black otherwise)
                if bg_l < 0.5 { chat_list_bg } else { (0, 0, 0) }
            } else {
                // message too dark vs name → brighten a bit (towards white)
                (255, 255, 255)
            };
            msg_rgb_v = ensure_contrast(lerp(msg_rgb_v, target, 0.15), chat_list_bg, MIN_CR_MSG);
        }
    }

    // Write back adjusted colors.
    if let Some(s) = theme.chat_list.get_mut("item_chat_name") {
        set_fg_rgb(s, name_rgb_v);
    }
    if let Some(s) = theme.chat_list.get_mut("item_message_content") {
        set_fg_rgb(s, msg_rgb_v);
    }
}

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
    use crate::configs::config_file::CONFIG_DIR_HIERARCHY;

    let mut theme_names = Vec::new();

    // Search in config directory hierarchy (same as CONFIG_DIR_HIERARCHY)
    let search_dirs: &Vec<PathBuf> = &CONFIG_DIR_HIERARCHY;

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
                let mut new_theme_config: ThemeConfig =
                    ThemeConfig::from_raw_with_palette(raw, &palette_for_conversion);
                adapt_theme_for_readability(theme_name, &mut new_theme_config);
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
            assert!(!themes.is_empty(), "Should discover at least one theme");
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
