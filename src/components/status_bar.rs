use {
    crate::{
        action::Action,
        app_context::AppContext,
        components::component_traits::{Component, HandleFocus},
        event::Event,
    },
    ratatui::{
        layout::{Alignment, Rect},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph, Wrap},
    },
    std::sync::Arc,
    tokio::sync::mpsc::UnboundedSender,
};

/// `StatusBar` is a struct that represents a status bar.
/// It is responsible for managing the layout and rendering of the status bar.
pub struct StatusBar {
    /// The application configuration.
    app_context: Arc<AppContext>,
    /// The name of the `StatusBar`.
    name: String,
    /// An unbounded sender that send action for processing.
    command_tx: Option<UnboundedSender<Action>>,
    /// Indicates whether the `StatusBar` is focused or not.
    focused: bool,
    /// The area of the terminal where the all the content will be rendered.
    terminal_area: Rect,
    /// The last key pressed.
    last_key: Event,
}
/// Implementation of `StatusBar` struct.
impl StatusBar {
    /// Create a new instance of the `StatusBar` struct.
    ///
    /// # Arguments
    /// * `app_context` - An Arc wrapped AppContext struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `StatusBar` struct.
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let command_tx = None;
        let name = "".to_string();
        let terminal_area = Rect::default();
        let last_key = Event::Unknown;
        let focused = false;

        StatusBar {
            app_context,
            command_tx,
            name,
            terminal_area,
            last_key,
            focused,
        }
    }
    /// Set the name of the `StatusBar`.
    ///
    /// # Arguments
    /// * `name` - The name of the `StatusBar`.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `StatusBar`.
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }
}

/// Implement the `HandleFocus` trait for the `StatusBar` struct.
/// This trait allows the `StatusBar` to be focused or unfocused.
impl HandleFocus for StatusBar {
    /// Set the `focused` flag for the `StatusBar`.
    fn focus(&mut self) {
        self.focused = true;
    }
    /// Set the `focused` flag for the `StatusBar`.
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

/// Implement the `Component` trait for the `ChatListWindow` struct.
impl Component for StatusBar {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> std::io::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::UpdateArea(area) => {
                self.terminal_area = area;
            }
            Action::Key(key, modifiers) => self.last_key = Event::Key(key, modifiers.into()),
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> std::io::Result<()> {
        let selected_chat = self
            .app_context
            .tg_context()
            .name_of_open_chat_id()
            .unwrap_or_default();
        let text = vec![Line::from(vec![
            Span::styled(
                "Press ",
                self.app_context.style_status_bar_message_quit_text(),
            ),
            Span::styled("q ", self.app_context.style_status_bar_message_quit_key()),
            Span::styled("or ", self.app_context.style_status_bar_message_quit_text()),
            Span::styled(
                "ctrl+c ",
                self.app_context.style_status_bar_message_quit_key(),
            ),
            Span::styled(
                "to quit",
                self.app_context.style_status_bar_message_quit_text(),
            ),
            //
            Span::raw("     "),
            Span::styled(
                "Help: ",
                self.app_context.style_status_bar_message_quit_text(),
            ),
            Span::styled(
                "alt+F1",
                self.app_context.style_status_bar_message_quit_key(),
            ),
            //
            Span::raw("     "),
            Span::styled(
                "Open chat: ",
                self.app_context.style_status_bar_open_chat_text(),
            ),
            Span::styled(
                selected_chat,
                self.app_context.style_status_bar_open_chat_name(),
            ),
            //
            Span::raw("     "),
            Span::styled(
                "Key pressed: ",
                self.app_context.style_status_bar_press_key_text(),
            ),
            Span::styled(
                self.last_key.to_string(),
                self.app_context.style_status_bar_press_key_key(),
            ),
            //
            Span::raw("     "),
            Span::styled("Size: ", self.app_context.style_status_bar_size_info_text()),
            Span::styled(
                self.terminal_area.width.to_string(),
                self.app_context.style_status_bar_size_info_numbers(),
            ),
            Span::styled(" x ", self.app_context.style_status_bar_size_info_text()),
            Span::styled(
                self.terminal_area.height.to_string(),
                self.app_context.style_status_bar_size_info_numbers(),
            ),
        ])];

        let paragraph = Paragraph::new(text)
            .block(Block::new().title(self.name.as_str()).borders(Borders::ALL))
            .style(self.app_context.style_status_bar())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);

        Ok(())
    }
}
