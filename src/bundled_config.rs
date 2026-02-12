//! Default config files embedded in the binary so they are always available (e.g. after `cargo install`).
//! Paths are relative to the crate root (parent of src/).

macro_rules! bundled {
    ($path:expr) => {
        include_str!(concat!("../", $path))
    };
}

/// Top-level config files (config/*.toml).
const BUNDLED_APP: &str = bundled!("config/app.toml");
const BUNDLED_KEYMAP: &str = bundled!("config/keymap.toml");
const BUNDLED_LOGGER: &str = bundled!("config/logger.toml");
const BUNDLED_TELEGRAM: &str = bundled!("config/telegram.toml");
/// Top-level theme.toml (same as themes/theme.toml for default theme config).
const BUNDLED_THEME: &str = bundled!("config/themes/theme.toml");

/// Theme files (config/themes/*.toml).
const BUNDLED_THEME_CATPPUCCIN: &str = bundled!("config/themes/catppuccin.toml");
const BUNDLED_THEME_FIRST: &str = bundled!("config/themes/first_theme.toml");
const BUNDLED_THEME_GITHUB: &str = bundled!("config/themes/github.toml");
const BUNDLED_THEME_GRUVBOX: &str = bundled!("config/themes/gruvbox.toml");
const BUNDLED_THEME_MONOKAI: &str = bundled!("config/themes/monokai.toml");
const BUNDLED_THEME_NORD: &str = bundled!("config/themes/nord.toml");
const BUNDLED_THEME_ONEDARK: &str = bundled!("config/themes/onedark.toml");
const BUNDLED_THEME_DEFAULT: &str = bundled!("config/themes/theme.toml");
const BUNDLED_THEME_TOKYO: &str = bundled!("config/themes/tokyo.toml");

/// Return embedded content for a config file (e.g. "keymap.toml"). Used for default-merge at load time.
pub fn bundled_config_content(filename: &str) -> Option<&'static str> {
    bundled_config_files()
        .iter()
        .find(|(p, _)| *p == filename)
        .map(|(_, c)| *c)
}

/// List of (relative path, content) for all bundled config files. Used to write missing files.
pub fn bundled_config_files() -> &'static [(&'static str, &'static str)] {
    &[
        ("app.toml", BUNDLED_APP),
        ("keymap.toml", BUNDLED_KEYMAP),
        ("logger.toml", BUNDLED_LOGGER),
        ("telegram.toml", BUNDLED_TELEGRAM),
        ("theme.toml", BUNDLED_THEME),
        ("themes/catppuccin.toml", BUNDLED_THEME_CATPPUCCIN),
        ("themes/first_theme.toml", BUNDLED_THEME_FIRST),
        ("themes/github.toml", BUNDLED_THEME_GITHUB),
        ("themes/gruvbox.toml", BUNDLED_THEME_GRUVBOX),
        ("themes/monokai.toml", BUNDLED_THEME_MONOKAI),
        ("themes/nord.toml", BUNDLED_THEME_NORD),
        ("themes/onedark.toml", BUNDLED_THEME_ONEDARK),
        ("themes/theme.toml", BUNDLED_THEME_DEFAULT),
        ("themes/tokyo.toml", BUNDLED_THEME_TOKYO),
    ]
}
