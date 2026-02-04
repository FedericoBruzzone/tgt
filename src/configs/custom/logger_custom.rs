use crate::{
    app_error::AppError,
    configs::{self, config_file::ConfigFile, config_type::ConfigType, raw::logger_raw::LoggerRaw},
    utils,
};
use std::path::Path;

#[derive(Clone, Debug)]
/// The logger configuration.
pub struct LoggerConfig {
    /// The folder where the log file is stored.
    pub log_dir: String,
    /// The name of the log file.
    pub log_file: String,
    /// The rotation frequency of the log.
    /// The log rotation frequency can be one of the following:
    /// * minutely: A new log file in the format of log_dir/log_file.yyyy-MM-dd-HH-mm will be created minutely (once per minute)
    /// * hourly: A new log file in the format of log_dir/log_file.yyyy-MM-dd-HH will be created hourly
    /// * daily: A new log file in the format of log_dir/log_file.yyyy-MM-dd will be created daily
    /// * never: This will result in log file located at log_dir/log_file
    pub rotation_frequency: String,
    /// The maximum number of old log files that will be stored
    pub max_old_log_files: usize,
    /// The level of the log.
    /// The log level can be one of the following:
    /// * error: only log error
    /// * warn: log error and warning
    /// * info: log error, warning, and info
    /// * debug: log error, warning, info, and debug
    /// * trace: log error, warning, info, debug, and trace
    /// * off: turn off logging
    pub log_level: String,
}
/// The logger configuration implementation.
impl LoggerConfig {
    /// Get the default logger configuration.
    ///
    /// # Returns
    /// The default logger configuration.
    pub fn default_result() -> Result<Self, AppError<()>> {
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
                if let Some(log_dir) = other.log_dir {
                    if !Path::new(&log_dir).exists() {
                        std::fs::create_dir_all(&log_dir).unwrap();
                    }
                    self.log_dir = log_dir;
                }
                if let Some(log_file) = other.log_file {
                    self.log_file = log_file;
                }
                if let Some(log_level) = other.log_level {
                    self.log_level = log_level;
                }
                if let Some(rotation_frequency) = other.rotation_frequency {
                    self.rotation_frequency = rotation_frequency;
                }
                if let Some(max_old_log_files) = other.max_old_log_files {
                    self.max_old_log_files = max_old_log_files;
                }
                self.clone()
            }
        }
    }
}
/// The default logger configuration.
impl Default for LoggerConfig {
    fn default() -> Self {
        // Try to load from config file, but fall back to hardcoded defaults if it doesn't exist
        Self::default_result().unwrap_or_else(|e| {
            tracing::warn!("Failed to load logger config, using hardcoded defaults: {e}");

            // Hardcoded defaults matching config/logger.toml
            let log_dir = utils::tgt_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap())
                .join(".data/logs")
                .to_string_lossy()
                .to_string();

            // Create log directory if it doesn't exist
            if !Path::new(&log_dir).exists() {
                let _ = std::fs::create_dir_all(&log_dir);
            }

            Self {
                log_dir,
                log_file: "tgt.log".to_string(),
                rotation_frequency: "daily".to_string(),
                max_old_log_files: 7,
                log_level: "info".to_string(),
            }
        })
    }
}
/// The conversion from the raw logger configuration to the logger
/// configuration.
impl From<LoggerRaw> for LoggerConfig {
    fn from(raw: LoggerRaw) -> Self {
        let log_dir = utils::tgt_dir()
            .unwrap()
            .join(raw.log_dir.unwrap())
            .to_string_lossy()
            .to_string();
        if !Path::new(&log_dir).exists() {
            std::fs::create_dir_all(&log_dir).unwrap();
        }

        Self {
            log_dir,
            log_file: raw.log_file.unwrap(),
            rotation_frequency: raw.rotation_frequency.unwrap(),
            max_old_log_files: raw.max_old_log_files.unwrap(),
            log_level: raw.log_level.unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::config_file::ConfigFile;
    use crate::configs::{custom::logger_custom::LoggerConfig, raw::logger_raw::LoggerRaw};
    use crate::utils::tgt_dir;

    #[test]
    fn test_logger_config_default() {
        let logger_config = LoggerConfig::default();
        assert_eq!(
            logger_config.log_dir,
            tgt_dir()
                .unwrap()
                .join(".data/logs")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(logger_config.log_file, "tgt.log");
        assert_eq!(logger_config.log_level, "info");
    }

    #[test]
    fn test_logger_config_from_raw() {
        let logger_raw = LoggerRaw {
            log_dir: Some(".data/logs".to_string()),
            log_file: Some("tgt.log".to_string()),
            rotation_frequency: Some("hourly".to_string()),
            max_old_log_files: Some(3),
            log_level: Some("debug".to_string()),
        };
        let logger_config = LoggerConfig::from(logger_raw);
        assert_eq!(
            logger_config.log_dir,
            tgt_dir()
                .unwrap()
                .join(".data/logs")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(logger_config.log_file, "tgt.log");
        assert_eq!(logger_config.rotation_frequency, "hourly");
        assert_eq!(logger_config.max_old_log_files, 3);
        assert_eq!(logger_config.log_level, "debug");
    }

    #[test]
    fn test_logger_config_merge() {
        let mut logger_config = LoggerConfig::from(LoggerRaw {
            log_dir: Some(".data/logs".to_string()),
            log_file: Some("tgt.log".to_string()),
            rotation_frequency: Some("never".to_string()),
            max_old_log_files: Some(5),
            log_level: Some("info".to_string()),
        });
        let logger_raw = LoggerRaw {
            log_dir: None,
            log_file: None,
            rotation_frequency: None,
            max_old_log_files: None,
            log_level: Some("debug".to_string()),
        };
        logger_config = logger_config.merge(Some(logger_raw));
        assert_eq!(
            logger_config.log_dir,
            tgt_dir()
                .unwrap()
                .join(".data/logs")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(logger_config.log_file, "tgt.log");
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
            log_dir: None,
            log_file: None,
            rotation_frequency: None,
            max_old_log_files: None,
            log_level: None,
        };
        logger_config = logger_config.merge(Some(logger_raw));
        assert_eq!(
            logger_config.log_dir,
            tgt_dir()
                .unwrap()
                .join(".data/logs")
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
