use crate::{
    action::Action,
    app_context::AppContext,
    components::component_traits::{Component, HandleFocus},
    event::Event,
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

/// `ChatWindow` is a struct that represents a window for displaying a chat.
/// It is responsible for managing the layout and rendering of the chat window.
pub struct ChatWindow {
    /// The application context.
    app_context: Rc<AppContext>,
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
    pub fn new(app_context: Rc<AppContext>) -> Self {
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

    /// Select the next message item in the list.
    fn next(&mut self) {
        let i = match self.message_list_state.selected() {
            Some(i) => {
                if i == self.message_list.len() / 2 {
                    if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                        event_tx.send(Event::GetChatHistory).unwrap();
                    }
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
                    if let Some(event_tx) = self.app_context.tg_context().event_tx().as_ref() {
                        event_tx.send(Event::GetChatHistory).unwrap();
                    }
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
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> std::io::Result<()> {
        if !self.focused {
            self.message_list_state.select(None);
        }

        self.message_list
            .clone_from(&self.app_context.tg_context().open_chat_messages());

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
