// Test helper module for search functionality tests
use crate::{
    app_context::AppContext,
    cli::CliArgs,
    components::chat_list_window::ChatListEntry,
    configs::custom::{
        app_custom::AppConfig, keymap_custom::KeymapConfig, palette_custom::PaletteConfig,
        telegram_custom::TelegramConfig, theme_custom::ThemeConfig,
    },
    tg::{message_entry::MessageEntry, tg_context::TgContext},
};
use clap::Parser;
use std::sync::Arc;
use tdlib_rs::{
    enums::{MessageContent, MessageSender},
    types::{FormattedText, Message, MessageSenderUser, MessageText},
};

/// Helper function to create a test AppContext
pub fn create_test_app_context() -> Arc<AppContext> {
    let app_config = AppConfig::default();
    let keymap_config = KeymapConfig::default();
    let theme_config = ThemeConfig::default();
    let palette_config = PaletteConfig::default();
    let telegram_config = TelegramConfig::default();
    let tg_context = TgContext::default();
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

/// Helper function to create a mock MessageEntry
/// Since MessageEntry fields are private, we create it via the From<Message> trait
/// by constructing a minimal tdlib Message
pub fn create_mock_message(id: i64, content: &str) -> MessageEntry {
    // Create a minimal Message - MessageEntry::from only uses:
    // id, sender_id, content, reply_to, date, edit_date
    let message = Message {
        id,
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
                text: content.to_string(),
                entities: vec![],
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
    };
    MessageEntry::from(&message)
}

/// Helper function to create a mock ChatListEntry
pub fn create_mock_chat(chat_id: i64, chat_name: &str) -> ChatListEntry {
    let mut chat = ChatListEntry::new();
    chat.set_chat_id(chat_id);
    chat.set_chat_name(chat_name.to_string());
    chat
}
