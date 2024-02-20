use crossterm::event;

#[derive(Debug, Clone)]
pub enum Action {
  Init, // Unused
  Quit,
  Render,
  Key(event::KeyEvent),
}
