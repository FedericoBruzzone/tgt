use {
    crate::{
        components::component::{Component, HandleFocus, HandleSmallArea},
        configs::config_theme::{style_border_component_focused, style_prompt},
        enums::action::Action,
    },
    ratatui::{
        layout::Rect,
        symbols::{
            border::{Set, PLAIN},
            line::NORMAL,
        },
        widgets::{block::Block, Borders},
    },
    tokio::sync::mpsc::UnboundedSender,
};

/// `PromptWindow` is a struct that represents a window for displaying a prompt.
/// It is responsible for managing the layout and rendering of the prompt
/// window.
pub struct PromptWindow {
    /// The name of the `PromptWindow`.
    name: String,
    /// An unbounded sender that send action for processing.
    command_tx: Option<UnboundedSender<Action>>,
    /// A flag indicating whether the `PromptWindow` should be displayed as a
    /// smaller version of itself.
    small_area: bool,
    /// Indicates whether the `PromptWindow` is focused or not.
    focused: bool,
}

impl Default for PromptWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptWindow {
    /// Create a new instance of the `PromptWindow` struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `PromptWindow` struct.
    pub fn new() -> Self {
        let name = "".to_string();
        let command_tx = None;
        let small_area = false;
        let focused = false;

        PromptWindow {
            name,
            command_tx,
            small_area,
            focused,
        }
    }
    /// Set the name of the `PromptWindow`.
    ///
    /// # Arguments
    /// * `name` - The name of the `PromptWindow`
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `PromptWindow`.
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }
}

/// Implement the `HandleFocus` trait for the `PromptWindow` struct.
/// This trait allows the `PromptWindow` to be focused or unfocused.
impl HandleFocus for PromptWindow {
    /// Set the `focused` flag for the `PromptWindow`.
    fn focus(&mut self) {
        self.focused = true;
    }
    /// Set the `focused` flag for the `PromptWindow`.
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

/// Implement the `HandleSmallArea` trait for the `PromptWindow` struct.
/// This trait allows the `PromptWindow` to display a smaller version of itself
/// if necessary.
impl HandleSmallArea for PromptWindow {
    /// Set the `small_area` flag for the `PromptWindow`.
    ///
    /// # Arguments
    /// * `small_area` - A boolean flag indicating whether the `PromptWindow`
    ///   should be displayed as a smaller version of itself.
    fn with_small_area(&mut self, small_area: bool) {
        self.small_area = small_area;
    }
}

/// Implement the `Component` trait for the `ChatListWindow` struct.
impl Component for PromptWindow {
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
        let collapsed_top_and_left_border_set = Set {
            top_left: NORMAL.vertical_right,
            top_right: NORMAL.vertical_left,
            bottom_left: if self.small_area {
                NORMAL.bottom_left
            } else {
                NORMAL.horizontal_up
            },
            ..PLAIN
        };
        let style_border_focused = if self.focused {
            style_border_component_focused()
        } else {
            style_prompt()
        };

        let block = Block::new()
            .border_set(collapsed_top_and_left_border_set)
            .border_style(style_border_focused)
            .borders(Borders::ALL)
            .style(style_prompt())
            .title(self.name.as_str());

        frame.render_widget(block, area);

        Ok(())
    }
}
