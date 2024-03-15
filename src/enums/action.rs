use {
    crate::app_error::AppError,
    crossterm::event::{KeyEvent, MouseEvent},
    std::str::FromStr,
};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
// Action` is an enum that represents an action that can be handled by the
/// main application loop and the components of the user interface.
pub enum Action {
    Unknown,
    Init,
    Quit,
    Render,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

/// Implement the `FormStr` trait for `Action`.
impl FromStr for Action {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "quit" => Ok(Action::Quit),
            "render" => Ok(Action::Render),
            _ => Err(AppError::InvalidAction(s.to_string())),
        }
    }
}
