use {
    crate::{app_context::AppContext, tg::ordered_chat::OrderedChat},
    chrono::{DateTime, Utc},
    ratatui::{
        style::{Modifier, Style},
        text::{Line, Span, Text},
    },
    std::{
        collections::{BTreeSet, HashMap, VecDeque},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex, MutexGuard,
        },
        time::{Duration, UNIX_EPOCH},
    },
    tdlib::{
        enums::{self, AuthorizationState, LogStream, Update},
        functions,
        types::{
            BasicGroup, BasicGroupFullInfo, Chat, ChatPosition, LogStreamFile, SecretChat,
            Supergroup, SupergroupFullInfo, User, UserFullInfo,
        },
    },
    tokio::{
        sync::mpsc::{UnboundedReceiver, UnboundedSender},
        task::JoinHandle,
    },
};

pub trait Item {
    fn get_text_styled(&self) -> Text;
}

#[derive(Debug, Default)]
pub struct DateTimeEntry {
    pub timestamp: i32,
}

impl DateTimeEntry {
    pub fn convert_time(timestamp: i32) -> String {
        let d = UNIX_EPOCH + Duration::from_secs(timestamp as u64);
        let datetime = DateTime::<Utc>::from(d);
        datetime.format("%Y-%m-%d").to_string()
    }
}
impl Item for DateTimeEntry {
    fn get_text_styled(&self) -> Text {
        let mut entry = Text::default();
        entry.extend(vec![Line::from(Span::styled(
            Self::convert_time(self.timestamp),
            Style::default(),
        ))]);
        entry
    }
}

#[derive(Debug, Default)]
pub struct MessageEntry {
    _id: i64,
    content: Line<'static>,
    timestamp: DateTimeEntry,
}
impl Item for MessageEntry {
    fn get_text_styled(&self) -> Text {
        let mut entry = Text::default();
        entry.extend(vec![Line::from(format!(
            "{} | {}",
            self.content,
            self.timestamp.get_text_styled(),
        ))]);
        entry
    }
}
impl MessageEntry {
    fn message_content_line(content: &enums::MessageContent) -> Line<'static> {
        match content {
            enums::MessageContent::MessageText(m) => Self::format_message_content(&m.text),
            _ => Line::from(""),
        }
    }

    fn format_message_content(message: &tdlib::types::FormattedText) -> Line<'static> {
        let text = &message.text;
        let entities = &message.entities;

        if entities.is_empty() {
            return Line::from(Span::raw(text.clone()));
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
                _ => {}
            }
            message_vec.push(Span::raw(
                text.chars().skip(offset + length).collect::<String>(),
            ));
        });
        Line::from(message_vec)
    }
}
impl From<&tdlib::types::Message> for MessageEntry {
    fn from(message: &tdlib::types::Message) -> Self {
        Self {
            _id: message.id,
            content: Self::message_content_line(&message.content),
            timestamp: DateTimeEntry {
                timestamp: message.date,
            },
        }
    }
}

#[derive(Debug)]
pub struct ChatListEntry {
    chat_id: i64,
    chat_name: String,
    last_message: Option<MessageEntry>,
    muted: bool,
    status: tdlib::enums::UserStatus,
    verificated: bool,
}
impl Default for ChatListEntry {
    fn default() -> Self {
        Self::new()
    }
}
impl ChatListEntry {
    pub fn new() -> Self {
        Self {
            chat_id: 0,
            chat_name: String::new(),
            last_message: None,
            muted: false,
            status: tdlib::enums::UserStatus::Empty,
            verificated: false,
        }
    }

