use crossterm::event::{KeyEvent, MouseEvent};

/// `Action` is an enum that represents an action that can be handled by the main application loop
/// and the components of the user interface.
#[derive(Debug, Clone)]
pub enum Action {
    Init, // Unused
    Quit,
    Render,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}
