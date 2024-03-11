use {
  crate::{
    app_error::AppError,
    configs::{
      config_dir_hierarchy::ConfigFile, config_type::ConfigType, custom::default_config_logger_file_path,
      raw::logger_raw::LoggerRaw,
    },
  },
  config::{Config, File, FileFormat},
  std::path::Path,
};

#[derive(Clone, Debug)]
pub struct LoggerConfig {
  pub log_folder: String,
  pub log_file: String,
  pub log_level: String,
}

impl LoggerConfig {
  pub fn default_result() -> Result<Self, AppError> {
    let builder: LoggerRaw = Config::builder()
      .add_source(File::from(Path::new(&default_config_logger_file_path()?)).format(FileFormat::Toml))
      .build()?
      .try_deserialize()?;
    Ok(builder.into())
  }
}

impl ConfigFile for LoggerConfig {
  type Raw = LoggerRaw;

  fn get_type() -> ConfigType {
    ConfigType::Logger
  }
}

impl std::default::Default for LoggerConfig {
  fn default() -> Self {
    Self::default_result().unwrap()
  }
}

impl From<LoggerRaw> for LoggerConfig {
  fn from(raw: LoggerRaw) -> Self {
    Self {
      log_folder: raw.log_folder,
      log_file: raw.log_file,
      log_level: raw.log_level,
    }
  }
}
