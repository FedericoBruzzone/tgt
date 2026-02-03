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
        search_overlay::SearchOverlay,
        theme_selector::ThemeSelector,
    },
    components::{MAX_CHAT_LIST_SIZE, MAX_PROMPT_SIZE, MIN_CHAT_LIST_SIZE, MIN_PROMPT_SIZE},
    configs::custom::keymap_custom::ActionBinding,
    event::Event,
    theme_switcher::{discover_available_themes, ThemeSwitcher},
};
use crossterm::event::{MouseButton, MouseEventKind};
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
    /// Indicates whether the theme selector should be shown.
    show_theme_selector: bool,
    /// Indicates whether the search overlay (server-side chat search) should be shown.
    show_search_overlay: bool,
    /// Last known screen areas for focusable sections (chat list, chat, prompt) for click-to-focus.
    last_focusable_areas: HashMap<ComponentName, Rect>,
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
            (
                ComponentName::ThemeSelector,
                ThemeSelector::new(Arc::clone(&app_context))
                    .with_name(ComponentName::ThemeSelector.to_string())
                    .new_boxed(),
            ),
            (
                ComponentName::SearchOverlay,
                SearchOverlay::new(Arc::clone(&app_context))
                    .with_name(ComponentName::SearchOverlay.to_string())
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
        let show_theme_selector = false;
        let show_search_overlay = false;
        let last_focusable_areas = HashMap::new();

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
            show_theme_selector,
            show_search_overlay,
            last_focusable_areas,
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
        let Some(ev) = event else {
            return Ok(None);
        };
        // Forward mouse events to the focused component (scroll wheel, click, etc.)
        if let Event::Mouse(mouse) = ev {
            // Left click: focus the section under the cursor (chat list, chat window, prompt)
            if let MouseEventKind::Down(MouseButton::Left) = mouse.kind {
                let col = mouse.column;
                let row = mouse.row;
                for name in [
                    ComponentName::ChatList,
                    ComponentName::Chat,
                    ComponentName::Prompt,
                ] {
                    if let Some(rect) = self.last_focusable_areas.get(&name) {
                        let in_rect = col >= rect.x
                            && col < rect.x + rect.width
                            && row >= rect.y
                            && row < rect.y + rect.height;
                        if in_rect && self.component_focused != Some(name) {
                            return Ok(Some(Action::FocusComponent(name)));
                        }
                    }
                }
            }
            if let Some(name) = self.component_focused {
                if let Some(component) = self.components.get_mut(&name) {
                    return component.handle_mouse_events(mouse).map_err(AppError::from);
                }
            }
            return Ok(None);
        }
        let binding = self.app_context.keymap_config();
        let map = binding.get_map_of(self.component_focused);
        if let Some(action_binding) = map.get(&ev) {
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
                self.app_context
                    .set_focused_component(self.component_focused);
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
                self.app_context.set_focused_component(None);
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
                // Reply uses the prompt only (same as edit). Do not set show_reply_message.
                // Dispatch to Chat so reply_selected() runs and sends FocusComponent(Prompt) + ReplyMessage.
                if let Some(component) = self.components.get_mut(&ComponentName::Chat) {
                    component.update(action.clone());
                }
            }
            Action::HideChatWindowReply => {
                self.show_reply_message = false;
                self.app_context
                    .tg_context()
                    .set_reply_message_i64(-1, String::new());
            }
            Action::ShowCommandGuide => {
                // Toggle command guide: if already visible, hide it; otherwise show it
                if self.show_command_guide {
                    self.show_command_guide = false;
                    if let Some(component) = self.components.get_mut(&ComponentName::CommandGuide) {
                        component.update(Action::HideCommandGuide);
                    }
                    // Unfocus when hiding
                    self.component_focused = None;
                    self.app_context.set_focused_component(None);
                } else {
                    self.show_command_guide = true;
                    // Focus the command guide so its keymap is active
                    self.component_focused = Some(ComponentName::CommandGuide);
                    self.app_context.set_focused_component(self.component_focused);
                    if let Some(component) = self.components.get_mut(&ComponentName::CommandGuide) {
                        component.focus();
                        component.update(action.clone());
                    }
                    // Unfocus other components
                    self.components
                        .iter_mut()
                        .filter(|(name, _)| *name != &ComponentName::CommandGuide)
                        .for_each(|(_, component)| component.unfocus());
                }
            }
            Action::HideCommandGuide => {
                self.show_command_guide = false;
                if let Some(component) = self.components.get_mut(&ComponentName::CommandGuide) {
                    component.update(action.clone());
                }
                // Unfocus when hiding
                if self.component_focused == Some(ComponentName::CommandGuide) {
                    self.component_focused = None;
                    self.app_context.set_focused_component(None);
                }
            }
            Action::ShowThemeSelector => {
                // Toggle theme selector: if already visible, hide it; otherwise show it
                if self.show_theme_selector {
                    self.show_theme_selector = false;
                    if let Some(component) = self.components.get_mut(&ComponentName::ThemeSelector)
                    {
                        component.update(Action::HideThemeSelector);
                    }
                    // Unfocus when hiding
                    self.component_focused = None;
                    self.app_context.set_focused_component(None);
                } else {
                    self.show_theme_selector = true;
                    // Focus the theme selector so its keymap is active
                    self.component_focused = Some(ComponentName::ThemeSelector);
                    self.app_context.set_focused_component(self.component_focused);
                    if let Some(component) = self.components.get_mut(&ComponentName::ThemeSelector)
                    {
                        component.focus();
                        component.update(action.clone());
                    }
                    // Unfocus other components
                    self.components
                        .iter_mut()
                        .filter(|(name, _)| *name != &ComponentName::ThemeSelector)
                        .for_each(|(_, component)| component.unfocus());
                }
            }
            Action::HideThemeSelector => {
                self.show_theme_selector = false;
                if let Some(component) = self.components.get_mut(&ComponentName::ThemeSelector) {
                    component.update(action.clone());
                }
                // Unfocus when hiding
                if self.component_focused == Some(ComponentName::ThemeSelector) {
                    self.component_focused = None;
                    self.app_context.set_focused_component(None);
                }
            }
            Action::SwitchTheme => {
                // Discover available themes and find current theme index
                let themes = discover_available_themes();
                if themes.is_empty() {
                    tracing::warn!("No themes available");
                    return;
                }

                // Get current theme from app config
                let current_theme_filename = self.app_context.app_config().theme_filename.clone();
                let current_theme_name = current_theme_filename
                    .strip_suffix(".toml")
                    .and_then(|s| s.strip_prefix("themes/"))
                    .unwrap_or_else(|| {
                        current_theme_filename
                            .strip_suffix(".toml")
                            .unwrap_or(&current_theme_filename)
                    });

                // Find current index and switch to next
                let current_index = themes
                    .iter()
                    .position(|t| t == current_theme_name)
                    .unwrap_or(0);
                let next_index = (current_index + 1) % themes.len();
                let next_theme = &themes[next_index];

                // Apply the next theme
                if ThemeSwitcher::apply_theme(&self.app_context, next_theme).is_ok() {
                    self.app_context.mark_dirty();
                } else {
                    tracing::error!("Failed to switch theme");
                }
            }
            Action::SwitchThemeTo(theme_name) => {
                // Apply the specified theme directly using static method
                if ThemeSwitcher::apply_theme(&self.app_context, &theme_name).is_ok() {
                    self.app_context.mark_dirty();
                } else {
                    tracing::error!("Failed to switch theme to {}", theme_name);
                }
            }
            Action::Key(key_code, modifiers) => {
                // Note: Popup components (CommandGuide, ThemeSelector, SearchOverlay) are now focusable and use keymaps.
                // Key events are routed to the focused component via the keymap system in run.rs.
                // Send key events to focused component
                if let Some(focused) = self.component_focused {
                    self.components
                        .get_mut(&focused)
                        .unwrap_or_else(|| panic!("Failed to get component: {focused}"))
                        .update(Action::Key(key_code, modifiers));
                }
                // Don't pass action to focused component again below
            }
            Action::ChatListSearch => {
                // If search overlay is open, close it first then focus ChatList
                if self.show_search_overlay {
                    self.show_search_overlay = false;
                    if let Some(component) = self.components.get_mut(&ComponentName::SearchOverlay)
                    {
                        component.update(Action::CloseSearchOverlay);
                        component.unfocus();
                    }
                }
                // Activate ChatList search when ChatList is focused, nothing focused, or Chat focused
                let should_activate_search = match self.component_focused {
                    None => true,
                    Some(ComponentName::ChatList) => true,
                    Some(ComponentName::Chat) => true,
                    Some(ComponentName::SearchOverlay) => true, // switching from overlay to ChatList search
                    Some(ComponentName::Prompt) => false,
                    _ => false,
                };

                if should_activate_search {
                    // Focus ChatList and activate search mode
                    self.component_focused = Some(ComponentName::ChatList);
                    self.app_context
                        .set_focused_component(self.component_focused);
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
                // Show server-side search overlay and focus it
                self.show_search_overlay = true;
                if let Some(component) = self.components.get_mut(&ComponentName::SearchOverlay) {
                    component.update(Action::ShowSearchOverlay);
                    component.focus();
                }
                self.component_focused = Some(ComponentName::SearchOverlay);
                self.app_context
                    .set_focused_component(self.component_focused);
                self.components
                    .iter_mut()
                    .filter(|(name, _)| *name != &ComponentName::SearchOverlay)
                    .for_each(|(_, component)| component.unfocus());
            }
            Action::CloseSearchOverlay => {
                self.show_search_overlay = false;
                if let Some(component) = self.components.get_mut(&ComponentName::SearchOverlay) {
                    component.update(Action::CloseSearchOverlay);
                    component.unfocus();
                }
                // Unfocus when hiding
                if self.component_focused == Some(ComponentName::SearchOverlay) {
                    self.component_focused = None;
                    self.app_context.set_focused_component(None);
                }
                // Refocus Chat after closing search
                self.component_focused = Some(ComponentName::Chat);
                self.app_context
                    .set_focused_component(self.component_focused);
                if let Some(component) = self.components.get_mut(&ComponentName::Chat) {
                    component.focus();
                }
                self.components
                    .iter_mut()
                    .filter(|(name, _)| *name != &ComponentName::Chat)
                    .for_each(|(_, component)| component.unfocus());
            }
            Action::SearchResults(_) => {
                // Propagate to SearchOverlay when visible
                if self.show_search_overlay {
                    if let Some(component) = self.components.get_mut(&ComponentName::SearchOverlay)
                    {
                        component.update(action.clone());
                    }
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
            Action::LoadChats(..) | Action::ChatHistoryAppended | Action::Resize(..) => {
                // Always forward to ChatList so it can rebuild_visible_chats (populates visible_chats).
                if let Some(component) = self.components.get_mut(&ComponentName::ChatList) {
                    component.update(action.clone());
                }
                if let Some(focused) = self.component_focused {
                    if focused != ComponentName::ChatList {
                        if let Some(component) = self.components.get_mut(&focused) {
                            component.update(action);
                        }
                    }
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

        // Store areas for click-to-focus (chat list, chat window, prompt)
        self.last_focusable_areas
            .insert(ComponentName::ChatList, core_layout[0]);
        self.last_focusable_areas
            .insert(ComponentName::Chat, sub_core_layout[0]);
        self.last_focusable_areas
            .insert(ComponentName::Prompt, sub_core_layout[2]);

        self.components
            .get_mut(&ComponentName::ChatList)
            .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::ChatList))
            .draw(frame, core_layout[0])?;

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

        if self.show_theme_selector {
            self.components
                .get_mut(&ComponentName::ThemeSelector)
                .unwrap_or_else(|| {
                    panic!("Failed to get component: {}", ComponentName::ThemeSelector)
                })
                .draw(frame, area)?;
        }

        if self.show_search_overlay {
            self.components
                .get_mut(&ComponentName::SearchOverlay)
                .unwrap_or_else(|| {
                    panic!("Failed to get component: {}", ComponentName::SearchOverlay)
                })
                .draw(frame, area)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{action::Action, components::search_tests::create_test_app_context, event::Event};

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

        // ChatWindowSearch opens the server search overlay
        window.update(Action::ChatWindowSearch);

        assert_eq!(
            window.component_focused,
            Some(ComponentName::SearchOverlay),
            "Search overlay should be focused after ChatWindowSearch"
        );
        assert!(
            window.show_search_overlay,
            "Search overlay should be visible"
        );
    }

    #[test]
    fn test_alt_r_switches_from_message_search_to_chat_list_search() {
        let mut window = create_test_core_window();
        window.component_focused = Some(ComponentName::Chat);
        if let Some(component) = window.components.get_mut(&ComponentName::Chat) {
            component.focus();
        }
        // Open search overlay (server search)
        window.update(Action::ChatWindowSearch);

        // Now Alt+R (ChatListSearch) should close overlay and focus ChatList
        window.update(Action::ChatListSearch);

        assert_eq!(
            window.component_focused,
            Some(ComponentName::ChatList),
            "Should switch to ChatList search when Alt+R pressed during message search"
        );
        assert!(
            !window.show_search_overlay,
            "Search overlay should be closed"
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
    fn test_chat_window_search_opens_overlay() {
        let mut window = create_test_core_window();
        window.component_focused = Some(ComponentName::Chat);

        window.update(Action::ChatWindowSearch);

        // Search overlay should open and be focused
        assert_eq!(
            window.component_focused,
            Some(ComponentName::SearchOverlay),
            "Search overlay should be focused"
        );
        assert!(window.show_search_overlay);
    }

    #[test]
    fn test_mouse_click_in_chat_list_when_not_focused_focuses_chat_list() {
        use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
        use ratatui::layout::Rect;

        let mut window = create_test_core_window();
        window.component_focused = Some(ComponentName::Chat);

        // Set ChatList area so click is inside it
        let chat_list_rect = Rect::new(0, 0, 20, 10);
        window
            .last_focusable_areas
            .insert(ComponentName::ChatList, chat_list_rect);

        let mouse = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 5,
            row: 5,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };
        let result = window.handle_events(Some(Event::Mouse(mouse)));

        assert_eq!(
            result.unwrap(),
            Some(Action::FocusComponent(ComponentName::ChatList)),
            "First click in chat list when Chat focused should focus ChatList (not open a chat)"
        );
    }
}
