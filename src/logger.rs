use {
        lazy_static::lazy_static,
        std::{fs::File, path::PathBuf},
        tracing_error::ErrorLayer,
        tracing_subscriber::{
                filter::EnvFilter, prelude::__tracing_subscriber_SubscriberExt, registry::Registry,
                util::SubscriberInitExt, Layer,
        },
};

lazy_static! {
        pub static ref LOG_FOLDER: Option<PathBuf> = Some(PathBuf::from(".data"));
        pub static ref LOG_FILE: Option<String> = Some("tgt.log".to_string());
        pub static ref LOG_LEVEL: String = "LOG_LEVEL".to_string();
}

/// Set the `RUST_LOG` environment variable to the value of the `LOG_LEVEL` environment variable if it is set.
/// If the `LOG_LEVEL` environment variable is not set, then the `RUST_LOG` environment variable is set to the value of the `CARGO_CRATE_NAME` environment variable with a log level of `info`.
fn set_rust_log_variable() {
        std::env::set_var(
                "RUST_LOG",
                std::env::var("RUST_LOG")
                        .or_else(|_| std::env::var(LOG_LEVEL.clone()))
                        .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME"))),
        );
}

/// Get the log folder for the application.
/// By default, the log folder is the `.data` directory in the current working directory.
///
/// # Returns
/// * `PathBuf` - The path to the log folder.
fn log_folder() -> PathBuf {
        if let Some(folder) = LOG_FOLDER.clone() {
                folder
        } else {
                PathBuf::from(".").join(".data")
        }
}

/// Get the log file for the application.
/// By default, the log file is `tgt.log` in the log folder.
///
/// # Returns
/// * `String` - The log file.
fn log_file() -> String {
        if let Some(file) = LOG_FILE.clone() {
                file
        } else {
                "tgt.log".to_string()
        }
}

/// Create the log file for the application.
/// By default, the log file is `tgt.log` in the log folder.
///
/// # Returns
/// * `std::io::Result<File>` - The log file.
fn create_log_file() -> std::io::Result<File> {
        let folder = log_folder();
        std::fs::create_dir_all(folder.clone())?;
        let log_path = folder.join(log_file());
        File::create(log_path)
}

/// Initialize the logging system for the application.
/// By default, the logging system will write log messages to a file in the `.data` directory of the current working directory.
pub fn initialize_logging() -> std::io::Result<()> {
        set_rust_log_variable();
        let directory = log_folder();
        std::fs::create_dir_all(directory.clone())?;
        let log_file = create_log_file()?;

        let file_subscriber = tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true)
                .with_target(true)
                .with_ansi(false)
                .with_writer(log_file)
                // Parsing an EnvFilter from the default environment variable (RUST_LOG)
                .with_filter(EnvFilter::from_default_env()); //tracing_subscriber::filter::LevelFilter::TRACE

        Registry::default()
                .with(file_subscriber)
                .with(ErrorLayer::default())
                .init();

        Ok(())
}
