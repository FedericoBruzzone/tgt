use crate::{
    action::Action,
    app_context::AppContext,
    components::component_traits::{Component, HandleFocus},
    theme_switcher::ThemeSwitcher,
};
use ratatui::{
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame,
};
use std::{io, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

/// `ThemeSelector` is a struct that represents a popup window for selecting themes.
/// It displays available themes and allows cycling through them.
pub struct ThemeSelector {
    /// The application context.
    app_context: Arc<AppContext>,
    /// The name of the `ThemeSelector`.
    name: String,
    /// An unbounded sender that send action for processing.
    action_tx: Option<UnboundedSender<Action>>,
    /// Indicates whether the `ThemeSelector` is focused or not.
    focused: bool,
    /// Indicates whether the theme selector should be shown.
    visible: bool,
    /// The theme switcher instance.
    theme_switcher: ThemeSwitcher,
    /// The list state for tracking selected theme.
    list_state: ListState,
}

impl ThemeSelector {
    /// Create a new instance of the `ThemeSelector` struct.
    ///
    /// # Arguments
    /// * `app_context` - An Arc wrapped AppContext struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `ThemeSelector` struct.
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let mut theme_switcher = ThemeSwitcher::new();

        // Initialize theme switcher to match current theme
        let current_theme_filename = app_context.app_config().theme_filename.clone();
        if let Some(theme_name) = current_theme_filename.strip_suffix(".toml") {
            // Remove "themes/" prefix if present
            let theme_name = theme_name.strip_prefix("themes/").unwrap_or(theme_name);
            if theme_switcher.switch_to_theme(theme_name).is_err() {
                tracing::warn!(
                    "Current theme '{}' not found in available themes, using default",
                    theme_name
                );
            }
        }

        let mut list_state = ListState::default();
        if let Some(current_index) = theme_switcher.current_theme().and_then(|name| {
            theme_switcher
                .available_themes()
                .iter()
                .position(|t| t == name)
        }) {
            list_state.select(Some(current_index));
        }

        ThemeSelector {
            app_context,
            name: "".to_string(),
            action_tx: None,
            focused: false,
            visible: false,
            theme_switcher,
            list_state,
        }
    }

    /// Set the name of the `ThemeSelector`.
    ///
    /// # Arguments
    /// * `name` - The name of the `ThemeSelector`.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `ThemeSelector`.
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }

    /// Show the theme selector.
    pub fn show(&mut self) {
        self.visible = true;
        // Update selection to match current theme
        if let Some(current_index) = self.theme_switcher.current_theme().and_then(|name| {
            self.theme_switcher
                .available_themes()
                .iter()
                .position(|t| t == name)
        }) {
            self.list_state.select(Some(current_index));
        }
    }

    /// Hide the theme selector.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if the theme selector is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Move selection up.
    fn select_previous(&mut self) {
        let themes = self.theme_switcher.available_themes();
        if themes.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new_index = if current == 0 {
            themes.len() - 1
        } else {
            current - 1
        };
        self.list_state.select(Some(new_index));

        // Apply the selected theme
        if let Some(theme_name) = themes.get(new_index) {
            let theme_name = theme_name.clone();
            if self.theme_switcher.switch_to_theme(&theme_name).is_ok()
                && self
                    .theme_switcher
                    .apply_current_theme(&self.app_context)
                    .is_ok()
            {
                self.app_context.mark_dirty();
            }
        }
    }

    /// Move selection down.
    fn select_next(&mut self) {
        let themes = self.theme_switcher.available_themes();
        if themes.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let new_index = (current + 1) % themes.len();
        self.list_state.select(Some(new_index));

        // Apply the selected theme
        if let Some(theme_name) = themes.get(new_index) {
            let theme_name = theme_name.clone();
            if self.theme_switcher.switch_to_theme(&theme_name).is_ok()
                && self
                    .theme_switcher
                    .apply_current_theme(&self.app_context)
                    .is_ok()
            {
                self.app_context.mark_dirty();
            }
        }
    }
}

impl HandleFocus for ThemeSelector {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl Component for ThemeSelector {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> io::Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::ShowThemeSelector => {
                self.show();
            }
            Action::HideThemeSelector => {
                self.hide();
            }
            Action::Key(key_code, _modifiers) => {
                if self.visible {
                    match key_code {
                        crossterm::event::KeyCode::Esc => {
                            self.hide();
                            // Send HideThemeSelector action to CoreWindow
                            if let Some(tx) = self.action_tx.as_ref() {
                                tx.send(Action::HideThemeSelector).unwrap_or(());
                            }
                        }
                        crossterm::event::KeyCode::Up => {
                            self.select_previous();
                        }
                        crossterm::event::KeyCode::Down => {
                            self.select_next();
                        }
                        crossterm::event::KeyCode::Enter => {
                            // Apply current selection and close
                            self.hide();
                            if let Some(tx) = self.action_tx.as_ref() {
                                tx.send(Action::HideThemeSelector).unwrap_or(());
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

        // Calculate popup size (60% width, 50% height, centered)
        let popup_width = (area.width as f32 * 0.6) as u16;
        let popup_height = (area.height as f32 * 0.5) as u16;
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

        let themes = self.theme_switcher.available_themes();
        let current_theme = self.theme_switcher.current_theme().unwrap_or("unknown");

        let block = Block::new()
            .borders(Borders::ALL)
            .title(format!("Theme Selector - Current: {}", current_theme))
            .title_alignment(Alignment::Center)
            .border_style(self.app_context.style_border_component_focused())
            .style(self.app_context.style_chat());

        // Create inner area for content
        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Build theme list items
        let items: Vec<ListItem> = themes
            .iter()
            .enumerate()
            .map(|(idx, theme)| {
                let is_selected = self.list_state.selected() == Some(idx);
                let is_current = theme == current_theme;

                let mut spans = vec![];
                if is_current {
                    spans.push(Span::styled("● ", self.app_context.style_item_selected()));
                } else {
                    spans.push(Span::raw("  "));
                }

                let style = if is_selected {
                    self.app_context.style_item_selected()
                } else {
                    self.app_context.style_chat()
                };

                spans.push(Span::styled(theme.clone(), style));

                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items)
            .style(self.app_context.style_chat())
            .highlight_style(self.app_context.style_item_selected());

        frame.render_stateful_widget(list, inner_area, &mut self.list_state);

        // Draw instructions at the bottom
        let instructions_area = Rect::new(
            inner_area.x,
            inner_area.y + inner_area.height.saturating_sub(3),
            inner_area.width,
            3,
        );

        let instructions = vec![Line::from(vec![Span::styled(
            "↑/↓: Navigate  Enter: Apply  Esc: Cancel",
            self.app_context.style_timestamp(),
        )])];

        let instructions_paragraph =
            ratatui::widgets::Paragraph::new(instructions).style(self.app_context.style_chat());
        frame.render_widget(instructions_paragraph, instructions_area);

        Ok(())
    }
}
