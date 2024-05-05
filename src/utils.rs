use std::{env, io, path::PathBuf};

pub const TGT_PROGRAM_NAME: &str = "tgt";
pub const TGT_CONFIG_HOME: &str = "TGT_CONFIG_HOME";

/// Get the project directory.
///
/// # Returns
/// The project directory.
pub fn project_dir() -> io::Result<PathBuf> {
    env::current_dir()
}
/// Get the default configuration directory.
///
/// # Returns
/// The default configuration directory.
pub fn default_config_dir() -> io::Result<PathBuf> {
    Ok(project_dir()?.join("config"))
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
    eprintln!("[ERROR]: {} {:?}", msg, e);
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
