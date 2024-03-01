use {
  crate::{
    action::Action,
    traits::{component::Component, handle_small_area::HandleSmallArea},
  },
  ratatui::{
    layout,
    widgets::{
      block::{self, Position, Title},
      Borders,
    },
  },
  std::io,
  tokio::sync::mpsc,
};

pub const STATUS_BAR: &str = "status_bar";

pub struct StatusBar {
  name: String,
  command_tx: Option<mpsc::UnboundedSender<Action>>,
  small_area: bool,
}

impl StatusBar {
  pub fn new() -> Self {
    let command_tx = None;
    let name = "".to_string();
    let small_area = false;
    StatusBar {
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

impl HandleSmallArea for StatusBar {
  fn small_area(&mut self, small: bool) {
    self.small_area = small;
  }
}

impl Component for StatusBar {
  fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: layout::Rect) -> io::Result<()> {
    frame.render_widget(
      block::Block::new()
        .borders(Borders::BOTTOM)
        .title(Title::from(self.name.as_str()).position(Position::Bottom))
        .title(
          Title::from(area.width.to_string() + "x" + area.height.to_string().as_str())
            .position(Position::Bottom)
            .alignment(layout::Alignment::Center),
        ),
      area,
    );

    Ok(())
  }
}
