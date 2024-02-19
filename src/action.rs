use crossterm::event;

#[derive(Debug, Clone)]
pub enum Action {
  Quit,
  Key(event::KeyEvent),
}
