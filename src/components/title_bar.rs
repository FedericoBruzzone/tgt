use super::Component;
use crate::action::Action;
use ratatui::{
  layout::{self, Alignment},
  widgets::{
    block::{self, Position, Title},
    Borders,
  },
};
use std::io;
use tokio::sync::mpsc;

pub struct TitleBar {
  name: String,
  command_tx: Option<mpsc::UnboundedSender<Action>>,
}

impl TitleBar {
  pub fn new() -> Self {
    let command_tx = None;
    let name = "".to_string();
    TitleBar { command_tx, name }
  }

  pub fn name(mut self, name: String) -> Self {
    self.name = name;
    self
  }

  pub fn new_boxed(mut self) -> Box<Self> {
    Box::new(self)
  }
}

impl Component for TitleBar {
  fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: layout::Rect) -> io::Result<()> {
    frame.render_widget(
      block::Block::new().borders(Borders::TOP).title(
        Title::from(self.name.clone())
          .position(Position::Top)
          .alignment(Alignment::Center),
      ),
      area,
    );

    Ok(())
  }
}
