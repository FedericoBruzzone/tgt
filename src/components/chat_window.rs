use crate::{
    action::Action,
    app_context::AppContext,
    components::component_traits::{Component, HandleFocus},
    event::Event,
    tg::message_entry::MessageEntry,
};
use arboard::Clipboard;
use crossterm::event::KeyCode;
use nucleo_matcher::{Matcher, Utf32Str};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    symbols::{
        border::{self, Set},
        line,
    },
    text::{Line, Span},
    widgets::{Block, Borders, List, ListDirection, ListItem, ListState, Paragraph},
};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

/// `ChatWindow` is a struct that represents a window for displaying a chat.
/// It is responsible for managing the layout and rendering of the chat window.
pub struct ChatWindow {
    /// The application context.
    app_context: Arc<AppContext>,
    /// The name of the `ChatWindow`.
    name: String,
    /// An unbounded sender that send action for processing.
    action_tx: Option<UnboundedSender<Action>>,
    /// A list of message items to be displayed in the `ChatWindow`.
    message_list: Vec<MessageEntry>,
    /// The state of the list.
    message_list_state: ListState,
    /// Indicates whether the `ChatWindow` is focused or not.
    focused: bool,
    /// The string used to filter messages in the chat window.
    sort_string: Option<String>,
    /// Search input string for filtering messages.
    search_input: String,
    /// Whether search mode is active.
    search_mode: bool,
}
/// Implementation of the `ChatWindow` struct.
impl ChatWindow {
    /// Create a new instance of the `ChatWindow` struct.
    ///
    /// # Arguments
    /// * `app_context` - An Arc wrapped AppContext struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `ChatWindow` struct.
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let name = "".to_string();
        let action_tx = None;
        let message_list = vec![];
        let message_list_state = ListState::default();
        let focused = false;
        ChatWindow {
            app_context,
            name,
            action_tx,
            message_list,
            message_list_state,
            focused,
            sort_string: None,
            search_input: String::new(),
            search_mode: false,
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

    /// Select the next message item in the list.
    fn next(&mut self) {
        // Only load more history if not in search mode (to avoid disrupting search)
        if !self.search_mode {
            if let Some(i) = self.message_list_state.selected() {
                if i == self.message_list.len() / 2 {
                    if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                        event_tx.send(Event::GetChatHistory).unwrap();
                    }
                }
            }
        }

        let i = match self.message_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.message_list_state.select(Some(i));
    }

    /// Select the previous message item in the list.
    fn previous(&mut self) {
        // Only load more history if not in search mode (to avoid disrupting search)
        if !self.search_mode {
            if let Some(i) = self.message_list_state.selected() {
                if i == self.message_list.len() / 2 {
                    if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                        event_tx.send(Event::GetChatHistory).unwrap();
                    }
                }
            }
        }

        let i = match self.message_list_state.selected() {
            Some(i) => {
                if i >= self.message_list.len() - 1 {
                    i
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.message_list_state.select(Some(i));
    }

    /// Unselect the message item in the list.
    fn unselect(&mut self) {
        self.message_list_state.select(None);
    }

    /// Delete the selected message item in the list.
    ///
    /// # Arguments
    /// * `revoke` - A boolean flag indicating whether the message should be revoked or not.
    fn delete_selected(&mut self, revoke: bool) {
        if let Some(selected) = self.message_list_state.selected() {
            if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                let sender_id = self.message_list[selected].sender_id();
                if sender_id != self.app_context.tg_context().me() {
                    return;
                }
                let message_id = self.message_list[selected].id();
                event_tx
                    .send(Event::DeleteMessages(vec![message_id], revoke))
                    .unwrap();
                self.app_context.tg_context().delete_message(message_id);
            }
        }
    }

    /// Copy the selected message item in the list.
    fn copy_selected(&self) {
        if let Some(selected) = self.message_list_state.selected() {
            let message = self.message_list[selected].message_content_to_string();
            if let Ok(mut clipboard) = Clipboard::new() {
                clipboard.set_text(message).unwrap();
            }
        }
    }

    /// Edit the selected message item in the list.
    fn edit_selected(&self) {
        if let Some(selected) = self.message_list_state.selected() {
            let sender_id = self.message_list[selected].sender_id();
            if sender_id != self.app_context.tg_context().me() {
                return;
            }
            let message = self.message_list[selected].message_content_to_string();
            let message_id = self.message_list[selected].id();
            if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                event_tx
                    .send(Event::EditMessage(message_id, message))
                    .unwrap();
            }
        }
    }

