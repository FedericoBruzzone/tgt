use crate::action::Action;
use crate::traits::component::Component;
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
  small_area: u16,
  command_tx: Option<mpsc::UnboundedSender<Action>>,
}

impl PromptWindow {
  pub fn new() -> Self {
    let name = "".to_string();
    let command_tx = None;
    let small_area = 50;

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

  pub fn small_area(mut self, small_area: u16) -> Self {
    self.small_area = small_area;
    self
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
      bottom_left: if area.width < self.small_area {
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
