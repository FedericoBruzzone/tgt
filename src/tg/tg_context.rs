use super::message_entry::MessageEntry;
use super::open_chat_store::OpenChatMessageStore;
use crate::tg::message_entry::DateTimeEntry;
use crate::{
    app_error::AppError, components::chat_list_window::ChatListEntry, event::Event,
    tg::ordered_chat::OrderedChat,
};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::{
    collections::{BTreeSet, HashMap},
    sync::{Mutex, MutexGuard},
};
use tdlib_rs::{
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

    // Ordered
    chats_index: Mutex<BTreeSet<OrderedChat>>,

    users_full_info: Mutex<HashMap<i64, UserFullInfo>>,
    basic_groups_full_info: Mutex<HashMap<i64, BasicGroupFullInfo>>,
    supergroups_full_info: Mutex<HashMap<i64, SupergroupFullInfo>>,

    event_tx: Mutex<Option<UnboundedSender<Event>>>,
    me: AtomicI64,
    open_chat_id: AtomicI64,
    /// Message cache and view window for the open chat (PR1 data layer).
    open_chat_messages: Mutex<OpenChatMessageStore>,
    open_chat_user: Mutex<Option<User>>,

    last_acknowledged_message_id: AtomicI64,

    /// The message id from which to start loading the chat history (set by backend when appending).
    from_message_id: AtomicI64,

    /// True while a "load more history" request is in flight; avoids duplicate requests.
    history_loading: AtomicBool,

    /// reply message id
    reply_message_id: AtomicI64,
    /// reply message text
    reply_message_text: Mutex<String>,
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
    pub fn open_chat_messages(&self) -> MutexGuard<'_, OpenChatMessageStore> {
        self.open_chat_messages.lock().unwrap()
    }
    pub fn event_tx(&self) -> MutexGuard<'_, Option<UnboundedSender<Event>>> {
        self.event_tx.lock().unwrap()
    }
    pub fn me(&self) -> i64 {
        self.me.load(Ordering::Relaxed)
    }
    pub fn from_message_id(&self) -> i64 {
        self.from_message_id.load(Ordering::Relaxed)
    }
    pub fn open_chat_user(&self) -> MutexGuard<'_, Option<User>> {
        self.open_chat_user.lock().unwrap()
    }
    pub fn last_acknowledged_message_id(&self) -> i64 {
        self.last_acknowledged_message_id.load(Ordering::Relaxed)
    }

    pub fn set_open_chat_user(&self, user: Option<User>) {
        *self.open_chat_user() = user;
    }

    pub fn set_open_chat_id(&self, chat_id: i64) {
        self.open_chat_id.store(chat_id, Ordering::Relaxed);
    }

    pub fn clear_open_chat_messages(&self) {
        self.open_chat_messages().clear();
    }

    pub fn set_from_message_id(&self, from_message_id: i64) {
        self.from_message_id
            .store(from_message_id, Ordering::Relaxed);
    }

    // ----- Read-only API for UI (PR1) -----

    /// Ordered message IDs for the open chat (oldest to newest). UI uses this to build the list.
    pub fn ordered_message_ids(&self) -> Vec<i64> {
        self.open_chat_messages().ordered_message_ids()
    }

    /// Get a message by ID (clone). UI read-only.
    pub fn get_message(&self, id: i64) -> Option<MessageEntry> {
        self.open_chat_messages().get_message(id)
    }

    /// Oldest loaded message ID; None if empty.
    pub fn oldest_message_id(&self) -> Option<i64> {
        self.open_chat_messages().oldest_message_id()
    }

    /// Newest loaded message ID; None if empty.
    pub fn newest_message_id(&self) -> Option<i64> {
        self.open_chat_messages().newest_message_id()
    }

    /// True while a "load more history" request is in flight.
    pub fn is_history_loading(&self) -> bool {
        self.history_loading.load(Ordering::Acquire)
    }

    /// Set history loading flag (backend/task only).
    pub fn set_history_loading(&self, value: bool) {
        self.history_loading.store(value, Ordering::Release);
    }

    pub fn set_me(&self, me: i64) {
        self.me.store(me, Ordering::Relaxed);
    }

    pub fn set_last_acknowledged_message_id(&self, message_id: i64) {
        self.last_acknowledged_message_id
            .store(message_id, Ordering::Relaxed);
    }

    pub fn set_event_tx(&self, event_tx: UnboundedSender<Event>) {
        *self.event_tx() = Some(event_tx);
    }

    // This is used to know if a message is being replied to.
    pub fn set_reply_message(&self, message_id: i64, text: String) {
        self.reply_message_id.store(message_id, Ordering::Relaxed);
        *self.reply_message_text.lock().unwrap() = text;
    }

    pub fn delete_message(&self, message_id: i64) {
        self.open_chat_messages().remove_message(message_id);
    }

    pub fn open_chat_user_status(&self) -> String {
        if let Some(user) = self.open_chat_user().as_ref() {
            return match &user.status {
                tdlib_rs::enums::UserStatus::Empty => "Empty".to_string(),
                tdlib_rs::enums::UserStatus::Online(_) => "Online".to_string(),
                tdlib_rs::enums::UserStatus::Offline(offline) => {
                    format!(
                        "Last seen {}",
                        DateTimeEntry::convert_time(offline.was_online)
                    )
                }
                tdlib_rs::enums::UserStatus::Recently(_) => "Last seen recently ".to_string(),
                tdlib_rs::enums::UserStatus::LastWeek(_) => "Last seen LastWeek".to_string(),
                tdlib_rs::enums::UserStatus::LastMonth(_) => "Last seen LastMonth ".to_string(),
            };
        }
        "".to_string()
    }

    pub fn unread_messages(&self) -> Vec<i64> {
        let mut unread_messages: Vec<i64> = Vec::new();
        for id in self.open_chat_messages().ordered_message_ids() {
            unread_messages.push(id);
            if id == self.last_read_inbox_message_id() {
                break;
            }
        }
        unread_messages
    }

    pub fn last_read_inbox_message_id(&self) -> i64 {
        let opened_chat = self.chats().get(&self.open_chat_id()).cloned();
        if let Some(opened_chat) = opened_chat {
            return opened_chat.last_read_inbox_message_id;
        }
        -1
    }

    pub fn last_read_outbox_message_id(&self) -> i64 {
        let opened_chat = self.chats().get(&self.open_chat_id()).cloned();
        if let Some(opened_chat) = opened_chat {
            return opened_chat.last_read_outbox_message_id;
        }
        -1
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

    pub fn reply_message_id(&self) -> i64 {
        self.reply_message_id.load(Ordering::Relaxed)
    }

    pub fn reply_message_text(&self) -> MutexGuard<'_, String> {
        self.reply_message_text.lock().unwrap()
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
                chat_list_item.set_is_marked_as_unread(chat.unread_count > 0);
                chat_list_item.set_chat_name(chat.title.clone());
                chat_list_item.set_last_read_inbox_message_id(chat.last_read_inbox_message_id);
                chat_list_item.set_last_read_outbox_message_id(chat.last_read_outbox_message_id);
                chat_list_item.set_unread_count(chat.unread_count);
                if let Some(chat_message) = &chat.last_message {
                    chat_list_item.set_last_message(MessageEntry::from(chat_message));
                }
                match &chat.r#type {
                    ChatType::Private(p) => {
                        if let Some(user) = self.users().get(&p.user_id) {
                            chat_list_item.set_user(user.clone());
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
