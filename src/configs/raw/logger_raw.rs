use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
/// The raw logger configuration.
pub struct LoggerRaw {
    /// The folder where the log file is stored.
    pub log_dir: Option<String>,
    /// The name of the log file.
    pub log_file: Option<String>,
    /// The rotation frequency of the log.
    /// The log rotation frequency can be one of the following:
    /// * minutely: A new log file in the format of log_folder/log_file.yyyy-MM-dd-HH-mm will be created minutely (once per minute)
    /// * hourly: A new log file in the format of log_folder/log_file.yyyy-MM-dd-HH will be created hourly
    /// * daily: A new log file in the format of log_folder/log_file.yyyy-MM-dd will be created daily
    /// * never: This will result in log file located at log_folder/log_file
    pub rotation_frequency: Option<String>,
    /// The maximum number of old log files that will be stored
    pub max_old_log_files: Option<usize>,
    /// The level of the log.
    /// The log level can be one of the following:
    /// * error: only log error
    /// * warn: log error and warning
    /// * info: log error, warning, and info
    /// * debug: log error, warning, info, and debug
    /// * trace: log error, warning, info, debug, and trace
    /// * off: turn off logging
    pub log_level: Option<String>,
}
