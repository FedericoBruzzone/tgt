use crate::{
    action::Action,
    app_context::AppContext,
    app_error::AppError,
    component_name::ComponentName,
    components::{
        chat_list_window::ChatListWindow,
        chat_window::ChatWindow,
        command_guide::CommandGuide,
        component_traits::{Component, HandleFocus},
        prompt_window::PromptWindow,
    },
    components::{MAX_CHAT_LIST_SIZE, MAX_PROMPT_SIZE, MIN_CHAT_LIST_SIZE, MIN_PROMPT_SIZE},
    configs::custom::keymap_custom::ActionBinding,
    event::Event,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::{collections::HashMap, io, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

use super::reply_message::ReplyMessage;

/// `CoreWindow` is a struct that represents the core window of the application.
/// It is responsible for managing the layout and rendering of the core window.
pub struct CoreWindow {
    /// The application context.
    app_context: Arc<AppContext>,
    /// The name of the `CoreWindow`.
    name: String,
    /// An unbounded sender that send action for processing.
    action_tx: Option<UnboundedSender<Action>>,
    /// A map of components that are part of the `CoreWindow`.
    components: HashMap<ComponentName, Box<dyn Component>>,
    /// A flag indicating whether the `CoreWindow` should be displayed as a
    /// smaller version of itself.
    small_area: bool,
    /// The size of the prompt component.
    size_prompt: u16,
    /// The size of the message reply component.
    size_message_reply: u16,
    /// The size of the chat list component.
    size_chat_list: u16,
    /// The name of the component that currently has focus. It is an optional
    /// value because no component may have focus. The focus is a component
    /// inside the `CoreWindow`.
    component_focused: Option<ComponentName>,
    /// Indicates whether the `CoreWindow` is focused or not.
    focused: bool,
    /// Indicates whether the reply message should be shown.
    show_reply_message: bool,
    /// Indicates whether the command guide should be shown.
    show_command_guide: bool,
}

impl CoreWindow {
    /// Create a new instance of the `CoreWindow` struct.
    ///
    /// # Arguments
    /// * `app_context` - An Arc wrapped AppContext struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `CoreWindow` struct.
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let components_iter: Vec<(ComponentName, Box<dyn Component>)> = vec![
            (
                ComponentName::ChatList,
                ChatListWindow::new(Arc::clone(&app_context))
                    .with_name(ComponentName::ChatList.to_string())
                    .new_boxed(),
            ),
            (
                ComponentName::Chat,
                ChatWindow::new(Arc::clone(&app_context))
                    .with_name(ComponentName::Chat.to_string())
                    .new_boxed(),
            ),
            (
                ComponentName::Prompt,
                PromptWindow::new(Arc::clone(&app_context))
                    .with_name(ComponentName::Prompt.to_string())
                    .new_boxed(),
            ),
            (
                ComponentName::ReplyMessage,
                ReplyMessage::new(Arc::clone(&app_context))
                    .with_name(ComponentName::ReplyMessage.to_string())
                    .new_boxed(),
            ),
            (
                ComponentName::CommandGuide,
                CommandGuide::new(Arc::clone(&app_context))
                    .with_name(ComponentName::CommandGuide.to_string())
                    .new_boxed(),
            ),
        ];

        let app_context = app_context;
        let name = "".to_string();
        let action_tx = None;
        let components: HashMap<ComponentName, Box<dyn Component>> =
            components_iter.into_iter().collect();
        let size_prompt = 3;
        let size_message_reply = 2;
        let size_chat_list = 20;
        let small_area = false;
        let component_focused = None;
        let focused = true;
        let show_reply_message = false;
        let show_command_guide = false;

        CoreWindow {
            app_context,
            name,
            action_tx,
            components,
            size_chat_list,
            size_prompt,
            size_message_reply,
            small_area,
            component_focused,
            focused,
            show_reply_message,
            show_command_guide,
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

    /// Set small area flag.
    ///
    /// # Arguments
    /// * `small_area` - A flag indicating whether the `CoreWindow` should be displayed as a
    pub fn with_small_area(&mut self, small_area: bool) {
        self.small_area = small_area;
    }

    /// Toggle the chat list component.
    pub fn toggle_chat_list(&mut self) {
        self.size_chat_list = if self.size_chat_list == 0 { 20 } else { 0 };
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
        if self.size_chat_list == MIN_CHAT_LIST_SIZE {
            return;
        }
        self.size_chat_list -= 1;
    }
    /// Decrease the size of the chat list component.
    pub fn decrease_size_prompt(&mut self) {
        if self.size_prompt == MIN_PROMPT_SIZE {
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

/// Implement the `Component` trait for the `ChatListWindow` struct.
impl Component for CoreWindow {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> std::io::Result<()> {
        self.action_tx = Some(tx.clone());
        for (_, component) in self.components.iter_mut() {
            component.register_action_handler(tx.clone())?;
        }
        Ok(())
    }

    fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>, AppError<Action>> {
        let binding = self.app_context.keymap_config();
        let map = binding.get_map_of(self.component_focused);
        if let Some(action_binding) = map.get(&event.unwrap()) {
            match action_binding {
                ActionBinding::Single { action, .. } => {
                    return Ok(Some(action.clone()));
                }
                ActionBinding::Multiple(_map_event_action) => {
                    tracing::warn!("Multiple action bindings are not supported yet. They are supported only for default key bindings because there are not the app context to handle them here.");
                    todo!();
                }
            }
        }
        Ok(Some(Action::Unknown))
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::FocusComponent(component_name) => {
                self.component_focused = Some(component_name);
                self.components
                    .get_mut(&component_name)
                    .unwrap_or_else(|| panic!("Failed to get component: {component_name}"))
                    .focus();
                self.components
                    .iter_mut()
                    .filter(|(name, _)| *name != &component_name)
                    .for_each(|(_, component)| component.unfocus());
            }
            Action::UnfocusComponent => {
                self.component_focused = None;
                self.show_reply_message = false;
                for (_, component) in self.components.iter_mut() {
                    component.unfocus();
                }
            }
            Action::ToggleChatList => {
                self.toggle_chat_list();
            }
            Action::IncreaseChatListSize => {
                self.increase_chat_list_size();
            }
            Action::DecreaseChatListSize => {
                self.decrease_chat_list_size();
            }
            Action::IncreasePromptSize => {
                self.increase_size_prompt();
            }
            Action::DecreasePromptSize => {
                self.decrease_size_prompt();
            }
            Action::TryQuit => {
                if self.component_focused != Some(ComponentName::Prompt) {
                    self.action_tx
                        .as_ref()
                        .unwrap_or_else(|| panic!("Failed to get action_tx on CoreWindow"))
                        .send(Action::Quit)
                        .unwrap_or_else(|_| panic!("Failed to send action Quit from CoreWindow"));
                }
            }
            Action::ShowChatWindowReply => {
                self.show_reply_message = true;
            }
            Action::HideChatWindowReply => {
                self.show_reply_message = false;
            }
            Action::ShowCommandGuide => {
                // Toggle command guide: if already visible, hide it; otherwise show it
                if self.show_command_guide {
                    self.show_command_guide = false;
                    if let Some(component) = self.components.get_mut(&ComponentName::CommandGuide) {
                        component.update(Action::HideCommandGuide);
                    }
                } else {
                    self.show_command_guide = true;
                    if let Some(component) = self.components.get_mut(&ComponentName::CommandGuide) {
                        component.update(action.clone());
                    }
                }
            }
            Action::HideCommandGuide => {
                self.show_command_guide = false;
                if let Some(component) = self.components.get_mut(&ComponentName::CommandGuide) {
                    component.update(action.clone());
                }
            }
            Action::Key(key_code, modifiers) => {
                // If command guide is visible, handle Esc/F1 to close it
                if self.show_command_guide {
                    match key_code {
                        crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::F(1) => {
                            self.show_command_guide = false;
                            if let Some(component) =
                                self.components.get_mut(&ComponentName::CommandGuide)
                            {
                                component.update(Action::HideCommandGuide);
                            }
                            return; // Don't process key event further
                        }
                        _ => {
                            // Send other keys to command guide when visible
                            if let Some(component) =
                                self.components.get_mut(&ComponentName::CommandGuide)
                            {
                                component.update(Action::Key(key_code, modifiers.clone()));
                            }
                            return; // Don't send to focused component when guide is visible
                        }
                    }
                }
                // Note: Alt+F1 to open is handled by the keymap system via ShowCommandGuide action
                // Send key events to focused component
                if let Some(focused) = self.component_focused {
                    self.components
                        .get_mut(&focused)
                        .unwrap_or_else(|| panic!("Failed to get component: {focused}"))
                        .update(Action::Key(key_code, modifiers));
                }
                return; // Don't pass action to focused component again below
            }
            Action::ChatListSearch => {
                // Always activate search if ChatList is focused, nothing is focused, or Chat is focused
                // ChatWindow will handle switching from its search mode to ChatList search
                let should_activate_search = match self.component_focused {
                    None => true,                          // No component focused - activate search
                    Some(ComponentName::ChatList) => true, // ChatList focused - activate search
                    Some(ComponentName::Chat) => true, // Chat focused - allow switching to ChatList search
                    Some(ComponentName::Prompt) => false, // Prompt focused - don't activate
                    _ => false,                        // Other components - don't activate
                };

                if should_activate_search {
                    // First, if ChatWindow is focused, let it handle the switch (it will stop search if in search mode)
                    if let Some(ComponentName::Chat) = self.component_focused {
                        if let Some(component) = self.components.get_mut(&ComponentName::Chat) {
                            component.update(action.clone());
                        }
                    }

                    // Focus ChatList and activate search mode
                    self.component_focused = Some(ComponentName::ChatList);
                    self.components
                        .get_mut(&ComponentName::ChatList)
                        .unwrap_or_else(|| {
                            panic!("Failed to get component: {}", ComponentName::ChatList)
                        })
                        .focus();
                    self.components
                        .iter_mut()
                        .filter(|(name, _)| *name != &ComponentName::ChatList)
                        .for_each(|(_, component)| component.unfocus());
                    // Activate search mode
                    if let Some(component) = self.components.get_mut(&ComponentName::ChatList) {
                        component.update(action.clone());
                    }
                }
            }
            Action::ChatWindowSearch => {
                // Focus Chat and activate search mode
                self.component_focused = Some(ComponentName::Chat);
                self.components
                    .get_mut(&ComponentName::Chat)
                    .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::Chat))
                    .focus();
                self.components
                    .iter_mut()
                    .filter(|(name, _)| *name != &ComponentName::Chat)
                    .for_each(|(_, component)| component.unfocus());
                // Activate search mode
                if let Some(component) = self.components.get_mut(&ComponentName::Chat) {
                    component.update(action.clone());
                }
            }
            Action::ChatListSortWithString(_) => {
                // Propagate to ChatList component even if not focused
                if let Some(component) = self.components.get_mut(&ComponentName::ChatList) {
                    component.update(action.clone());
                }
                // Also update focused component if it's not ChatList
                if let Some(focused) = self.component_focused {
                    if focused != ComponentName::ChatList {
                        self.components
                            .get_mut(&focused)
                            .unwrap_or_else(|| panic!("Failed to get component: {focused}"))
                            .update(action);
                    }
                }
            }
            Action::ChatWindowSortWithString(_) => {
                // Propagate to Chat component even if not focused
                if let Some(component) = self.components.get_mut(&ComponentName::Chat) {
                    component.update(action.clone());
                }
                // Also update focused component if it's not Chat
                if let Some(focused) = self.component_focused {
                    if focused != ComponentName::Chat {
                        self.components
                            .get_mut(&focused)
                            .unwrap_or_else(|| panic!("Failed to get component: {focused}"))
                            .update(action);
                    }
                }
            }
            Action::ChatListRestoreSort => {
                // Propagate to ChatList component even if not focused
                if let Some(component) = self.components.get_mut(&ComponentName::ChatList) {
                    component.update(action.clone());
                }
                // Also update focused component if it's not ChatList
                if let Some(focused) = self.component_focused {
                    if focused != ComponentName::ChatList {
                        self.components
                            .get_mut(&focused)
                            .unwrap_or_else(|| panic!("Failed to get component: {focused}"))
                            .update(action);
                    }
                }
            }
            Action::ChatWindowRestoreSort => {
                // Propagate to Chat component even if not focused
                if let Some(component) = self.components.get_mut(&ComponentName::Chat) {
                    component.update(action.clone());
                }
                // Also update focused component if it's not Chat
                if let Some(focused) = self.component_focused {
                    if focused != ComponentName::Chat {
                        self.components
                            .get_mut(&focused)
                            .unwrap_or_else(|| panic!("Failed to get component: {focused}"))
                            .update(action);
                    }
                }
            }
            Action::Key(_, _) => {
                // Send key events only to the focused component to prevent double processing
                // Components will handle search mode internally
                if let Some(focused) = self.component_focused {
                    self.components
                        .get_mut(&focused)
                        .unwrap_or_else(|| panic!("Failed to get component: {focused}"))
                        .update(action);
                }
            }
            _ => {
                if let Some(focused) = self.component_focused {
                    self.components
                        .get_mut(&focused)
                        .unwrap_or_else(|| panic!("Failed to get component: {focused}"))
                        .update(action);
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> io::Result<()> {
        let core_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(self.size_chat_list),
                Constraint::Percentage(100 - self.size_chat_list),
            ])
            .split(area);

        self.components
            .get_mut(&ComponentName::ChatList)
            .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::ChatList))
            .draw(frame, core_layout[0])?;

        let sub_core_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                {
                    if self.show_reply_message {
                        Constraint::Length(self.size_message_reply)
                    } else {
                        Constraint::Length(0)
                    }
                },
                Constraint::Length(self.size_prompt),
            ])
            .split(core_layout[1]);

        self.components
            .get_mut(&ComponentName::Chat)
            .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::Chat))
            .draw(frame, sub_core_layout[0])?;

        if self.show_reply_message {
            self.components
                .get_mut(&ComponentName::ReplyMessage)
                .unwrap_or_else(|| {
                    panic!("Failed to get component: {}", ComponentName::ReplyMessage)
                })
                .draw(frame, sub_core_layout[1])?;
        }
        self.components
            .get_mut(&ComponentName::Prompt)
            .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::Prompt))
            .draw(frame, sub_core_layout[2])?;

        // Draw command guide popup if visible (draws on top of everything)
        if self.show_command_guide {
            self.components
                .get_mut(&ComponentName::CommandGuide)
                .unwrap_or_else(|| {
                    panic!("Failed to get component: {}", ComponentName::CommandGuide)
                })
                .draw(frame, area)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{action::Action, components::search_tests::create_test_app_context};

    fn create_test_core_window() -> CoreWindow {
        let app_context = create_test_app_context();
        CoreWindow::new(app_context)
    }

    #[test]
    fn test_alt_r_activates_chat_list_search_when_nothing_focused() {
        let mut window = create_test_core_window();
        window.component_focused = None;

        // Simulate Alt+R keypress (this would come from keymap, but we test the action directly)
        window.update(Action::ChatListSearch);

        // Should focus ChatList and activate search
        assert_eq!(
            window.component_focused,
            Some(ComponentName::ChatList),
            "ChatList should be focused after Alt+R when nothing is focused"
        );
    }

    #[test]
    fn test_alt_r_activates_chat_list_search_when_chat_list_focused() {
        let mut window = create_test_core_window();
        window.component_focused = Some(ComponentName::ChatList);
        if let Some(component) = window.components.get_mut(&ComponentName::ChatList) {
            component.focus();
        }

        window.update(Action::ChatListSearch);

        assert_eq!(
            window.component_focused,
            Some(ComponentName::ChatList),
            "ChatList should remain focused"
        );
    }

    #[test]
    fn test_alt_r_activates_message_search_when_chat_focused() {
        let mut window = create_test_core_window();
        window.component_focused = Some(ComponentName::Chat);
        if let Some(component) = window.components.get_mut(&ComponentName::Chat) {
            component.focus();
        }

        window.update(Action::ChatWindowSearch);

        assert_eq!(
            window.component_focused,
            Some(ComponentName::Chat),
            "Chat should be focused after ChatWindowSearch"
        );
    }

    #[test]
    fn test_alt_r_switches_from_message_search_to_chat_list_search() {
        let mut window = create_test_core_window();
        window.component_focused = Some(ComponentName::Chat);
        if let Some(component) = window.components.get_mut(&ComponentName::Chat) {
            component.focus();
            // Put ChatWindow in search mode
            component.update(Action::ChatWindowSearch);
        }

        // Now Alt+R should switch to ChatList search
        window.update(Action::ChatListSearch);

        assert_eq!(
            window.component_focused,
            Some(ComponentName::ChatList),
            "Should switch to ChatList search when Alt+R pressed during message search"
        );
    }

    #[test]
    fn test_alt_c_restores_chat_window_sort() {
        let mut window = create_test_core_window();
        window.component_focused = Some(ComponentName::Chat);

        // Set a sort string first
        window.update(Action::ChatWindowSortWithString("test".to_string()));

        // Then restore
        window.update(Action::ChatWindowRestoreSort);

        // The action should be propagated to Chat component
        // We can't directly check the internal state, but we verify the action is handled
        assert_eq!(
            window.component_focused,
            Some(ComponentName::Chat),
            "Chat should remain focused"
        );
    }

    #[test]
    fn test_alt_c_restores_chat_list_sort() {
        let mut window = create_test_core_window();
        window.component_focused = Some(ComponentName::ChatList);

        // Set a sort string first
        window.update(Action::ChatListSortWithString("test".to_string()));

        // Then restore
        window.update(Action::ChatListRestoreSort);

        assert_eq!(
            window.component_focused,
            Some(ComponentName::ChatList),
            "ChatList should remain focused"
        );
    }

    #[test]
    fn test_chat_list_search_propagates_to_component() {
        let mut window = create_test_core_window();
        window.component_focused = Some(ComponentName::ChatList);

        window.update(Action::ChatListSearch);

        // ChatList should receive the search action
        assert_eq!(
            window.component_focused,
            Some(ComponentName::ChatList),
            "ChatList should be focused"
        );
    }

    #[test]
    fn test_chat_window_search_propagates_to_component() {
        let mut window = create_test_core_window();
        window.component_focused = Some(ComponentName::Chat);

        window.update(Action::ChatWindowSearch);

        // Chat should receive the search action
        assert_eq!(
            window.component_focused,
            Some(ComponentName::Chat),
            "Chat should be focused"
        );
    }
}
