use super::message_entry::MessageEntry;
use crate::{
    app_error::AppError, components::chat_list_window::ChatListEntry, event::Event,
    tg::ordered_chat::OrderedChat,
};
use std::sync::atomic::{AtomicI64, Ordering};
use std::{
    collections::{BTreeSet, HashMap},
    sync::{Mutex, MutexGuard},
};
use tdlib::{
    enums::ChatType,
    types::{
        BasicGroup, BasicGroupFullInfo, Chat, SecretChat, Supergroup, SupergroupFullInfo, User,
        UserFullInfo,
    },
};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Default)]
pub struct TgContext {
    users: Mutex<HashMap<i64, User>>,
    basic_groups: Mutex<HashMap<i64, BasicGroup>>,
    supergroups: Mutex<HashMap<i64, Supergroup>>,
    secret_chats: Mutex<HashMap<i32, SecretChat>>,

    chats: Mutex<HashMap<i64, Chat>>,
    // Only ordered
    chats_index: Mutex<BTreeSet<OrderedChat>>,

    users_full_info: Mutex<HashMap<i64, UserFullInfo>>,
    basic_groups_full_info: Mutex<HashMap<i64, BasicGroupFullInfo>>,
    supergroups_full_info: Mutex<HashMap<i64, SupergroupFullInfo>>,

    event_tx: Mutex<Option<UnboundedSender<Event>>>,
    me: AtomicI64,
    open_chat_id: AtomicI64,
    open_chat_messages: Mutex<Vec<MessageEntry>>,

    /// The chat id from which to start loading the chat history.
    from_message_id: Mutex<i64>,
}

impl TgContext {
    pub fn users(&self) -> MutexGuard<'_, HashMap<i64, User>> {
        self.users.lock().unwrap()
    }
    pub fn basic_groups(&self) -> MutexGuard<'_, HashMap<i64, BasicGroup>> {
        self.basic_groups.lock().unwrap()
    }
    pub fn supergroups(&self) -> MutexGuard<'_, HashMap<i64, Supergroup>> {
        self.supergroups.lock().unwrap()
    }
    pub fn secret_chats(&self) -> MutexGuard<'_, HashMap<i32, SecretChat>> {
        self.secret_chats.lock().unwrap()
    }
    pub fn chats(&self) -> MutexGuard<'_, HashMap<i64, Chat>> {
        self.chats.lock().unwrap()
    }
    pub fn chats_index(&self) -> MutexGuard<'_, BTreeSet<OrderedChat>> {
        self.chats_index.lock().unwrap()
    }
    pub fn users_full_info(&self) -> MutexGuard<'_, HashMap<i64, UserFullInfo>> {
        self.users_full_info.lock().unwrap()
    }
    pub fn basic_groups_full_info(&self) -> MutexGuard<'_, HashMap<i64, BasicGroupFullInfo>> {
        self.basic_groups_full_info.lock().unwrap()
    }
    pub fn supergroups_full_info(&self) -> MutexGuard<'_, HashMap<i64, SupergroupFullInfo>> {
        self.supergroups_full_info.lock().unwrap()
    }
    pub fn open_chat_id(&self) -> i64 {
        self.open_chat_id.load(Ordering::Relaxed)
    }
    pub fn open_chat_messages(&self) -> MutexGuard<'_, Vec<MessageEntry>> {
        self.open_chat_messages.lock().unwrap()
    }
    pub fn event_tx(&self) -> MutexGuard<'_, Option<UnboundedSender<Event>>> {
        self.event_tx.lock().unwrap()
    }
    pub fn me(&self) -> i64 {
        self.me.load(Ordering::Relaxed)
    }
    pub fn from_message_id(&self) -> MutexGuard<'_, i64> {
        self.from_message_id.lock().unwrap()
    }

    pub fn set_open_chat_id(&self, chat_id: i64) {
        self.open_chat_id.store(chat_id, Ordering::Relaxed);
    }

    pub fn clear_open_chat_messages(&self) {
        *self.open_chat_messages() = Vec::new();
    }

    pub fn set_from_message_id(&self, from_message_id: i64) {
        *self.from_message_id() = from_message_id;
    }

    pub fn set_me(&self, me: i64) {
        self.me.store(me, Ordering::Relaxed);
    }

    pub fn set_event_tx(&self, event_tx: UnboundedSender<Event>) {
        *self.event_tx() = Some(event_tx);
    }

    pub fn try_name_from_chats_or_users(&self, user_id: i64) -> Option<String> {
        if self.name_from_chats(user_id).is_some() {
            return self.name_from_chats(user_id);
        }
        if let Some(user) = self.users().get(&user_id) {
            match user.usernames.as_ref() {
                Some(usernames) => {
                    if let Some(username) = usernames.active_usernames.first() {
                        return Some(username.clone());
                    }
                }
                None => {
                    return Some(user.first_name.clone());
                }
            }
        }
        None
    }

    pub fn name_from_chats(&self, chat_id: i64) -> Option<String> {
        if let Some(chat) = self.chats().get(&chat_id) {
            return Some(chat.title.clone());
        }
        None
    }

    pub fn name_of_open_chat_id(&self) -> Option<String> {
        if let Some(chat) = self.chats().get(&self.open_chat_id()) {
            return Some(chat.title.clone());
        }
        None
    }

    pub fn get_chats_index(&self) -> Result<Option<Vec<ChatListEntry>>, AppError<Event>> {
        let chats_index = self.chats_index();
        let chats = self.chats();
        let mut chat_list: Vec<ChatListEntry> = Vec::new();
        for ord_chat in chats_index.iter() {
            let mut chat_list_item = ChatListEntry::new();
            chat_list_item.set_chat_id(ord_chat.chat_id);
            if let Some(chat) = chats.get(&ord_chat.chat_id) {
                if let Some(chat_message) = &chat.last_message {
                    chat_list_item.set_last_message(MessageEntry::from(chat_message));
                }
                match &chat.r#type {
                    ChatType::Private(p) => {
                        if let Some(user) = self.users().get(&p.user_id) {
                            chat_list_item.set_chat_name(chat.title.clone());
                            chat_list_item.set_verificated(user.is_verified);
                            chat_list_item.set_status(user.status.clone());
                        }
                    }
                    ChatType::BasicGroup(bg) => {
                        if let Some(_basic_group) = self.basic_groups().get(&bg.basic_group_id) {
                            chat_list_item.set_chat_name(chat.title.clone());
                        }
                    }
                    ChatType::Supergroup(sg) => {
                        if let Some(_supergroup) = self.supergroups().get(&sg.supergroup_id) {
                            chat_list_item.set_chat_name(chat.title.clone());
                        }
                    }
                    ChatType::Secret(s) => {
                        if let Some(_secret_chat) = self.secret_chats().get(&s.secret_chat_id) {
                            chat_list_item.set_chat_name(chat.title.clone());
                        }
                    }
                }
            }

            chat_list.push(chat_list_item);
        }

        Ok(Some(chat_list))
    }
}
