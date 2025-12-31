use {
    crate::{
        action::Action,
        app_context::AppContext,
        components::component_traits::{Component, HandleFocus},
    },
    ratatui::{
        layout::Rect,
        symbols::{
            border::{Set, PLAIN},
            line::NORMAL,
        },
        text::{Line, Span, Text},
        widgets::{Block, Borders, Paragraph, Wrap},
    },
    std::{io, sync::Arc},
    tokio::sync::mpsc,
};

/// `ReplyMessage` is a struct that represents a window for replying to messages.
/// It is responsible for managing the layout and rendering of the reply message window.
pub struct ReplyMessage {
    /// The application configuration.
    app_context: Arc<AppContext>,
    /// The name of the `ReplyMessage`.
    name: String,
    /// An unbounded sender that send action for processing.
    command_tx: Option<mpsc::UnboundedSender<Action>>,
    /// Indicates whether the `ReplyMessage` is focused or not.
    focused: bool,
}
/// Implementation of `ReplyMessage` struct.
impl ReplyMessage {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let command_tx = None;
        let name = "".to_string();
        let focused = false;
        ReplyMessage {
            app_context,
            command_tx,
            name,
            focused,
        }
    }
    /// Set the name of the `ReplyMessage`.
    ///
    /// # Arguments
    /// * `name` - The name of the `ReplyMessage`.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `ReplyMessage`.
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }
}

/// Implement the `HandleFocus` trait for the `ReplyMessage` struct.
/// This trait allows the `ReplyMessage` to be focused or unfocused.
impl HandleFocus for ReplyMessage {
    /// Set the `focused` flag for the `ReplyMessage`.
    fn focus(&mut self) {
        self.focused = true;
    }
    /// Set the `focused` flag for the `ReplyMessage`.
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

/// Implement the `Component` trait for the `ReplyMessage` struct.
impl Component for ReplyMessage {
    fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> io::Result<()> {
        let mut text = Text::default();
        text.extend(vec![Line::from(vec![Span::styled(
            (*self.app_context.tg_context().reply_message_text()).to_string(),
            self.app_context.style_reply_message_message_text(),
        )])]);

        let collapsed_border = Set {
            top_left: NORMAL.vertical_right,
            top_right: NORMAL.vertical_left,
            bottom_left: NORMAL.vertical_right,
            bottom_right: NORMAL.vertical_left,
            ..PLAIN
        };

        let block = Block::new()
            .border_set(collapsed_border)
            .borders(Borders::RIGHT | Borders::LEFT | Borders::TOP)
            .title("Reply Message")
            .border_style(self.app_context.style_reply_message())
            .style(self.app_context.style_reply_message());

        let paragraph = Paragraph::new(text)
            .block(block)
            .style(self.app_context.style_reply_message())
            // .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);

        Ok(())
    }
}
