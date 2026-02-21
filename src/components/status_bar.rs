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
    std::time::Instant,
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
    /// Optional short status message; cleared after STATUS_MESSAGE_DURATION or on next key press.
    status_message: Option<String>,
    /// When the status message was set (for 5s auto-clear).
    status_message_set_at: Option<Instant>,
}

/// How long a status message is shown before reverting to "Open chat" on line 2.
const STATUS_MESSAGE_DURATION_SECS: u64 = 5;
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
        let status_message = None;
        let status_message_set_at = None;

        StatusBar {
            app_context,
            command_tx,
            name,
            terminal_area,
            last_key,
            focused,
            status_message,
            status_message_set_at,
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
            Action::Key(key, modifiers) => {
                self.last_key = Event::Key(key, modifiers.into());
                self.status_message = None;
                self.status_message_set_at = None;
            }
            Action::StatusMessage(msg) => {
                self.status_message = Some(msg);
                self.status_message_set_at = Some(Instant::now());
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> std::io::Result<()> {
        let selected_chat = self
            .app_context
            .tg_context()
            .name_of_open_chat_id()
            .unwrap_or_default();
        let mut spans: Vec<Span<'_>> = vec![
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
            Span::raw("     "),
            Span::styled(
                "Help: ",
                self.app_context.style_status_bar_message_quit_text(),
            ),
            Span::styled(
                "alt+F1",
                self.app_context.style_status_bar_message_quit_key(),
            ),
            Span::raw("     "),
            Span::styled(
                "Key pressed: ",
                self.app_context.style_status_bar_press_key_text(),
            ),
            Span::styled(
                self.last_key.to_string(),
                self.app_context.style_status_bar_press_key_key(),
            ),
            Span::raw("     "),
        ];
        spans.extend([
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
        ]);
        // Expire status message after 5 seconds so line 2 reverts to "Open chat".
        if let Some(set_at) = self.status_message_set_at {
            if set_at.elapsed() >= std::time::Duration::from_secs(STATUS_MESSAGE_DURATION_SECS) {
                self.status_message = None;
                self.status_message_set_at = None;
            }
        }
        // Line 1: key hints, key pressed, size. Line 2: voice playback (while playing) else status message (for 5s) else "Open chat" (default).
        let line1 = Line::from(spans);
        let line2_spans: Vec<Span<'_>> = {
            #[cfg(feature = "rodio")]
            let show_voice = {
                let state = self.app_context.voice_playback_state();
                state.is_playing && state.message_id.is_some()
            };
            #[cfg(not(feature = "rodio"))]
            let show_voice = false;
            if show_voice {
                #[cfg(feature = "rodio")]
                {
                    let state = self.app_context.voice_playback_state();
                    let m = state.position_secs / 60;
                    let s = state.position_secs % 60;
                    let dm = state.duration_secs / 60;
                    let ds = state.duration_secs % 60;
                    vec![Span::styled(
                        format!("V: {}:{:02}/{}:{:02}", m, s, dm, ds),
                        self.app_context.style_status_bar_size_info_numbers(),
                    )]
                }
                #[cfg(not(feature = "rodio"))]
                vec![]
            } else if let Some(ref msg) = self.status_message {
                vec![Span::styled(
                    msg.as_str(),
                    self.app_context.style_status_bar_message_quit_key(),
                )]
            } else {
                vec![
                    Span::styled(
                        "Open chat: ",
                        self.app_context.style_status_bar_open_chat_text(),
                    ),
                    Span::styled(
                        selected_chat,
                        self.app_context.style_status_bar_open_chat_name(),
                    ),
                ]
            }
        };
        let mut text = vec![line1];
        text.push(Line::from(line2_spans));

        let paragraph = Paragraph::new(text)
            .block(Block::new().title(self.name.as_str()).borders(Borders::ALL))
            .style(self.app_context.style_status_bar())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);

        Ok(())
    }
}
