use {
    super::config_type::ConfigType,
    crate::utils::tgt_config_dir,
    std::io,
    std::path::{Path, PathBuf},
};

pub mod app_custom;
pub mod keymap_custom;
pub mod logger_custom;
pub mod palette_custom;
pub mod telegram_custom;
pub mod theme_custom;

/// Path to the bundled config file (repo config/ at compile time). None if not available (e.g. installed binary).
/// Does not check existence so that tests running from any cwd can still resolve the repo path.
fn bundled_config_file_path(config_type: ConfigType) -> Option<PathBuf> {
    let manifest = option_env!("CARGO_MANIFEST_DIR")?;
    Some(
        Path::new(manifest)
            .join("config")
            .join(config_type.as_default_filename()),
    )
}

/// Path to use when loading the default config: user config dir first, then bundled if user file is missing.
/// This avoids panics when ~/.tgt was removed and XDG dir is empty (e.g. before next build).
pub fn path_to_load_default_config(config_type: ConfigType) -> io::Result<PathBuf> {
    let filename = config_type.as_default_filename();
    let user_path = tgt_config_dir()?.join(&filename);
    if user_path.exists() {
        return Ok(user_path);
    }
    bundled_config_file_path(config_type).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "configuration file \"{}\" not found in user config dir or bundled config",
                filename
            ),
        )
    })
}

/// Get the default configuration file path of the specified configuration type.
/// Prefers user config dir; falls back to bundled path if the user path does not exist.
fn default_config_file_path_of(config_type: ConfigType) -> io::Result<String> {
    path_to_load_default_config(config_type).and_then(|p| {
        p.to_str()
            .map(String::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "config path not UTF-8"))
    })
}

/// Get the default configuration file path for the logger.
///
/// # Returns
/// The default configuration file path for the logger.
pub fn default_config_logger_file_path() -> io::Result<String> {
    default_config_file_path_of(ConfigType::Logger)
}

/// Get the default configuration file path for the keymap.
///
/// # Returns
/// The default configuration file path for the keymap.
pub fn default_config_keymap_file_path() -> io::Result<String> {
    default_config_file_path_of(ConfigType::Keymap)
}

/// Get the default configuration file path for the application.
///
/// # Returns
/// The default configuration file path for the application.
pub fn default_config_app_file_path() -> io::Result<String> {
    default_config_file_path_of(ConfigType::App)
}

/// Get the default configuration file path for the theme.
///
/// # Returns
/// The default configuration file path for the theme.
pub fn default_config_theme_file_path() -> io::Result<String> {
    default_config_file_path_of(ConfigType::Theme)
}

/// Get the default configuration file path for the palette.
///
/// # Returns
/// The default configuration file path for the palette.
pub fn default_config_palette_file_path() -> io::Result<String> {
    default_config_file_path_of(ConfigType::Palette)
}

/// Get the default configuration file path for the telegram.
///
/// # Returns
/// The default configuration file path for the telegram.
pub fn default_config_telegram_file_path() -> io::Result<String> {
    default_config_file_path_of(ConfigType::Telegram)
}

// #[cfg(not(target_os = "windows"))]
// pub const DEFAULT_CONFIG_LOGGER_FILE_PATH: &str =
// include_str!("../../../config/logger.toml");
//
// #[cfg(target_os = "windows")]
// pub const DEFAULT_CONFIG_LOGGER_FILE_PATH: &str =
// include_str!("..\\..\\..\\config\\icons.toml");
