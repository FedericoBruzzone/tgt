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
        sending_state: None,
        scheduling_state: None,
        is_outgoing: false,
        is_pinned: false,
        is_from_offline: false,
        can_be_saved: false,
        has_timestamped_media: false,
        is_channel_post: false,
        is_paid_star_suggested_post: false,
        is_paid_ton_suggested_post: false,
        contains_unread_mention: false,
        date: 0,
        edit_date: 0,
        forward_info: None,
        import_info: None,
        interaction_info: None,
        unread_reactions: vec![],
        fact_check: None,
        suggested_post_info: None,
        reply_to: None,
        topic_id: None,
        self_destruct_type: None,
        self_destruct_in: 0.0,
        auto_delete_in: 0.0,
        via_bot_user_id: 0,
        sender_business_bot_user_id: 0,
        sender_boost_count: 0,
        paid_message_star_count: 0,
        author_signature: String::new(),
        media_album_id: 0,
        effect_id: 0,
        restriction_info: None,
        summary_language_code: String::new(),
        content: MessageContent::MessageText(MessageText {
            text: FormattedText {
                text: content.to_string(),
                entities: vec![],
            },
            link_preview: None,
            link_preview_options: None,
        }),
        reply_markup: None,
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
