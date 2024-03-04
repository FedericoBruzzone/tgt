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
  tokio::sync::mpsc::UnboundedSender,
};

pub const CHAT: &str = "chat_window";

/// `ChatWindow` is a struct that represents a window for displaying a chat.
/// It is responsible for managing the layout and rendering of the chat window.
pub struct ChatWindow {
  /// The name of the `ChatWindow`.
  name: String,
  /// An unbounded sender that send action for processing.
  command_tx: Option<UnboundedSender<Action>>,
  /// A flag indicating whether the `ChatWindow` should be displayed as a smaller version of itself.
  small_area: bool,
}

impl ChatWindow {
  /// Create a new instance of the `ChatWindow` struct.
  ///
  /// # Returns
  /// * `Self` - The new instance of the `ChatWindow` struct.
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
  /// Set the name of the `ChatWindow`.
  ///
  /// # Arguments
  /// * `name` - The name of the `ChatWindow`.
  ///
  /// # Returns
  /// * `Self` - The modified instance of the `ChatWindow`.
  pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
    self.name = name.as_ref().to_string();
    self
  }
}

/// Implement the `HandleSmallArea` trait for the `ChatWindow` struct.
/// This trait allows the `ChatWindow` to display a smaller version of itself if necessary.
impl HandleSmallArea for ChatWindow {
  /// Set the `small_area` flag for the `ChatWindow`.
  ///
  /// # Arguments
  /// * `small_area` - A boolean flag indicating whether the `ChatWindow` should be displayed as a smaller version of itself.
  fn with_small_area(&mut self, small_area: bool) {
    self.small_area = small_area;
  }
}

impl Component for ChatWindow {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> std::io::Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> std::io::Result<()> {
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
