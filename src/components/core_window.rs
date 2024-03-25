use {
    super::{MAX_CHAT_LIST_SIZE, MAX_PROMPT_SIZE},
    crate::{
        components::{
            chat_list_window::ChatListWindow,
            chat_window::ChatWindow,
            component::{Component, HandleFocus, HandleSmallArea},
            prompt_window::PromptWindow,
        },
        enums::{action::Action, component_name::ComponentName},
    },
    ratatui::layout::{Constraint, Direction, Layout, Rect},
    std::{collections::HashMap, io},
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
    /// A flag indicating whether the `CoreWindow` should be displayed as a
    /// smaller version of itself.
    small_area: bool,
    /// The size of the prompt component.
    size_prompt: u16,
    /// The size of the chat list component.
    size_chat_list: u16,
    /// The name of the component that currently has focus. It is an optional
    /// value because no component may have focus. The focus is a component
    /// inside the `CoreWindow`.
    component_focused: Option<ComponentName>,
    /// Indicates whether the `CoreWindow` is focused or not.
    focused: bool,
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
                ChatListWindow::new().with_name("Chat list").new_boxed(),
            ),
            (
                ComponentName::Chat,
                ChatWindow::new().with_name("Name").new_boxed(),
            ),
            (
                ComponentName::Prompt,
                PromptWindow::new().with_name("Prompt").new_boxed(),
            ),
        ];

        let name = "".to_string();
        let command_tx = None;
        let components: HashMap<ComponentName, Box<dyn Component>> =
            components_iter.into_iter().collect();
        let size_prompt = 3;
        let size_chat_list = 20;
        let small_area = false;
        let component_focused = None;
        let focused = false;

        CoreWindow {
            name,
            command_tx,
            components,
            size_chat_list,
            size_prompt,
            small_area,
            component_focused,
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

    /// Increase the size of the chat list component.
    pub fn increase_chat_list_size(&mut self) {
        if self.size_chat_list == MAX_CHAT_LIST_SIZE {
            return;
        }
        self.size_chat_list += 1;
    }

    /// Increase the size of the chat list component.
    pub fn increase_size_prompt(&mut self) {
        if self.size_prompt == MAX_PROMPT_SIZE {
            return;
        }
        self.size_prompt += 1;
    }

    /// Decrease the size of the chat list component.
    pub fn decrease_chat_list_size(&mut self) {
        if self.size_chat_list == 0 {
            return;
        }
        self.size_chat_list -= 1;
    }

    /// Decrease the size of the chat list component.
    pub fn decrease_size_prompt(&mut self) {
        if self.size_prompt == 0 {
            return;
        }
        self.size_prompt -= 1;
    }
}

/// Implement the `HandleFocus` trait for the `CoreWindow` struct.
/// This trait allows the `CoreWindow` to be focused or unfocused.
impl HandleFocus for CoreWindow {
    /// Set the `focused` flag for the `CoreWindow`.
    fn focus(&mut self) {
        self.focused = true;
    }
    /// Set the `focused` flag for the `CoreWindow`.
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

/// Implement the `HandleSmallArea` trait for the `CoreWindow` struct.
/// This trait allows the `CoreWindow` to display a smaller version of itself if
/// necessary.
impl HandleSmallArea for CoreWindow {
    /// Set the `small_area` flag for the `CoreWindow`.
    ///
    /// # Arguments
    /// * `small_area` - A boolean flag indicating whether the `CoreWindow`
    ///   should be displayed as a smaller version of itself.
    fn with_small_area(&mut self, small_area: bool) {
        self.small_area = small_area;
        for (_, component) in self.components.iter_mut() {
            component.with_small_area(small_area);
        }
    }
}

/// Implement the `Component` trait for the `ChatListWindow` struct.
impl Component for CoreWindow {
    fn register_action_handler(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> std::io::Result<()> {
        self.command_tx = Some(tx.clone());
        for (_, component) in self.components.iter_mut() {
            component.register_action_handler(tx.clone())?;
        }
        Ok(())
    }

    fn update(&mut self, action: Action) -> io::Result<Option<Action>> {
        match action {
            Action::FocusComponent(component_name) => {
                self.component_focused = Some(component_name);
                self.components
                    .get_mut(&component_name)
                    .unwrap_or_else(|| {
                        panic!("Failed to get component: {}", component_name)
                    })
                    .focus();
                self.components
                    .iter_mut()
                    .filter(|(name, _)| *name != &component_name)
                    .for_each(|(_, component)| component.unfocus());
                Ok(None)
            }
            Action::UnfocusComponent => {
                self.component_focused = None;
                for (_, component) in self.components.iter_mut() {
                    component.unfocus();
                }
                Ok(None)
            }
            Action::IncreaseChatListSize => {
                self.increase_chat_list_size();
                Ok(None)
            }
            Action::DecreaseChatListSize => {
                self.decrease_chat_list_size();
                Ok(None)
            }
            Action::IncreasePromptSize => {
                self.increase_size_prompt();
                Ok(None)
            }
            Action::DecreasePromptSize => {
                self.decrease_size_prompt();
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame<'_>,
        area: Rect,
    ) -> io::Result<()> {
        let size_chat_list = if self.small_area {
            0
        } else {
            self.size_chat_list
        };

        let core_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(size_chat_list),
                Constraint::Percentage(100 - size_chat_list),
            ])
            .split(area);

        self.components
            .get_mut(&ComponentName::ChatList)
            .unwrap_or_else(|| {
                panic!("Failed to get component: {}", ComponentName::ChatList)
            })
            .draw(frame, core_layout[0])?;

        let sub_core_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(self.size_prompt),
            ])
            .split(core_layout[1]);

        self.components
            .get_mut(&ComponentName::Chat)
            .unwrap_or_else(|| {
                panic!("Failed to get component: {}", ComponentName::Chat)
            })
            .draw(frame, sub_core_layout[0])?;

        self.components
            .get_mut(&ComponentName::Prompt)
            .unwrap_or_else(|| {
                panic!("Failed to get component: {}", ComponentName::Prompt)
            })
            .draw(frame, sub_core_layout[1])?;

        Ok(())
    }
}
