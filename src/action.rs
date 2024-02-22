use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone)]
pub enum Action {
  Init, // Unused
  Quit,
  Render,
  Key(KeyEvent),
  Mouse(MouseEvent),
  Resize(u16, u16),
}
