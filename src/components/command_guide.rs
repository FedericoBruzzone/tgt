use crate::{
    action::Action,
    app_context::AppContext,
    components::component_traits::{Component, HandleFocus},
    configs::custom::keymap_custom::ActionBinding,
    event::Event,
};
use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::{collections::HashMap, io, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

/// `CommandGuide` is a struct that represents a popup window displaying keybindings.
/// It shows all available keybindings organized by component and explains configuration options.
pub struct CommandGuide {
    /// The application context.
    app_context: Arc<AppContext>,
    /// The name of the `CommandGuide`.
    name: String,
    /// An unbounded sender that send action for processing.
    action_tx: Option<UnboundedSender<Action>>,
    /// Indicates whether the `CommandGuide` is focused or not.
    focused: bool,
    /// Indicates whether the command guide should be shown.
    visible: bool,
}

impl CommandGuide {
    /// Create a new instance of the `CommandGuide` struct.
    ///
    /// # Arguments
    /// * `app_context` - An Arc wrapped AppContext struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `CommandGuide` struct.
    pub fn new(app_context: Arc<AppContext>) -> Self {
        CommandGuide {
            app_context,
            name: "".to_string(),
            action_tx: None,
            focused: false,
            visible: false,
        }
    }

    /// Set the name of the `CommandGuide`.
    ///
    /// # Arguments
    /// * `name` - The name of the `CommandGuide`.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `CommandGuide`.
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }

    /// Show the command guide.
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the command guide.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if the command guide is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Format a keybinding for display.
    fn format_keybinding(event: &Event, description: Option<&String>) -> String {
        let key_str = format!("{}", event);
        let desc_str = description.map(|d| format!(" - {}", d)).unwrap_or_default();
        format!("{:<20}{}", key_str, desc_str)
    }

    /// Get keybindings for a component section.
    fn get_keybindings_section(
        &self,
        title: &str,
        keymap: &HashMap<Event, ActionBinding>,
    ) -> Vec<String> {
        let mut bindings = Vec::new();
        bindings.push(format!("[{}]", title));
        bindings.push(String::new());

        let mut sorted_bindings: Vec<(&Event, &ActionBinding)> = keymap.iter().collect();
        sorted_bindings.sort_by(|a, b| format!("{}", a.0).cmp(&format!("{}", b.0)));

        for (event, binding) in sorted_bindings {
            match binding {
                ActionBinding::Single { description, .. } => {
                    bindings.push(Self::format_keybinding(event, description.as_ref()));
                }
                ActionBinding::Multiple(_) => {
                    // For multiple key bindings, just show the first key
                    bindings.push(Self::format_keybinding(
                        event,
                        Some(&"Multiple keys...".to_string()),
                    ));
                }
            }
        }

        bindings.push(String::new());
        bindings
    }

    /// Build the help text content.
    fn build_help_text(&self) -> Text<'_> {
        let keymap_config = self.app_context.keymap_config();
        let mut lines = Vec::new();

        // Title
        lines.push(Line::from(vec![Span::styled(
            "Command Guide - Keybindings",
            self.app_context.style_title_bar(),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Press Alt+F1 or Esc to close",
            self.app_context.style_timestamp(),
        )]));
        lines.push(Line::from(""));

        // Core Window Keybindings
        let core_window_lines = self.get_keybindings_section(
            "Core Window (Available everywhere)",
            &keymap_config.core_window,
        );
        for line in core_window_lines {
            lines.push(Line::from(line));
        }

        // Chat List Keybindings
        let chat_list_lines = self.get_keybindings_section("Chat List", &keymap_config.chat_list);
        for line in chat_list_lines {
            lines.push(Line::from(line));
        }

        // Chat Keybindings
        let chat_lines = self.get_keybindings_section("Chat Window", &keymap_config.chat);
        for line in chat_lines {
            lines.push(Line::from(line));
        }

        // Prompt Keybindings
        let prompt_lines = self.get_keybindings_section("Prompt", &keymap_config.prompt);
        for line in prompt_lines {
            lines.push(Line::from(line));
        }

        // Configuration section
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Configuration",
            self.app_context.style_title_bar(),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from("Keybindings can be customized by editing:"));
        lines.push(Line::from(vec![Span::styled(
            "  config/keymap.toml",
            self.app_context.style_chat(),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from("Format:"));
        lines.push(Line::from("  [component_name]"));
        lines.push(Line::from("  keymap = ["));
        lines.push(Line::from(
            "    { keys = [\"key\"], command = \"action\", description = \"...\" },",
        ));
        lines.push(Line::from("  ]"));
        lines.push(Line::from(""));
        lines.push(Line::from(
            "Available components: core_window, chat_list, chat, prompt",
        ));
        lines.push(Line::from(""));
        lines.push(Line::from("Example:"));
        lines.push(Line::from(vec![Span::styled(
            "  [core_window]",
            self.app_context.style_chat(),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "  keymap = [",
            self.app_context.style_chat(),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "    { keys = [\"q\"], command = \"try_quit\", description = \"Quit\" },",
            self.app_context.style_chat(),
        )]));
        lines.push(Line::from(vec![Span::styled(
            "  ]",
            self.app_context.style_chat(),
        )]));

        Text::from(lines)
    }
}

impl HandleFocus for CommandGuide {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl Component for CommandGuide {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> io::Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::ShowCommandGuide => {
                self.show();
            }
            Action::HideCommandGuide => {
                self.hide();
            }
            Action::Key(key_code, _modifiers) => {
                if self.visible {
                    match key_code {
                        crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::F(1) => {
                            self.hide();
                            // Send HideCommandGuide action to CoreWindow
                            if let Some(tx) = self.action_tx.as_ref() {
                                tx.send(Action::HideCommandGuide).unwrap_or(());
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> io::Result<()> {
        if !self.visible {
            return Ok(());
        }

        // Calculate popup size (80% of screen, centered)
        let popup_width = (area.width as f32 * 0.8) as u16;
        let popup_height = (area.height as f32 * 0.8) as u16;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect::new(
            area.x + popup_x,
            area.y + popup_y,
            popup_width,
            popup_height,
        );

        // Clear the popup area first
        frame.render_widget(Clear, popup_area);

        let block = Block::new()
            .borders(Borders::ALL)
            .title("Command Guide")
            .title_alignment(Alignment::Center)
            .border_style(self.app_context.style_border_component_focused())
            .style(self.app_context.style_chat());

        // Create inner area for content
        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Build and render help text
        let help_text = self.build_help_text();
        let paragraph = Paragraph::new(help_text)
            .wrap(Wrap { trim: true })
            .style(self.app_context.style_chat());

        frame.render_widget(paragraph, inner_area);

        Ok(())
    }
}
