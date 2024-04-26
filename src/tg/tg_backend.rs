use crate::event::Event;
use crate::{app_context::AppContext, tg::ordered_chat::OrderedChat};
use std::collections::{BTreeSet, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, MutexGuard};
use tdlib::enums::{self, AuthorizationState, ChatList, LogStream, Update};
use tdlib::functions;
use tdlib::types::{Chat, ChatPosition, LogStreamFile};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

pub struct TgBackend {
    pub handle_updates: JoinHandle<()>,
    pub auth_rx: UnboundedReceiver<AuthorizationState>,
    pub auth_tx: UnboundedSender<AuthorizationState>,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
    pub client_id: i32,
    pub have_authorization: bool,
    pub need_quit: bool,
    pub can_quit: Arc<AtomicBool>,
    pub app_context: Arc<AppContext>,
    full_chats_list: bool,
}

impl TgBackend {
    pub fn new(app_context: Arc<AppContext>) -> Result<Self, std::io::Error> {
        tracing::info!("Creating TgBackend");
        let handle_updates = tokio::spawn(async {});
        let (auth_tx, auth_rx) = tokio::sync::mpsc::unbounded_channel::<AuthorizationState>();
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
        let client_id = tdlib::create_client();
        let have_authorization = false;
        let need_quit = false;
        let can_quit = Arc::new(AtomicBool::new(false));
        let full_chats_list = false;
        app_context.tg_context().set_event_tx(event_tx.clone());
        tracing::info!("Created TDLib client with client_id: {}", client_id);

        Ok(Self {
            handle_updates,
            auth_tx,
            auth_rx,
            event_tx,
            event_rx,
            client_id,
            have_authorization,
            need_quit,
            can_quit,
            app_context,
            full_chats_list,
        })
    }

    pub async fn load_chats(&mut self, chat_list: ChatList, limit: i32) {
        if self.full_chats_list {
            return;
        }

        if let Err(e) = functions::load_chats(Some(chat_list), limit, self.client_id).await {
            tracing::error!("Failed to load chats: {e:?}");
            self.full_chats_list = true;
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.try_recv().ok()
    }

    pub fn start(&mut self) {
        let auth_tx = self.auth_tx.clone();
        let can_quit = self.can_quit.clone();
        let tg_context = self.app_context.tg_context();

        self.handle_updates = tokio::spawn(async move {
            while !can_quit.load(Ordering::Acquire) {
                let mut update_dequeue: VecDeque<Update> = VecDeque::new();
                if let Some((update, _client_id)) = tdlib::receive() {
                    update_dequeue.push_back(update);
                    let update = update_dequeue.pop_front().unwrap();
                    match update.clone() {
                        Update::AuthorizationState(update) => {
                            auth_tx.send(update.authorization_state).unwrap();
                        }
                        Update::User(update_user) => {
                            tg_context
                                .users()
                                .insert(update_user.user.id, update_user.user);
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
                                tg_context.chats_index(),
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
                                        tg_context.chats_index(),
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
                                            tg_context.chats_index(),
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
                                        tg_context.chats_index(),
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
                    tracing::info!("Logging out");
                }
                AuthorizationState::Closing => {
                    self.have_authorization = false;
                    tracing::info!("Closing");
                }
                AuthorizationState::Closed => {
                    tracing::info!("Closed");
                    if self.need_quit {
                        self.can_quit.store(true, Ordering::Release);
                    }
                    break;
                }
            }
        }
    }

    fn set_chat_positions(
        mut chats_index: MutexGuard<'_, BTreeSet<OrderedChat>>,
        chat: &mut Chat,
        positions: Vec<ChatPosition>,
    ) {
        for position in &chat.positions {
            if let enums::ChatList::Main = position.list {
                let is_removed = chats_index.remove(&OrderedChat {
                    position: position.clone(),
                    chat_id: chat.id,
                });
                assert!(is_removed);
            }
        }

        chat.positions = positions;

        for position in &chat.positions {
            if let enums::ChatList::Main = position.list {
                let is_inserted = chats_index.insert(OrderedChat {
                    position: position.clone(),
                    chat_id: chat.id,
                });
                assert!(is_inserted);
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
            tracing::error!("Failed to set log stream to file: {error:?}");
        }
    }
}

fn ask_user(string: &str) -> String {
    println!("{}", string);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
