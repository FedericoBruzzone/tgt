use {
    crate::{app_error::AppError, configs::custom::logger_custom::LoggerConfig},
    std::{
        fs::{self, File},
        path::PathBuf,
    },
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
    log_folder: PathBuf,
    /// The log file.
    log_file: String,
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
    ///
    /// # Returns
    /// * `Result<&Self, AppError>` - The result of the operation.
    pub fn init(&self) {
        self.set_rust_log_variable();
        let file = self.create_log_file().unwrap();

        let file_subscriber = tracing_subscriber::fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_target(true)
            .with_ansi(false)
            .with_writer(file)
            // Parsing an EnvFilter from the default environment variable
            // (RUST_LOG)
            .with_filter(EnvFilter::from_default_env()); //tracing_subscriber::filter::LevelFilter::TRACE

        Registry::default()
            .with(file_subscriber)
            .with(ErrorLayer::default())
            .init();
    }

    /// Create the log file for the application.
    /// By default, the log file is `tgt.log` in the `.data` directory of the
    /// current working directory.
    ///
    /// # Returns
    /// * `Result<(), AppError>` - The result of the operation.
    fn create_log_file(&self) -> Result<File, AppError> {
        let folder = self.log_folder.clone();
        fs::create_dir_all(folder.clone())?;
        let file = File::create(folder.join(self.log_file.clone()))?;
        Ok(file)
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
            log_folder: PathBuf::from(config.log_folder),
            log_file: config.log_file,
            log_level: config.log_level,
        }
    }
}
