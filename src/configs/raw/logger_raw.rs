use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
/// The raw logger configuration.
pub struct LoggerRaw {
    /// The folder where the log file is stored.
    pub log_folder: Option<String>,
    /// The name of the log file.
    pub log_file: Option<String>,
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
