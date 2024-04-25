use crate::{
    components::chat_list_window::{ChatListEntry, MessageEntry},
    tg::ordered_chat::OrderedChat,
};
use std::{
    collections::{BTreeSet, HashMap},
    sync::{Mutex, MutexGuard},
};
use tdlib::{
    enums::{ChatList, ChatType},
    functions,
    types::{
        BasicGroup, BasicGroupFullInfo, Chat, SecretChat, Supergroup, SupergroupFullInfo, User,
        UserFullInfo,
    },
};

#[derive(Debug, Default)]
pub struct TgContext {
    pub users: Mutex<HashMap<i64, User>>,
    pub basic_groups: Mutex<HashMap<i64, BasicGroup>>,
    pub supergroups: Mutex<HashMap<i64, Supergroup>>,
    pub secret_chats: Mutex<HashMap<i32, SecretChat>>,

    pub chats: Mutex<HashMap<i64, Chat>>,
    // Only ordered
    pub main_chat_list: Mutex<BTreeSet<OrderedChat>>,
    pub have_full_main_chat_list: bool,

    pub users_full_info: Mutex<HashMap<i64, UserFullInfo>>,
    pub basic_groups_full_info: Mutex<HashMap<i64, BasicGroupFullInfo>>,
    pub supergroups_full_info: Mutex<HashMap<i64, SupergroupFullInfo>>,

    client_id: Mutex<i32>,
    open_chat_id: Mutex<i64>,
}

impl TgContext {
    pub fn new(client_id: i32) -> Self {
        Self {
            client_id: Mutex::new(client_id),
            ..Default::default()
        }
    }

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
    pub fn main_chat_list(&self) -> MutexGuard<'_, BTreeSet<OrderedChat>> {
        self.main_chat_list.lock().unwrap()
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
    pub fn client_id(&self) -> MutexGuard<'_, i32> {
        self.client_id.lock().unwrap()
    }

    pub fn open_chat_id(&self) -> MutexGuard<'_, i64> {
        self.open_chat_id.lock().unwrap()
    }

    pub fn get_name_of_open_chat(&self) -> Option<String> {
        if let Some(chat) = self.chats().get(&self.open_chat_id()) {
            return Some(chat.title.clone());
        }
        None
    }

    pub fn get_main_chat_list(&self) -> Option<Vec<ChatListEntry>> {
        let client_id = *self.client_id();
        tokio::spawn(
            async move { functions::load_chats(Some(ChatList::Main), 20, client_id).await },
        );

        let main_chat_list = self.main_chat_list();
        let chats = self.chats();
        let mut chat_list: Vec<ChatListEntry> = Vec::new();

        for ord_chat in main_chat_list.iter() {
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

        Some(chat_list)
    }
}
