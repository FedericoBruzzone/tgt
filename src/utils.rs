use dirs;
use std::{env, io, path::PathBuf};

pub const TGT: &str = "tgt";
pub const TGT_CONFIG_DIR: &str = "TGT_CONFIG_DIR";

/// Get the project directory, creating it if it doesn't exist.
///
/// # Returns
/// The project directory.
pub fn tgt_dir() -> io::Result<PathBuf> {
    // Debug
    if cfg!(debug_assertions) {
        return env::current_dir();
    }

    // Release
    let home = dirs::home_dir().unwrap().to_str().unwrap().to_owned();
    let tgt_path = PathBuf::from(format!("{home}/.tgt"));

    // Create the directory if it doesn't exist
    if !tgt_path.exists() {
        std::fs::create_dir_all(&tgt_path)?;
        tracing::info!("Created .tgt directory at: {}", tgt_path.display());
    }

    Ok(tgt_path)
}
/// Get the default configuration directory, creating it if it doesn't exist.
///
/// # Returns
/// The default configuration directory.
pub fn tgt_config_dir() -> io::Result<PathBuf> {
    let config_dir = tgt_dir()?.join("config");

    // Create the config directory if it doesn't exist
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
        tracing::info!("Created config directory at: {}", config_dir.display());
    }

    Ok(config_dir)
}

/// Fail with an error message and exit the application.
///
/// # Arguments
/// * `msg` - A string slice that holds the error message.
/// * `e` - A generic type that holds the error.
///
/// # Returns
/// * `!` - This function does not return a value.
fn fail_with<E: std::fmt::Debug>(msg: &str, e: E) -> ! {
    eprintln!("[ERROR]: {msg} {e:?}");
    std::process::exit(1);
}

/// Unwrap a result or fail with an error message.
/// This function will unwrap a result and return the value if it is Ok.
/// If the result is an error, this function will fail with an error message.
///
/// # Arguments
/// * `result` - A result that holds a value or an error.
/// * `msg` - A string slice that holds the error message.
///
/// # Returns
/// * `T` - The value if the result is Ok.
pub fn unwrap_or_fail<T, E: std::fmt::Debug>(result: Result<T, E>, msg: &str) -> T {
    match result {
        Ok(v) => v,
        Err(e) => fail_with(msg, e),
    }
}
