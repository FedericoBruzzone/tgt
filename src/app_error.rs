use {
    crate::enums::action::Action,
    config::ConfigError,
    std::{fmt::Display, io},
    tokio::sync::mpsc::error::SendError,
};

#[derive(Debug)]
/// An error type for the application.
/// It is used to represent the application error.
/// It can be one of the following:
/// * Io: An IO error.
/// * Send: A send error.
/// * Config: A configuration error.
/// * ConfigFile: A configuration file error.
pub enum AppError {
    /// It is a wrapper for the `std::io::Error`.
    Io(io::Error),
    /// It is a wrapper for the `tokio::sync::mpsc::error::SendError`.
    Send(SendError<Action>),
    /// It is a wrapper for the `config::ConfigError`.
    Config(ConfigError),
    /// It is an invalid action.
    InvalidAction(String),
    /// It is an invalid event.
    InvalidEvent(String),
    /// It is a configuration file error. It is used when a key is already
    /// bound.
    AlreadyBound,
}
/// Convert an `std::io::Error` into an `AppError`.
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
/// Convert a `tokio::sync::mpsc::error::SendError` into an `AppError`.
impl From<SendError<Action>> for AppError {
    fn from(error: SendError<Action>) -> Self {
        Self::Send(error)
    }
}
/// Convert a `config::ConfigError` into an `AppError`.
impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}
/// Implement the `Display` trait for `AppError`.
impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "IO error: {}", error),
            Self::Send(error) => write!(f, "Send error: {}", error),
            Self::Config(error) => write!(f, "Config error: {}", error),
            Self::InvalidAction(action) => {
                write!(f, "Invalid action: {}", action)
            }
            Self::InvalidEvent(event) => {
                write!(f, "Invalid event: {}", event)
            }
            Self::AlreadyBound => {
                write!(f, "Key already bound")
            }
        }
    }
}
