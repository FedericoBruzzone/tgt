use {
  crate::{
    enums::action::Action,
    traits::{component::Component, handle_small_area::HandleSmallArea},
  },
  ratatui::{
    layout::Rect,
    symbols::{border, line},
    widgets::{block, Borders},
  },
  std::io,
  tokio::sync::mpsc::UnboundedSender,
};

pub const CHAT: &str = "chat_window";

pub struct ChatWindow {
  name: String,
  command_tx: Option<UnboundedSender<Action>>,
  small_area: bool,
}

impl ChatWindow {
  pub fn new() -> Self {
    let command_tx = None;
    let name = "".to_string();
    let small_area = false;
    ChatWindow {
      command_tx,
      name,
      small_area,
    }
  }

  pub fn name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    self
  }
}

impl HandleSmallArea for ChatWindow {
  fn small_area(&mut self, small: bool) {
    self.small_area = small;
  }
}

impl Component for ChatWindow {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> io::Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> io::Result<()> {
    let border = if self.small_area {
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
