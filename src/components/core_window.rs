use {
    crate::{
        components::{
            chat_list_window::ChatListWindow,
            chat_window::ChatWindow,
            component::{Component, HandleSmallArea},
            prompt_window::PromptWindow,
        },
        enums::{action::Action, component_name::ComponentName},
    },
    ratatui::layout::{Constraint, Direction, Layout, Rect},
    std::collections::HashMap,
    tokio::sync::mpsc::UnboundedSender,
};

/// `CoreWindow` is a struct that represents the core window of the application.
/// It is responsible for managing the layout and rendering of the core window.
pub struct CoreWindow {
    /// The name of the `CoreWindow`.
    name: String,
    /// An unbounded sender that send action for processing.
    command_tx: Option<UnboundedSender<Action>>,
    /// A map of components that are part of the `CoreWindow`.
    components: HashMap<ComponentName, Box<dyn Component>>,
    /// A flag indicating whether the `CoreWindow` should be displayed as a smaller version of itself.
    small_area: bool,
    #[allow(dead_code)]
    /// The name of the component that is currently focused.
    focused: ComponentName,
}

impl Default for CoreWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreWindow {
    /// Create a new instance of the `CoreWindow` struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `CoreWindow` struct.
    pub fn new() -> Self {
        let components_iter: Vec<(ComponentName, Box<dyn Component>)> = vec![
            (
                ComponentName::ChatList,
                ChatListWindow::new().with_name("Chats").new_boxed(),
            ),
            (ComponentName::Chat, ChatWindow::new().with_name("Name").new_boxed()),
            (
                ComponentName::Prompt,
                PromptWindow::new().with_name("Prompt").new_boxed(),
            ),
        ];

        let name = "".to_string();
        let command_tx = None;
        let components: HashMap<ComponentName, Box<dyn Component>> = components_iter.into_iter().collect();
        let small_area = false;
        let focused = ComponentName::ChatList;

        CoreWindow {
            name,
            command_tx,
            components,
            small_area,
            focused,
        }
    }
    /// Set the name of the `CoreWindow`.
    ///
    /// # Arguments
    /// * `name` - The name of the `CoreWindow`.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `CoreWindow`.
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }
}

/// Implement the `HandleSmallArea` trait for the `CoreWindow` struct.
/// This trait allows the `CoreWindow` to display a smaller version of itself if necessary.
impl HandleSmallArea for CoreWindow {
    /// Set the `small_area` flag for the `CoreWindow`.
    ///
    /// # Arguments
    /// * `small_area` - A boolean flag indicating whether the `CoreWindow` should be displayed as a smaller version of itself.
    fn with_small_area(&mut self, small_area: bool) {
        self.small_area = small_area;
        for (_, component) in self.components.iter_mut() {
            component.with_small_area(small_area);
        }
    }
}

/// Implement the `Component` trait for the `ChatListWindow` struct.
impl Component for CoreWindow {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> std::io::Result<()> {
        self.command_tx = Some(tx.clone());
        for (_, component) in self.components.iter_mut() {
            component.register_action_handler(tx.clone())?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> std::io::Result<()> {
        // let size_chats = if area.width < SMALL_AREA_WIDTH { 0 } else { 20 };
        let size_chats = if self.small_area { 0 } else { 20 };
        let size_prompt = 3;

        let core_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(size_chats),
                Constraint::Percentage(100 - size_chats),
            ])
            .split(area);

        self.components
            .get_mut(&ComponentName::ChatList)
            .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::ChatList))
            .draw(frame, core_layout[0])?;

        let sub_core_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(size_prompt)])
            .split(core_layout[1]);

        self.components
            .get_mut(&ComponentName::Chat)
            .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::Chat))
            .draw(frame, sub_core_layout[0])?;

        self.components
            .get_mut(&ComponentName::Prompt)
            .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::Prompt))
            .draw(frame, sub_core_layout[1])?;

        Ok(())
    }
}
