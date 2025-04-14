use crate::action::{Action, MessageEdited, SendMessageResult};
use crate::event::Event;
use crate::{app_context::AppContext, tg::ordered_chat::OrderedChat};
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, MutexGuard};
use tdlib_rs::enums::{
    self, AuthorizationState, ChatList, InputMessageContent, InputMessageReplyTo, LogStream,
    Messages, OptionValue, Update, User,
};
use tdlib_rs::functions;
use tdlib_rs::types::{
    BasicGroup, BasicGroupFullInfo, Chat, ChatPosition, InputMessageText, LogStreamFile,
    OptionValueBoolean, SecretChat, Supergroup, SupergroupFullInfo, UserFullInfo,
};
use tokio::spawn;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use super::message_entry::MessageEntry;
use super::td_enums::TdMessageReplyToMessage;

const QUERY_MESSAGE_WIN_SIZE: i32 = 100;

pub struct TgBackendOld {
    pub handle_updates: JoinHandle<()>,
    pub auth_rx: UnboundedReceiver<AuthorizationState>,
    pub auth_tx: UnboundedSender<AuthorizationState>,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
    pub client_id: i32,
    pub have_authorization: bool,
    pub can_quit: Arc<AtomicBool>,
    pub app_context: Rc<AppContext>,
    full_chats_list: bool,
}

