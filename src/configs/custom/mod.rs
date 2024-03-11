use {crate::configs::default_config_dir, std::io};

pub mod logger_custom;

// #[cfg(not(target_os = "windows"))]
// pub const DEFAULT_CONFIG_LOGGER_FILE_PATH: &str = include_str!("../../../config/logger.toml");
//
// #[cfg(target_os = "windows")]
// pub const DEFAULT_CONFIG_LOGGER_FILE_PATH: &str = include_str!("..\\..\\..\\config\\icons.toml");

pub fn default_config_logger_file_path() -> io::Result<String> {
  Ok(default_config_dir()?.join("logger.toml").to_str().unwrap().to_string())
}
