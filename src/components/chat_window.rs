use crate::{
    action::Action,
    app_context::AppContext,
    components::component_traits::{Component, HandleFocus},
    event::Event,
    tg::message_entry::MessageEntry,
};
use arboard::Clipboard;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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

    /// Build message_list from data layer (read-only API).
    fn refresh_message_list_from_store(&mut self) {
        let tg = self.app_context.tg_context();
        self.message_list = tg
            .ordered_message_ids()
            .into_iter()
            .filter_map(|id| tg.get_message(id))
            .collect();
    }

    /// Select the next message item in the list.
    fn next(&mut self) {
        let len = self.message_list.len();
        // Load more history when near top of loaded range (and not already loading)
        if len > 0 && !self.app_context.tg_context().is_history_loading() {
            let oldest = self.app_context.tg_context().oldest_message_id();
            let selected_id = self
                .message_list_state
                .selected()
                .and_then(|i| self.message_list.get(i).map(|m| m.id()));
            let near_top = match (oldest, selected_id) {
                (Some(old), Some(sel)) => sel == old,
                (_, Some(_)) => self.message_list_state.selected() == Some(0),
                _ => false,
            };
            if near_top {
                if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                    let _ = event_tx.send(Event::GetChatHistory);
                }
            }
        }

        let i = match (len, self.message_list_state.selected()) {
            (0, _) => {
                self.message_list_state.select(None);
                return;
            }
            (_, Some(i)) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            (_, None) => 0,
        };
        self.message_list_state.select(Some(i));
    }

    /// Select the previous message item in the list (down = towards newer messages).
    fn previous(&mut self) {
        let len = self.message_list.len();
        // Load newer messages when near bottom (so user can scroll forward in time)
        if len > 0 && !self.app_context.tg_context().is_history_loading() {
            let newest = self.app_context.tg_context().newest_message_id();
            let selected_id = self
                .message_list_state
                .selected()
                .and_then(|i| self.message_list.get(i).map(|m| m.id()));
            let near_bottom = match (newest, selected_id) {
                (Some(new), Some(sel)) => sel == new,
                (_, Some(_)) => self.message_list_state.selected() == Some(len.saturating_sub(1)),
                _ => false,
            };
            if near_bottom {
                if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                    let _ = event_tx.send(Event::GetChatHistoryNewer);
                }
            }
        }

        let i = match (len, self.message_list_state.selected()) {
            (0, _) => {
                self.message_list_state.select(None);
                return;
            }
            (_, Some(i)) => {
                if i >= len.saturating_sub(1) {
                    i
                } else {
                    i + 1
                }
            }
            (_, None) => 0,
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
            Action::ChatWindowNext => self.next(),
            Action::ChatWindowPrevious => self.previous(),
            Action::ChatWindowUnselect => self.unselect(),
            Action::ChatWindowDeleteForEveryone => self.delete_selected(true),
            Action::ChatWindowDeleteForMe => self.delete_selected(false),
            Action::ChatWindowCopy => self.copy_selected(),
            Action::ChatWindowEdit => self.edit_selected(),
            Action::ShowChatWindowReply => self.reply_selected(),
            Action::JumpCompleted(_message_id) => {
                // Selection by message_id is applied in draw() when jump_target_message_id is set
            }
            Action::ChatWindowSortWithString(_) | Action::ChatWindowRestoreSort => {
                // No-op: chat message search is server-side only (search overlay)
            }
            Action::ChatWindowSearch | Action::ChatListSearch => {
                // Handled by CoreWindow: opens search overlay or focuses ChatList
            }
            Action::Key(key_code, modifiers) => {
                if self.focused {
                    match key_code {
                        KeyCode::Char('r') if modifiers.alt => {
                            // Alt+R: switch to ChatList search (handled by CoreWindow)
                            if let Some(tx) = self.action_tx.as_ref() {
                                let _ = tx.send(Action::ChatListSearch);
                            }
                        }
                        KeyCode::Up => self.next(),
                        KeyCode::Down => self.previous(),
                        KeyCode::Tab => self.next(),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> std::io::Result<()> {
        // Capture selection by ID before any clear, so we can restore viewport on redraw (e.g. when unfocused)
        let selected_message_id_before = self
            .message_list_state
            .selected()
            .and_then(|idx| self.message_list.get(idx).map(|m| m.id()));

        if !self.focused {
            self.message_list_state.select(None);
        }

        // Always refresh message list from store (no local filtering; search is server-side).
        let selected_message_id = selected_message_id_before;
        let prev_len = self.message_list.len();
        self.refresh_message_list_from_store();

        // After jump-to-message: select the target and clear the flag
        let jump_target = self.app_context.tg_context().jump_target_message_id();
        if jump_target != 0 {
            self.app_context.tg_context().set_jump_target_message_id(0);
            if let Some(idx) = self.message_list.iter().position(|m| m.id() == jump_target) {
                self.message_list_state.select(Some(idx));
            }
        } else {
            // Restore selection by message ID when possible
            let selection_restored = selected_message_id.and_then(|id| {
                self.message_list.iter().position(|m| m.id() == id).map(|idx| {
                    self.message_list_state.select(Some(idx));
                    id
                })
            });
            // When we have no valid selection (e.g. just entered chat), jump to latest by ID.
            // When list grew and we were at bottom (selected was newest), stay at bottom.
            let at_bottom = selection_restored
                .zip(self.app_context.tg_context().newest_message_id())
                .map_or(false, |(sel_id, newest_id)| sel_id == newest_id);
            let should_jump_to_latest = selection_restored.is_none()
                || (at_bottom && prev_len < self.message_list.len());
            if should_jump_to_latest && !self.message_list.is_empty() {
                if let Some(newest_id) = self.app_context.tg_context().newest_message_id() {
                    if let Some(new_idx) =
                        self.message_list.iter().position(|m| m.id() == newest_id)
                    {
                        self.message_list_state.select(Some(new_idx));
                    }
                }
            }
        }

        let chat_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Percentage(100)])
            .split(area);

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
        let mut items: Vec<ListItem<'_>> = self
            .message_list
            .iter()
            .map(|message_entry| {
                let (myself, name_style, content_style, alignment) =
                    if message_entry.sender_id() == self.app_context.tg_context().me() {
                        if message_entry.id()
                            == self.app_context.tg_context().last_read_outbox_message_id()
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
                        if message_entry.id()
                            == self.app_context.tg_context().last_read_inbox_message_id()
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
            })
            .collect();
        if self.app_context.tg_context().is_history_loading() {
            items.push(ListItem::new(Line::from(Span::styled(
                "Loading…",
                self.app_context.style_timestamp(),
            ))));
        }

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
            .direction(ListDirection::TopToBottom);

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

        frame.render_widget(header, chat_layout[0]);
        frame.render_stateful_widget(list, chat_layout[1], &mut self.message_list_state);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        action::{Action, Modifiers},
        components::search_tests::{create_mock_message, create_test_app_context},
    };
    use crossterm::event::{KeyCode, KeyModifiers};

    fn create_test_chat_window() -> ChatWindow {
        let app_context = create_test_app_context();
        ChatWindow::new(app_context)
    }

    fn setup_messages(window: &mut ChatWindow, messages: Vec<MessageEntry>) {
        let tg_context = window.app_context.tg_context();
        {
            let mut store = tg_context.open_chat_messages();
            store.clear();
            store.insert_messages(messages);
        }
        window.refresh_message_list_from_store();
    }

    #[test]
    fn test_chat_window_selection_by_id() {
        let mut window = create_test_chat_window();
        let messages = vec![
            create_mock_message(100, "First"),
            create_mock_message(200, "Second"),
            create_mock_message(300, "Third"),
        ];
        setup_messages(&mut window, messages);
        window.message_list_state.select(Some(1));
        let selected_id = window
            .message_list_state
            .selected()
            .and_then(|idx| window.message_list.get(idx).map(|m| m.id()));
        assert_eq!(selected_id, Some(200));
    }

    #[test]
    fn test_chat_window_navigation() {
        let mut window = create_test_chat_window();
        let messages = vec![
            create_mock_message(1, "One"),
            create_mock_message(2, "Two"),
            create_mock_message(3, "Three"),
        ];
        setup_messages(&mut window, messages);
        let modifiers = Modifiers::from(KeyModifiers::empty());
        window.update(Action::Key(KeyCode::Down, modifiers.clone()));
        window.update(Action::Key(KeyCode::Up, modifiers.clone()));
        let selected = window.message_list_state.selected();
        assert_eq!(selected, Some(0));
    }

    #[test]
    fn test_chat_window_sort_actions_no_op() {
        let mut window = create_test_chat_window();
        window.update(Action::ChatWindowSortWithString("test".to_string()));
        window.update(Action::ChatWindowRestoreSort);
        // Chat message search is server-side only; these are no-ops
        assert_eq!(window.message_list.len(), 0);
    }
}
