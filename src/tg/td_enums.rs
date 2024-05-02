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
