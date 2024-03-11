use {
        crate::{
                app_error::AppError,
                configs::{
                        config_file::ConfigFile, config_type::ConfigType, custom::default_config_logger_file_path,
                        project_dir, raw::logger_raw::LoggerRaw,
                },
        },
        config::{Config, File, FileFormat},
        std::path::Path,
};

#[derive(Clone, Debug)]
/// The logger configuration.
pub struct LoggerConfig {
        pub log_folder: String,
        pub log_file: String,
        pub log_level: String,
}
/// The logger configuration implementation.
impl LoggerConfig {
        /// Get the default logger configuration.
        ///
        /// # Returns
        /// The default logger configuration.
        pub fn default_result() -> Result<Self, AppError> {
                let builder: LoggerRaw = Config::builder()
                        .add_source(File::from(Path::new(&default_config_logger_file_path()?)).format(FileFormat::Toml))
                        .build()?
                        .try_deserialize()?;
                Ok(builder.into())
        }
}
/// The implementation of the configuration file for the logger.
impl ConfigFile for LoggerConfig {
        type Raw = LoggerRaw;

        fn get_type() -> ConfigType {
                ConfigType::Logger
        }

        fn override_fields() -> bool {
                true
        }

        fn merge(&mut self, other: Option<Self::Raw>) -> Self {
                match other {
                        None => self.clone(),
                        Some(other) => {
                                if let Some(log_folder) = other.log_folder {
                                        self.log_folder = log_folder;
                                }
                                if let Some(log_file) = other.log_file {
                                        self.log_file = log_file;
                                }
                                if let Some(log_level) = other.log_level {
                                        self.log_level = log_level;
                                }
                                self.clone()
                        }
                }
        }
}
/// The default logger configuration.
impl std::default::Default for LoggerConfig {
        fn default() -> Self {
                Self::default_result().unwrap()
        }
}
/// The conversion from the raw logger configuration to the logger configuration.
impl From<LoggerRaw> for LoggerConfig {
        fn from(raw: LoggerRaw) -> Self {
                Self {
                        log_folder: project_dir()
                                .unwrap()
                                .join(raw.log_folder.unwrap())
                                .to_string_lossy()
                                .to_string(),
                        log_file: raw.log_file.unwrap(),
                        log_level: raw.log_level.unwrap(),
                }
        }
}
