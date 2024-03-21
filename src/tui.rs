use {
    crate::{
        app_error::AppError,
        components::{
            component::Component, core_window::CoreWindow,
            status_bar::StatusBar, title_bar::TitleBar, SMALL_AREA_HEIGHT,
            SMALL_AREA_WIDTH,
        },
        configs::custom::keymap_custom::KeymapConfig,
        enums::{action::Action, component_name::ComponentName, event::Event},
    },
    ratatui::layout::{Constraint, Direction, Layout, Rect},
    std::{collections::HashMap, io},
    tokio::sync::mpsc::UnboundedSender,
};

/// `Tui` is a struct that represents the main user interface for the
/// application. It is responsible for managing the layout and rendering of all
/// the components. It also handles the distribution of events and actions to
/// the appropriate components.
pub struct Tui {
    /// An optional unbounded sender that can send actions to be processed.
    action_tx: Option<UnboundedSender<Action>>,
    /// A hashmap of components that make up the user interface.
    components: HashMap<ComponentName, Box<dyn Component>>,
    /// The keymap configuration.
    keymap_config: Option<KeymapConfig>,
    #[allow(dead_code)]
    /// The name of the component that currently has focus. It is an optional
    /// value because no component may have focus. The focus is a component
    /// inside the `CoreWindow`.
    focused: Option<ComponentName>,
}
/// Implement the `Default` trait for the `Tui` struct.
impl Default for Tui {
    fn default() -> Self {
        Self::new()
    }
}
/// Implement the `Tui` struct.
impl Tui {
    /// Create a new instance of the `Tui` struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `Tui` struct.
    pub fn new() -> Self {
        let components_iter: Vec<(ComponentName, Box<dyn Component>)> = vec![
            (
                ComponentName::TitleBar,
                TitleBar::new().with_name("TG-TUI").new_boxed(),
            ),
            (
                ComponentName::CoreWindow,
                CoreWindow::new().with_name("CoreWindow").new_boxed(),
            ),
            (
                ComponentName::StatusBar,
                StatusBar::new().with_name("Status Bar").new_boxed(),
            ),
        ];

        let action_tx = None;
        let focused = None;
        let components: HashMap<ComponentName, Box<dyn Component>> =
            components_iter.into_iter().collect();
        let keymap_config = None;

        Tui {
            action_tx,
            components,
            keymap_config,
            focused,
        }
    }
    /// Set the keymap configuration for the user interface.
    /// The keymap configuration is used to map events to actions.
    ///
    /// # Arguments
    /// * `keymap_config` - The keymap configuration for the user interface.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `Tui` struct.
    pub fn with_keymap_config(mut self, keymap_config: KeymapConfig) -> Self {
        self.keymap_config = Some(keymap_config);
        self
    }
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
    pub fn register_action_handler(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> Result<(), AppError> {
        self.action_tx = Some(tx.clone());
        self.components.iter_mut().try_for_each(|(_, component)| {
            component.register_action_handler(tx.clone())
        })?;
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
    pub fn handle_events(
        &mut self,
        event: Option<Event>,
    ) -> Result<Option<Action>, AppError> {
        // It would be enough send the action to the `CoreWindow` component
        self.components
            .iter_mut()
            .try_fold(None, |acc, (_, component)| {
                match component.handle_events(event.clone()) {
                    Ok(Some(action)) => Ok(Some(action)),
                    Ok(None) => Ok(acc),
                    Err(e) => Err(e),
                }
            })
    }
    /// Update the state of the component based on a received action.
    ///
    /// # Arguments
    ///
    /// * `action` - An action that may modify the state of the component.
    ///
    /// # Returns
    ///
    /// * `Result<Option<Action>>` - An action to be processed or none.
    pub fn update(&mut self, action: Action) -> io::Result<Option<Action>> {
        match action {
            Action::FocusComponent(component_name) => {
                self.focused = Some(component_name);
            }
            Action::UnfocusComponent => {
                self.focused = None;
            }
            _ => {}
        }
        // It would be enough send the action to the `CoreWindow` component
        self.components
            .iter_mut()
            .try_fold(None, |acc, (_, component)| {
                match component.update(action.clone()) {
                    Ok(Some(action)) => Ok(Some(action)),
                    Ok(None) => Ok(acc),
                    Err(e) => Err(e),
                }
            })
    }
    /// Render the user interface to the screen.
    ///
    /// # Arguments
    /// * `frame` - A mutable reference to the frame to be rendered.
    /// * `area` - A rectangular area to render the user interface within.
    ///
    /// # Returns
    /// * `Result<()>` - An Ok result or an error.
    pub fn draw(
        &mut self,
        frame: &mut ratatui::Frame<'_>,
        area: Rect,
    ) -> Result<(), AppError> {
        if area.width < SMALL_AREA_WIDTH {
            self.components.iter_mut().for_each(|(_, component)| {
                component.with_small_area(true);
            });
        } else {
            self.components.iter_mut().for_each(|(_, component)| {
                component.with_small_area(false);
            });
        }

        let main_layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Min(SMALL_AREA_HEIGHT),
                Constraint::Length(1),
            ],
        )
        .split(area);

        self.components
            .get_mut(&ComponentName::TitleBar)
            .unwrap_or_else(|| {
                panic!("Failed to get component: {}", ComponentName::TitleBar)
            })
            .draw(frame, main_layout[0])?;

        self.components
            .get_mut(&ComponentName::CoreWindow)
            .unwrap_or_else(|| {
                panic!("Failed to get component: {}", ComponentName::CoreWindow)
            })
            .draw(frame, main_layout[1])?;

        self.components
            .get_mut(&ComponentName::StatusBar)
            .unwrap_or_else(|| {
                panic!("Failed to get component: {}", ComponentName::StatusBar)
            })
            .draw(frame, main_layout[2])?;

        Ok(())
    }
}
