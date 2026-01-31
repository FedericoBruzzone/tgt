use crate::action::Action;
use crate::app_context::AppContext;
use crate::component_name::ComponentName::Prompt;
use crate::components::component_traits::{Component, HandleFocus};
use crate::event::Event;
use crate::tg::message_entry::MessageEntry;
use crossterm::event::{KeyCode, MouseEventKind};
use nucleo_matcher::{Matcher, Utf32Str};
use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};
use ratatui::symbols::border::PLAIN;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListDirection, ListState, Paragraph};
use ratatui::Frame;
use std::sync::Arc;
use tdlib_rs::enums::{ChatList, UserStatus};
use tdlib_rs::types::User;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct ChatListEntry {
    chat_id: i64,
    chat_name: String,
    last_message: Option<MessageEntry>,
    user: Option<User>,
    is_marked_as_unread: bool,
    unread_count: i32,
    /// Identifier of the last read incoming message
    last_read_inbox_message_id: Option<i64>,
    /// Identifier of the last read outgoing message
    last_read_outbox_message_id: Option<i64>,
}
impl Default for ChatListEntry {
    fn default() -> Self {
        Self::new()
    }
}
impl ChatListEntry {
    pub fn new() -> Self {
        Self {
            chat_id: 0,
            chat_name: String::new(),
            last_message: None,
            user: None,
            is_marked_as_unread: false,
            unread_count: 0,
            last_read_inbox_message_id: None,
            last_read_outbox_message_id: None,
        }
    }

    pub fn set_chat_id(&mut self, chat_id: i64) {
        self.chat_id = chat_id;
    }
    pub fn set_chat_name(&mut self, chat_name: String) {
        self.chat_name = chat_name;
    }
    pub fn set_last_message(&mut self, last_message: MessageEntry) {
        self.last_message = Some(last_message);
    }
    pub fn set_user(&mut self, user: User) {
        self.user = Some(user);
    }
    pub fn set_is_marked_as_unread(&mut self, is_marked_as_unread: bool) {
        self.is_marked_as_unread = is_marked_as_unread;
    }
    pub fn set_unread_count(&mut self, unread_count: i32) {
        self.unread_count = unread_count;
    }
    pub fn set_last_read_inbox_message_id(&mut self, last_read_inbox_message_id: i64) {
        self.last_read_inbox_message_id = Some(last_read_inbox_message_id);
    }
    pub fn set_last_read_outbox_message_id(&mut self, last_read_outbox_message_id: i64) {
        self.last_read_outbox_message_id = Some(last_read_outbox_message_id);
    }

    fn get_text_styled(&self, app_context: &AppContext) -> Text<'_> {
        let mut online_symbol = "";
        let mut verificated_symbol = "";
        if let Some(user) = &self.user {
            online_symbol = match user.status {
                UserStatus::Online(_) => "🟢 ",
                UserStatus::Offline(_) => "",
                UserStatus::Empty => "",
                UserStatus::Recently(_) => "",
                UserStatus::LastWeek(_) => "",
                UserStatus::LastMonth(_) => "",
            };
            verificated_symbol = if user.is_verified { "✅" } else { "" };
        }
        let unread_info = if self.is_marked_as_unread {
            format!("({})", self.unread_count)
        } else {
            "".to_string()
        };

        let preview_lines = -1;
        let mut entry = Text::default();
        entry.extend(vec![Line::from(vec![
            Span::raw(online_symbol),
            Span::styled(
                self.chat_name.clone(),
                app_context.style_chat_list_item_chat_name(),
            ),
            Span::raw(" "),
            Span::styled(
                unread_info,
                app_context.style_chat_list_item_unread_counter(),
            ),
            Span::raw(" "),
            Span::raw(verificated_symbol),
            Span::raw(" | "),
            self.last_message.as_ref().map_or_else(Span::default, |e| {
                e.timestamp().get_span_styled(app_context)
            }),
        ])]);
        entry.extend(self.last_message.as_ref().map_or_else(Line::default, |e| {
            e.get_lines_styled_with_style(
                app_context.style_chat_list_item_message_content(),
                preview_lines,
            )[0]
            .clone()
        }));

