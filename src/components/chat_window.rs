use crate::action::Action;
use crate::components::SMALL_AREA_WIDTH;
use crate::traits::component::Component;
use ratatui::{
  layout::Rect,
  symbols::{border, line},
  widgets::{block, Borders},
};
use std::io;
use tokio::sync::mpsc::UnboundedSender;

pub const CHAT: &str = "chat_window";

pub struct ChatWindow {
  name: String,
  command_tx: Option<UnboundedSender<Action>>,
}

impl ChatWindow {
  pub fn new() -> Self {
    let command_tx = None;
    let name = "".to_string();
    ChatWindow { command_tx, name }
  }

  pub fn name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    self
  }
}

impl Component for ChatWindow {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> io::Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> io::Result<()> {
    let border = if area.width < SMALL_AREA_WIDTH {
      border::PLAIN
    } else {
      border::Set {
        top_left: line::NORMAL.horizontal_down,
        bottom_left: line::NORMAL.horizontal_up,
        ..border::PLAIN
      }
    };

    frame.render_widget(
      block::Block::new()
        .border_set(border)
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .title(self.name.as_str()),
      area,
    );

    Ok(())
  }
}
