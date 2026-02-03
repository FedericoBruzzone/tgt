use crate::{
    action::Action,
    app_context::AppContext,
    component_name::ComponentName,
    components::component_traits::{Component, HandleFocus},
    event::Event,
    tg::message_entry::MessageEntry,
};
use arboard::Clipboard;
use crossterm::event::{KeyCode, MouseEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    symbols::{
        border::{self, Set},
        line,
    },
    text::{Line, Span, Text},
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
    /// When true, next draw will select the newest message (Alt+C restore order).
    request_jump_to_latest: bool,
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
            request_jump_to_latest: false,
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

    /// Build message_list from data layer (read-only API). Uses a single-lock snapshot to avoid
    /// TOCTOU: another thread clearing the store between ordered_message_ids() and get_message().
    fn refresh_message_list_from_store(&mut self) {
        self.message_list = self.app_context.tg_context().ordered_messages_snapshot();
    }

    /// Select the next message item in the list (down = towards newer messages).
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

        // Handle empty list: unselect and return early
        if len == 0 {
            self.message_list_state.select(None);
            return;
        }

        // Bounds check: saturating_sub prevents going below 0 when already at oldest message (index 0).
        // If no selection, start at index 0 (oldest message).
        // Without these checks, scrolling past the ends could cause panics or invalid indices.
        let i = self
            .message_list_state
            .selected()
            .map(|i| i.saturating_sub(1))
            .unwrap_or(0);
        self.message_list_state.select(Some(i));
    }

    /// Select the previous message item in the list (up = towards older messages).
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

        // Handle empty list: unselect and return early
        if len == 0 {
            self.message_list_state.select(None);
            return;
        }

        // Bounds check: min(max_idx) prevents going above len-1 when already at newest message (index len-1).
        // If no selection, start at index 0 (oldest message).
        // Without these checks, scrolling past the ends could cause panics or invalid indices.
        let max_idx = len.saturating_sub(1);
        let i = self
            .message_list_state
            .selected()
            .map(|i| (i + 1).min(max_idx))
            .unwrap_or(0);
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
        let Some(selected) = self.message_list_state.selected() else {
            return;
        };
        let Some(entry) = self.message_list.get(selected) else {
            return;
        };
        let message = entry.message_content_to_string();
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if clipboard.set_text(&message).is_ok() {
                    if let Some(tx) = self.action_tx.as_ref() {
                        let _ = tx.send(Action::StatusMessage("Message yanked".into()));
                    }
                } else {
                    tracing::warn!("Clipboard set_text failed");
                }
            }
            Err(e) => {
                tracing::warn!("Clipboard unavailable (copy message): {}", e);
            }
        }
    }

    /// Edit the selected message item in the list. Only our own messages can be edited.
    fn edit_selected(&self) {
        let Some(selected) = self.message_list_state.selected() else {
            return;
        };
        let Some(entry) = self.message_list.get(selected) else {
            return;
        };
        if entry.sender_id() != self.app_context.tg_context().me() {
            // Not our message: do nothing to avoid bugging the chat.
            return;
        }
        let message_id = entry.id();
        let message = entry.message_content_to_string();
        if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
            let _ = event_tx.send(Event::EditMessage(message_id, message));
        }
    }

    /// Reply to the selected message item in the list.
    /// Focuses the prompt and sets reply mode so the user can type in the same prompt used for sending/editing.
    fn reply_selected(&self) {
        if let Some(selected) = self.message_list_state.selected() {
            let message_id = self.message_list[selected].id();
            let text = self.message_list[selected].message_content_to_string();
            if let Some(tx) = self.action_tx.as_ref() {
                let _ = tx.send(Action::FocusComponent(ComponentName::Prompt));
                let _ = tx.send(Action::ReplyMessage(message_id, text));
            }
        }
    }

    /// Wraps each line with a border span on one side only (reply-target border-only highlight).
    /// Messages from others: `│` at the start of each line. Messages from me: `│` at the end.
    /// This keeps borders aligned and avoids broken vertical bars under each other.
    fn wrap_text_with_reply_border(
        content: Text,
        border_style: Style,
        alignment: Alignment,
        myself: bool,
    ) -> Text {
        let wrapped_lines: Vec<Line> = content
            .into_iter()
            .map(|line| {
                if myself {
                    let mut spans: Vec<Span> = line.into_iter().collect();
                    spans.push(Span::styled(" │", border_style));
                    Line::from(spans)
                } else {
                    let mut spans = vec![Span::styled("│ ", border_style)];
                    spans.extend(line);
                    Line::from(spans)
                }
            })
            .collect();
        Text::from(wrapped_lines).alignment(alignment)
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

    fn handle_mouse_events(
        &mut self,
        mouse: crossterm::event::MouseEvent,
    ) -> std::io::Result<Option<Action>> {
        if !self.focused {
            return Ok(None);
        }
        match mouse.kind {
            MouseEventKind::ScrollDown => Ok(Some(Action::ChatWindowPrevious)),
            MouseEventKind::ScrollUp => Ok(Some(Action::ChatWindowNext)),
            _ => Ok(None),
        }
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
            Action::ChatWindowSortWithString(_) => {
                // No-op: chat message search is server-side only (search overlay)
            }
            Action::ChatWindowRestoreSort => {
                self.request_jump_to_latest = true;
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
        let jump_target = self
            .app_context
            .tg_context()
            .jump_target_message_id()
            .as_i64();
        if jump_target != 0 {
            self.app_context
                .tg_context()
                .set_jump_target_message_id_i64(0);
            if let Some(idx) = self.message_list.iter().position(|m| m.id() == jump_target) {
                self.message_list_state.select(Some(idx));
            }
        } else {
            // Alt+C restore order: jump to latest message
            if self.request_jump_to_latest {
                self.request_jump_to_latest = false;
                if !self.message_list.is_empty() {
                    if let Some(newest_id) = self.app_context.tg_context().newest_message_id() {
                        if let Some(new_idx) =
                            self.message_list.iter().position(|m| m.id() == newest_id)
                        {
                            self.message_list_state.select(Some(new_idx));
                        }
                    }
                }
            } else {
                // Restore selection by message ID when possible
                let selection_restored = selected_message_id.and_then(|id| {
                    self.message_list
                        .iter()
                        .position(|m| m.id() == id)
                        .map(|idx| {
                            self.message_list_state.select(Some(idx));
                            id
                        })
                });
                // When we have no valid selection (e.g. just entered chat), jump to latest by ID.
                // When list grew and we were at bottom (selected was newest), stay at bottom.
                let at_bottom = selection_restored
                    .zip(self.app_context.tg_context().newest_message_id())
                    .is_some_and(|(sel_id, newest_id)| sel_id == newest_id);
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

        let block = Block::new()
            .border_set(border)
            .border_style(style_border_focused)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .style(self.app_context.style_chat());
        let list_area = chat_layout[1];
        let list_inner = block.inner(list_area);
        // Leave margin so wrapped lines and wide chars don't overflow the right edge
        let wrap_width = list_inner.width.saturating_sub(2) as i32;

        let reply_message_id = self.app_context.tg_context().reply_message_id().as_i64();
        let mut is_unread_outbox = true;
        let mut is_unread_inbox = true;
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
                let content = message_entry
                    .get_text_styled(
                        myself,
                        &self.app_context,
                        is_unread_outbox,
                        name_style,
                        content_style,
                        wrap_width,
                    )
                    .alignment(alignment);
                let is_reply_target =
                    reply_message_id != 0 && message_entry.id() == reply_message_id;
                if is_reply_target {
                    let border_style = self.app_context.style_item_reply_target();
                    let content_with_border =
                        Self::wrap_text_with_reply_border(content, border_style, alignment, myself);
                    // Do not apply border_style to the whole ListItem: only the │ spans
                    // use it, so the message keeps its original formatting (name, reply-to, content).
                    ListItem::new(content_with_border)
                } else {
                    ListItem::new(content)
                }
            })
            .collect();
        if self.app_context.tg_context().is_history_loading() {
            items.push(ListItem::new(Line::from(Span::styled(
                "Loading…",
                self.app_context.style_timestamp(),
            ))));
        }

        // Render from bottom to top so content sits at the bottom (above the prompt) when
        // there are fewer messages than fit in the area. Reverse items so the list's
        // "first" item is newest; with BottomToTop that draws at the bottom, keeping
        // visual order: oldest at top, newest at bottom.
        items.reverse();
        let item_count = items.len();

        // Selection is stored as index into message_list (oldest=0). In reversed list,
        // newest=0, oldest=len-1. Convert to list index for the widget, then back after render.
        let orig_selected = self.message_list_state.selected();
        let list_selected = orig_selected.map(|i| item_count.saturating_sub(1).saturating_sub(i));
        self.message_list_state.select(list_selected);

        // Use normal selection highlight so message text keeps its original formatting.
        // Reply-target is already indicated by the │ border spans only.
        let highlight_style = self.app_context.style_item_selected();
        let list = List::new(items)
            .block(block)
            .style(self.app_context.style_chat())
            .highlight_style(highlight_style)
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

        frame.render_widget(header, chat_layout[0]);
        frame.render_stateful_widget(list, chat_layout[1], &mut self.message_list_state);

        // Restore selection to message_list index (oldest=0) for next/previous and other logic.
        let list_sel = self.message_list_state.selected();
        self.message_list_state
            .select(list_sel.map(|i| item_count.saturating_sub(1).saturating_sub(i)));

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
        window.focus(); // Key events only handled when focused
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

    /// Edit on someone else's message must be a no-op (does not send EditMessage, does not bug the chat).
    /// create_mock_message uses sender user_id 1; test app context has me() = 0, so all mock messages are "others".
    #[test]
    fn test_edit_on_others_message_is_no_op() {
        let mut window = create_test_chat_window();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        window.register_action_handler(tx).unwrap();
        let messages = vec![
            create_mock_message(100, "Other's message"),
            create_mock_message(200, "Another other"),
        ];
        setup_messages(&mut window, messages);
        window.message_list_state.select(Some(0));
        let selected_before = window.message_list_state.selected();
        let selected_id_before =
            selected_before.and_then(|i| window.message_list.get(i).map(|m| m.id()));
        window.update(Action::ChatWindowEdit);
        // Selection unchanged; no EditMessage is sent (we can't easily assert no event without mock event_tx).
        assert_eq!(window.message_list_state.selected(), selected_before);
        assert_eq!(
            selected_before.and_then(|i| window.message_list.get(i).map(|m| m.id())),
            selected_id_before
        );
    }
}