        entry
    }
}
/// `ChatListWindow` is a struct that represents a window for displaying a list
/// of chat items. It is responsible for managing the layout and rendering of
/// the chat list.
pub struct ChatListWindow {
    /// The application context.
    app_context: Arc<AppContext>,
    /// The name of the `ChatListWindow`.
    name: String,
    /// An unbounded sender that send action for processing.
    command_tx: Option<UnboundedSender<Action>>,
    /// A list of chat items to be displayed in the `ChatListWindow`.
    chat_list: Vec<ChatListEntry>,
    /// The state of the list.
    chat_list_state: ListState,
    /// Indicates whether the `ChatListWindow` is focused or not.
    focused: bool,
    /// String used to sort chats.
    sort_string: Option<String>,
    /// Search input string for filtering chats.
    search_input: String,
    /// Whether search mode is active.
    search_mode: bool,
}
/// Implementation of the `ChatListWindow` struct.
impl ChatListWindow {
    /// Create a new instance of the `ChatListWindow` struct.
    ///
    /// # Arguments
    /// * `app_context` - An Arc wrapped AppContext struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `ChatListWindow` struct.
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let name = "".to_string();
        let command_tx = None;
        let chat_list = vec![];
        let chat_list_state = ListState::default();
        let focused = false;
        let sort_string = None;

        ChatListWindow {
            app_context,
            name,
            command_tx,
            chat_list,
            chat_list_state,
            focused,
            sort_string,
            search_input: String::new(),
            search_mode: false,
        }
    }
    /// Set the name of the `ChatListWindow`.
    ///
    /// # Arguments
    /// * `name` - The name of the `ChatListWindow`.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `ChatListWindow`.
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }
    /// Select the next chat item in the list.
    fn next(&mut self) {
        let i = match self.chat_list_state.selected() {
            Some(i) => {
                if i == self.chat_list.len() / 2 {
                    if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                        event_tx
                            .send(Event::LoadChats(ChatList::Main.into(), 20))
                            .unwrap();
                    }
                }

                if i >= self.chat_list.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.chat_list_state.select(Some(i));
    }
    /// Select the previous chat item in the list.
    fn previous(&mut self) {
        let i = match self.chat_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.chat_list_state.select(Some(i));
    }
    /// Unselect the chat item in the list.
    fn unselect(&mut self) {
        self.chat_list_state.select(None);
    }
    /// Confirm the selection of the chat item in the list.
    fn confirm_selection(&mut self) {
        if let Some(i) = self.chat_list_state.selected() {
            if let Some(chat) = self.chat_list.get(i) {
                self.app_context
                    .tg_context()
                    .set_open_chat_user(chat.user.clone());
                self.app_context.tg_context().set_open_chat_id(chat.chat_id);
                self.app_context.tg_context().clear_open_chat_messages();
                self.app_context.tg_context().set_jump_target_message_id(0);
                self.app_context
                    .action_tx()
                    .send(Action::FocusComponent(Prompt))
                    .unwrap();

                self.app_context.tg_context().set_from_message_id(0);
                // Start loading chat history immediately (same handle_app_actions run).
                // Event::GetChatHistory is also sent for tg_backend; Action ensures we spawn the task now.
                let _ = self.app_context.action_tx().send(Action::GetChatHistory);

                if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                    // Mark all unread messages as read
                    let _ = event_tx.send(Event::ViewAllMessages);
                }
            }
        }
    }

    /// Sets the string used to order the entries of the chat list.
    fn sort(&mut self, s: String) {
        self.sort_string = Some(s);
    }

    /// Unsets the string used to order the entries of the chat list.
    fn default_sort(&mut self) {
        self.sort_string = None;
    }

    /// Enter search mode.
    fn start_search(&mut self) {
        self.search_mode = true;
        self.search_input.clear();
    }

    /// Exit search mode.
    fn stop_search(&mut self) {
        self.search_mode = false;
        self.search_input.clear();
        self.default_sort();
    }

    /// Handle character input in search mode.
    fn handle_search_char(&mut self, c: char) {
        self.search_input.push(c);
        self.sort(self.search_input.clone());
    }

    /// Handle backspace in search mode.
    fn handle_search_backspace(&mut self) {
        self.search_input.pop();
        if self.search_input.is_empty() {
            self.default_sort();
        } else {
            self.sort(self.search_input.clone());
        }
    }
}

