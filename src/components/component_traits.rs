use crate::action::Action;
use crate::app_error::AppError;
use crate::event::Event;
use crossterm::event;
use ratatui::layout;
use std::io;
use tokio::sync::mpsc;

/// `Component` is a trait that represents a visual and interactive element of
/// the user interface. Implementors of this trait can be registered with the
/// main application loop and will be able to receive events, update state, and
/// be rendered on the screen.
pub trait Component: HandleSmallArea + HandleFocus {
    /// Register an action handler that can send actions for processing if
    /// necessary.
    ///
    /// # Arguments
    ///
    /// * `tx` - An unbounded sender that can send actions.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - An Ok result or an error.
    #[allow(unused_variables)]
    fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
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
    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>, AppError<Action>> {
        let r = match event {
            Some(Event::Key(key, modifiers)) => {
                self.handle_key_events(Event::Key(key, modifiers))?
            }
            Some(Event::Mouse(mouse_event)) => self.handle_mouse_events(mouse_event)?,
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
    fn handle_key_events(&mut self, key: Event) -> io::Result<Option<Action>> {
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
    fn handle_mouse_events(&mut self, mouse: event::MouseEvent) -> io::Result<Option<Action>> {
        Ok(None)
    }
    /// Update the state of the component based on a received action. (REQUIRED)
    ///
    /// # Arguments
    ///
    /// * `action` - An action that may modify the state of the component.
    #[allow(unused_variables)]
    fn update(&mut self, action: Action) {}
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
    fn draw(&mut self, f: &mut ratatui::Frame<'_>, area: layout::Rect) -> io::Result<()>;
    /// Create a new boxed instance of the component.
    ///
    /// # Returns
    ///
    /// * `Box<Self>` - A boxed instance of the component.
    fn new_boxed(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}
/// `HandleSmallArea` is a trait that represents a component that can handle
/// small area events. Implementors of this trait can be notified when the area
/// they are rendering in is too small to be useful. This can be useful for
/// components that require a minimum amount of space to be useful.
pub trait HandleSmallArea {
    /// This method is called when the area is too small to be useful.
    /// This should set the state of the component to reflect the fact that the
    /// area is too small.
    ///
    /// # Arguments
    ///
    /// * `small_area` - A boolean indicating if the area is too small.
    #[allow(unused_variables)]
    fn with_small_area(&mut self, small_area: bool);
}

/// `HandleFocus` is a trait that represents a component that can handle focus
/// actions. Implementors of this trait can be notified when they have focus
/// and when they lose focus.
pub trait HandleFocus {
    /// This method is called when the component should focus.
    /// This should set the state of the component to reflect the fact that it
    /// has focus.
    fn focus(&mut self);
    /// This method is called when the component should unfocus.
    /// This should set the state of the component to reflect the fact that it
    /// has lost focus.
    fn unfocus(&mut self);
}