    /// Reply to the selected message item in the list.
    fn reply_selected(&self) {
        if let Some(selected) = self.message_list_state.selected() {
            let message_id = self.message_list[selected].id();
            let text = self.message_list[selected].message_content_to_string();
            if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                event_tx
                    .send(Event::ReplyMessage(message_id, text))
                    .unwrap();
            }
        }
    }

    /// Sets the string used to filter messages in the chat window.
    fn sort(&mut self, s: String) {
        self.sort_string = Some(s);
    }

    /// Unsets the string used to filter messages in the chat window.
    fn default_sort(&mut self) {
        self.sort_string = None;
    }

    /// Enter search mode.
    fn start_search(&mut self) {
        self.search_mode = true;
        self.search_input.clear();
        // Initialize filtered list from current messages
        self.message_list
            .clone_from(&self.app_context.tg_context().open_chat_messages());
    }

    /// Exit search mode.
    fn stop_search(&mut self) {
        // Preserve the selected message ID before exiting search mode
        let selected_message_id = self
            .message_list_state
            .selected()
            .and_then(|idx| self.message_list.get(idx).map(|m| m.id()));

        self.search_mode = false;
        self.search_input.clear();
        self.default_sort();

        // Restore full message list
        self.message_list
            .clone_from(&self.app_context.tg_context().open_chat_messages());

        // Find and select the message by ID in the full list
        if let Some(message_id) = selected_message_id {
            if let Some(full_list_idx) = self.message_list.iter().position(|m| m.id() == message_id)
            {
                self.message_list_state.select(Some(full_list_idx));
            }
        }
    }

    /// Handle character input in search mode.
    fn handle_search_char(&mut self, c: char) {
        self.search_input.push(c);
        self.sort(self.search_input.clone());
        // Re-filter the message list when search input changes
        // Preserve selection if possible
        let selected_id = self
            .message_list_state
            .selected()
            .and_then(|idx| self.message_list.get(idx).map(|m| m.id()));

        let source_messages: Vec<MessageEntry> =
            self.app_context.tg_context().open_chat_messages().clone();
        self.message_list = source_messages;
        // Apply filter
        if !self.search_input.is_empty() {
            let mut config = nucleo_matcher::Config::DEFAULT;
            config.prefer_prefix = true;
            let mut matcher = Matcher::new(config);
            let search_chars: Vec<char> = self.search_input.chars().collect();
            self.message_list.retain(|message| {
                let message_text = message.message_content_to_string().to_lowercase();
                let message_chars: Vec<char> = message_text.chars().collect();
                matcher
                    .fuzzy_indices(
                        Utf32Str::Unicode(&message_chars),
                        Utf32Str::Unicode(&search_chars),
                        &mut Vec::new(),
                    )
                    .is_some()
            });
        }

        // Restore selection if message still exists in filtered list
        if let Some(id) = selected_id {
            if let Some(new_idx) = self.message_list.iter().position(|m| m.id() == id) {
                self.message_list_state.select(Some(new_idx));
            } else {
                // If selected message no longer matches, select first item
                if !self.message_list.is_empty() {
                    self.message_list_state.select(Some(0));
                }
            }
        }
    }

    /// Handle backspace in search mode.
    fn handle_search_backspace(&mut self) {
        // Preserve selection if possible
        let selected_id = self
            .message_list_state
            .selected()
            .and_then(|idx| self.message_list.get(idx).map(|m| m.id()));

        self.search_input.pop();
        if self.search_input.is_empty() {
            self.default_sort();
            // Restore full message list
            self.message_list
                .clone_from(&self.app_context.tg_context().open_chat_messages());
        } else {
            self.sort(self.search_input.clone());
            // Re-filter the message list when search input changes
            let source_messages: Vec<MessageEntry> =
                self.app_context.tg_context().open_chat_messages().clone();
            self.message_list = source_messages;
            // Apply filter
            let mut config = nucleo_matcher::Config::DEFAULT;
            config.prefer_prefix = true;
            let mut matcher = Matcher::new(config);
            let search_chars: Vec<char> = self.search_input.chars().collect();
            self.message_list.retain(|message| {
                let message_text = message.message_content_to_string().to_lowercase();
                let message_chars: Vec<char> = message_text.chars().collect();
                matcher
                    .fuzzy_indices(
                        Utf32Str::Unicode(&message_chars),
                        Utf32Str::Unicode(&search_chars),
                        &mut Vec::new(),
                    )
                    .is_some()
            });
        }

        // Restore selection if message still exists in filtered list
        if let Some(id) = selected_id {
            if let Some(new_idx) = self.message_list.iter().position(|m| m.id() == id) {
                self.message_list_state.select(Some(new_idx));
            } else if !self.message_list.is_empty() {
                // If selected message no longer matches, select first item
                self.message_list_state.select(Some(0));
            }
        }
    }
}

