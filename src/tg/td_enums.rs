use std::hash::Hash;

use tdlib::{enums::ChatList, types::ChatListFolder};

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
pub struct TdMessageReplyToMessage {
    /// The identifier of the chat to which the replied message belongs; ignored for outgoing replies. For example, messages in the Replies chat are replies to messages in different chats
    pub chat_id: i64,
    /// The identifier of the replied message
    pub message_id: i64,
}

impl From<TdMessageReplyToMessage> for tdlib::types::MessageReplyToMessage {
    fn from(reply_to_message: TdMessageReplyToMessage) -> Self {
        tdlib::types::MessageReplyToMessage {
            chat_id: reply_to_message.chat_id,
            message_id: reply_to_message.message_id,
        }
    }
}

impl From<tdlib::types::MessageReplyToMessage> for TdMessageReplyToMessage {
    fn from(reply_to_message: tdlib::types::MessageReplyToMessage) -> Self {
        TdMessageReplyToMessage {
            chat_id: reply_to_message.chat_id,
            message_id: reply_to_message.message_id,
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
