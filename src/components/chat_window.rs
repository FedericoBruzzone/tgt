use crate::{
    action::Action,
    app_context::AppContext,
    component_name::ComponentName,
    components::component_traits::{Component, HandleFocus},
    tg::message_entry::MessageEntry,
};
use arboard::Clipboard;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    symbols::{
        border::{self, Set},
        line,
    },
    text::{Line, Span},
    widgets::{Block, Borders, List, ListDirection, ListItem, ListState, Paragraph},
};
use std::rc::Rc;
use tokio::sync::mpsc::UnboundedSender;

// TODO: Rework UI configs in order to remove app_context
/// `ChatWindow` is a struct that represents a window for displaying a chat.
/// It is responsible for managing the layout and rendering of the chat window.
pub struct ChatWindow {
    /// The application context.
    app_context: Rc<AppContext>,
    /// The name of the `ChatWindow`.
    name: String,
    /// An unbounded sender that send action for processing.
    app_tx: Option<UnboundedSender<Action>>,
    tgbackend_tx: Option<UnboundedSender<Action>>,
    /// A list of message items to be displayed in the `ChatWindow`.
    message_list: Vec<MessageEntry>,
    /// The state of the list.
    message_list_state: ListState,
    /// Indicates whether the `ChatWindow` is focused or not.
    focused: bool,
    chat_id: i64,
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
    pub fn new(app_context: Rc<AppContext>) -> Self {
        let name = "".to_string();
        let app_tx = None;
        let tgbackend_tx = None;
        let message_list = vec![];
        let message_list_state = ListState::default();
        let focused = false;
        let chat_id = 0;
        ChatWindow {
            app_context,
            name,
            app_tx,
            tgbackend_tx,
            message_list,
            message_list_state,
            focused,
            chat_id,
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

    pub fn register_tgbackend_tx(&mut self, tgbackend_tx: UnboundedSender<Action>) {
        self.tgbackend_tx = Some(tgbackend_tx);
    }

    /// Select the next message item in the list.
    fn next(&mut self) {
        let i = match self.message_list_state.selected() {
            Some(i) => {
                if i == self.message_list.len() / 2 {
                    self.app_context
                        .action_tx()
                        .send(Action::GetChatHistoryOld)
                        .unwrap();
                }

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
        let i = match self.message_list_state.selected() {
            Some(i) => {
                if i == self.message_list.len() / 2 {
                    self.app_context
                        .action_tx()
                        .send(Action::GetChatHistoryOld)
                        .unwrap();
                }

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
            let sender_id = self.message_list[selected].sender_id();
            if sender_id != self.app_context.tg_context().me() {
                return;
            }
            let message_id = self.message_list[selected].id();
            self.tgbackend_tx
                .as_ref()
                .unwrap()
                .send(Action::DeleteMessages(
                    self.chat_id,
                    vec![message_id],
                    revoke,
                ))
                .unwrap();
            // Should probably wait for Action::DeleteMessagesResponse()
            // but it is rare for this to fail. Will do in future.
            self.message_list.retain(|m| m.id() != message_id);
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

            self.app_tx
                .as_ref()
                .unwrap()
                .send(Action::FocusComponent(ComponentName::Prompt))
                .unwrap();
            self.app_tx
                .as_ref()
                .unwrap()
                .send(Action::EditMessage(message_id, message))
                .unwrap();
        }
    }

    /// Reply to the selected message item in the list.
    fn reply_selected(&self) {
        if let Some(selected) = self.message_list_state.selected() {
            let message_id = self.message_list[selected].id();
            let text = self.message_list[selected].message_content_to_string();

            self.app_tx
                .as_ref()
                .unwrap()
                .send(Action::FocusComponent(ComponentName::Prompt))
                .unwrap();
            self.app_tx
                .as_ref()
                .unwrap()
                .send(Action::ReplyMessage(message_id, text))
                .unwrap();
        }
    }

    fn update_messages(&mut self, chat_id: i64, resp: Vec<MessageEntry>) {
        if chat_id != self.chat_id {
            return;
        }
        self.message_list.extend_from_slice(&resp);
        self.message_list.sort_by(|a, b| a.id().cmp(&b.id()));
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
        self.app_tx = Some(tx);
        Ok(())
    }

    // Read comments in delete_selected() for missing checks
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
            Action::GetChatHistoryResponse(chat_id, resp) => self.update_messages(chat_id, resp),
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> std::io::Result<()> {
        if !self.focused {
            self.message_list_state.select(None);
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

        frame.render_widget(header, chat_layout[0]);
        frame.render_stateful_widget(list, chat_layout[1], &mut self.message_list_state);

        Ok(())
    }
}
