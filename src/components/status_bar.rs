use {
    crate::{
        components::component::{Component, HandleFocus, HandleSmallArea},
        enums::{action::Action, event::Event},
    },
    ratatui::{
        layout::{Alignment, Rect},
        style::{Style, Stylize},
        text::{Line, Span},
        widgets::{block::Block, Borders, Paragraph, Wrap},
    },
    tokio::sync::mpsc::UnboundedSender,
};

/// `StatusBar` is a struct that represents a status bar.
/// It is responsible for managing the layout and rendering of the status bar.
pub struct StatusBar {
    /// The name of the `StatusBar`.
    name: String,
    /// An unbounded sender that send action for processing.
    command_tx: Option<UnboundedSender<Action>>,
    /// A flag indicating whether the `StatusBar` should be displayed as a
    /// smaller version of itself.
    small_area: bool,
    /// Indicates whether the `StatusBar` is focused or not.
    focused: bool,
    /// The area of the terminal where the all the content will be rendered.
    terminal_area: Rect,
    /// The last key pressed.
    last_key: Event,
}
/// Default implementation for `StatusBar`.
impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
/// Implementation of `StatusBar`.
impl StatusBar {
    /// Create a new instance of the `StatusBar` struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `StatusBar` struct.
    pub fn new() -> Self {
        let command_tx = None;
        let name = "".to_string();
        let small_area = false;
        let terminal_area = Rect::default();
        let last_key = Event::Unknown;
        let focused = false;

        StatusBar {
            command_tx,
            name,
            small_area,
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

/// Implement the `HandleSmallArea` trait for the `StatusBar` struct.
/// This trait allows the `StatusBar` to display a smaller version of itself if
/// necessary.
impl HandleSmallArea for StatusBar {
    /// Set the `small_area` flag for the `StatusBar`.
    ///
    /// # Arguments
    /// * `small_area` - A boolean flag indicating whether the `StatusBar`
    ///   should be displayed as a smaller version of itself.
    fn with_small_area(&mut self, small_area: bool) {
        self.small_area = small_area;
    }
}

/// Implement the `Component` trait for the `ChatListWindow` struct.
impl Component for StatusBar {
    fn register_action_handler(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> std::io::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::UpdateArea(area) => {
                self.terminal_area = area;
            }
            Action::Key(key, modifiers) => {
                self.last_key = Event::Key(key, modifiers);
            }
            _ => {}
        }
    }

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame<'_>,
        area: Rect,
    ) -> std::io::Result<()> {
        let text = vec![Line::from(vec![
            Span::raw("Press "),
            Span::styled("q ", Style::new().green().italic()),
            Span::raw("or "),
            Span::styled("ctrl+c ", Style::new().green().italic()),
            Span::raw("to quit"),
            //
            Span::raw("     "),
            Span::styled("Press key: ", Style::new().bold()),
            Span::styled(
                self.last_key.to_string(),
                Style::new().yellow().italic(),
            ),
            //
            Span::raw("     "),
            Span::styled("Size: ", Style::new().bold()),
            Span::styled(
                self.terminal_area.width.to_string(),
                Style::new().blue().italic(),
            ),
            Span::raw(" x "),
            Span::styled(
                self.terminal_area.height.to_string(),
                Style::new().blue().italic(),
            ),
        ])];

        let paragraph = Paragraph::new(text)
            .block(Block::new().title(self.name.as_str()).borders(Borders::ALL))
            .style(Style::new().white().on_black())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);

        Ok(())
    }
}
