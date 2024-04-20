use std::sync::Arc;

use crate::{
    action::Action,
    app_context::AppContext,
    components::component::{Component, HandleFocus, HandleSmallArea},
};
use ratatui::{
    layout::{Alignment, Rect},
    symbols::{
        border::{self, Set},
        line,
    },
    text::Line,
    widgets::{Block, Borders, List, ListDirection, ListItem, ListState},
};
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
    /// A flag indicating whether the `ChatWindow` should be displayed as a
    /// smaller version of itself.
    small_area: bool,
    /// A list of message items to be displayed in the `ChatWindow`.
    message_list: Vec<String>, // [TODO] Use message_item struct
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
        let small_area = false;
        let message_list = vec![
            "My message".to_string(),
            "Your message".to_string(),
            "My message".to_string(),
            "Your message".to_string(),
            "My message".to_string(),
            "Your message".to_string(),
            "My message".to_string(),
            "Your message".to_string(),
        ];
        let message_list_state = ListState::default();
        let focused = false;
        ChatWindow {
            app_context,
            name,
            action_tx,
            small_area,
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

/// Implement the `HandleSmallArea` trait for the `ChatWindow` struct.
/// This trait allows the `ChatWindow` to display a smaller version of itself if
/// necessary.
impl HandleSmallArea for ChatWindow {
    /// Set the `small_area` flag for the `ChatWindow`.
    ///
    /// # Arguments
    /// * `small_area` - A boolean flag indicating whether the `ChatWindow`
    ///   should be displayed as a smaller version of itself.
    fn with_small_area(&mut self, small_area: bool) {
        self.small_area = small_area;
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
            Action::MessageListNext => self.next(),
            Action::MessageListPrevious => self.previous(),
            Action::MessageListUnselect => self.unselect(),
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> std::io::Result<()> {
        let border = Set {
            top_left: line::NORMAL.horizontal_down,
            bottom_left: line::NORMAL.horizontal_up,
            ..border::PLAIN
        };
        let style_border_focused = if self.focused {
            self.app_context.style_border_component_focused()
        } else {
            self.app_context.style_chat()
        };

        let items = self.message_list.iter().enumerate().map(|(i, item)| {
            let alignment = if i % 2 == 0 {
                Alignment::Right
            } else {
                Alignment::Left
            };
            let style = if i % 2 == 0 {
                self.app_context.style_chat_message_myself()
            } else {
                self.app_context.style_chat_message_other()
            };

            // ListItem::new(Text::from(item.as_str()).alignment(alignment))
            ListItem::new(
                Line::from(
                    item.as_str(), /* .fg(style.fg.unwrap()).bg(style.bg.
                                    * unwrap()), */
                )
                .alignment(alignment)
                .style(style),
            )
        });

        let block = Block::new()
            .border_set(border)
            .border_style(style_border_focused)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .style(self.app_context.style_chat())
            .title(self.name.as_str());

        let list = List::new(items)
            .block(block)
            .style(self.app_context.style_chat_list())
            .highlight_style(self.app_context.style_item_selected())
            .repeat_highlight_symbol(true)
            .direction(ListDirection::BottomToTop);

        frame.render_stateful_widget(list, area, &mut self.message_list_state);

        Ok(())
    }
}
