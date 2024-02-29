use crate::action::Action;
use crate::traits::{component::Component, handle_small_area::HandleSmallArea};
use ratatui::{
  layout,
  symbols::{border, line},
  widgets::{block::Block, Borders},
};
use std::io;
use tokio::sync::mpsc;

pub const PROMPT: &str = "prompt_window";

pub struct PromptWindow {
  name: String,
  command_tx: Option<mpsc::UnboundedSender<Action>>,
  small_area: bool,
}

impl PromptWindow {
  pub fn new() -> Self {
    let name = "".to_string();
    let command_tx = None;
    let small_area = false;

    PromptWindow {
      name,
      command_tx,
      small_area,
    }
  }

  pub fn name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    self
  }
}

impl HandleSmallArea for PromptWindow {
  fn small_area(&mut self, small_area: bool) {
    self.small_area = small_area;
  }
}

impl Component for PromptWindow {
  fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
    self.command_tx = Some(tx.clone());
    Ok(())
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: layout::Rect) -> io::Result<()> {
    let collapsed_top_and_left_border_set = border::Set {
      top_left: line::NORMAL.vertical_right,
      top_right: line::NORMAL.vertical_left,
      bottom_left: if self.small_area {
        line::NORMAL.bottom_left
      } else {
        line::NORMAL.horizontal_up
      },
      ..border::PLAIN
    };

    frame.render_widget(
      Block::new()
        .border_set(collapsed_top_and_left_border_set)
        .borders(Borders::ALL)
        .title(self.name.as_str()),
      area,
    );

    Ok(())
  }
}
