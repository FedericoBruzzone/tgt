use crate::{
    action::Action,
    app_context::AppContext,
    components::component_traits::{Component, HandleFocus},
    tg::message_entry::MessageEntry,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use std::io;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

/// Modal popup listing pinned messages for the open chat (newest first).
pub struct PinnedMessagesPopup {
    app_context: Arc<AppContext>,
    name: String,
    action_tx: Option<UnboundedSender<Action>>,
    focused: bool,
    visible: bool,
    messages: Vec<MessageEntry>,
    index: usize,
}

impl PinnedMessagesPopup {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        PinnedMessagesPopup {
            app_context,
            name: String::new(),
            action_tx: None,
            focused: false,
            visible: false,
            messages: Vec::new(),
            index: 0,
        }
    }

    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }

    fn reload_from_context(&mut self) {
        self.messages = self.app_context.tg_context().open_chat_pinned_snapshot();
        self.index = self.index.min(self.messages.len().saturating_sub(1));
        self.visible = !self.messages.is_empty();
    }

    fn go_previous(&mut self) {
        if self.messages.is_empty() {
            return;
        }
        let max = self.messages.len() - 1;
        self.index = (self.index + 1).min(max);
    }

    fn go_next(&mut self) {
        self.index = self.index.saturating_sub(1);
    }

    fn current_message_id(&self) -> Option<i64> {
        self.messages.get(self.index).map(MessageEntry::id)
    }

    /// Photo/voice handlers use `get_message` on the open-chat cache; pinned-only entries may be absent.
    fn ensure_current_in_open_cache(&self) {
        let Some(entry) = self.messages.get(self.index) else {
            return;
        };
        let tg = self.app_context.tg_context();
        let mut store = tg.open_chat_messages();
        if store.get_message(entry.id()).is_none() {
            store.insert_messages(std::iter::once(entry.clone()));
        }
    }
}

impl HandleFocus for PinnedMessagesPopup {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl Component for PinnedMessagesPopup {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> io::Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::ShowPinnedMessagesPopup => {
                self.reload_from_context();
            }
            Action::HidePinnedMessagesPopup => {
                self.visible = false;
            }
            Action::PinnedPopupPrevious => self.go_previous(),
            Action::PinnedPopupNext => self.go_next(),
            Action::PinnedPopupConfirmJump => {
                if let (Some(id), Some(tx)) = (self.current_message_id(), self.action_tx.as_ref()) {
                    let _ = tx.send(Action::JumpToMessage(id));
                    let _ = tx.send(Action::HidePinnedMessagesPopup);
                }
            }
            Action::PinnedPopupViewPhoto => {
                self.ensure_current_in_open_cache();
                if let (Some(id), Some(tx)) = (self.current_message_id(), self.action_tx.as_ref()) {
                    let _ = tx.send(Action::ViewPhotoMessage(id));
                }
            }
            Action::PinnedPopupPlayVoice => {
                self.ensure_current_in_open_cache();
                if let (Some(id), Some(tx)) = (self.current_message_id(), self.action_tx.as_ref()) {
                    let _ = tx.send(Action::PlayVoiceMessage(id));
                }
            }
            Action::LoadPinnedMessages => {
                if self.visible {
                    self.reload_from_context();
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> io::Result<()> {
        if !self.visible || self.messages.is_empty() {
            return Ok(());
        }

        let popup_width = (area.width as f32 * 0.85).max(20.0).min(area.width as f32) as u16;
        let popup_height = (area.height as f32 * 0.7).max(8.0).min(area.height as f32) as u16;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(
            area.x + popup_x,
            area.y + popup_y,
            popup_width,
            popup_height,
        );

        frame.render_widget(Clear, popup_area);

        let title = format!("{}/{}", self.index + 1, self.messages.len());
        let block = Block::new()
            .borders(Borders::ALL)
            .title(title)
            .title_alignment(Alignment::Center)
            .border_style(self.app_context.style_border_component_focused())
            .style(self.app_context.style_chat());

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let entry = &self.messages[self.index];
        let myself = entry.sender_id() == self.app_context.tg_context().me();
        let wrap_width = inner.width.saturating_sub(2) as i32;
        let (name_style, content_style, alignment) = if myself {
            (
                self.app_context.style_chat_message_myself_name(),
                self.app_context.style_chat_message_myself_content(),
                Alignment::Right,
            )
        } else {
            (
                self.app_context.style_chat_message_other_name(),
                self.app_context.style_chat_message_other_content(),
                Alignment::Left,
            )
        };
        let text = entry.get_text_styled(
            myself,
            &self.app_context,
            true,
            name_style,
            content_style,
            wrap_width,
        );
        let body_style: Style = self.app_context.style_chat();
        let paragraph = Paragraph::new(text)
            .alignment(alignment)
            .style(body_style)
            .wrap(Wrap { trim: true });

        let footer_h = 2u16;
        let header_h = 1u16;
        let main = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_h),
                Constraint::Min(1),
                Constraint::Length(footer_h),
            ])
            .split(inner);

        let header_para = Paragraph::new(Line::from(vec![Span::styled(
            "Pinned messages:",
            self.app_context.style_chat_chat_name(),
        )]))
        .style(self.app_context.style_chat());

        frame.render_widget(header_para, main[0]);
        frame.render_widget(paragraph, main[1]);

        let footer_area = main[2];

        let hint = Line::from(vec![Span::styled(
            "↑/k older  ↓/j newer  Enter jump  Alt+V photo  Alt+P audio  Esc close",
            self.app_context.style_timestamp(),
        )]);
        frame.render_widget(
            Paragraph::new(hint).alignment(Alignment::Center),
            footer_area,
        );

        Ok(())
    }
}
