use {
    crate::{
        app_context::AppContext,
        components::component::{Component, HandleFocus, HandleSmallArea},
        enums::action::Action,
    },
    ratatui::{
        layout::{Alignment, Rect},
        text::{Line, Span},
        widgets::{block::Block, Borders, Paragraph, Wrap},
    },
    std::{io, sync::Arc},
    tokio::sync::mpsc,
};

/// `TitleBar` is a struct that represents a title bar.
/// It is responsible for managing the layout and rendering of the title bar.
pub struct TitleBar {
    /// The application configuration.
    app_context: Arc<AppContext>,
    /// The name of the `TitleBar`.
    name: String,
    /// An unbounded sender that send action for processing.
    command_tx: Option<mpsc::UnboundedSender<Action>>,
    /// A flag indicating whether the `TitleBar` should be displayed as a
    /// smaller version of itself.
    small_area: bool,
    /// Indicates whether the `TitleBar` is focused or not.
    focused: bool,
}
/// Implementation of `TitleBar` struct.
impl TitleBar {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let command_tx = None;
        let name = "".to_string();
        let small_area = false;
        let focused = false;
        TitleBar {
            app_context,
            command_tx,
            name,
            small_area,
            focused,
        }
    }
    /// Set the name of the `TitleBar`.
    ///
    /// # Arguments
    /// * `name` - The name of the `TitleBar`.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `TitleBar`.
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }
}

/// Implement the `HandleFocus` trait for the `TitleBar` struct.
/// This trait allows the `TitleBar` to be focused or unfocused.
impl HandleFocus for TitleBar {
    /// Set the `focused` flag for the `TitleBar`.
    fn focus(&mut self) {
        self.focused = true;
    }
    /// Set the `focused` flag for the `TitleBar`.
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

/// Implement the `HandleSmallArea` trait for the `TitleBar` struct.
/// This trait allows the `TitleBar` to display a smaller version of itself if
/// necessary.
impl HandleSmallArea for TitleBar {
    /// Set the `small_area` flag for the `TitleBar`.
    ///
    /// # Arguments
    /// * `small_area` - A boolean flag indicating whether the `TitleBar` should
    ///   be displayed as a smaller version of itself.
    fn with_small_area(&mut self, small: bool) {
        self.small_area = small;
    }
}
/// Implement the `Component` trait for the `ChatListWindow` struct.
impl Component for TitleBar {
    fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> io::Result<()> {
        let name: Vec<char> = self.name.chars().collect::<Vec<char>>();
        // Span::raw(" - A TUI for Telegram"),
        let text = vec![Line::from(vec![
            Span::styled(
                name[0].to_string(),
                self.app_context.style_title_bar_title1(),
            ),
            Span::styled(
                name[1].to_string(),
                self.app_context.style_title_bar_title2(),
            ),
            Span::styled(
                name[2].to_string(),
                self.app_context.style_title_bar_title3(),
            ),
            Span::styled(" - ", self.app_context.style_title_bar_title1()),
            Span::styled("A", self.app_context.style_title_bar_title2()),
            Span::styled(" T", self.app_context.style_title_bar_title3()),
            Span::styled("U", self.app_context.style_title_bar_title1()),
            Span::styled("I", self.app_context.style_title_bar_title2()),
            Span::styled(" f", self.app_context.style_title_bar_title3()),
            Span::styled("o", self.app_context.style_title_bar_title1()),
            Span::styled("r", self.app_context.style_title_bar_title2()),
            Span::styled(" T", self.app_context.style_title_bar_title3()),
            Span::styled("e", self.app_context.style_title_bar_title1()),
            Span::styled("l", self.app_context.style_title_bar_title2()),
            Span::styled("e", self.app_context.style_title_bar_title3()),
            Span::styled("g", self.app_context.style_title_bar_title1()),
            Span::styled("r", self.app_context.style_title_bar_title2()),
            Span::styled("a", self.app_context.style_title_bar_title3()),
            Span::styled("m", self.app_context.style_title_bar_title1()),
        ])];
        let paragraph = Paragraph::new(text)
            .block(Block::new().borders(Borders::ALL))
            .style(self.app_context.style_title_bar())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);

        Ok(())
    }
}
