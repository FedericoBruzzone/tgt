use crossterm::event;

#[derive(Debug, Clone)]
pub enum Action {
  Init, // Unused
  Quit,
  Render,
  Key(event::KeyEvent),
  Resize(u16, u16),
}
