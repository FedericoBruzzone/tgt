use crate::app_context::AppContext;
use chrono::{DateTime, Utc};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use std::time::{Duration, UNIX_EPOCH};
use tdlib::enums::{MessageContent, MessageSender};
use tdlib::types::FormattedText;

#[derive(Debug, Default, Clone)]
pub struct DateTimeEntry {
    pub timestamp: i32,
}
impl DateTimeEntry {
    pub fn convert_time(timestamp: i32) -> String {
        let d = UNIX_EPOCH + Duration::from_secs(timestamp as u64);
        let datetime = DateTime::<Utc>::from(d);
        datetime.format("%Y-%m-%d").to_string()
    }

    pub fn get_span_styled(&self, app_context: &AppContext) -> Span {
        Span::styled(
            Self::convert_time(self.timestamp),
            app_context.style_chat_list_item_message_timestamp(),
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct MessageEntry {
    _id: i64,
    sender_id: i64,
    content: Line<'static>,
    timestamp: DateTimeEntry,
}
impl MessageEntry {
    pub fn get_line_styled_with_only_content(&self, content_style: Style) -> Line<'static> {
        Line::from(Span::styled(format!("{}", self.content), content_style))
    }

    pub fn timestamp(&self) -> &DateTimeEntry {
        &self.timestamp
    }

    pub fn sender_id(&self) -> i64 {
        self.sender_id
    }

    pub fn get_text_styled(
        &self,
        app_context: &AppContext,
        name_style: Style,
        content_style: Style,
    ) -> Text {
        let mut entry = Text::default();
        entry.extend(vec![
            Line::from(vec![Span::styled(
                app_context
                    .tg_context()
                    .get_name_of_chat_id(self.sender_id)
                    .unwrap_or_default(),
                name_style,
            )]),
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
            tracing::info!("entities is empty {:?}", text);
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
            _id: message.id,
            sender_id: match &message.sender_id {
                MessageSender::User(user_id) => user_id.user_id,
                MessageSender::Chat(chat_id) => chat_id.chat_id,
            },
            content: Self::message_content_line(&message.content),
            timestamp: DateTimeEntry {
                timestamp: message.date,
            },
        }
    }
}
