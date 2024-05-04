use crate::app_context::AppContext;
use chrono::{DateTime, Utc};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use std::time::{Duration, UNIX_EPOCH};
use tdlib::enums::{MessageContent, MessageSender};
use tdlib::types::FormattedText;

use super::td_enums::TdMessageSender;

#[derive(Debug, Default, Clone)]
pub struct DateTimeEntry {
    pub timestamp: i32,
}
impl DateTimeEntry {
    pub fn convert_time(timestamp: i32) -> String {
        let d = UNIX_EPOCH + Duration::from_secs(timestamp as u64);
        let datetime = DateTime::<Utc>::from(d);
        if datetime.date_naive() == Utc::now().date_naive() {
            return datetime.format("Today %H:%M:%S").to_string();
        }
        if datetime.date_naive() == (Utc::now() - chrono::Duration::days(1)).date_naive() {
            return datetime.format("Yesterday %H:%M:%S").to_string();
        }
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn get_span_styled(&self, app_context: &AppContext) -> Span {
        Span::styled(
            Self::convert_time(self.timestamp),
            app_context.style_chat_list_item_message_timestamp(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct MessageEntry {
    id: i64,
    sender_id: TdMessageSender,
    message_content: Line<'static>,
    timestamp: DateTimeEntry,
}
impl MessageEntry {
    pub fn get_line_styled_with_only_content(&self, content_style: Style) -> Line<'static> {
        Line::from(Span::styled(
            format!("{}", self.message_content),
            content_style,
        ))
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn timestamp(&self) -> &DateTimeEntry {
        &self.timestamp
    }

    pub fn sender_id(&self) -> i64 {
        match self.sender_id {
            TdMessageSender::User(user_id) => user_id,
            TdMessageSender::Chat(chat_id) => chat_id,
        }
    }

    pub fn set_message_content(&mut self, content: &MessageContent) {
        self.message_content = Self::message_content_line(content);
    }

    pub fn get_text_styled(
        &self,
        app_context: &AppContext,
        name_style: Style,
        content_style: Style,
    ) -> Text {
        let mut entry = Text::default();
        entry.extend(vec![
            Line::from(vec![
                Span::styled(
                    match self.sender_id {
                        TdMessageSender::User(user_id) => app_context
                            .tg_context()
                            .try_name_from_chats_or_users(user_id)
                            .unwrap_or_default(),
                        TdMessageSender::Chat(chat_id) => app_context
                            .tg_context()
                            .name_from_chats(chat_id)
                            .unwrap_or_default(),
                    },
                    name_style,
                ),
                Span::raw(" "),
                self.timestamp.get_span_styled(app_context),
            ]),
            self.get_line_styled_with_only_content(content_style),
        ]);
        entry
    }

    fn message_content_line(content: &MessageContent) -> Line<'static> {
        match content {
            MessageContent::MessageText(m) => Self::format_message_content(&m.text),
            _ => Line::from(""),
        }
    }

    fn format_message_content(message: &FormattedText) -> Line<'static> {
        let text = &message.text;
        let entities = &message.entities;

        if entities.is_empty() {
            return Line::from(Span::raw(text.clone()));
        }

        let mut message_vec = Vec::new();
        entities.iter().for_each(|e| {
            let offset = e.offset as usize;
            let length = e.length as usize;
            message_vec.push(Span::raw(text.chars().take(offset).collect::<String>()));
            match &e.r#type {
                tdlib::enums::TextEntityType::Italic => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::ITALIC),
                    ));
                }
                tdlib::enums::TextEntityType::Bold => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ));
                }
                tdlib::enums::TextEntityType::Underline => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::UNDERLINED),
                    ));
                }
                _ => {}
            }
            message_vec.push(Span::raw(
                text.chars().skip(offset + length).collect::<String>(),
            ));
        });
        Line::from(message_vec)
    }
}
impl From<&tdlib::types::Message> for MessageEntry {
    fn from(message: &tdlib::types::Message) -> Self {
        Self {
            id: message.id,
            sender_id: match &message.sender_id {
                MessageSender::User(user) => TdMessageSender::User(user.user_id),
                MessageSender::Chat(chat) => TdMessageSender::Chat(chat.chat_id),
            },
            message_content: Self::message_content_line(&message.content),
            timestamp: DateTimeEntry {
                timestamp: message.date,
            },
        }
    }
}
