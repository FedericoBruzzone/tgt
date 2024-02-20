use crate::{action::Action, tui::Event};
use crossterm::event;
use ratatui::layout;
use std::io;
use tokio::sync::mpsc;

pub mod home;

/// `Component` is a trait that represents a visual and interactive element of the user interface.
/// Implementors of this trait can be registered with the main application loop and will be able to receive events,
/// update state, and be rendered on the screen.
pub trait Component {
  /// Register an action handler that can send actions for processing if necessary.
  ///
  /// # Arguments
  ///
  /// * `tx` - An unbounded sender that can send actions.
  ///
  /// # Returns
  ///
  /// * `Result<()>` - An Ok result or an error.
  #[allow(unused_variables)]
  fn register_action_handler(
    &mut self,
    tx: mpsc::UnboundedSender<Action>,
  ) -> io::Result<()> {
    Ok(())
  }
  /// Initialize the component with a specified area if necessary.
  ///
  /// # Arguments
  ///
  /// * `area` - Rectangular area to initialize the component within.
  ///
  /// # Returns
  ///
  /// * `Result<()>` - An Ok result or an error.
  #[allow(unused_variables)]
  fn init(&mut self, area: layout::Rect) -> io::Result<()> {
    Ok(())
  }
  /// Handle incoming events and produce actions if necessary.
  ///
  /// # Arguments
  ///
  /// * `event` - An optional event to be processed.
  ///
  /// # Returns
  ///
  /// * `Result<Option<Action>>` - An action to be processed or none.
  fn handle_events(
    &mut self,
    event: Option<Event>,
  ) -> io::Result<Option<Action>> {
    let r = match event {
      Some(Event::Key(key_event)) => self.handle_key_events(key_event)?,
      // Some(Event::Mouse(mouse_event)) => {
      //   self.handle_mouse_events(mouse_event)?
      // }
      _ => None,
    };
    Ok(r)
  }
  /// Handle key events and produce actions if necessary.
  ///
  /// # Arguments
  ///
  /// * `key` - A key event to be processed.
  ///
  /// # Returns
  ///
  /// * `Result<Option<Action>>` - An action to be processed or none.
  #[allow(unused_variables)]
  fn handle_key_events(
    &mut self,
    key: event::KeyEvent,
  ) -> io::Result<Option<Action>> {
    Ok(None)
  }
  /// Handle mouse events and produce actions if necessary.
  ///
  /// # Arguments
  ///
  /// * `mouse` - A mouse event to be processed.
  ///
  /// # Returns
  ///
  /// * `Result<Option<Action>>` - An action to be processed or none.
  #[allow(unused_variables)]
  fn handle_mouse_events(
    &mut self,
    mouse: event::MouseEvent,
  ) -> io::Result<Option<Action>> {
    Ok(None)
  }
  /// Update the state of the component based on a received action. (REQUIRED)
  ///
  /// # Arguments
  ///
  /// * `action` - An action that may modify the state of the component.
  ///
  /// # Returns
  ///
  /// * `Result<Option<Action>>` - An action to be processed or none.
  #[allow(unused_variables)]
  fn update(&mut self, action: Action) -> io::Result<Option<Action>> {
    Ok(None)
  }
  /// Render the component on the screen. (REQUIRED)
  ///
  /// # Arguments
  ///
  /// * `f` - A frame used for rendering.
  /// * `area` - The area in which the component should be drawn.
  ///
  /// # Returns
  ///
  /// * `Result<()>` - An Ok result or an error.
  fn draw(
    &mut self,
    f: &mut ratatui::Frame<'_>,
    area: layout::Rect,
  ) -> io::Result<()>;
}
