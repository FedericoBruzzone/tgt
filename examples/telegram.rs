// Run it with `cargo run --bin telegram`
// telegram getting started -> https://core.telegram.org/tdlib_rs/getting-started
// tdlib_rs rust docs -> https://docs.rs/tdlib_rs/latest/tdlib_rs/
// tdlib_rs telegram docs -> https://core.telegram.org/tdlib_rs/docs/
// java example -> https://github.com/tdlib_rs/td/blob/master/example/java/org/drinkless/tdlib_rs/example/Example.java

use {
    std::{
        collections::{BTreeSet, HashMap, VecDeque},
        hash::Hash,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex,
        },
    },
    tdlib_rs::{
        enums::{self, AuthorizationState, LogStream, Update},
        functions,
        types::{
            BasicGroup, BasicGroupFullInfo, Chat, ChatPosition, FormattedText, InputMessageText,
            LogStreamFile, SecretChat, Supergroup, SupergroupFullInfo, User, UserFullInfo,
        },
    },
    tokio::{
        sync::mpsc::{UnboundedReceiver, UnboundedSender},
        task::JoinHandle,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct OrderedChat {
    pub chat_id: i64,
    pub position: ChatPosition, // maybe can be changed with position.order
}

impl Hash for OrderedChat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.chat_id.hash(state);

        // self.position.hash(state);
        format!("{:?}", self.position.list).hash(state);
        self.position.order.hash(state);
        self.position.is_pinned.hash(state);
        format!("{:?}", self.position.source).hash(state);
    }
}

impl Eq for OrderedChat {}

impl PartialOrd for OrderedChat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedChat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.position.order != other.position.order {
            if self.position.order > other.position.order {
                return core::cmp::Ordering::Less;
            } else {
                return core::cmp::Ordering::Greater;
            }
        }
        if self.chat_id != other.chat_id {
            if self.chat_id > other.chat_id {
                return core::cmp::Ordering::Less;
            } else {
                return core::cmp::Ordering::Greater;
            }
        }
        core::cmp::Ordering::Equal
    }
}

pub struct TgBackend {
    // thread for receiving updates from tdlib_rs
    pub handle_updates: JoinHandle<()>,

    pub auth_rx: UnboundedReceiver<AuthorizationState>,

    pub auth_tx: UnboundedSender<AuthorizationState>,

    // TODO need thread to receive action/events from app
    pub client_id: i32,

    pub have_authorization: bool,
    pub need_quit: bool,
    pub can_quit: Arc<AtomicBool>,

    pub users: Arc<Mutex<HashMap<i64, User>>>,
    pub basic_groups: Arc<Mutex<HashMap<i64, BasicGroup>>>,
    pub supergroups: Arc<Mutex<HashMap<i64, Supergroup>>>,
    pub secret_chats: Arc<Mutex<HashMap<i32, SecretChat>>>,

    pub chats: Arc<Mutex<HashMap<i64, Chat>>>,
    pub main_chat_list: Arc<Mutex<BTreeSet<OrderedChat>>>,
    pub have_full_main_chat_list: bool,

    pub users_full_info: Arc<Mutex<HashMap<i64, UserFullInfo>>>,
    pub basic_groups_full_info: Arc<Mutex<HashMap<i64, BasicGroupFullInfo>>>,
    pub supergroups_full_info: Arc<Mutex<HashMap<i64, SupergroupFullInfo>>>,
}

