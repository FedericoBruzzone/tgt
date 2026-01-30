//! Server-side search overlay: query input and results list; jump to message on select.

use crate::{
    action::Action,
    app_context::AppContext,
    components::component_traits::{Component, HandleFocus},
    tg::message_entry::{DateTimeEntry, MessageEntry},
};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

/// Search overlay: server-side chat message search. Query input + results list; Enter on result jumps.
pub struct SearchOverlay {
    app_context: Arc<AppContext>,
    name: String,
    action_tx: Option<UnboundedSender<Action>>,
    focused: bool,
    visible: bool,
    /// Current query string (user types here).
    query: String,
    /// Search results from TDLib.
    results: Vec<MessageEntry>,
    list_state: ListState,
}

impl SearchOverlay {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        SearchOverlay {
            app_context,
            name: "".to_string(),
            action_tx: None,
            focused: false,
            visible: false,
            query: String::new(),
            results: Vec::new(),
            list_state: ListState::default(),
        }
    }

    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.results.clear();
        self.list_state.select(None);
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    fn submit_search(&self) {
        if self.query.is_empty() {
            return;
        }
        if let Some(tx) = self.action_tx.as_ref() {
            let _ = tx.send(Action::SearchChatMessages(self.query.clone()));
        }
    }

    fn select_previous(&mut self) {
        let len = self.results.len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            None => len.saturating_sub(1),
            Some(i) => i.saturating_sub(1),
        };
        self.list_state.select(Some(i));
    }

    fn select_next(&mut self) {
        let len = self.results.len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            None => 0,
            Some(i) => (i + 1).min(len.saturating_sub(1)),
        };
        self.list_state.select(Some(i));
    }

    fn confirm_selection(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if let Some(entry) = self.results.get(selected) {
                let message_id = entry.id();
                if let Some(tx) = self.action_tx.as_ref() {
                    let _ = tx.send(Action::JumpToMessage(message_id));
                    let _ = tx.send(Action::CloseSearchOverlay);
                }
            }
        }
    }
}

impl HandleFocus for SearchOverlay {
    fn focus(&mut self) {
        self.focused = true;
    }
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl Component for SearchOverlay {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> std::io::Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::ShowSearchOverlay => self.show(),
            Action::CloseSearchOverlay => self.hide(),
            Action::SearchResults(entries) => {
                self.results = entries;
                self.list_state.select(if self.results.is_empty() {
                    None
                } else {
                    Some(0)
                });
            }
            Action::Key(key_code, _modifiers) => {
                if !self.visible || !self.focused {
                    return;
                }
                match key_code {
                    KeyCode::Esc => {
                        if let Some(tx) = self.action_tx.as_ref() {
                            let _ = tx.send(Action::CloseSearchOverlay);
                        }
                    }
                    KeyCode::Enter => {
                        if self.results.is_empty() {
                            self.submit_search();
                        } else {
                            self.confirm_selection();
                        }
                    }
                    KeyCode::Up => self.select_previous(),
                    KeyCode::Down => self.select_next(),
                    KeyCode::Backspace => {
                        self.query.pop();
                    }
                    KeyCode::Char(c) => {
                        self.query.push(c);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> std::io::Result<()> {
        if !self.visible {
            return Ok(());
        }

        // Centered overlay
        let overlay_w = area.width.min(80);
        let overlay_h = area.height.min(24);
        let overlay_x = area.x + area.width.saturating_sub(overlay_w) / 2;
        let overlay_y = area.y + area.height.saturating_sub(overlay_h) / 2;
        let overlay_rect = Rect::new(overlay_x, overlay_y, overlay_w, overlay_h);

        // Clear background
        let clear = Clear;
        frame.render_widget(clear, overlay_rect);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(4),
            ])
            .split(overlay_rect);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Search messages (server) ")
            .style(self.app_context.style_chat());
        frame.render_widget(block, overlay_rect);

        let query_text = format!("{}_", self.query);
        let query_para = Paragraph::new(query_text)
            .block(Block::default().borders(Borders::ALL).title(" Query (Enter to search) "))
            .style(self.app_context.style_chat());
        frame.render_widget(query_para, chunks[0]);

        let items: Vec<ListItem<'_>> = self
            .results
            .iter()
            .map(|m| {
                let ts_str = DateTimeEntry::convert_time(m.timestamp().timestamp);
                let preview = m.message_content_to_string();
                let preview = if preview.chars().count() > 50 {
                    format!("{}…", preview.chars().take(47).collect::<String>())
                } else {
                    preview
                };
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("[{}] ", ts_str),
                        self.app_context.style_timestamp(),
                    ),
                    Span::raw(preview),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Results (Enter to jump) "))
            .style(self.app_context.style_chat())
            .highlight_style(self.app_context.style_item_selected())
            .repeat_highlight_symbol(true);
        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);

        Ok(())
    }
}