    pub fn set_chat_id(&mut self, chat_id: i64) {
        self.chat_id = chat_id;
    }
    pub fn set_chat_name(&mut self, chat_name: String) {
        self.chat_name = chat_name;
    }
    pub fn set_last_message(&mut self, last_message: MessageEntry) {
        self.last_message = Some(last_message);
    }
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }
    pub fn set_status(&mut self, status: tdlib::enums::UserStatus) {
        self.status = status;
    }
    pub fn set_verificated(&mut self, verificated: bool) {
        self.verificated = verificated;
    }
}
impl Item for ChatListEntry {
    fn get_text_styled(&self) -> Text {
        let mut entry = Text::default();
        let online_symbol = match self.status {
            tdlib::enums::UserStatus::Online(_) => "🟢",
            tdlib::enums::UserStatus::Offline(_) => "🔴",
            _ => "🔴",
        };
        let verificated_symbol = if self.verificated { "✅" } else { "" };
        let muted_symbol = if self.muted { "🔇" } else { "" };
        entry.extend(vec![Line::from(format!(
            "{} {} {} {}",
            online_symbol, self.chat_name, verificated_symbol, muted_symbol
        ))]);
        entry.extend(
            self.last_message
                .as_ref()
                .map_or_else(Text::default, |m| m.get_text_styled()),
        );
        entry
    }
}

#[derive(Debug, Default)]
pub struct TgContext {
    pub users: Mutex<HashMap<i64, User>>,
    pub basic_groups: Mutex<HashMap<i64, BasicGroup>>,
    pub supergroups: Mutex<HashMap<i64, Supergroup>>,
    pub secret_chats: Mutex<HashMap<i32, SecretChat>>,

    pub chats: Mutex<HashMap<i64, Chat>>,
    // Only ordered chats
    pub main_chat_list: Mutex<BTreeSet<OrderedChat>>,
    pub have_full_main_chat_list: bool,

    pub users_full_info: Mutex<HashMap<i64, UserFullInfo>>,
    pub basic_groups_full_info: Mutex<HashMap<i64, BasicGroupFullInfo>>,
    pub supergroups_full_info: Mutex<HashMap<i64, SupergroupFullInfo>>,

    client_id: Mutex<Option<i32>>,
}

