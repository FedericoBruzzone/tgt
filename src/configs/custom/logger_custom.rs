use crate::{
    app_error::AppError,
    configs::{
        self, config_file::ConfigFile, config_type::ConfigType, project_dir,
        raw::logger_raw::LoggerRaw,
    },
};
use std::path::Path;

#[derive(Clone, Debug)]
/// The logger configuration.
pub struct LoggerConfig {
    pub log_folder: String,
    pub log_file: String,
    pub rotation_frequency: String,
    pub max_old_log_files: usize,
    pub log_level: String,
}
/// The logger configuration implementation.
impl LoggerConfig {
    /// Get the default logger configuration.
    ///
    /// # Returns
    /// The default logger configuration.
    pub fn default_result() -> Result<Self, AppError> {
        configs::deserialize_to_config_into::<LoggerRaw, Self>(Path::new(
            &configs::custom::default_config_logger_file_path()?,
        ))
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
                tracing::info!("Merging logger config");
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
impl Default for LoggerConfig {
    fn default() -> Self {
        Self::default_result().unwrap()
    }
}
/// The conversion from the raw logger configuration to the logger
/// configuration.
impl From<LoggerRaw> for LoggerConfig {
    fn from(raw: LoggerRaw) -> Self {
        Self {
            log_folder: project_dir()
                .unwrap()
                .join(raw.log_folder.unwrap())
                .to_string_lossy()
                .to_string(),
            log_file: raw.log_file.unwrap(),
            rotation_frequency: raw.rotation_frequency.unwrap(),
            max_old_log_files: raw.max_old_log_files.unwrap(),
            log_level: raw.log_level.unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::{
        config_file::ConfigFile, custom::logger_custom::LoggerConfig, project_dir,
        raw::logger_raw::LoggerRaw,
    };

    #[test]
    fn test_logger_config_default() {
        let logger_config = LoggerConfig::default();
        assert_eq!(
            logger_config.log_folder,
            project_dir()
                .unwrap()
                .join(".data")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(logger_config.log_file, "tgt.log");
        assert_eq!(logger_config.log_level, "info");
    }

    #[test]
    fn test_logger_config_from_raw() {
        let logger_raw = LoggerRaw {
            log_folder: Some(".data_raw".to_string()),
            log_file: Some("tgt_raw.log".to_string()),
            rotation_frequency: Some("hourly".to_string()),
            max_old_log_files: Some(3),
            log_level: Some("debug".to_string()),
        };
        let logger_config = LoggerConfig::from(logger_raw);
        assert_eq!(
            logger_config.log_folder,
            project_dir()
                .unwrap()
                .join(".data_raw")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(logger_config.log_file, "tgt_raw.log");
        assert_eq!(logger_config.rotation_frequency, "hourly");
        assert_eq!(logger_config.max_old_log_files, 3);
        assert_eq!(logger_config.log_level, "debug");
    }

    #[test]
    fn test_logger_config_merge() {
        let mut logger_config = LoggerConfig::from(LoggerRaw {
            log_folder: Some(".data_raw".to_string()),
            log_file: Some("tgt_raw.log".to_string()),
            rotation_frequency: Some("never".to_string()),
            max_old_log_files: Some(5),
            log_level: Some("info".to_string()),
        });
        let logger_raw = LoggerRaw {
            log_folder: None,
            log_file: None,
            rotation_frequency: None,
            max_old_log_files: None,
            log_level: Some("debug".to_string()),
        };
        logger_config = logger_config.merge(Some(logger_raw));
        assert_eq!(
            logger_config.log_folder,
            project_dir()
                .unwrap()
                .join(".data_raw")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(logger_config.log_file, "tgt_raw.log");
        assert_eq!(logger_config.rotation_frequency, "never");
        assert_eq!(logger_config.max_old_log_files, 5);
        assert_eq!(logger_config.log_level, "debug");
    }

    #[test]
    fn test_logger_config_override_fields() {
        assert!(LoggerConfig::override_fields());
    }

    #[test]
    fn test_merge_all_fields() {
        let mut logger_config = LoggerConfig::default();
        let logger_raw = LoggerRaw {
            log_folder: None,
            log_file: None,
            rotation_frequency: None,
            max_old_log_files: None,
            log_level: None,
        };
        logger_config = logger_config.merge(Some(logger_raw));
        assert_eq!(
            logger_config.log_folder,
            project_dir()
                .unwrap()
                .join(".data")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(logger_config.log_file, "tgt.log");
        assert_eq!(logger_config.rotation_frequency, "daily");
        assert_eq!(logger_config.max_old_log_files, 7);
        assert_eq!(logger_config.log_level, "info");
    }

    #[test]
    fn test_logger_config_get_type() {
        assert_eq!(
            LoggerConfig::get_type(),
            crate::configs::config_type::ConfigType::Logger
        );
    }
}