// NOTE: To improve performance, requests to update the telegram state should
// only be received through a channel so that they may be handled in their
// own thread. Instead the only functions that should be exposed are getters,
// that way the UI can draw the data that it currently has while the rest of
// the data is pulled in the background. Upon receiving new data the backend
// should send a signal to the frontend that new data is available so that it
// may redraw with the new data.
impl TgBackendOld {
    pub fn new(app_context: Rc<AppContext>) -> Result<Self, std::io::Error> {
        tracing::info!("Creating TgBackend");
        let handle_updates = spawn(async {});
        let (auth_tx, auth_rx) = unbounded_channel::<AuthorizationState>();
        let (event_tx, event_rx) = unbounded_channel::<Event>();
        let client_id = tdlib_rs::create_client();
        let have_authorization = false;
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
            can_quit,
            app_context,
            full_chats_list,
        })
    }

    pub async fn get_me(&mut self) {
        match functions::get_me(self.client_id).await {
            Ok(User::User(me)) => {
                self.app_context.tg_context().set_me(me.id);
            }
            Err(error) => tracing::error!("Failed to get me: {error:?}"),
        }
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

    // Used only in CLI
    pub async fn load_all_chats(&mut self) {
        while !self.full_chats_list {
            self.load_chats(ChatList::Main, 50).await;
        }
    }

    // Used only in CLI
    pub async fn search_chats(
        &self,
        username: String,
    ) -> Result<tdlib_rs::types::Chats, tdlib_rs::types::Error> {
        match functions::search_chats(username.clone(), 50, self.client_id).await {
            Ok(tdlib_rs::enums::Chats::Chats(chats)) => Ok(chats),
            Err(e) => {
                tracing::error!("Failed to search chats: {:?} with query: {}", e, username);
                Err(e)
            }
        }
    }

    pub async fn close(&self) {
        match functions::close(self.client_id).await {
            Ok(me) => tracing::info!("TDLib client closed: {:?}", me),
            Err(error) => tracing::error!("Error closing TDLib client: {:?}", error),
        }
    }

    pub async fn view_all_messages(&self) {
        if let Err(e) = functions::view_messages(
            self.app_context.tg_context().open_chat_id(),
            self.app_context.tg_context().unread_messages(),
            None,
            true,
            self.client_id,
        )
        .await
        {
            tracing::error!("Failed to view all messages: {e:?}");
        }
    }

    #[allow(clippy::await_holding_lock)]
    // By default telegram send us only one message the first time
    // NOTE: Never used
    pub async fn prepare_to_get_chat_history(&mut self, chat_id: i64) {
        match functions::get_chat_history(chat_id, 0, 0, 100, false, self.client_id).await {
            Ok(_m) => {}
            Err(_e) => {}
        }
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn get_chat_history(&mut self, chat_id: i64) {
        let start_open_chat_messages_len = self.app_context.tg_context().open_chat_messages().len();
        let mut mut_open_chat_messages_len =
            self.app_context.tg_context().open_chat_messages().len();
        let win_size = 100;

        while mut_open_chat_messages_len < start_open_chat_messages_len + win_size {
            let from_message_id = self.app_context.tg_context().from_message_id();
            match functions::get_chat_history(
                chat_id,
                from_message_id,
                0,
                50,
                false,
                self.client_id,
            )
            .await
            {
                Ok(Messages::Messages(messages)) => {
                    if messages.messages.is_empty() {
                        tracing::info!("No more messages to get");
                        break;
                    }

                    let message_flatten = messages.messages.into_iter().flatten();
                    for message in message_flatten.clone() {
                        self.app_context
                            .tg_context()
                            .open_chat_messages()
                            .push(MessageEntry::from(&message));
                        mut_open_chat_messages_len += 1;
                    }
                    if let Some(message) = message_flatten.last() {
                        self.app_context
                            .tg_context()
                            .set_from_message_id(message.id);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to get chat history: {e:?}");
                    break;
                }
            }
        }
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn send_message(
        &mut self,
        message: String,
        chat_id: i64,
        reply_to: Option<TdMessageReplyToMessage>,
    ) -> Result<tdlib_rs::types::Message, tdlib_rs::types::Error> {
        self.app_context
            .tg_context()
            .set_reply_message(-1, "".to_string());

        self.app_context
            .action_tx()
            .send(Action::HideChatWindowReply)
            .unwrap();

        let text = InputMessageContent::InputMessageText(InputMessageText {
            text: tdlib_rs::types::FormattedText {
                text: message,
                entities: vec![], // TODO: Add entities
            },
            link_preview_options: None,
            clear_draft: true,
        });
        let reply_to: Option<InputMessageReplyTo> =
            reply_to.map(|reply_to| InputMessageReplyTo::Message((&reply_to).into()));
        match functions::send_message(chat_id, 0, reply_to, None, text, self.client_id).await {
            Ok(tdlib_rs::enums::Message::Message(message)) => Ok(message),
            Err(e) => {
                tracing::error!("Failed to send message: {e:?}");
                Err(e)
            }
        }
    }

    pub async fn send_message_edited(&self, message_id: i64, message: String) {
        let text = InputMessageContent::InputMessageText(InputMessageText {
            text: tdlib_rs::types::FormattedText {
                text: message,
                entities: vec![],
            },
            link_preview_options: None,
            clear_draft: true,
        });
        match functions::edit_message_text(
            self.app_context.tg_context().open_chat_id(),
            message_id,
            text,
            self.client_id,
        )
        .await
        {
            Ok(_) => tracing::info!("Message edited"),
            Err(e) => tracing::error!("Failed to edit message: {e:?}"),
        }
    }

    pub async fn delete_messages(&self, chat_id: i64, message_ids: Vec<i64>, revoke: bool) {
        match functions::delete_messages(chat_id, message_ids, revoke, self.client_id).await {
            Ok(_) => tracing::info!("Messages deleted"),
            Err(e) => tracing::error!("Failed to delete messages: {e:?}"),
        }
    }

    pub async fn use_quick_ack(&self) {
        match functions::set_option(
            String::from("use_quick_ack"),
            Some(OptionValue::Boolean(OptionValueBoolean { value: true })),
            self.client_id,
        )
        .await
        {
            Ok(_) => {
                tracing::info!("Quick ack enabled");
            }
            Err(e) => {
                tracing::error!("Failed to enable quick ack: {e:?}");
            }
        }
    }

    async fn set_online(&self, online: bool) -> Result<(), tdlib_rs::types::Error> {
        functions::set_option(
            String::from("online"),
            Some(OptionValue::Boolean(OptionValueBoolean { value: online })),
            self.client_id,
        )
        .await
    }

    pub async fn online(&mut self) {
        match self.set_online(true).await {
            Ok(_) => tracing::info!("Went online"),
            Err(error) => tracing::error!("Error going online: {error:?}"),
        }
    }
    pub async fn offline(&mut self) {
        match self.set_online(false).await {
            Ok(_) => tracing::info!("Went offline"),
            Err(error) => tracing::error!("Error going offline: {error:?}"),
        }
    }

    pub async fn disable_animated_emoji(&mut self, disable: bool) {
        match functions::set_option(
            String::from("disable_animated_emoji"),
            Some(OptionValue::Boolean(OptionValueBoolean { value: disable })),
            self.client_id,
        )
        .await
        {
            Ok(_) => {
                tracing::info!("Animated emoji set to: {}", disable);
            }
            Err(error) => {
                tracing::error!("Error setting animated emoji: {error:?}");
            }
        }
    }

    pub async fn log_out(&mut self) {
        match functions::log_out(self.client_id).await {
            Ok(_) => {
                tracing::info!("Logged out");
            }
            Err(error) => tracing::error!("Error logging out: {error:?}"),
        }
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn handle_authorization_state(&mut self) {
        tracing::info!("Handling authorization state");
        let telegram_config = self.app_context.telegram_config();
        let api_id: i32 = {
            if !self
                .app_context
                .app_config()
                .take_api_id_from_telegram_config
            {
                // `env!("API_ID").parse().unwrap()` generates a compile time error
                if let Ok(api_id) = std::env::var("API_ID") {
                    api_id.parse().unwrap()
                } else {
                    tracing::error!("API_ID not found in environment");
                    "-1".parse().unwrap() // This will throw the tdlib-rs error message
                }
            } else {
                telegram_config.api_id.parse().unwrap()
            }
        };
        let api_hash: String = {
            if !self
                .app_context
                .app_config()
                .take_api_hash_from_telegram_config
            {
                // `env!("API_HASH").into()` generates a compile time error
                if let Ok(api_hash) = std::env::var("API_HASH") {
                    api_hash
                } else {
                    tracing::error!("API_HASH not found in environment");
                    "".into() // This will throw the tdlib-rs error message
                }
            } else {
                telegram_config.api_hash.clone()
            }
        };
        let database_dir = telegram_config.database_dir.clone();
        let use_file_database = telegram_config.use_file_database;
        let use_chat_info_database = telegram_config.use_chat_info_database;
        let use_message_database = telegram_config.use_message_database;
        let system_language_code = telegram_config.system_language_code.clone();
        let device_model = telegram_config.device_model.clone();

        while let Some(state) = self.auth_rx.recv().await {
            match state {
                AuthorizationState::WaitTdlibParameters => {
                    let response = functions::set_tdlib_parameters(
                        false,
                        database_dir.clone(),
                        String::new(),
                        String::new(),
                        use_file_database,
                        use_chat_info_database,
                        use_message_database,
                        false,
                        api_id,
                        api_hash.clone(),
                        system_language_code.clone(),
                        device_model.clone(),
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
                    tracing::info!("Logging out");
                }
                AuthorizationState::Closing => {
                    self.have_authorization = false;
                    tracing::info!("Closing");
                }
                AuthorizationState::Closed => {
                    tracing::info!("Closed");
                    self.can_quit.store(true, Ordering::Release);
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
        let verbosity_level = self.app_context.telegram_config().verbosity_level;
        let log_path = self.app_context.telegram_config().log_path.clone();
        let redirect_stderr = self.app_context.telegram_config().redirect_stderr;

        // Set a fairly low verbosity level. We mainly do this because tdlib_rs
        // requires to perform a random request with the client to start
        // receiving updates for it.
        functions::set_log_verbosity_level(verbosity_level, self.client_id)
            .await
            .unwrap();

        // Create log file
        let log_stream_file = LogStreamFile {
            path: log_path,
            max_file_size: 1 << 27,
            redirect_stderr,
        };

        // Set log stream to file
        if let Err(error) =
            functions::set_log_stream(LogStream::File(log_stream_file), self.client_id).await
        {
            tracing::error!("Failed to set log stream to file: {error:?}");
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.try_recv().ok()
    }

    pub fn start(&mut self) {
        let auth_tx = self.auth_tx.clone();
        let can_quit = self.can_quit.clone();
        let tg_context = self.app_context.tg_context();

        self.handle_updates = spawn(async move {
            tracing::info!("Starting handling updates from TDLib");
            while !can_quit.load(Ordering::Acquire) {
                let mut update_dequeue: VecDeque<Update> = VecDeque::new();
                if let Some((update, _client_id)) = tdlib_rs::receive() {
                    update_dequeue.push_back(update);
                    let update = update_dequeue.pop_front().unwrap();
                    match update.clone() {
                        Update::MessageSendSucceeded(update_message) => {
                            tracing::info!("Message sent: {:?}", update_message);
                        }
                        Update::MessageSendAcknowledged(update_message) => {
                            tg_context.set_last_acknowledged_message_id(update_message.message_id);
                        }
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
                            // This update is received also when a message is edited
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
                        Update::NewMessage(update_new_message) => {
                            // New message update only the opened chat in order to have
                            // in real time the message displayed
                            let message = update_new_message.message;
                            let chat_id = message.chat_id;
                            if tg_context.open_chat_id() == chat_id {
                                tg_context
                                    .open_chat_messages()
                                    .insert(0, MessageEntry::from(&message));
                            }
                        }
                        Update::MessageEdited(_) => {}
                        Update::MessageContent(message) => {
                            if tg_context.open_chat_id() == message.chat_id {
                                for m in tg_context.open_chat_messages().iter_mut() {
                                    if m.id() == message.message_id {
                                        m.set_message_content(&message.new_content);
                                        m.set_is_edited(true);
                                    }
                                }
                            }
                        }
                        Update::DeleteMessages(update_delete_messages) => {
                            if tg_context.open_chat_id() == update_delete_messages.chat_id {
                                let mut i = 0;
                                while i < tg_context.open_chat_messages().len() {
                                    if update_delete_messages
                                        .message_ids
                                        .contains(&tg_context.open_chat_messages()[i].id())
                                    {
                                        tg_context.open_chat_messages().remove(i);
                                    } else {
                                        i += 1;
                                    }
                                }
                            }
                        }
                        // Update::Option(option) => {
                        //     tracing::info!("{:?}", option);
                        // }
                        // _ => eprintln!("[HANDLE UPDATE]: {update:?}"),
                        _ => {
                            // tracing::info!("Unhandled update: {:?}", update);
                        }
                    }
                }
            }
        });
    }
}

fn ask_user(string: &str) -> String {
    println!("{}", string);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub struct TgBackend {
    pub auth_rx: UnboundedReceiver<AuthorizationState>,
    pub auth_tx: UnboundedSender<AuthorizationState>,
    pub action_rx: UnboundedReceiver<Action>,
    pub action_tx: UnboundedSender<Action>,
    pub client_id: i32,
    pub have_authorization: bool,
    full_chats_list: RwLock<bool>,

    users: RwLock<HashMap<i64, User>>,
    basic_groups: RwLock<HashMap<i64, BasicGroup>>,
    supergroups: RwLock<HashMap<i64, Supergroup>>,
    secret_chats: RwLock<HashMap<i32, SecretChat>>,
    chats: RwLock<HashMap<i64, Chat>>,
    chats_index: RwLock<BTreeSet<OrderedChat>>,
    users_full_info: RwLock<HashMap<i64, UserFullInfo>>,
    basic_groups_full_info: RwLock<HashMap<i64, BasicGroupFullInfo>>,
    supergroups_full_info: RwLock<HashMap<i64, SupergroupFullInfo>>,
    me: RwLock<i64>,
    last_acknowledged_message_id: RwLock<i64>,
}

// TODO: Implement functions called only through the CLI interface
impl TgBackend {
    async fn new() -> Self {
        tracing::info!("Creating TgBackend");
        let (auth_tx, auth_rx) = unbounded_channel::<AuthorizationState>();
        let (event_tx, event_rx) = unbounded_channel::<Action>();
        let client_id = tdlib_rs::create_client();
        let have_authorization = false;
        let full_chats_list = RwLock::new(false);
        tracing::info!("Created TDLib client with client_id: {}", client_id);

        Self {
            auth_tx,
            auth_rx,
            action_tx: event_tx,
            action_rx: event_rx,
            client_id,
            have_authorization,
            full_chats_list,
            users: Default::default(),
            basic_groups: Default::default(),
            supergroups: Default::default(),
            secret_chats: Default::default(),
            chats: Default::default(),
            chats_index: Default::default(),
            users_full_info: Default::default(),
            basic_groups_full_info: Default::default(),
            supergroups_full_info: Default::default(),
            me: Default::default(),
            last_acknowledged_message_id: Default::default(),
        }
    }

    /// Need to be called in a spawned thread
    pub async fn start(&mut self) {
        match functions::get_me(self.client_id).await {
            Ok(User::User(me)) => {
                let mut m = self.me.write().await;
                *m = me.id;
            }
            Err(error) => tracing::error!("Failed to get me: {error:?}"),
        }

        self.process().await;
    }

    async fn process(&mut self) {
        while let Some(ev) = self.action_rx.recv().await {
            match ev {
                Action::LoadChats(chat_list, limit) => {
                    self.load_chats(chat_list.into(), limit).await
                }
                Action::ViewAllMessages(chat_id) => self.view_all_messages(chat_id).await,
                Action::GetChatHistory(chat_id, from_id, resp_c) => {
                    self.get_chat_history(chat_id, from_id, resp_c.inner).await
                }
                Action::SendMessage(chat_id, m, r_id, resp_c) => {
                    self.send_message(chat_id, m, r_id, resp_c.inner).await
                }
                Action::SendMessageEdited(chat_id, m_id, m, resp_c) => {
                    self.send_message_edited(chat_id, m_id, m, resp_c.inner)
                        .await
                }
                _ => (),
            }
        }

        match functions::close(self.client_id).await {
            Ok(me) => tracing::info!("TDLib client closed: {:?}", me),
            Err(error) => tracing::error!("Error closing TDLib client: {:?}", error),
        }
    }

    async fn load_chats(&mut self, chat_list: ChatList, limit: i32) {
        let full_cl = self.full_chats_list.read().await;
        if *full_cl {
            return;
        }
        drop(full_cl);

        if let Err(e) = functions::load_chats(Some(chat_list), limit, self.client_id).await {
            tracing::error!("Failed to load chats: {e:?}");
            let mut full_cl = self.full_chats_list.write().await;
            *full_cl = true;
        }
    }

    async fn view_all_messages(&mut self, chat_id: i64) {
        let to_read = match self.chats.read().await.get(&chat_id) {
            Some(c) => match c.last_message.as_ref() {
                Some(m) => m.id,
                None => -1,
            },
            None => -1,
        };

        if let Err(e) =
            functions::view_messages(chat_id, vec![to_read], None, true, self.client_id).await
        {
            tracing::error!("Failed to view all messages: {e:?}");
        }
    }

    /// Sends back, through the relevant channel, multiple responses to the relevant
    /// component until all of the requested history is returned.
    async fn get_chat_history(
        &mut self,
        chat_id: i64,
        from_message_id: i64,
        response_c: UnboundedSender<Action>,
    ) {
        let mut counter = 0;

        while (counter as usize) < QUERY_MESSAGE_WIN_SIZE as usize {
            match functions::get_chat_history(
                chat_id,
                from_message_id,
                counter,
                QUERY_MESSAGE_WIN_SIZE,
                false,
                self.client_id,
            )
            .await
            {
                Ok(Messages::Messages(messages)) => {
                    if messages.messages.is_empty() {
                        tracing::info!("No more messages to get");
                        break;
                    }

                    let resp: Vec<MessageEntry> = messages
                        .messages
                        .into_iter()
                        .flatten()
                        .map(|m| MessageEntry::from(&m))
                        .collect();
                    let many = resp.len();
                    let _ = response_c.send(Action::GetChatHistoryResponse(chat_id, resp));
                    counter += many as i32;
                }
                Err(e) => {
                    tracing::error!("Failed to get chat history: {e:?}");
                    break;
                }
            }
        }
    }

    // The relevant component should hide the reply window on its own.
    // In the previous version this function sent back a Action::HideChatWindowReply
    async fn send_message(
        &mut self,
        chat_id: i64,
        message: String,
        reply_to: Option<TdMessageReplyToMessage>,
        response_c: UnboundedSender<Action>,
    ) {
        let text = InputMessageContent::InputMessageText(InputMessageText {
            text: tdlib_rs::types::FormattedText {
                text: message,
                entities: vec![], // TODO: Add entities
            },
            link_preview_options: None,
            clear_draft: true,
        });
        let reply_to: Option<InputMessageReplyTo> =
            reply_to.map(|reply_to| InputMessageReplyTo::Message((&reply_to).into()));
        let r =
            match functions::send_message(chat_id, 0, reply_to, None, text, self.client_id).await {
                Ok(tdlib_rs::enums::Message::Message(message)) => {
                    SendMessageResult::Ok(Box::new(message))
                }
                Err(e) => {
                    tracing::error!("Failed to send message: {e:?}");
                    SendMessageResult::Err(e)
                }
            };
        let _ = response_c.send(Action::SendMessageResponse(r));
    }

    async fn send_message_edited(
        &self,
        chat_id: i64,
        message_id: i64,
        message: String,
        response_c: UnboundedSender<Action>,
    ) {
        let text = InputMessageContent::InputMessageText(InputMessageText {
            text: tdlib_rs::types::FormattedText {
                text: message.clone(),
                entities: vec![],
            },
            link_preview_options: None,
            clear_draft: true,
        });
        let r = match functions::edit_message_text(chat_id, message_id, text, self.client_id).await
        {
            Ok(_) => {
                tracing::info!("Message edited");
                MessageEdited::Ok(chat_id, message_id, message)
            }
            Err(e) => {
                tracing::error!("Failed to edit message: {e:?}");
                MessageEdited::Err(e)
            }
        };
        let _ = response_c.send(Action::SendMessageEditedResponse(r));
    }
}
