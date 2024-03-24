use {
    super::config_type::ConfigType, crate::configs::default_config_dir, std::io,
};

pub mod app_custom;
pub mod keymap_custom;
pub mod logger_custom;

/// Get the default configuration file path of the specified configuration type.
/// It is cross-platform.
///
/// # Arguments
/// * `config_type` - The configuration type.
///
/// # Returns
/// The default configuration file path of the specified configuration type.
fn default_config_file_path_of(config_type: ConfigType) -> io::Result<String> {
    Ok(default_config_dir()?
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

// #[cfg(not(target_os = "windows"))]
// pub const DEFAULT_CONFIG_LOGGER_FILE_PATH: &str =
// include_str!("../../../config/logger.toml");
//
// #[cfg(target_os = "windows")]
// pub const DEFAULT_CONFIG_LOGGER_FILE_PATH: &str =
// include_str!("..\\..\\..\\config\\icons.toml");
