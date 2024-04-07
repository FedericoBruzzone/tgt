use {
    crate::{
        components::component::{Component, HandleFocus, HandleSmallArea},
        configs::config_theme::{
            style_border_component_focused, style_chat_list,
            style_item_selected,
        },
        enums::action::Action,
    },
    ratatui::{
        layout::Rect,
        symbols::border::PLAIN,
        widgets::{
            block::{Block, Title},
            Borders, List, ListDirection, ListState,
        },
    },
    tokio::sync::mpsc::UnboundedSender,
};

/// `ChatListWindow` is a struct that represents a window for displaying a list
/// of chat items. It is responsible for managing the layout and rendering of
/// the chat list.
pub struct ChatListWindow {
    /// The name of the `ChatListWindow`.
    name: String,
    /// An unbounded sender that send action for processing.
    command_tx: Option<UnboundedSender<Action>>,
    /// A flag indicating whether the `ChatListWindow` should be displayed as a
    /// smaller version of itself.
    small_area: bool,
    /// A list of chat items to be displayed in the `ChatListWindow`.
    chat_list: Vec<String>, // [TODO] Use chat_item struct
    /// The state of the list.
    chat_list_state: ListState,
    /// Indicates whether the `ChatListWindow` is focused or not.
    focused: bool,
}

impl Default for ChatListWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatListWindow {
    /// Create a new instance of the `ChatListWindow` struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `ChatListWindow` struct.
    pub fn new() -> Self {
        let name = "".to_string();
        let command_tx = None;
        let small_area = false;
        let chat_list = vec![
            "Chat 1".to_string(),
            "Chat 2".to_string(),
            "Chat 2".to_string(),
            "Chat 2".to_string(),
            "Chat 2".to_string(),
            "Chat 2".to_string(),
        ];
        let chat_list_state = ListState::default();
        let focused = false;

        ChatListWindow {
            name,
            command_tx,
            small_area,
            chat_list,
            chat_list_state,
            focused,
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

/// Implement the `HandleSmallArea` trait for the `ChatListWindow` struct.
/// This trait allows the `ChatListWindow` to display a smaller version of
/// itself if necessary.
impl HandleSmallArea for ChatListWindow {
    /// Set the `small_area` flag for the `ChatListWindow`.
    ///
    /// # Arguments
    /// * `small_area` - A boolean flag indicating whether the `ChatListWindow`
    ///   should be displayed as a smaller version of itself.
    fn with_small_area(&mut self, small_area: bool) {
        self.small_area = small_area;
    }
}

/// Implement the `Component` trait for the `ChatListWindow` struct.
impl Component for ChatListWindow {
    fn register_action_handler(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> std::io::Result<()> {
        self.command_tx = Some(tx.clone());
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::ChatListNext => self.next(),
            Action::ChatListPrevious => self.previous(),
            Action::ChatListUnselect => self.unselect(),
            _ => {}
        }
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame<'_>,
        area: Rect,
    ) -> std::io::Result<()> {
        let style_border_focused = if self.focused {
            style_border_component_focused()
        } else {
            style_chat_list()
        };

        let items = self.chat_list.iter().map(|item| item.as_str());
        let block = Block::default()
            .border_set(PLAIN)
            .border_style(style_border_focused)
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .title(Title::from(self.name.as_str()));

        let list = List::new(items)
            .block(block)
            .style(style_chat_list())
            .highlight_style(style_item_selected())
            .highlight_symbol("➤ ")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        frame.render_stateful_widget(list, area, &mut self.chat_list_state);
        Ok(())
    }
}