/// Implement the `HandleFocus` trait for the `ChatListWindow` struct.
/// This trait allows the `ChatListWindow` to be focused or unfocused.
impl HandleFocus for ChatListWindow {
    /// Set the `focused` flag for the `ChatListWindow`.
    fn focus(&mut self) {
        self.focused = true;
    }
    /// Set the `focused` flag for the `ChatListWindow`.
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

/// Implement the `Component` trait for the `ChatListWindow` struct.
impl Component for ChatListWindow {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> std::io::Result<()> {
        self.command_tx = Some(tx.clone());
        Ok(())
    }

    fn handle_mouse_events(&mut self, mouse: crossterm::event::MouseEvent) -> std::io::Result<Option<Action>> {
        if !self.focused {
            return Ok(None);
        }
        match mouse.kind {
            MouseEventKind::ScrollDown => Ok(Some(Action::ChatListNext)),
            MouseEventKind::ScrollUp => Ok(Some(Action::ChatListPrevious)),
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::ChatListNext => {
                // Allow navigation in and out of search mode (arrow keys move list selection)
                self.next();
            }
            Action::ChatListPrevious => {
                // Allow navigation in and out of search mode (arrow keys move list selection)
                self.previous();
            }
            Action::ChatListUnselect => {
                if self.search_mode {
                    self.stop_search();
                } else {
                    self.unselect();
                }
            }
            Action::ChatListOpen => {
                // Open selected chat; in search mode also exit search after opening
                self.confirm_selection();
                if self.search_mode {
                    self.stop_search();
                }
            }
            Action::ChatListSortWithString(s) => self.sort(s),
            Action::ChatListRestoreSort => self.default_sort(),
            Action::ChatListSearch => self.start_search(),
            Action::Key(key_code, modifiers) => {
                if self.search_mode {
                    match key_code {
                        KeyCode::Char(c)
                            if !modifiers.control && !modifiers.alt && !modifiers.shift =>
                        {
                            self.handle_search_char(c);
                        }
                        KeyCode::Backspace => {
                            self.handle_search_backspace();
                        }
                        KeyCode::Enter => {
                            // Open selected chat and exit search mode
                            self.confirm_selection();
                            self.stop_search();
                        }
                        KeyCode::Esc => {
                            self.stop_search();
                        }
                        KeyCode::Down => {
                            // Allow arrow key navigation in search mode
                            self.next();
                        }
                        KeyCode::Up => {
                            // Allow arrow key navigation in search mode
                            self.previous();
                        }
                        KeyCode::Tab => {
                            // Allow tab navigation in search mode
                            self.next();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> std::io::Result<()> {
        let style_border_focused = if self.focused {
            self.app_context.style_border_component_focused()
        } else {
            self.app_context.style_chat_list()
        };

        // Split area to include search bar if in search mode
        let (list_area, search_area) = if self.search_mode {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Fill(1)])
                .split(area);
            (layout[1], Some(layout[0]))
        } else {
            (area, None)
        };

        if let Ok(Some(items)) = self.app_context.tg_context().get_chats_index() {
            self.chat_list = items;

            // Filter and sort before drawing
            if let Some(s) = self.sort_string.as_ref() {
                if !s.is_empty() {
                    let mut config = nucleo_matcher::Config::DEFAULT;
                    config.prefer_prefix = true;
                    let mut matcher = Matcher::new(config.clone());
                    let search_chars: Vec<char> = s.chars().collect();
                    // Filter chats that match the search query
                    self.chat_list.retain(|chat| {
                        let chat_name_chars: Vec<char> = chat.chat_name.chars().collect();
                        matcher
                            .fuzzy_indices(
                                Utf32Str::Unicode(&chat_name_chars),
                                Utf32Str::Unicode(&search_chars),
                                &mut Vec::new(),
                            )
                            .is_some()
                    });
                    // Sort remaining chats by relevance score
                    let mut matcher = Matcher::new(config);
                    self.chat_list.sort_by(|a, b| {
                        let a: Vec<char> = a.chat_name.chars().collect();
                        let b: Vec<char> = b.chat_name.chars().collect();
                        let a_score = matcher
                            .fuzzy_indices(
                                Utf32Str::Unicode(&a),
                                Utf32Str::Unicode(&search_chars),
                                &mut Vec::new(),
                            )
                            .unwrap_or(0);
                        let b_score = matcher
                            .fuzzy_indices(
                                Utf32Str::Unicode(&b),
                                Utf32Str::Unicode(&search_chars),
                                &mut Vec::new(),
                            )
                            .unwrap_or(0);
                        a_score.cmp(&b_score)
                    });
                    self.chat_list.reverse();
                }
            }
        }

        // Draw search bar if in search mode
        if let Some(search_rect) = search_area {
            let search_block = Block::default()
                .border_set(PLAIN)
                .border_style(style_border_focused)
                .borders(Borders::ALL)
                .title("Search chats");
            let search_text = format!("{}_", self.search_input);
            let search_paragraph = Paragraph::new(search_text)
                .block(search_block)
                .style(self.app_context.style_chat_list());
            frame.render_widget(search_paragraph, search_rect);

            // Set cursor position in search bar
            if self.focused && self.search_mode {
                frame.set_cursor_position(Position {
                    x: search_rect.x + self.search_input.len() as u16 + 1,
                    y: search_rect.y + 1,
                });
            }
        }

        let items = self
            .chat_list
            .iter()
            .map(|item| item.get_text_styled(&self.app_context));
        let block = Block::default()
            .border_set(PLAIN)
            .border_style(style_border_focused)
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .title(Line::from(self.name.as_str()));

        let list = List::new(items)
            .block(block)
            .style(self.app_context.style_chat_list())
            .highlight_style(self.app_context.style_chat_list_item_selected())
            .direction(ListDirection::TopToBottom);
        // .highlight_symbol("➤ ")
        // .repeat_highlight_symbol(true)

        frame.render_stateful_widget(list, list_area, &mut self.chat_list_state);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        action::{Action, Modifiers},
        components::search_tests::{create_mock_chat, create_test_app_context},
    };
    use crossterm::event::{KeyCode, KeyModifiers};

    fn create_test_chat_list_window() -> ChatListWindow {
        let app_context = create_test_app_context();
        ChatListWindow::new(app_context)
    }

    fn setup_chats(window: &mut ChatListWindow, chat_names: &[&str]) {
        let mut chats = Vec::new();
        for (i, name) in chat_names.iter().enumerate() {
            chats.push(create_mock_chat(i as i64 + 1, name));
        }
        // Directly set the chat_list (using internal access for testing)
        window.chat_list = chats;
    }

    #[test]
    fn test_chat_list_search_initial_state() {
        let window = create_test_chat_list_window();
        assert!(!window.search_mode, "Search mode should be false initially");
    }

    #[test]
    fn test_chat_list_start_search() {
        let mut window = create_test_chat_list_window();
        window.update(Action::ChatListSearch);
        assert!(
            window.search_mode,
            "Search mode should be true after ChatListSearch"
        );
    }

    #[test]
    fn test_chat_list_stop_search() {
        let mut window = create_test_chat_list_window();
        window.update(Action::ChatListSearch);
        assert!(window.search_mode);

        window.update(Action::ChatListUnselect);
        assert!(
            !window.search_mode,
            "Search mode should be false after Esc/Unselect"
        );
    }

    #[test]
    fn test_chat_list_filtering_basic() {
        let mut window = create_test_chat_list_window();
        setup_chats(&mut window, &["Alice", "Bob", "Charlie", "David"]);

        // Start search
        window.update(Action::ChatListSearch);
        assert!(window.search_mode);

        // Type 'a' to filter
        let modifiers = Modifiers::from(KeyModifiers::empty());
        window.update(Action::Key(KeyCode::Char('a'), modifiers));
        window.update(Action::ChatListSortWithString("a".to_string()));

        // Simulate draw to trigger filtering
        window.chat_list = vec![
            create_mock_chat(1, "Alice"),
            create_mock_chat(3, "Charlie"),
            create_mock_chat(4, "David"),
        ];
        window.sort("a".to_string());

        // Filter should match "Alice" and "Charlie" (contains 'a')
        // Note: Actual filtering happens in draw(), but we can test the sort_string
        assert_eq!(window.sort_string, Some("a".to_string()));
    }

    #[test]
    fn test_chat_list_filtering_fuzzy() {
        let mut window = create_test_chat_list_window();
        setup_chats(&mut window, &["Alice", "Bob", "Charlie", "David"]);

        window.update(Action::ChatListSearch);
        let modifiers = Modifiers::from(KeyModifiers::empty());

        // Type 'al' - should match "Alice"
        window.update(Action::Key(KeyCode::Char('a'), modifiers.clone()));
        window.update(Action::Key(KeyCode::Char('l'), modifiers.clone()));
        window.update(Action::ChatListSortWithString("al".to_string()));

        assert_eq!(window.sort_string, Some("al".to_string()));
        assert_eq!(window.search_input, "al");
    }

    #[test]
    fn test_chat_list_search_backspace() {
        let mut window = create_test_chat_list_window();
        window.update(Action::ChatListSearch);

        let modifiers = Modifiers::from(KeyModifiers::empty());
        window.update(Action::Key(KeyCode::Char('a'), modifiers.clone()));
        window.update(Action::Key(KeyCode::Char('b'), modifiers.clone()));
        assert_eq!(window.search_input, "ab");

        window.update(Action::Key(KeyCode::Backspace, modifiers));
        assert_eq!(window.search_input, "a");
    }

    #[test]
    fn test_chat_list_search_exit_with_enter() {
        let mut window = create_test_chat_list_window();
        window.update(Action::ChatListSearch);
        assert!(window.search_mode);

        let modifiers = Modifiers::from(KeyModifiers::empty());
        window.update(Action::Key(KeyCode::Enter, modifiers));
        assert!(!window.search_mode, "Search should exit with Enter");
    }

    #[test]
    fn test_chat_list_search_exit_with_esc() {
        let mut window = create_test_chat_list_window();
        window.update(Action::ChatListSearch);
        assert!(window.search_mode);

        window.update(Action::ChatListUnselect);
        assert!(!window.search_mode, "Search should exit with Esc/Unselect");
    }

    #[test]
    fn test_chat_list_navigation_in_search_mode() {
        let mut window = create_test_chat_list_window();
        setup_chats(&mut window, &["Alice", "Bob", "Charlie"]);

        window.update(Action::ChatListSearch);
        let modifiers = Modifiers::from(KeyModifiers::empty());

        // Type to filter
        window.update(Action::Key(KeyCode::Char('a'), modifiers.clone()));
        window.update(Action::ChatListSortWithString("a".to_string()));

        // Navigate with arrow keys
        window.update(Action::Key(KeyCode::Down, modifiers.clone()));
        // Navigation should work in search mode
        // Note: Actual navigation state is managed by ListState which is harder to test directly
        assert!(window.search_mode, "Should still be in search mode");
    }

    #[test]
    fn test_chat_list_restore_sort() {
        let mut window = create_test_chat_list_window();
        window.update(Action::ChatListSortWithString("test".to_string()));
        assert_eq!(window.sort_string, Some("test".to_string()));

        window.update(Action::ChatListRestoreSort);
        assert_eq!(window.sort_string, None, "Sort should be restored to None");
    }

    #[test]
    fn test_chat_list_search_clear_on_exit() {
        let mut window = create_test_chat_list_window();
        window.update(Action::ChatListSearch);

        let modifiers = Modifiers::from(KeyModifiers::empty());
        window.update(Action::Key(KeyCode::Char('t'), modifiers.clone()));
        window.update(Action::Key(KeyCode::Char('e'), modifiers.clone()));
        window.update(Action::Key(KeyCode::Char('s'), modifiers.clone()));
        window.update(Action::Key(KeyCode::Char('t'), modifiers));
        assert_eq!(window.search_input, "test");

        window.update(Action::ChatListUnselect);
        assert!(
            window.search_input.is_empty(),
            "Search input should be cleared on exit"
        );
        assert_eq!(
            window.sort_string, None,
            "Sort string should be cleared on exit"
        );
    }
}
