use {
    crate::{app_error::AppError, configs::custom::logger_custom::LoggerConfig},
    std::fs,
    tracing_error::ErrorLayer,
    tracing_subscriber::{
        filter::EnvFilter, prelude::__tracing_subscriber_SubscriberExt, registry::Registry,
        util::SubscriberInitExt, Layer,
    },
};

#[derive(Clone, Debug)]
/// The logger.
/// This struct is used to initialize the logger for the application.
pub struct Logger {
    /// The log folder.
    log_folder: String,
    /// The log file.
    log_file: String,
    /// The rotation frequency.
    rotation_frequency: tracing_appender::rolling::Rotation,
    /// The maximum number of old log files to keep.
    max_old_log_files: usize,
    /// The log level.
    log_level: String,
}

impl Logger {
    /// Create a new logger from the logger configuration.
    ///
    /// # Arguments
    /// * `logger_config` - The logger configuration.
    ///
    /// # Returns
    /// The new logger.
    pub fn from_config(logger_config: LoggerConfig) -> Self {
        logger_config.into()
    }

    /// Initialize the logger.
    /// This function initializes the logger for the application.
    /// The logger is initialized with the following layers:
    /// - a file subscriber
    /// - an error layer
    /// The file subscriber is initialized with the following settings:
    /// - file: true
    /// - line_number: true
    /// - target: true
    /// - ansi: false
    /// - writer: the log file
    /// - filter: the `RUST_LOG` environment variable
    /// The error layer is initialized with the default settings.
    pub fn init(&self) {
        self.set_rust_log_variable();
        let _ = self.delete_old_log_files();

        let file_appender = tracing_appender::rolling::RollingFileAppender::new(
            self.rotation_frequency.clone(),
            self.log_folder.clone(),
            self.log_file.clone(),
        );

        let file_subscriber = tracing_subscriber::fmt::layer()
            .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
                "%Y-%m-%dT%H:%M:%S%.6fZ".to_string(),
            ))
            .with_file(true)
            .with_line_number(true)
            .with_target(true)
            .with_ansi(false)
            .with_writer(file_appender)
            // Parsing an EnvFilter from the default environment variable
            // (RUST_LOG)
            .with_filter(EnvFilter::from_default_env()); //tracing_subscriber::filter::LevelFilter::TRACE

        Registry::default()
            .with(file_subscriber)
            .with(ErrorLayer::default())
            .init();
    }
    /// Deletes old log files from the specified log folder.
    ///
    /// This function iterates through the log files in the specified log folder, filters out files
    /// whose filenames match the provided prefix, sorts the remaining log files, and deletes the oldest ones
    /// if the number of log files exceeds a certain threshold.
    ///
    /// # Returns
    /// * `Result<(), AppError>` - The result of the operation.
    fn delete_old_log_files(&self) -> Result<(), AppError<()>> {
        let mut logs: Vec<_> = fs::read_dir(&self.log_folder)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|filename| filename.to_str())
                    .map(|name| name.starts_with(&self.log_file))
                    .unwrap_or(false)
            })
            .collect();

        logs.sort();

        let logs_to_delete = logs.len().saturating_sub(self.max_old_log_files);

        for log in logs.iter().take(logs_to_delete) {
            fs::remove_file(log)?;
        }
        Ok(())
    }
    /// Set the `RUST_LOG` environment variable.
    /// This function try to set the `RUST_LOG` environment variable to:
    /// - the value of the `RUST_LOG` environment variable
    /// - the value of `log_level` field of the `Logger` struct
    /// or to `CARGO_CRATE_NAME=info` if the `RUST_LOG` environment variable is
    /// not set.
    fn set_rust_log_variable(&self) {
        std::env::set_var(
            "RUST_LOG",
            std::env::var("RUST_LOG")
                .or_else(|_| Ok(self.log_level.clone()))
                .unwrap_or_else(|_: String| format!("{}=info", env!("CARGO_CRATE_NAME"))),
        );
    }
}
/// The conversion from the logger configuration to the logger.
impl From<LoggerConfig> for Logger {
    fn from(config: LoggerConfig) -> Self {
        Self {
            log_folder: config.log_folder,
            log_file: config.log_file,
            rotation_frequency: match config.rotation_frequency.as_str() {
                "minutely" => tracing_appender::rolling::Rotation::MINUTELY,
                "hourly" => tracing_appender::rolling::Rotation::HOURLY,
                "daily" => tracing_appender::rolling::Rotation::DAILY,
                _ => tracing_appender::rolling::Rotation::NEVER,
            },
            max_old_log_files: config.max_old_log_files,
            log_level: config.log_level,
        }
    }
}
