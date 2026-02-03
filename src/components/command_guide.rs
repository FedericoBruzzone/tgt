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
    /// Vertical scroll offset (number of lines scrolled).
    scroll_offset: u16,
    /// Last inner area height (from previous draw) for clamping scroll in update.
    last_inner_height: u16,
    /// Last content line count (from previous draw) for max scroll.
    last_content_lines: usize,
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
            scroll_offset: 0,
            last_inner_height: 0,
            last_content_lines: 0,
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
        self.scroll_offset = 0;
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
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "Command Guide - Keybindings",
                self.app_context.style_title_bar(),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Scroll: ↑/↓ PgUp/PgDn  |  Close: Esc or Alt+F1",
                self.app_context.style_timestamp(),
            )]),
            Line::from(""),
        ];

        // None state (global) — keybindings available everywhere
        let core_window_lines =
            self.get_keybindings_section("None state (global)", &keymap_config.core_window);
        for line in core_window_lines {
            lines.push(Line::from(line));
        }

        // Chat List
        let chat_list_lines = self.get_keybindings_section("Chat List", &keymap_config.chat_list);
        for line in chat_list_lines {
            lines.push(Line::from(line));
        }

        // Chat — copy (y/Ctrl+C), edit (e), reply (r), delete (d/D)
        let chat_lines = self.get_keybindings_section("Chat", &keymap_config.chat);
        for line in chat_lines {
            lines.push(Line::from(line));
        }

        // Prompt
        let prompt_lines = self.get_keybindings_section("Prompt", &keymap_config.prompt);
        for line in prompt_lines {
            lines.push(Line::from(line));
        }

        // Mouse
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Mouse",
            self.app_context.style_title_bar(),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(
            "  Scroll: chat list / chat to move selection or messages.",
        ));
        lines.push(Line::from(
            "  Chat list: first click focuses list, second click opens selected chat.",
        ));
        lines.push(Line::from(""));

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
                        crossterm::event::KeyCode::Up => {
                            self.scroll_offset = self.scroll_offset.saturating_sub(1);
                        }
                        crossterm::event::KeyCode::Down => {
                            let max_scroll =
                                self.last_content_lines
                                    .saturating_sub(self.last_inner_height as usize)
                                    .min(u16::MAX as usize) as u16;
                            self.scroll_offset = (self.scroll_offset + 1).min(max_scroll);
                        }
                        crossterm::event::KeyCode::PageUp => {
                            let page = self.last_inner_height;
                            self.scroll_offset = self.scroll_offset.saturating_sub(page);
                        }
                        crossterm::event::KeyCode::PageDown => {
                            let max_scroll =
                                self.last_content_lines
                                    .saturating_sub(self.last_inner_height as usize)
                                    .min(u16::MAX as usize) as u16;
                            let page = self.last_inner_height;
                            self.scroll_offset = (self.scroll_offset + page).min(max_scroll);
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

        // Build help text and update scroll state (build once for line count to avoid borrow conflict)
        let content_lines = self.build_help_text().lines.len();
        self.last_content_lines = content_lines;
        self.last_inner_height = inner_area.height;

        // Clamp scroll offset to valid range
        let max_scroll = content_lines
            .saturating_sub(inner_area.height as usize)
            .min(u16::MAX as usize) as u16;
        self.scroll_offset = self.scroll_offset.min(max_scroll);

        let help_text = self.build_help_text();
        let paragraph = Paragraph::new(help_text)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_offset, 0))
            .style(self.app_context.style_chat());

        frame.render_widget(paragraph, inner_area);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        action::{Action, Modifiers},
        app_context::AppContext,
        cli::CliArgs,
        configs::custom::{
            app_custom::AppConfig, keymap_custom::KeymapConfig, palette_custom::PaletteConfig,
            telegram_custom::TelegramConfig, theme_custom::ThemeConfig,
        },
        tg::tg_context::TgContext,
    };
    use clap::Parser;
    use crossterm::event::{KeyCode, KeyModifiers};
    use std::sync::Arc;
    use tokio::sync::mpsc;

    /// Helper function to create a test AppContext
    fn create_test_app_context() -> Arc<AppContext> {
        let app_config = AppConfig::default();
        let keymap_config = KeymapConfig::default();
        let theme_config = ThemeConfig::default();
        let palette_config = PaletteConfig::default();
        let telegram_config = TelegramConfig::default();
        let tg_context = TgContext::default();
        // Create CliArgs by parsing empty args (default behavior)
        let cli_args = CliArgs::parse_from::<[&str; 0], &str>([]);

        Arc::new(
            AppContext::new(
                app_config,
                keymap_config,
                theme_config,
                palette_config,
                telegram_config,
                tg_context,
                cli_args,
            )
            .unwrap(),
        )
    }

    #[test]
    fn test_command_guide_initial_state() {
        let app_context = create_test_app_context();
        let guide = CommandGuide::new(app_context);

        assert!(
            !guide.is_visible(),
            "Command guide should not be visible initially"
        );
        assert!(
            !guide.focused,
            "Command guide should not be focused initially"
        );
    }

    #[test]
    fn test_command_guide_show() {
        let app_context = create_test_app_context();
        let mut guide = CommandGuide::new(app_context);

        guide.show();
        assert!(
            guide.is_visible(),
            "Command guide should be visible after show()"
        );
    }

    #[test]
    fn test_command_guide_hide() {
        let app_context = create_test_app_context();
        let mut guide = CommandGuide::new(app_context);

        guide.show();
        assert!(
            guide.is_visible(),
            "Command guide should be visible after show()"
        );

        guide.hide();
        assert!(
            !guide.is_visible(),
            "Command guide should not be visible after hide()"
        );
    }

    #[test]
    fn test_command_guide_toggle() {
        let app_context = create_test_app_context();
        let mut guide = CommandGuide::new(app_context);

        // Initially hidden
        assert!(!guide.is_visible());

        // Show
        guide.show();
        assert!(guide.is_visible());

        // Hide
        guide.hide();
        assert!(!guide.is_visible());

        // Show again
        guide.show();
        assert!(guide.is_visible());
    }

    #[test]
    fn test_command_guide_show_action() {
        let app_context = create_test_app_context();
        let mut guide = CommandGuide::new(app_context);

        assert!(!guide.is_visible());

        guide.update(Action::ShowCommandGuide);
        assert!(
            guide.is_visible(),
            "Command guide should be visible after ShowCommandGuide action"
        );
    }

    #[test]
    fn test_command_guide_hide_action() {
        let app_context = create_test_app_context();
        let mut guide = CommandGuide::new(app_context);

        guide.show();
        assert!(guide.is_visible());

        guide.update(Action::HideCommandGuide);
        assert!(
            !guide.is_visible(),
            "Command guide should not be visible after HideCommandGuide action"
        );
    }

    #[test]
    fn test_command_guide_close_with_esc_when_visible() {
        let app_context = create_test_app_context();
        let mut guide = CommandGuide::new(app_context);

        guide.show();
        assert!(guide.is_visible(), "Guide should be visible before HideCommandGuide");

        guide.update(Action::HideCommandGuide);
        assert!(
            !guide.is_visible(),
            "Command guide should be hidden after HideCommandGuide action when visible"
        );
    }

    #[test]
    fn test_command_guide_close_with_f1_when_visible() {
        let app_context = create_test_app_context();
        let mut guide = CommandGuide::new(app_context);

        guide.show();
        assert!(guide.is_visible(), "Guide should be visible before HideCommandGuide");

        guide.update(Action::HideCommandGuide);
        assert!(
            !guide.is_visible(),
            "Command guide should be hidden after HideCommandGuide action when visible"
        );
    }

    #[test]
    fn test_command_guide_ignores_keys_when_hidden() {
        let app_context = create_test_app_context();
        let mut guide = CommandGuide::new(app_context);

        assert!(!guide.is_visible());

        // Send various keys when hidden - should not affect visibility
        let modifiers = Modifiers::from(KeyModifiers::empty());
        guide.update(Action::Key(KeyCode::Esc, modifiers.clone()));
        assert!(
            !guide.is_visible(),
            "Esc should not affect visibility when hidden"
        );

        guide.update(Action::Key(KeyCode::F(1), modifiers.clone()));
        assert!(
            !guide.is_visible(),
            "F1 should not affect visibility when hidden"
        );

        guide.update(Action::Key(KeyCode::Char('a'), modifiers));
        assert!(
            !guide.is_visible(),
            "Other keys should not affect visibility when hidden"
        );
    }

    #[test]
    fn test_command_guide_ignores_other_keys_when_visible() {
        let app_context = create_test_app_context();
        let mut guide = CommandGuide::new(app_context);

        guide.show();
        assert!(guide.is_visible());

        // Other keys should not close the guide
        let modifiers = Modifiers::from(KeyModifiers::empty());
        guide.update(Action::Key(KeyCode::Char('a'), modifiers.clone()));
        assert!(guide.is_visible(), "Other keys should not close the guide");

        guide.update(Action::Key(KeyCode::Enter, modifiers));
        assert!(guide.is_visible(), "Enter should not close the guide");
    }

    #[test]
    fn test_command_guide_register_action_handler() {
        let app_context = create_test_app_context();
        let mut guide = CommandGuide::new(app_context);
        let (tx, _rx) = mpsc::unbounded_channel();

        let result = guide.register_action_handler(tx);
        assert!(result.is_ok(), "register_action_handler should succeed");
        assert!(
            guide.action_tx.is_some(),
            "action_tx should be set after registration"
        );
    }

    #[test]
    fn test_command_guide_with_name() {
        let app_context = create_test_app_context();
        let guide = CommandGuide::new(app_context).with_name("Test Guide");

        assert_eq!(guide.name, "Test Guide");
    }

    #[test]
    fn test_command_guide_focus_unfocus() {
        let app_context = create_test_app_context();
        let mut guide = CommandGuide::new(app_context);

        assert!(!guide.focused);

        guide.focus();
        assert!(guide.focused, "Guide should be focused after focus()");

        guide.unfocus();
        assert!(
            !guide.focused,
            "Guide should not be focused after unfocus()"
        );
    }
}
