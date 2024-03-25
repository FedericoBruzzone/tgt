use {
    crate::{
        components::component::{Component, HandleFocus, HandleSmallArea},
        enums::action::Action,
    },
    ratatui::{
        layout::Rect,
        style::{Color, Modifier, Style},
        symbols::border::PLAIN,
        widgets::{
            block::{Block, Title},
            Borders, List, ListDirection,
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
        let focused = false;

        ChatListWindow {
            name,
            command_tx,
            small_area,
            chat_list,
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

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame<'_>,
        area: Rect,
    ) -> std::io::Result<()> {
        let color_focused = if self.focused {
            Color::Cyan
        } else {
            Color::White
        };
        let list = List::new(
            self.chat_list
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>(),
        )
        .block(
            Block::default()
                .title("Chat List")
                .border_set(PLAIN)
                .border_style(Style::default().fg(color_focused))
                .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
                .title(Title::from(self.name.as_str())),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::BottomToTop);
        frame.render_widget(list, area);
        Ok(())
    }
}
