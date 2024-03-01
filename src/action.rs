use crossterm::event::{KeyEvent, MouseEvent};

/// `Action` is an enum that represents an action that can be taken by a component.
#[derive(Debug, Clone)]
pub enum Action {
  Init, // Unused
  Quit,
  Render,
  Key(KeyEvent),
  Mouse(MouseEvent),
  Resize(u16, u16),
}
