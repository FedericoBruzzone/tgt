use {
  crate::{
    enums::action::Action,
    traits::{component::Component, handle_small_area::HandleSmallArea},
  },
  ratatui::{
    layout::{Alignment, Rect},
    widgets::{
      block::{Block, Position, Title},
      Borders,
    },
  },
  tokio::sync::mpsc::UnboundedSender,
};

pub const STATUS_BAR: &str = "status_bar";

/// `StatusBar` is a struct that represents a status bar.
/// It is responsible for managing the layout and rendering of the status bar.
pub struct StatusBar {
  /// The name of the `StatusBar`.
  name: String,
  /// An unbounded sender that send action for processing.
  command_tx: Option<UnboundedSender<Action>>,
  /// A flag indicating whether the `StatusBar` should be displayed as a smaller version of itself.
  small_area: bool,
}

impl StatusBar {
  /// Create a new instance of the `StatusBar` struct.
  ///
  /// # Returns
  /// * `Self` - The new instance of the `StatusBar` struct.
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
  /// Set the name of the `StatusBar`.
  ///
  /// # Arguments
  /// * `name` - The name of the `StatusBar`.
  ///
  /// # Returns
  /// * `Self` - The modified instance of the `StatusBar`.
  pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
    self.name = name.as_ref().to_string();
    self
  }
}

/// Implement the `HandleSmallArea` trait for the `StatusBar` struct.
/// This trait allows the `StatusBar` to display a smaller version of itself if necessary.
impl HandleSmallArea for StatusBar {
  /// Set the `small_area` flag for the `StatusBar`.
  ///
  /// # Arguments
  /// * `small_area` - A boolean flag indicating whether the `StatusBar` should be displayed as a smaller version of itself.
  fn with_small_area(&mut self, small_area: bool) {
    self.small_area = small_area;
  }
}

impl Component for StatusBar {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> std::io::Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> std::io::Result<()> {
    frame.render_widget(
      Block::new()
        .borders(Borders::BOTTOM)
        .title(Title::from(self.name.as_str()).position(Position::Bottom))
        .title(
          Title::from(area.width.to_string() + "x" + area.height.to_string().as_str())
            .position(Position::Bottom)
            .alignment(Alignment::Center),
        ),
      area,
    );

    Ok(())
  }
}
