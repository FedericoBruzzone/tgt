use crate::action::Action;
use crate::traits::component::Component;
use ratatui::{
  layout,
  symbols::{border, line},
  widgets::{block::{Block, Title}, Borders},
};
use std::{collections::HashMap, io};
use tokio::sync::mpsc;

pub const PROMPT: &str = "prompt_window";

pub struct PromptWindow {
  name: String,
  command_tx: Option<mpsc::UnboundedSender<Action>>,
}

impl PromptWindow{
  pub fn new() -> Self {
    let name = "".to_string();
    let command_tx = None;

    ChatsWindow  { name, command_tx }
  }

  pub fn name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    self
  }
}

impl Component for PromptWindow {
  fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
    self.command_tx = Some(tx.clone());
    Ok(())
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: layout::Rect) -> io::Result<()> {
    frame.render_widget(
      Block::new()
        .border_set(border::PLAIN)
        .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
        .title(Title::from(self.name.as_str())),
      area,
    );

    Ok(())
  }
}