/// Implement the `HandleFocus` trait for the `ChatWindow` struct.
/// This trait allows the `ChatListWindow` to be focused or unfocused.
impl HandleFocus for ChatWindow {
    /// Set the `focused` flag for the `ChatWindow`.
    fn focus(&mut self) {
        self.focused = true;
    }
    /// Set the `focused` flag for the `ChatWindow`.
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

/// Implement the `Component` trait for the `ChatListWindow` struct.
impl Component for ChatWindow {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> std::io::Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::ChatWindowNext => {
                // Allow navigation even in search mode
                // But skip if we're handling the key event directly (to avoid double processing)
                if !self.search_mode {
                    self.next();
                }
            }
            Action::ChatWindowPrevious => {
                // Allow navigation even in search mode
                // But skip if we're handling the key event directly (to avoid double processing)
                if !self.search_mode {
                    self.previous();
                }
            }
            Action::ChatWindowUnselect => {
                if self.search_mode {
                    self.stop_search();
                } else {
                    self.unselect();
                }
            }
            Action::ChatWindowDeleteForEveryone => {
                if !self.search_mode {
                    self.delete_selected(true);
                }
            }
            Action::ChatWindowDeleteForMe => {
                if !self.search_mode {
                    self.delete_selected(false);
                }
            }
            Action::ChatWindowCopy => {
                if !self.search_mode {
                    self.copy_selected();
                }
            }
            Action::ChatWindowEdit => {
                if !self.search_mode {
                    self.edit_selected();
                }
            }
            Action::ShowChatWindowReply => {
                if !self.search_mode {
                    self.reply_selected();
                }
            }
            Action::ChatWindowSortWithString(s) => self.sort(s),
            Action::ChatWindowRestoreSort => self.default_sort(),
            Action::ChatWindowSearch => {
                // If already in search mode, switch to ChatList search
                if self.search_mode {
                    self.stop_search();
                    if let Some(tx) = self.action_tx.as_ref() {
                        tx.send(Action::ChatListSearch).unwrap();
                    }
                } else {
                    self.start_search();
                }
            }
            Action::ChatListSearch => {
                // If ChatWindow is in search mode and ChatListSearch is triggered, switch to ChatList search
                if self.search_mode {
                    self.stop_search();
                    // The action will be handled by CoreWindow to focus ChatList
                }
            }
            Action::Key(key_code, modifiers) => {
                if self.search_mode {
                    match key_code {
                        KeyCode::Char('r') if modifiers.alt => {
                            // Alt+R in search mode: switch to ChatList search
                            self.stop_search();
                            if let Some(tx) = self.action_tx.as_ref() {
                                tx.send(Action::ChatListSearch).unwrap();
                            }
                        }
                        KeyCode::Char(c)
                            if !modifiers.control && !modifiers.alt && !modifiers.shift =>
                        {
                            self.handle_search_char(c);
                        }
                        KeyCode::Backspace => {
                            self.handle_search_backspace();
                        }
                        KeyCode::Enter | KeyCode::Esc => {
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

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> std::io::Result<()> {
        if !self.focused {
            self.message_list_state.select(None);
        }

        // Only update message list from source when not in search mode
        // This prevents jumping back to bottom when selecting messages during search
        if !self.search_mode {
            // Preserve selection by message ID before updating list
            let selected_message_id = self
                .message_list_state
                .selected()
                .and_then(|idx| self.message_list.get(idx).map(|m| m.id()));

            self.message_list
                .clone_from(&self.app_context.tg_context().open_chat_messages());

            // Filter messages based on search string
            if let Some(s) = self.sort_string.as_ref() {
                if !s.is_empty() {
                    let mut config = nucleo_matcher::Config::DEFAULT;
                    config.prefer_prefix = true;
                    let mut matcher = Matcher::new(config);
                    let search_chars: Vec<char> = s.chars().collect();
                    self.message_list.retain(|message| {
                        let message_text = message.message_content_to_string().to_lowercase();
                        let message_chars: Vec<char> = message_text.chars().collect();
                        matcher
                            .fuzzy_indices(
                                Utf32Str::Unicode(&message_chars),
                                Utf32Str::Unicode(&search_chars),
                                &mut Vec::new(),
                            )
                            .is_some()
                    });
                }
            }

            // Restore selection by message ID in the updated list
            if let Some(message_id) = selected_message_id {
                if let Some(new_idx) = self.message_list.iter().position(|m| m.id() == message_id) {
                    self.message_list_state.select(Some(new_idx));
                }
            }
        } else {
            // When in search mode, preserve the filtered list and selection
            // Don't update from source to prevent jumping back to bottom
            // The filtered list is already set when search mode was activated
        }

        // Split area to include search bar if in search mode
        let (main_area, search_area) = if self.search_mode {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Fill(1)])
                .split(area);
            (layout[1], Some(layout[0]))
        } else {
            (area, None)
        };

        let chat_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Percentage(100)])
            .split(main_area);

        let border = Set {
            top_left: line::NORMAL.vertical_right,
            top_right: line::NORMAL.vertical_left,
            bottom_left: line::NORMAL.horizontal_up,
            ..border::PLAIN
        };
        let style_border_focused = if self.focused {
            self.app_context.style_border_component_focused()
        } else {
            self.app_context.style_chat()
        };

        let mut is_unread_outbox = true;
        let mut is_unread_inbox = true;
        let wrap_width = (area.width / 2) as i32;
        let items = self.message_list.iter().map(|message_entry| {
            let (myself, name_style, content_style, alignment) = if message_entry.sender_id()
                == self.app_context.tg_context().me()
            {
                if message_entry.id() == self.app_context.tg_context().last_read_outbox_message_id()
                {
                    is_unread_outbox = false;
                }
                (
                    true,
                    self.app_context.style_chat_message_myself_name(),
                    self.app_context.style_chat_message_myself_content(),
                    Alignment::Right,
                )
            } else {
                if message_entry.id() == self.app_context.tg_context().last_read_inbox_message_id()
                {
                    is_unread_inbox = false;
                }
                (
                    false,
                    self.app_context.style_chat_message_other_name(),
                    self.app_context.style_chat_message_other_content(),
                    Alignment::Left,
                )
            };
            ListItem::new(
                message_entry
                    .get_text_styled(
                        myself,
                        &self.app_context,
                        is_unread_outbox,
                        name_style,
                        content_style,
                        wrap_width,
                    )
                    .alignment(alignment),
            )
        });

        let block = Block::new()
            .border_set(border)
            .border_style(style_border_focused)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .style(self.app_context.style_chat());
        let list = List::new(items)
            .block(block)
            .style(self.app_context.style_chat())
            .highlight_style(self.app_context.style_item_selected())
            .repeat_highlight_symbol(true)
            .direction(ListDirection::BottomToTop);

        let border_header = Set {
            top_left: line::NORMAL.horizontal_down,
            bottom_left: line::NORMAL.horizontal_up,
            ..border::PLAIN
        };
        let block_header = Block::new()
            .border_set(border_header)
            .border_style(style_border_focused)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .style(self.app_context.style_chat())
            .title(self.name.as_str());
        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                self.app_context
                    .tg_context()
                    .name_of_open_chat_id()
                    .unwrap_or_default(),
                self.app_context.style_chat_chat_name(),
            ),
            Span::raw(" "),
            Span::styled(
                self.app_context.tg_context().open_chat_user_status(),
                self.app_context.style_timestamp(),
            ),
        ]))
        .block(block_header)
        .alignment(Alignment::Center);

        // Draw search bar if in search mode
        if let Some(search_rect) = search_area {
            let search_block = Block::new()
                .border_set(border)
                .border_style(style_border_focused)
                .borders(Borders::ALL)
                .title("Search messages");
            let search_text = format!("{}_", self.search_input);
            let search_paragraph = Paragraph::new(search_text)
                .block(search_block)
                .style(self.app_context.style_chat());
            frame.render_widget(search_paragraph, search_rect);

            // Set cursor position in search bar
            if self.focused && self.search_mode {
                frame.set_cursor_position(Position {
                    x: search_rect.x + self.search_input.len() as u16 + 1,
                    y: search_rect.y + 1,
                });
            }
        }

        frame.render_widget(header, chat_layout[0]);
        frame.render_stateful_widget(list, chat_layout[1], &mut self.message_list_state);

        Ok(())
    }
}
