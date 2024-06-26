use {super::config_type::ConfigType, crate::utils::tgt_config_dir, std::io};

pub mod app_custom;
pub mod keymap_custom;
pub mod logger_custom;
pub mod palette_custom;
pub mod telegram_custom;
pub mod theme_custom;

/// Get the default configuration file path of the specified configuration type.
/// It is cross-platform.
///
/// # Arguments
/// * `config_type` - The configuration type.
///
/// # Returns
/// The default configuration file path of the specified configuration type.
fn default_config_file_path_of(config_type: ConfigType) -> io::Result<String> {
    Ok(tgt_config_dir()?
        .join(config_type.as_default_filename())
        .to_str()
        .unwrap()
        .to_string())
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
