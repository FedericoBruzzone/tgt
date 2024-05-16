use crate::app_context::AppContext;
use chrono::{DateTime, Local};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use std::time::{Duration, UNIX_EPOCH};
use tdlib::enums::{MessageContent, MessageReplyTo, MessageSender};
use tdlib::types::FormattedText;

use super::td_enums::{TdMessageReplyTo, TdMessageSender};

#[derive(Debug, Default, Clone)]
pub struct DateTimeEntry {
    pub timestamp: i32,
}
impl DateTimeEntry {
    pub fn convert_time(timestamp: i32) -> String {
        let d = UNIX_EPOCH + Duration::from_secs(timestamp as u64);
        let datetime = DateTime::<Local>::from(d);
        if datetime.date_naive() == Local::now().date_naive() {
            return datetime.format("%H:%M").to_string();
        }
        if datetime.date_naive() == (Local::now() - chrono::Duration::days(1)).date_naive() {
            return datetime.format("Yesterday %H:%M").to_string();
        }
        datetime.format("%Y-%m-%d %H:%M").to_string() // :%S
    }

    pub fn get_span_styled(&self, app_context: &AppContext) -> Span {
        Span::styled(
            Self::convert_time(self.timestamp),
            app_context.style_timestamp(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct MessageEntry {
    id: i64,
    sender_id: TdMessageSender,
    message_content: Vec<Line<'static>>,
    reply_to: Option<TdMessageReplyTo>,
    timestamp: DateTimeEntry,
    is_edited: bool,
}

impl MessageEntry {
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

    pub fn message_content_to_string(&self) -> String {
        self.message_content
            .iter()
            .map(|l| l.iter().map(|s| s.content.clone()).collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn set_message_content(&mut self, content: &MessageContent) {
        self.message_content = Self::message_content_lines(content);
    }

    pub fn set_is_edited(&mut self, is_edited: bool) {
        self.is_edited = is_edited;
    }

    pub fn get_text_styled(
        &self,
        app_context: &AppContext,
        is_unread: Option<bool>,
        name_style: Style,
        content_style: Style,
    ) -> Text {
        let reply_text = match &self.reply_to {
            Some(reply_to) => match reply_to {
                TdMessageReplyTo::Message(message) => {
                    if app_context.tg_context().open_chat_id() == message.chat_id {
                        let mut entry = Text::default();
                        entry.extend(vec![Line::from(vec![
                            Span::styled(
                                "↩️ Reply to: ",
                                app_context.style_chat_message_reply_text(),
                            ),
                            Span::styled(
                                app_context
                                    .tg_context()
                                    .try_name_from_chats_or_users(
                                        match app_context
                                            .tg_context()
                                            .open_chat_messages()
                                            .iter()
                                            .find(|m| m.id() == message.message_id)
                                        {
                                            Some(m) => m.sender_id(),
                                            None => -1,
                                        },
                                    )
                                    .unwrap_or_default(),
                                app_context.style_chat_message_reply_name(),
                            ),
                        ])]);
                        entry.extend(
                            match app_context
                                .tg_context()
                                .open_chat_messages()
                                .iter()
                                .find(|m| m.id() == message.message_id)
                            {
                                Some(m) => m.get_lines_styled_with_style(
                                    app_context.style_chat_message_reply_content(),
                                ),
                                None => vec![Line::from("")],
                            },
                        );
                        // entry.extend(vec![Line::from("")]);
                        Some(entry)
                    } else {
                        None
                    }
                }
                TdMessageReplyTo::Story(_) => {
                    let mut entry = Text::default();
                    entry.extend(vec![Line::from(vec![
                        Span::styled("↩️ Reply to: ", app_context.style_chat_message_reply_text()),
                        Span::styled("Story", app_context.style_chat_message_reply_name()),
                    ])]);
                    // entry.extend(vec![Line::from("")]);
                    Some(entry)
                }
            },
            None => None,
        };

        let mut entry = Text::default();
        entry.extend(vec![Line::from(vec![
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
            Span::raw(if self.is_edited { "✏️" } else { "" }),
            Span::raw(" "),
            Span::raw(match is_unread {
                Some(is_unread) => {
                    if is_unread {
                        "📤"
                    } else {
                        "👀"
                    }
                }
                None => "",
            }),
            Span::raw(" "),
            self.timestamp.get_span_styled(app_context),
        ])]);
        entry.extend(reply_text.unwrap_or_default());
        entry.extend(self.get_lines_styled_with_style(content_style));
        entry
    }

    fn message_content_lines(content: &MessageContent) -> Vec<Line<'static>> {
        match content {
            MessageContent::MessageText(m) => Self::format_message_content(&m.text),
            MessageContent::MessageAudio(_) => vec![Line::from("🎵 Audio")],
            MessageContent::MessagePhoto(_) => vec![Line::from("📷 Photo")],
            MessageContent::MessageSticker(_) => vec![Line::from("🎨 Sticker")],
            MessageContent::MessageVideo(_) => vec![Line::from("🎥 Video")],
            MessageContent::MessageAnimation(_) => vec![Line::from("🎞️ Animation")],
            MessageContent::MessageVoiceNote(_) => vec![Line::from("🎤 Voice Note")],
            MessageContent::MessageDocument(_) => vec![Line::from("📄 Document")],
            _ => vec![Line::from("")],
        }
    }

    fn from_span_to_lines(span: Span) -> Vec<Line<'static>> {
        span.content
            .split('\n')
            .map(|s| Line::from(Span::styled(s.to_owned(), span.style)))
            .collect::<Vec<Line>>()
    }

    fn from_spans_to_lines(spans: Vec<Span>) -> Vec<Line<'static>> {
        let vec = spans
            .iter()
            .filter(|s| !s.content.is_empty())
            .flat_map(|s| Self::from_span_to_lines(s.clone()))
            .collect::<Vec<Line>>();
        if vec.is_empty() {
            vec![Line::from("")]
        } else {
            vec
        }
    }

    fn merge_two_style(a: Style, b: Style) -> Style {
        Style {
            fg: a.fg.or(b.fg),
            bg: a.bg.or(b.bg),
            add_modifier: a.add_modifier | b.add_modifier,
            sub_modifier: a.sub_modifier | b.sub_modifier,
            ..Style::default()
        }
    }

    pub fn get_lines_styled_with_style(&self, content_style: Style) -> Vec<Line<'static>> {
        self.message_content
            .iter()
            .map(|l| {
                l.iter()
                    .map(|s| {
                        Span::styled(
                            s.content.clone(),
                            Self::merge_two_style(s.style, content_style),
                        )
                    })
                    .collect()
            })
            .collect::<Vec<Line>>()
    }

    fn format_message_content(message: &FormattedText) -> Vec<Line<'static>> {
        let text = &message.text;
        let entities = &message.entities;

        if entities.is_empty() {
            return Self::from_span_to_lines(Span::raw(text));
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
                tdlib::enums::TextEntityType::Strikethrough => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::CROSSED_OUT),
                    ));
                }
                tdlib::enums::TextEntityType::Url => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::UNDERLINED),
                    ));
                }
                tdlib::enums::TextEntityType::TextUrl(text_url) => {
                    message_vec.push(Span::styled(
                        text_url.url.clone(),
                        Style::default().add_modifier(Modifier::UNDERLINED),
                    ));
                }
                tdlib::enums::TextEntityType::EmailAddress => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::UNDERLINED),
                    ));
                }
                tdlib::enums::TextEntityType::Mention => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ));
                }
                tdlib::enums::TextEntityType::Hashtag => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ));
                }
                tdlib::enums::TextEntityType::PhoneNumber => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::UNDERLINED),
                    ));
                }
                tdlib::enums::TextEntityType::MentionName(mention_name) => {
                    message_vec.push(Span::styled(
                        // TODO: Fix from user_id to username
                        mention_name.user_id.to_string(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ));
                }
                tdlib::enums::TextEntityType::Code => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::DIM),
                    ));
                }
                tdlib::enums::TextEntityType::Pre => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::DIM),
                    ));
                }
                tdlib::enums::TextEntityType::PreCode(_pre_code) => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::DIM),
                    ));
                }
                tdlib::enums::TextEntityType::Cashtag => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ));
                }
                tdlib::enums::TextEntityType::BankCardNumber => {
                    message_vec.push(Span::styled(
                        text.chars().skip(offset).take(length).collect::<String>(),
                        Style::default().add_modifier(Modifier::UNDERLINED),
                    ));
                }
                tdlib::enums::TextEntityType::Spoiler => {}
                tdlib::enums::TextEntityType::MediaTimestamp(_) => {}
                tdlib::enums::TextEntityType::CustomEmoji(_) => {}
                tdlib::enums::TextEntityType::BotCommand => {}
            }
            message_vec.push(Span::raw(
                text.chars().skip(offset + length).collect::<String>(),
            ));
        });

        Self::from_spans_to_lines(message_vec)
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
            message_content: Self::message_content_lines(&message.content),
            reply_to: match &message.reply_to {
                Some(reply) => match reply {
                    MessageReplyTo::Message(message) => {
                        Some(TdMessageReplyTo::Message(message.into()))
                    }
                    MessageReplyTo::Story(story) => Some(TdMessageReplyTo::Story(story.into())),
                },
                None => None,
            },
            timestamp: DateTimeEntry {
                timestamp: message.date,
            },
            is_edited: message.edit_date != 0,
        }
    }
}