impl TgBackend {
    pub fn new() -> Result<Self, std::io::Error> {
        let handle_updates = tokio::spawn(async {});

        let (auth_tx, auth_rx) = tokio::sync::mpsc::unbounded_channel::<AuthorizationState>();

        let client_id = tdlib_rs::create_client();

        // probably useless in real app
        let have_authorization = false;
        // probably useless in real app
        let need_quit = false;
        // probably useless in real app
        let can_quit = Arc::new(AtomicBool::new(false));

        // maybe all datastructures needs to be thread safe
        let users: Arc<Mutex<HashMap<i64, User>>> = Arc::new(Mutex::new(HashMap::new()));
        let basic_groups: Arc<Mutex<HashMap<i64, BasicGroup>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let supergroups: Arc<Mutex<HashMap<i64, Supergroup>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let secret_chats: Arc<Mutex<HashMap<i32, SecretChat>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let chats: Arc<Mutex<HashMap<i64, Chat>>> = Arc::new(Mutex::new(HashMap::new()));
        let main_chat_list: Arc<Mutex<BTreeSet<OrderedChat>>> =
            Arc::new(Mutex::new(BTreeSet::new()));
        let have_full_main_chat_list = false;

        let users_full_info: Arc<Mutex<HashMap<i64, UserFullInfo>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let basic_groups_full_info: Arc<Mutex<HashMap<i64, BasicGroupFullInfo>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let supergroups_full_info: Arc<Mutex<HashMap<i64, SupergroupFullInfo>>> =
            Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            handle_updates,
            auth_tx,
            auth_rx,
            client_id,
            have_authorization,
            need_quit,
            can_quit,
            users,
            basic_groups,
            supergroups,
            secret_chats,
            chats,
            main_chat_list,
            have_full_main_chat_list,
            users_full_info,
            basic_groups_full_info,
            supergroups_full_info,
        })
    }

    pub fn start(&mut self) {
        let auth_tx = self.auth_tx.clone();

        let can_quit = self.can_quit.clone();

        let users = self.users.clone();
        let basic_groups = self.basic_groups.clone();
        let supergroups = self.supergroups.clone();
        let secret_chats = self.secret_chats.clone();

        let chats = self.chats.clone();
        let main_chat_list = self.main_chat_list.clone();

        let users_full_info = self.users_full_info.clone();
        let basic_groups_full_info = self.basic_groups_full_info.clone();
        let supergroups_full_info = self.supergroups_full_info.clone();

        self.handle_updates = tokio::spawn(async move {
            while !can_quit.load(Ordering::Acquire) {
                // TODO check that the client_ids are equal
                let mut update_dequeue: VecDeque<Update> = VecDeque::new();
                if let Some((update, _client_id)) = tdlib_rs::receive() {
                    update_dequeue.push_back(update);
                    let update = update_dequeue.pop_front().unwrap();
                    match update.clone() {
                        Update::AuthorizationState(update) => {
                            auth_tx.send(update.authorization_state).unwrap();
                        }
                        Update::User(update_user) => {
                            // eprintln!(
                            //     "[USER UPDATE]: {:?} {:?}",
                            //     update_user.user.usernames,
                            // update_user.user.id
                            // );

                            users
                                .lock()
                                .unwrap()
                                .insert(update_user.user.id, update_user.user);
                        }
                        Update::UserStatus(update_user) => {
                            let mut _users = users.lock().unwrap();
                            match _users.get_mut(&update_user.user_id) {
                                Some(user) => {
                                    user.status = update_user.status;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::BasicGroup(update_basic_group) => {
                            basic_groups.lock().unwrap().insert(
                                update_basic_group.basic_group.id,
                                update_basic_group.basic_group,
                            );
                        }
                        Update::Supergroup(update_supergroup) => {
                            supergroups.lock().unwrap().insert(
                                update_supergroup.supergroup.id,
                                update_supergroup.supergroup,
                            );
                        }
                        Update::SecretChat(update_secret_chat) => {
                            secret_chats.lock().unwrap().insert(
                                update_secret_chat.secret_chat.id,
                                update_secret_chat.secret_chat,
                            );
                        }
                        Update::NewChat(update_new_chat) => {
                            let mut chat = update_new_chat.chat;

                            let mut _chats = chats.lock().unwrap();
                            _chats.insert(chat.id, chat.clone());

                            let positions = chat.positions;

                            chat.positions = Vec::new();

                            Self::set_chat_positions(main_chat_list.clone(), &mut chat, positions);
                        }
                        Update::ChatTitle(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => chat.title = update_chat.title,
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatPhoto(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => chat.photo = update_chat.photo,
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatPermissions(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => chat.permissions = update_chat.permissions,
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatLastMessage(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.last_message = update_chat.last_message;

                                    Self::set_chat_positions(
                                        main_chat_list.clone(),
                                        chat,
                                        update_chat.positions,
                                    );
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatPosition(update_chat) => {
                            if let enums::ChatList::Main = update_chat.position.list {
                                let mut _chats = chats.lock().unwrap();

                                match _chats.get_mut(&update_chat.chat_id) {
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
                                            main_chat_list.clone(),
                                            chat,
                                            new_position,
                                        );
                                    }
                                    None => update_dequeue.push_back(update),
                                }
                            }
                        }
                        Update::ChatReadInbox(update_chat) => {
                            let mut _chats = chats.lock().unwrap();

                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.last_read_inbox_message_id =
                                        update_chat.last_read_inbox_message_id;
                                    chat.unread_count = update_chat.unread_count;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatReadOutbox(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.last_read_outbox_message_id =
                                        update_chat.last_read_outbox_message_id;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatActionBar(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.action_bar = update_chat.action_bar;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatAvailableReactions(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.available_reactions = update_chat.available_reactions;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatUnreadMentionCount(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.unread_mention_count = update_chat.unread_mention_count;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::MessageMentionRead(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.unread_mention_count = update_chat.unread_mention_count;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatReplyMarkup(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.reply_markup_message_id =
                                        update_chat.reply_markup_message_id;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatDraftMessage(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.draft_message = update_chat.draft_message;
                                    Self::set_chat_positions(
                                        main_chat_list.clone(),
                                        chat,
                                        update_chat.positions,
                                    );
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatMessageSender(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.message_sender_id = update_chat.message_sender_id;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatMessageAutoDeleteTime(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.message_auto_delete_time =
                                        update_chat.message_auto_delete_time;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatNotificationSettings(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.notification_settings = update_chat.notification_settings;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatPendingJoinRequests(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.pending_join_requests = update_chat.pending_join_requests;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatBackground(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.background = update_chat.background;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatTheme(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.theme_name = update_chat.theme_name;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatUnreadReactionCount(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.unread_reaction_count = update_chat.unread_reaction_count;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatDefaultDisableNotification(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.default_disable_notification =
                                        update_chat.default_disable_notification;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatIsMarkedAsUnread(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.is_marked_as_unread = update_chat.is_marked_as_unread;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatBlockList(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => chat.block_list = update_chat.block_list,
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::ChatHasScheduledMessages(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.has_scheduled_messages =
                                        update_chat.has_scheduled_messages;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::MessageUnreadReactions(update_chat) => {
                            let mut _chats = chats.lock().unwrap();
                            match _chats.get_mut(&update_chat.chat_id) {
                                Some(chat) => {
                                    chat.unread_mention_count = update_chat.unread_reaction_count;
                                }
                                None => update_dequeue.push_back(update),
                            }
                        }
                        Update::UserFullInfo(update_user_full_info) => {
                            users_full_info.lock().unwrap().insert(
                                update_user_full_info.user_id,
                                update_user_full_info.user_full_info,
                            );
                        }
                        Update::BasicGroupFullInfo(update_basic_group_full_info) => {
                            basic_groups_full_info.lock().unwrap().insert(
                                update_basic_group_full_info.basic_group_id,
                                update_basic_group_full_info.basic_group_full_info,
                            );
                        }
                        Update::SupergroupFullInfo(update_supergroup_full_info) => {
                            supergroups_full_info.lock().unwrap().insert(
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

    async fn handle_authorization_state(&mut self) {
        let api_id: i32 = {
            // `env!("API_ID").parse().unwrap()` generates a compile time error
            if let Ok(api_id) = std::env::var("API_ID") {
                api_id.parse().unwrap()
            } else {
                tracing::error!("API_ID not found in environment");
                "94575".parse().unwrap() // This will throw the tdlib-rs error message
            }
        };
        let api_hash: String = {
            // `env!("API_HASH").into()` generates a compile time error
            if let Ok(api_hash) = std::env::var("API_HASH") {
                api_hash
            } else {
                "a3406de8d171bb422bb6ddf3bbd800e2".into() // This will throw the tdlib-rs error message
            }
        };
        while let Some(state) = self.auth_rx.recv().await {
            match state {
                AuthorizationState::WaitTdlibParameters => {
                    let response = functions::set_tdlib_parameters(
                        false,
                        ".data/example".into(),
                        String::new(),
                        String::new(),
                        false,
                        false,
                        false,
                        false,
                        api_id,
                        api_hash.clone(),
                        "en".into(),
                        "Desktop".into(),
                        String::new(),
                        env!("CARGO_PKG_VERSION").into(),
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
                            tdlib_rs::types::EmailAddressAuthenticationCode { code },
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
                    functions::register_user(first_name, last_name, false, self.client_id)
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
        main_chat_list: Arc<Mutex<BTreeSet<OrderedChat>>>,
        chat: &mut Chat,
        positions: Vec<ChatPosition>,
    ) {
        let mut main_chat_list = main_chat_list.lock().unwrap();

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

    async fn get_command(&mut self) {
        let command = ask_user("Enter command (gcs - GetChats, gc <chatId> - GetChat, me - GetMe, sm <chatId> <message> - SendMessage, lo - LogOut, q - Quit, mcl - MainChatList, h <chatId> - GetChatHistory): ");
        let commands: Vec<&str> = command.split(' ').collect();
        match commands[0] {
            "gcs" => {
                let mut limit = 20;
                if commands.len() > 1 {
                    limit = commands[1].parse::<i32>().unwrap();
                }
                match functions::load_chats(Some(enums::ChatList::Main), limit, self.client_id)
                    .await
                {
                    Ok(()) => (),
                    Err(error) => eprintln!("[GET MAIN CHAT LIST]: {error:?}"),
                }
            }
            "gc" => match functions::get_chat(commands[1].parse::<i64>().unwrap(), self.client_id)
                .await
            {
                Ok(chat) => println!("[GET CHAT]: {chat:?}"),
                Err(error) => eprintln!("[GET CHAT]: {error:?}"),
            },
            "me" => match functions::get_me(self.client_id).await {
                Ok(me) => println!("[GET ME]: {me:?}"),
                Err(error) => eprintln!("[GET ME]: {error:?}"),
            },
            "sm" => {
                println!("[DEBUG]: {commands:?}");
                // let args: Vec<&str> = commands[1].split(' ').collect();
                let text = enums::InputMessageContent::InputMessageText(InputMessageText {
                    text: FormattedText {
                        text: commands[2].into(),
                        entities: Vec::new(),
                    },
                    link_preview_options: None,
                    clear_draft: true,
                });
                match functions::send_message(
                    commands[1].parse::<i64>().unwrap(),
                    0,
                    None,
                    None,
                    text,
                    self.client_id,
                )
                .await
                {
                    Ok(me) => println!("[SEND MESSAGE]: {me:?}"),
                    Err(error) => eprintln!("[SEND MESSAGE]: {error:?}"),
                };
            }
            "lo" => {
                self.have_authorization = false;
                match functions::log_out(self.client_id).await {
                    Ok(me) => println!("[LOG OUT]: {me:?}"),
                    Err(error) => eprintln!("[LOG OUT]: {error:?}"),
                }
            }
            "q" => {
                self.need_quit = true;
                self.have_authorization = false;
                match functions::close(self.client_id).await {
                    Ok(me) => println!("[CLOSE]: {me:?}"),
                    Err(error) => eprintln!("[CLOSE]: {error:?}"),
                }
            }
            "mcl" => {
                let mcl = self.main_chat_list.lock().unwrap();
                let chats = self.chats.lock().unwrap();

                for chat in mcl.iter() {
                    let c = chats.get(&chat.chat_id).unwrap();
                    let content = if let enums::MessageContent::MessageText(m) =
                        c.last_message.clone().unwrap().content
                    {
                        m.text.text
                    } else {
                        String::new()
                    };
                    println!(
                        "chat_id: {}, title: {}, last_message: {}",
                        chat.chat_id,
                        c.title,
                        content.split('\n').next().unwrap_or("")
                    );
                }
            }
            "h" => {
                let chat_id = commands[1].parse::<i64>().unwrap();
                match functions::get_chat_history(chat_id, 0, 0, 10, false, self.client_id).await {
                    Ok(enums::Messages::Messages(messages)) => {
                        for message in messages.messages.into_iter().flatten() {
                            let content =
                                if let enums::MessageContent::MessageText(m) = message.content {
                                    m.text.text
                                } else {
                                    String::new()
                                };
                            let sender_id = if let enums::MessageSender::User(u) = message.sender_id
                            {
                                u.user_id
                            } else {
                                0
                            };
                            println!("sender_id: {sender_id:?}, content: {content:?}")
                        }
                    }
                    Err(error) => eprintln!("[GET CHAT HISTORY]: {error:?}"),
                }
            }
            _ => (),
        }
    }

    pub async fn set_logging(&self) {
        // TODO read data from config file

        // Set a fairly low verbosity level. We mainly do this because tdlib_rs
        // requires to perform a random request with the client to start
        // receiving updates for it.
        functions::set_log_verbosity_level(2, self.client_id)
            .await
            .unwrap();

        // Create log file
        let log_stream_file = LogStreamFile {
            path: ".data/tdlib_rs.log".into(),
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
    println!("{string}");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

#[tokio::main]
async fn main() {
    // Create the client object
    let mut tg_backend = TgBackend::new().unwrap();

    // Spawn a task to receive updates/responses
    tg_backend.start();

    // Do the first request to start receiving updates
    tg_backend.set_logging().await;

    while !tg_backend.need_quit {
        while tg_backend.have_authorization {
            tg_backend.get_command().await;
        }
        tg_backend.handle_authorization_state().await;
    }
}
