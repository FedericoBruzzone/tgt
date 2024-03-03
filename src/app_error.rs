use {crate::enums::action::Action, std::io, tokio::sync::mpsc::error::SendError};

#[derive(Debug)]
/// An error type for the application.
/// This type is used to represent errors that occur during the execution of the application.
/// It is used to wrap errors from the `std::io` module and the `tokio::sync::mpsc` module.
/// This type is used as the error type for the `Result` type returned by the `main` function.
pub enum AppError {
  Io(io::Error),
  Send(SendError<Action>),
}
/// Convert an `std::io::Error` into an `AppError`.
impl From<io::Error> for AppError {
  fn from(error: io::Error) -> Self {
    Self::Io(error)
  }
}
/// Convert a `tokio::sync::mpsc::error::SendError` into an `AppError`.
impl From<SendError<Action>> for AppError {
  fn from(error: SendError<Action>) -> Self {
    Self::Send(error)
  }
}
/// Implement the `Display` trait for `AppError`.
impl std::fmt::Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::Io(error) => write!(f, "IO error: {}", error),
      Self::Send(error) => write!(f, "Send error: {}", error),
    }
  }
}
/// Implement the `Error` trait for `AppError`.
impl std::error::Error for AppError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    match self {
      Self::Io(error) => Some(error),
      Self::Send(error) => Some(error),
    }
  }
}