impl TgContext {
    // lock_users
    fn users(&self) -> MutexGuard<'_, HashMap<i64, User>> {
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
    pub fn client_id(&self) -> MutexGuard<'_, Option<i32>> {
        self.client_id.lock().unwrap()
    }

    pub fn insert_user(&self, user_id: i64, user: User) {
        self.users().insert(user_id, user);
    }

    pub fn set_client_id(&self, client_id: i32) {
        self.client_id().replace(client_id);
    }

    pub fn get_main_chat_list(&self) -> Option<Vec<ChatListEntry>> {
        let client_id = self.client_id().unwrap();
        tokio::spawn(async move {
            functions::load_chats(Some(enums::ChatList::Main), 20, client_id).await
        });

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
                    enums::ChatType::Private(p) => {
                        if let Some(user) = self.users().get(&p.user_id) {
                            // chat_list_item
                            //     .set_chat_name(format!("{} {}", user.first_name, user.last_name));
                            chat_list_item.set_chat_name(chat.title.clone());
                            chat_list_item.set_verificated(user.is_verified);
                            chat_list_item.set_status(user.status.clone());
                        }
                    }
                    enums::ChatType::BasicGroup(bg) => {
                        if let Some(_basic_group) = self.basic_groups().get(&bg.basic_group_id) {
                            chat_list_item.set_chat_name(chat.title.clone());
                        }
                    }
                    enums::ChatType::Supergroup(sg) => {
                        if let Some(_supergroup) = self.supergroups().get(&sg.supergroup_id) {
                            chat_list_item.set_chat_name(chat.title.clone());
                        }
                    }
                    enums::ChatType::Secret(s) => {
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

pub struct TgBackend {
    pub handle_updates: JoinHandle<()>,
    pub auth_rx: UnboundedReceiver<AuthorizationState>,
    pub auth_tx: UnboundedSender<AuthorizationState>,
    pub client_id: i32,
    pub have_authorization: bool,
    pub need_quit: bool,
    pub can_quit: Arc<AtomicBool>,
    pub app_context: Arc<AppContext>,
}

impl TgBackend {
    // pub async fn get_command(&mut self) {
    //     let command = ask_user("Enter command (gcs - GetChats, gc <chatId> - GetChat, me - GetMe, sm <chatId> <message> - SendMessage, lo - LogOut, q - Quit, mcl - MainChatList, h <chatId> - GetChatHistory): ");
    //     let commands: Vec<&str> = command.split(' ').collect();
    //     match commands[0] {
    //         "gcs" => {
    //             let mut limit = 20;
    //             if commands.len() > 1 {
    //                 limit = commands[1].parse::<i32>().unwrap();
    //             }
    //             match functions::load_chats(Some(enums::ChatList::Main), limit, self.client_id)
    //                 .await
    //             {
    //                 Ok(()) => (),
    //                 Err(error) => eprintln!("[GET MAIN CHAT LIST]: {error:?}"),
    //             }
    //         }
    //         "gc" => match functions::get_chat(commands[1].parse::<i64>().unwrap(), self.client_id)
    //             .await
    //         {
    //             Ok(chat) => println!("[GET CHAT]: {chat:?}"),
    //             Err(error) => eprintln!("[GET CHAT]: {error:?}"),
    //         },
    //         "me" => match functions::get_me(self.client_id).await {
    //             Ok(me) => println!("[GET ME]: {me:?}"),
    //             Err(error) => eprintln!("[GET ME]: {error:?}"),
    //         },
    //         "sm" => {
    //             println!("[DEBUG]: {commands:?}");
    //             // let args: Vec<&str> = commands[1].split(' ').collect();
    //             let text = enums::InputMessageContent::InputMessageText(InputMessageText {
    //                 text: FormattedText {
    //                     text: commands[2].into(),
    //                     entities: Vec::new(),
    //                 },
    //                 disable_web_page_preview: false,
    //                 clear_draft: true,
    //             });
    //             match functions::send_message(
    //                 commands[1].parse::<i64>().unwrap(),
    //                 0,
    //                 None,
    //                 None,
    //                 None,
    //                 text,
    //                 self.client_id,
    //             )
    //             .await
    //             {
    //                 Ok(me) => println!("[SEND MESSAGE]: {me:?}"),
    //                 Err(error) => eprintln!("[SEND MESSAGE]: {error:?}"),
    //             };
    //         }
    //         "lo" => {
    //             self.have_authorization = false;
    //             match functions::log_out(self.client_id).await {
    //                 Ok(me) => println!("[LOG OUT]: {me:?}"),
    //                 Err(error) => eprintln!("[LOG OUT]: {error:?}"),
    //             }
    //         }
    //         "q" => {
    //             self.need_quit = true;
    //             self.have_authorization = false;
    //             match functions::close(self.client_id).await {
    //                 Ok(me) => println!("[CLOSE]: {me:?}"),
    //                 Err(error) => eprintln!("[CLOSE]: {error:?}"),
    //             }
    //         }
    //         "mcl" => {
    //             let mcl = self.main_chat_list.lock().unwrap();
    //             let chats = self.chats.lock().unwrap();
    //
    //             for chat in mcl.iter() {
    //                 let c = chats.get(&chat.chat_id).unwrap();
    //                 let content = if let enums::MessageContent::MessageText(m) =
    //                     c.last_message.clone().unwrap().content
    //                 {
    //                     m.text.text
    //                 } else {
    //                     String::new()
    //                 };
    //                 println!(
    //                     "chat_id: {}, title: {}, last_message: {}",
    //                     chat.chat_id,
    //                     c.title,
    //                     content.split('\n').next().unwrap_or("")
    //                 );
    //             }
    //         }
    //         "h" => {
    //             let chat_id = commands[1].parse::<i64>().unwrap();
    //             match functions::get_chat_history(chat_id, 0, 0, 10, false, self.client_id).await {
    //                 Ok(enums::Messages::Messages(messages)) => {
    //                     for message in messages.messages.into_iter().flatten() {
    //                         let content =
    //                             if let enums::MessageContent::MessageText(m) = message.content {
    //                                 m.text.text
    //                             } else {
    //                                 String::new()
    //                             };
    //                         let sender_id = if let enums::MessageSender::User(u) = message.sender_id
    //                         {
    //                             u.user_id
    //                         } else {
    //                             0
    //                         };
    //                         println!("sender_id: {:?}, content: {:?}", sender_id, content,)
    //                     }
    //                 }
    //                 Err(error) => eprintln!("[GET CHAT HISTORY]: {error:?}"),
    //             }
    //         }
    //         _ => (),
    //     }
    // }

    pub fn new(app_context: Arc<AppContext>) -> Result<Self, std::io::Error> {
        let handle_updates = tokio::spawn(async {});

        let (auth_tx, auth_rx) = tokio::sync::mpsc::unbounded_channel::<AuthorizationState>();

        let client_id = tdlib::create_client();
        app_context.tg_context().set_client_id(client_id);

        // probably useless in real app
        let have_authorization = false;
        // probably useless in real app
        let need_quit = false;
        // probably useless in real app
        let can_quit = Arc::new(AtomicBool::new(false));

        Ok(Self {
            handle_updates,
            auth_tx,
            auth_rx,
            client_id,
            have_authorization,
            need_quit,
            can_quit,
            app_context,
        })
    }

    pub fn start(&mut self) {
        let auth_tx = self.auth_tx.clone();

        let can_quit = self.can_quit.clone();

        let tg_context = self.app_context.tg_context();

        self.handle_updates = tokio::spawn(async move {
            while !can_quit.load(Ordering::Acquire) {
                // TODO check that the client_ids are equal
                let mut update_dequeue: VecDeque<Update> = VecDeque::new();
                if let Some((update, _client_id)) = tdlib::receive() {
                    update_dequeue.push_back(update);
                    let update = update_dequeue.pop_front().unwrap();
                    match update.clone() {
                        Update::AuthorizationState(update) => {
                            auth_tx.send(update.authorization_state).unwrap();
                        }
                        Update::User(update_user) => {
                            tg_context.insert_user(update_user.user.id, update_user.user);
                        }
                        Update::UserStatus(update_user) => {
                            match tg_context.users().get_mut(&update_user.user_id) {
                                Some(user) => {
                                    user.status = update_user.status;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::BasicGroup(update_basic_group) => {
                            tg_context.basic_groups().insert(
                                update_basic_group.basic_group.id,
                                update_basic_group.basic_group,
                            );
                        }
                        Update::Supergroup(update_supergroup) => {
                            tg_context.supergroups().insert(
                                update_supergroup.supergroup.id,
                                update_supergroup.supergroup,
                            );
                        }
                        Update::SecretChat(update_secret_chat) => {
                            tg_context.secret_chats().insert(
                                update_secret_chat.secret_chat.id,
                                update_secret_chat.secret_chat,
                            );
                        }
                        Update::NewChat(update_new_chat) => {
                            let mut chat = update_new_chat.chat;

                            tg_context.chats().insert(chat.id, chat.clone());

                            let positions = chat.positions;

                            chat.positions = Vec::new();

                            Self::set_chat_positions(
                                tg_context.main_chat_list(),
                                &mut chat,
                                positions,
                            );
                        }
                        Update::ChatTitle(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => chat.title = update_chat.title,
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatPhoto(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => chat.photo = update_chat.photo,
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatPermissions(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => chat.permissions = update_chat.permissions,
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatLastMessage(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.last_message = update_chat.last_message;

                                    Self::set_chat_positions(
                                        tg_context.main_chat_list(),
                                        chat,
                                        update_chat.positions,
                                    );
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatPosition(update_chat) => {
                            if let enums::ChatList::Main = update_chat.position.list {
                                match tg_context.chats().get_mut(&update_chat.chat_id) {
                                    Some(chat) => {
                                        let mut i = 0;

                                        for p in &chat.positions {
                                            if let enums::ChatList::Main = p.list {
                                                break;
                                            }
                                            i += 1;
                                        }
                                        let mut new_position: Vec<ChatPosition> = Vec::new();
                                        let mut pos = 0;
                                        if update_chat.position.order != 0 {
                                            new_position.insert(pos, update_chat.position);
                                            pos += 1;
                                        }
                                        for j in 0..chat.positions.len() {
                                            if j != i {
                                                new_position.insert(pos, chat.positions[j].clone());
                                                pos += 1;
                                            }
                                        }
                                        assert!(pos == new_position.len());

                                        Self::set_chat_positions(
                                            tg_context.main_chat_list(),
                                            chat,
                                            new_position,
                                        );
                                    }
                                    None => update_dequeue.push_back(update),
                                }
                            }
                        }
                        Update::ChatReadInbox(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.last_read_inbox_message_id =
                                        update_chat.last_read_inbox_message_id;
                                    chat.unread_count = update_chat.unread_count;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatReadOutbox(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.last_read_outbox_message_id =
                                        update_chat.last_read_outbox_message_id;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatActionBar(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.action_bar = update_chat.action_bar;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatAvailableReactions(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.available_reactions = update_chat.available_reactions;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatUnreadMentionCount(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.unread_mention_count = update_chat.unread_mention_count;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::MessageMentionRead(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.unread_mention_count = update_chat.unread_mention_count;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatReplyMarkup(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.reply_markup_message_id =
                                        update_chat.reply_markup_message_id;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatDraftMessage(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.draft_message = update_chat.draft_message;
                                    Self::set_chat_positions(
                                        tg_context.main_chat_list(),
                                        chat,
                                        update_chat.positions,
                                    );
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatMessageSender(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.message_sender_id = update_chat.message_sender_id;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatMessageAutoDeleteTime(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.message_auto_delete_time =
                                        update_chat.message_auto_delete_time;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatNotificationSettings(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.notification_settings = update_chat.notification_settings;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatPendingJoinRequests(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.pending_join_requests = update_chat.pending_join_requests;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatBackground(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.background = update_chat.background;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatTheme(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.theme_name = update_chat.theme_name;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatUnreadReactionCount(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.unread_reaction_count = update_chat.unread_reaction_count;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatDefaultDisableNotification(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.default_disable_notification =
                                        update_chat.default_disable_notification;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatIsMarkedAsUnread(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.is_marked_as_unread = update_chat.is_marked_as_unread;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatBlockList(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => chat.block_list = update_chat.block_list,
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatHasScheduledMessages(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.has_scheduled_messages =
                                        update_chat.has_scheduled_messages;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::MessageUnreadReactions(update_chat) => {
                            match tg_context.chats().get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.unread_mention_count = update_chat.unread_reaction_count;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::UserFullInfo(update_user_full_info) => {
                            tg_context.users_full_info().insert(
                                update_user_full_info.user_id,
                                update_user_full_info.user_full_info,
                            );
                        }
                        Update::BasicGroupFullInfo(update_basic_group_full_info) => {
                            tg_context.basic_groups_full_info().insert(
                                update_basic_group_full_info.basic_group_id,
                                update_basic_group_full_info.basic_group_full_info,
                            );
                        }
                        Update::SupergroupFullInfo(update_supergroup_full_info) => {
                            tg_context.supergroups_full_info().insert(
                                update_supergroup_full_info.supergroup_id,
                                update_supergroup_full_info.supergroup_full_info,
                            );
                        }
                        // Too much prints
                        // _ => eprintln!("[HANDLE UPDATE]: {update:?}"),
                        _ => {}
                    }
                }
            }
        });
    }

    pub async fn handle_authorization_state(&mut self) {
        while let Some(state) = self.auth_rx.recv().await {
            match state {
                AuthorizationState::WaitTdlibParameters => {
                    let response = functions::set_tdlib_parameters(
                        false,
                        ".data/tg".into(),
                        String::new(),
                        String::new(),
                        false,
                        false,
                        false,
                        false,
                        env!("API_ID").parse().unwrap(),
                        env!("API_HASH").into(),
                        "en".into(),
                        "Desktop".into(),
                        String::new(),
                        env!("CARGO_PKG_VERSION").into(),
                        false,
                        true,
                        self.client_id,
                    )
                    .await;

                    if let Err(error) = response {
                        println!("{}", error.message);
                    }
                }
                AuthorizationState::WaitPhoneNumber => loop {
                    let phone_number =
                        ask_user("Enter your phone number (include the country calling code):");
                    let response = functions::set_authentication_phone_number(
                        phone_number,
                        None,
                        self.client_id,
                    )
                    .await;
                    match response {
                        Ok(_) => break,
                        Err(e) => println!("{}", e.message),
                    }
                },
                AuthorizationState::WaitOtherDeviceConfirmation(x) => {
                    println!(
                        "Please confirm this login link on another device: {}",
                        x.link
                    );
                }
                AuthorizationState::WaitEmailAddress(_x) => {
                    let email_address = ask_user("Please enter email address: ");
                    let response =
                        functions::set_authentication_email_address(email_address, self.client_id)
                            .await;
                    match response {
                        Ok(_) => break,
                        Err(e) => println!("{}", e.message),
                    }
                }
                AuthorizationState::WaitEmailCode(_x) => {
                    let code = ask_user("Please enter email authentication code: ");
                    let response = functions::check_authentication_email_code(
                        enums::EmailAddressAuthentication::Code(
                            tdlib::types::EmailAddressAuthenticationCode { code },
                        ),
                        self.client_id,
                    )
                    .await;
                    match response {
                        Ok(_) => break,
                        Err(e) => println!("{}", e.message),
                    }
                }
                AuthorizationState::WaitCode(_x) => loop {
                    // x contains info about verification code
                    let code = ask_user("Enter the verification code:");
                    let response = functions::check_authentication_code(code, self.client_id).await;
                    match response {
                        Ok(_) => break,
                        Err(e) => println!("{}", e.message),
                    }
                },
                AuthorizationState::WaitRegistration(_x) => {
                    // x useless but contains the TOS if we want to show it
                    let first_name = ask_user("Please enter your first name: ");
                    let last_name = ask_user("Please enter your last name: ");
                    functions::register_user(first_name, last_name, self.client_id)
                        .await
                        .unwrap();
                }
                AuthorizationState::WaitPassword(_x) => {
                    let password = ask_user("Please enter password: ");
                    functions::check_authentication_password(password, self.client_id)
                        .await
                        .unwrap();
                }
                AuthorizationState::Ready => {
                    // Maybe block all until this state is reached
                    self.have_authorization = true;
                    break;
                }
                AuthorizationState::LoggingOut => {
                    self.have_authorization = false;
                    println!("[HANDLE AUTH]: Logging out");
                }
                AuthorizationState::Closing => {
                    self.have_authorization = false;
                    println!("[HANDLE AUTH]: Closing");
                }
                AuthorizationState::Closed => {
                    println!("[HANDLE AUTH]: Closed");
                    // Set the flag to false to stop receiving updates
                    // from the spawned task
                    if self.need_quit {
                        self.can_quit.store(true, Ordering::Release);
                    }
                    break;
                }
            }
        }
    }

    fn set_chat_positions(
        mut main_chat_list: MutexGuard<'_, BTreeSet<OrderedChat>>,
        chat: &mut Chat,
        positions: Vec<ChatPosition>,
    ) {
        for position in &chat.positions {
            if let enums::ChatList::Main = position.list {
                let is_removed = main_chat_list.remove(&OrderedChat {
                    position: position.clone(),
                    chat_id: chat.id,
                });
                assert!(is_removed); // Too much
            }
        }

        chat.positions = positions;

        for position in &chat.positions {
            if let enums::ChatList::Main = position.list {
                let is_inserted = main_chat_list.insert(OrderedChat {
                    position: position.clone(),
                    chat_id: chat.id,
                });
                assert!(is_inserted); // Too much
            }
        }
    }
    pub async fn set_logging(&self) {
        // TODO read data from config file

        // Set a fairly low verbosity level. We mainly do this because tdlib
        // requires to perform a random request with the client to start
        // receiving updates for it.
        functions::set_log_verbosity_level(2, self.client_id)
            .await
            .unwrap();

        // Create log file
        let log_stream_file = LogStreamFile {
            path: ".data/tdlib.log".into(),
            max_file_size: 1 << 27,
            redirect_stderr: false,
        };

        // Set log stream to file
        if let Err(error) =
            functions::set_log_stream(LogStream::File(log_stream_file), self.client_id).await
        {
            eprintln!("[ERROR] \"Write access to the current directory is required\": {error:?}")
        }
    }
}

fn ask_user(string: &str) -> String {
    println!("{}", string);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
