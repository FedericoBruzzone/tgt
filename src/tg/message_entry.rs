use crate::app_context::AppContext;
use chrono::{DateTime, Local};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use std::time::{Duration, UNIX_EPOCH};
use tdlib_rs::enums::{MessageContent, MessageReplyTo, MessageSender};
use tdlib_rs::types::FormattedText;

use super::td_enums::{TdMessageReplyTo, TdMessageSender};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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

    pub fn get_span_styled(&self, app_context: &AppContext) -> Span<'_> {
        Span::styled(
            Self::convert_time(self.timestamp),
            app_context.style_timestamp(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageEntry {
    id: i64,
    sender_id: TdMessageSender,
    message_content: Vec<Line<'static>>,
    reply_to: Option<TdMessageReplyTo>,
    timestamp: DateTimeEntry,
    is_edited: bool,
}

impl MessageEntry {
    /// Test-only: create a minimal MessageEntry for unit tests (e.g. open_chat_store).
    #[cfg(test)]
    pub fn test_entry(id: i64) -> Self {
        Self {
            id,
            sender_id: TdMessageSender::User(0),
            message_content: vec![Line::from("")],
            reply_to: None,
            timestamp: DateTimeEntry { timestamp: 0 },
            is_edited: false,
        }
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
        myself: bool,
        app_context: &AppContext,
        is_unread: bool, // When myself is false, is_unread is useless
        name_style: Style,
        content_style: Style,
        wrap_width: i32,
    ) -> Text<'_> {
        let (message_reply_name, message_reply_content) = if myself {
            (
                app_context.style_chat_message_myself_reply_name(),
                app_context.style_chat_message_myself_reply_content(),
            )
        } else {
            (
                app_context.style_chat_message_other_reply_name(),
                app_context.style_chat_message_other_reply_content(),
            )
        };

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
                                        app_context
                                            .tg_context()
                                            .get_message(message.message_id)
                                            .map(|m| m.sender_id())
                                            .unwrap_or(-1),
                                    )
                                    .unwrap_or_default(),
                                message_reply_name,
                            ),
                        ])]);
                        entry.extend(
                            match app_context.tg_context().get_message(message.message_id) {
                                Some(m) => {
                                    m.get_lines_styled_with_style(message_reply_content, wrap_width)
                                }
                                None => vec![Line::from("")],
                            },
                        );
                        Some(entry)
                    } else {
                        None
                    }
                }
                TdMessageReplyTo::Story(_) => {
                    let mut entry = Text::default();
                    entry.extend(vec![Line::from(vec![
                        Span::styled("↩️ Reply to: ", app_context.style_chat_message_reply_text()),
                        Span::styled("Story", message_reply_name),
                    ])]);
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
            Span::raw(match myself {
                true => {
                    if is_unread {
                        "📤"
                    } else {
                        "👀"
                    }
                }
                false => "",
            }),
            Span::raw(" "),
            self.timestamp.get_span_styled(app_context),
        ])]);
        entry.extend(reply_text.unwrap_or_default());
        entry.extend(self.get_lines_styled_with_style(content_style, wrap_width));
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

    pub fn get_lines_styled_with_style(
        &self,
        content_style: Style,
        wrap_width: i32,
    ) -> Vec<Line<'static>> {
        if wrap_width == -1 {
            // No wrap
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
        } else {
            // Wrap the text
            let mut lines = Vec::new();
            let mut current_line = Line::default();
            let mut current_line_length = 0;
            // for span in self.message_content.iter().flat_map(|l| l.iter()) {
            for span in self.message_content.iter().flat_map(|l| l.iter()) {
                for c in span.content.chars() {
                    if c == ' ' && current_line_length >= wrap_width {
                        lines.push(current_line);
                        current_line = Line::default();
                        current_line_length = 0;
                    }
                    current_line.spans.push(Span::styled(
                        c.to_string(),
                        Self::merge_two_style(span.style, content_style),
                    ));
                    current_line_length += 1;
                }
                lines.push(current_line);
                current_line = Line::default();
                current_line_length = 0;
            }
            lines
        }
    }

    /// Extract a substring by character range (start..end). Used for entity segments.
    fn text_slice_chars(text: &str, start: usize, end: usize) -> String {
        text.chars()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect()
    }

    fn format_message_content(message: &FormattedText) -> Vec<Line<'static>> {
        let text = &message.text;
        let entities = &message.entities;

        if entities.is_empty() {
            return Self::from_span_to_lines(Span::raw(text.clone()));
        }

        // Build disjoint segments (start, end, style, optional url for TextUrl).
        // TDLib uses UTF-16 offset/length; we treat as char indices for simplicity (see PR3 doc).
        type Segment = (usize, usize, Style, Option<String>);
        let mut segments: Vec<Segment> = Vec::new();
        for e in entities.iter() {
            let offset = e.offset as usize;
            let length = e.length as usize;
            let end = offset.saturating_add(length);
            let style_content = match &e.r#type {
                tdlib_rs::enums::TextEntityType::Italic => {
                    Some((Style::default().add_modifier(Modifier::ITALIC), None))
                }
                tdlib_rs::enums::TextEntityType::Bold => {
                    Some((Style::default().add_modifier(Modifier::BOLD), None))
                }
                tdlib_rs::enums::TextEntityType::Underline => {
                    Some((Style::default().add_modifier(Modifier::UNDERLINED), None))
                }
                tdlib_rs::enums::TextEntityType::Strikethrough => {
                    Some((Style::default().add_modifier(Modifier::CROSSED_OUT), None))
                }
                tdlib_rs::enums::TextEntityType::Url => {
                    Some((Style::default().add_modifier(Modifier::UNDERLINED), None))
                }
                tdlib_rs::enums::TextEntityType::TextUrl(text_url) => Some((
                    Style::default().add_modifier(Modifier::UNDERLINED),
                    Some(text_url.url.clone()),
                )),
                tdlib_rs::enums::TextEntityType::EmailAddress => {
                    Some((Style::default().add_modifier(Modifier::UNDERLINED), None))
                }
                tdlib_rs::enums::TextEntityType::Mention => {
                    Some((Style::default().add_modifier(Modifier::BOLD), None))
                }
                tdlib_rs::enums::TextEntityType::Hashtag => {
                    Some((Style::default().add_modifier(Modifier::BOLD), None))
                }
                tdlib_rs::enums::TextEntityType::PhoneNumber => {
                    Some((Style::default().add_modifier(Modifier::UNDERLINED), None))
                }
                tdlib_rs::enums::TextEntityType::MentionName(mention_name) => Some((
                    Style::default().add_modifier(Modifier::BOLD),
                    Some(mention_name.user_id.to_string()),
                )),
                tdlib_rs::enums::TextEntityType::Code => {
                    Some((Style::default().add_modifier(Modifier::DIM), None))
                }
                tdlib_rs::enums::TextEntityType::Pre => {
                    Some((Style::default().add_modifier(Modifier::DIM), None))
                }
                tdlib_rs::enums::TextEntityType::PreCode(_) => {
                    Some((Style::default().add_modifier(Modifier::DIM), None))
                }
                tdlib_rs::enums::TextEntityType::Cashtag => {
                    Some((Style::default().add_modifier(Modifier::BOLD), None))
                }
                tdlib_rs::enums::TextEntityType::BankCardNumber => {
                    Some((Style::default().add_modifier(Modifier::UNDERLINED), None))
                }
                tdlib_rs::enums::TextEntityType::BlockQuote => {
                    Some((Style::default().add_modifier(Modifier::DIM), None))
                }
                tdlib_rs::enums::TextEntityType::Spoiler
                | tdlib_rs::enums::TextEntityType::MediaTimestamp(_)
                | tdlib_rs::enums::TextEntityType::CustomEmoji(_)
                | tdlib_rs::enums::TextEntityType::BotCommand => None,
            };
            if let Some((style, url_override)) = style_content {
                segments.push((offset, end, style, url_override));
            }
        }

        // Sort by start; merge overlapping (later segment wins for overlap).
        segments.sort_by_key(|(s, _, _, _)| *s);
        let mut disjoint: Vec<Segment> = Vec::new();
        for (start, end, style, url_override) in segments {
            if start >= end {
                continue;
            }
            if let Some(last) = disjoint.last_mut() {
                let (_, last_end, _, _) = *last;
                if start < last_end {
                    // Trim last segment so it doesn't overlap with this one
                    *last = (last.0, start, last.2.clone(), last.3.clone());
                }
            }
            disjoint.push((start, end, style, url_override));
        }

        // Build spans in order: raw before first, styled(seg), raw between, ... raw after last.
        let char_count = text.chars().count();
        let mut message_vec = Vec::new();
        let mut prev_end = 0usize;
        for (start, end, style, url_override) in disjoint {
            if start > prev_end {
                let raw_slice = Self::text_slice_chars(text, prev_end, start);
                if !raw_slice.is_empty() {
                    message_vec.push(Span::raw(raw_slice));
                }
            }
            let content = url_override
                .unwrap_or_else(|| Self::text_slice_chars(text, start, end));
            if !content.is_empty() {
                message_vec.push(Span::styled(content, style));
            }
            prev_end = end;
        }
        if prev_end < char_count {
            let raw_slice = Self::text_slice_chars(text, prev_end, char_count);
            if !raw_slice.is_empty() {
                message_vec.push(Span::raw(raw_slice));
            }
        }

        Self::from_spans_to_lines(message_vec)
    }
}
impl From<&tdlib_rs::types::Message> for MessageEntry {
    fn from(message: &tdlib_rs::types::Message) -> Self {
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

#[cfg(test)]
mod message_parsing_tests {
    use super::*;
    use tdlib_rs::enums::{MessageContent, MessageSender, TextEntityType};
    use tdlib_rs::types::{
        FormattedText, Message, MessageSenderUser, MessageText, TextEntity,
    };

    fn message_with_entities(text: &str, entities: Vec<TextEntity>) -> Message {
        Message {
            id: 1,
            sender_id: MessageSender::User(MessageSenderUser { user_id: 1 }),
            chat_id: 1,
            author_signature: String::new(),
            via_bot_user_id: 0,
            reply_to: None,
            date: 0,
            edit_date: 0,
            message_thread_id: 0,
            content: MessageContent::MessageText(MessageText {
                text: FormattedText {
                    text: text.to_string(),
                    entities,
                },
                link_preview_options: None,
                web_page: None,
            }),
            sending_state: None,
            scheduling_state: None,
            is_outgoing: false,
            is_pinned: false,
            is_from_offline: false,
            can_be_edited: false,
            can_be_forwarded: false,
            can_be_deleted_only_for_self: false,
            can_be_deleted_for_all_users: false,
            can_get_added_reactions: false,
            can_get_statistics: false,
            can_get_message_thread: false,
            can_get_viewers: false,
            can_get_media_timestamp_links: false,
            can_report_reactions: false,
            is_channel_post: false,
            is_topic_message: false,
            contains_unread_mention: false,
            interaction_info: None,
            unread_reactions: vec![],
            self_destruct_type: None,
            auto_delete_in: 0.0,
            media_album_id: 0,
            restriction_reason: String::new(),
            can_be_replied_in_another_chat: false,
            can_be_saved: false,
            can_get_read_date: false,
            has_timestamped_media: false,
            forward_info: None,
            import_info: None,
            saved_messages_topic_id: 0,
            sender_business_bot_user_id: 0,
            sender_boost_count: 0,
            reply_markup: None,
            self_destruct_in: 0.0,
        }
    }

    /// No duplicate or overlapping spans: output must equal the original text.
    #[test]
    fn format_message_content_single_entity_no_duplicate() {
        let msg = message_with_entities(
            "hello",
            vec![TextEntity {
                offset: 0,
                length: 5,
                r#type: TextEntityType::Bold,
            }],
        );
        let entry = MessageEntry::from(&msg);
        assert_eq!(entry.message_content_to_string(), "hello");
    }

    /// Two adjacent entities: output must be exact text, no duplicate segments.
    #[test]
    fn format_message_content_two_adjacent_entities_no_duplicate() {
        let msg = message_with_entities(
            "ab",
            vec![
                TextEntity {
                    offset: 0,
                    length: 1,
                    r#type: TextEntityType::Bold,
                },
                TextEntity {
                    offset: 1,
                    length: 1,
                    r#type: TextEntityType::Italic,
                },
            ],
        );
        let entry = MessageEntry::from(&msg);
        assert_eq!(entry.message_content_to_string(), "ab");
    }

    /// Two non-overlapping entities: output must be exact text.
    #[test]
    fn format_message_content_two_non_overlapping_no_duplicate() {
        let msg = message_with_entities(
            "hello world",
            vec![
                TextEntity {
                    offset: 0,
                    length: 5,
                    r#type: TextEntityType::Bold,
                },
                TextEntity {
                    offset: 6,
                    length: 5,
                    r#type: TextEntityType::Italic,
                },
            ],
        );
        let entry = MessageEntry::from(&msg);
        assert_eq!(entry.message_content_to_string(), "hello world");
    }

    /// Empty entities: plain text.
    #[test]
    fn format_message_content_empty_entities() {
        let msg = message_with_entities("plain", vec![]);
        let entry = MessageEntry::from(&msg);
        assert_eq!(entry.message_content_to_string(), "plain");
    }
}
