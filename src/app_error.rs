use config::ConfigError;
use std::{fmt::Display, io};
use tokio::sync::mpsc::error::SendError;

#[derive(Debug)]
/// An error type for the application.
/// It is used to represent the application error.
/// It can be one of the following:
/// * Io: An IO error.
/// * Send: A send error.
/// * Config: A configuration error.
/// * ConfigFile: A configuration file error.
pub enum AppError<T> {
    /// It is a wrapper for the `std::io::Error`.
    Io(io::Error),
    /// It is a wrapper for the `tokio::sync::mpsc::error::SendError`.
    Send(SendError<T>),
    /// It is a wrapper for the `config::ConfigError`.
    Config(ConfigError),
    /// It is an invalid action.
    InvalidAction(String),
    /// It is an invalid event.
    InvalidEvent(String),
    /// It is a configuration file error. It is used when a key is already
    /// bound.
    AlreadyBound,
    /// It is an invalid color.
    InvalidColor(String),
}
impl<T> From<io::Error> for AppError<T> {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl<T> From<SendError<T>> for AppError<T> {
    fn from(error: SendError<T>) -> Self {
        Self::Send(error)
    }
}
impl<T> From<ConfigError> for AppError<T> {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}
impl<T> Display for AppError<T> {
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
            Self::InvalidColor(color) => {
                write!(f, "Invalid color: {}", color)
            }
        }
    }
}
