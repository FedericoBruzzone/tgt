use std::hash::Hash;

use tdlib_rs::{enums::ChatList, types::ChatListFolder};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TdMessageSender {
    User(i64),
    Chat(i64),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TdChatList {
    Main,
    Archive,
    Folder(i32),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TdMessageReplyTo {
    Message(TdMessageReplyToMessage),
    Story(TdMessageReplyToStory),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TdMessageReplyToMessage {
    /// The identifier of the chat to which the replied message belongs; ignored for outgoing replies. For example, messages in the Replies chat are replies to messages in different chats
    pub chat_id: i64,
    /// The identifier of the replied message
    pub message_id: i64,
}

impl From<&TdMessageReplyToMessage> for tdlib_rs::types::InputMessageReplyToMessage {
    fn from(reply_to_message: &TdMessageReplyToMessage) -> Self {
        tdlib_rs::types::InputMessageReplyToMessage {
            chat_id: reply_to_message.chat_id,
            message_id: reply_to_message.message_id,
            quote: None,
        }
    }
}

impl From<&tdlib_rs::types::MessageReplyToMessage> for TdMessageReplyToMessage {
    fn from(reply_to_message: &tdlib_rs::types::MessageReplyToMessage) -> Self {
        TdMessageReplyToMessage {
            chat_id: reply_to_message.chat_id,
            message_id: reply_to_message.message_id,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TdMessageReplyToStory {
    /// The identifier of the sender of the replied story. Currently, stories can be replied only in the sender's chat
    pub story_sender_chat_id: i64,
    /// The identifier of the replied story
    pub story_id: i32,
}

impl From<&TdMessageReplyToStory> for tdlib_rs::types::MessageReplyToStory {
    fn from(reply_to_story: &TdMessageReplyToStory) -> Self {
        tdlib_rs::types::MessageReplyToStory {
            story_sender_chat_id: reply_to_story.story_sender_chat_id,
            story_id: reply_to_story.story_id,
        }
    }
}

impl From<&tdlib_rs::types::MessageReplyToStory> for TdMessageReplyToStory {
    fn from(reply_to_story: &tdlib_rs::types::MessageReplyToStory) -> Self {
        TdMessageReplyToStory {
            story_sender_chat_id: reply_to_story.story_sender_chat_id,
            story_id: reply_to_story.story_id,
        }
    }
}

impl From<ChatList> for TdChatList {
    fn from(chat_list: ChatList) -> Self {
        match chat_list {
            ChatList::Main => TdChatList::Main,
            ChatList::Archive => TdChatList::Archive,
            ChatList::Folder(folder_id) => TdChatList::Folder(folder_id.chat_folder_id),
        }
    }
}

impl From<TdChatList> for ChatList {
    fn from(td_chat_list: TdChatList) -> Self {
        match td_chat_list {
            TdChatList::Main => ChatList::Main,
            TdChatList::Archive => ChatList::Archive,
            TdChatList::Folder(folder_id) => ChatList::Folder(ChatListFolder {
                chat_folder_id: folder_id,
            }),
        }
    }
}
