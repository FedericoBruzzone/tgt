// Run it with `cargo run --bin example`
// tdlib rust docs -> https://docs.rs/tdlib/latest/tdlib/
// tdlib telegram docs -> https://core.telegram.org/tdlib/docs/
// java example -> https://github.com/tdlib/td/blob/master/example/java/org/drinkless/tdlib/example/Example.java

use {
    std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    tdlib::{
        enums::{self, AuthorizationState, LogStream, Update},
        functions,
        types::{InputMessageText, LogStreamFile},
    },
    tokio::sync::mpsc::{self, Receiver, Sender},
};

fn ask_user(string: &str) -> String {
    println!("{}", string);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

async fn get_command(client_id: i32) -> bool {
    let mut need_quit = false;
    let command = ask_user("Enter command (gcs - GetChats, gc <chatId> - GetChat, me - GetMe, sm <chatId> <message> - SendMessage, lo - LogOut, q - Quit): ");
    let commands: Vec<&str> = command.split(' ').collect();
    match commands[0] {
        "gcs" => {
            let mut limit = 20;
            if commands.len() > 1 {
                limit = commands[1].parse().unwrap();
            }
            match functions::load_chats(
                Some(enums::ChatList::Main),
                limit,
                client_id,
            )
            .await
            {
                Ok(()) => (),
                Err(error) => eprintln!("[GET MAIN CHAT LIST]: {error:?}"),
            }
        }
        "gc" => {
            match functions::get_chat(commands[1].parse().unwrap(), client_id)
                .await
            {
                Ok(chat) => println!("[GET CHAT]: {chat:?}"),
                Err(error) => eprintln!("[GET CHAT]: {error:?}"),
            }
        }
        "me" => match functions::get_me(client_id).await {
            Ok(me) => println!("[GET ME]: {me:?}"),
            Err(error) => eprintln!("[GET ME]: {error:?}"),
        },
        "sm" => {
            println!("[DEBUG]: {commands:?}");
            // let args: Vec<&str> = commands[1].split(' ').collect();
            let text = enums::InputMessageContent::InputMessageText(
                InputMessageText {
                    text: tdlib::types::FormattedText {
                        text: commands[2].into(),
                        entities: Vec::new(),
                    },
                    disable_web_page_preview: false,
                    clear_draft: true,
                },
            );
            match functions::send_message(
                commands[1].parse().unwrap(),
                0,
                None,
                None,
                None,
                text,
                client_id,
            )
            .await
            {
                Ok(me) => println!("[SEND MESSAGE]: {me:?}"),
                Err(error) => eprintln!("[SEND MESSAGE]: {error:?}"),
            };
        }
        "lo" => {
            match functions::log_out(client_id).await {
                Ok(me) => println!("[LOG OUT]: {me:?}"),
                Err(error) => eprintln!("[LOG OUT]: {error:?}"),
            }
            need_quit = true;
        }
        "q" => {
            match functions::close(client_id).await {
                Ok(me) => println!("[CLOSE]: {me:?}"),
                Err(error) => eprintln!("[CLOSE]: {error:?}"),
            }
            need_quit = true;
        }
        _ => (),
    }
    need_quit
}

async fn handle_update(update: Update, auth_tx: &Sender<AuthorizationState>) {
    match update {
        Update::AuthorizationState(update) => {
            auth_tx.send(update.authorization_state).await.unwrap();
        }
        Update::User(x) => {
            eprintln!("[HANDLE UPDATE]: {:?} {:?}", x.user.usernames, x.user.id)
        }
        // Update::UserStatus(_) => eprintln!("[HANDLE UPDATE]: UserStatus"),
        // Update::BasicGroup(_) => eprintln!("[HANDLE UPDATE]: BasicGroup"),
        // Update::Supergroup(_) => eprintln!("[HANDLE UPDATE]: Supergroup"),
        // Update::SecretChat(_) => eprintln!("[HANDLE UPDATE]: SecretChat"),
        // Update::NewChat(x) => eprintln!("[HANDLE UPDATE]: {x:?}"),
        // Update::ChatTitle(_) => eprintln!("[HANDLE UPDATE]: ChatTitle"),
        // Update::ChatPhoto(_) => eprintln!("[HANDLE UPDATE]: ChatPhoto"),
        // Update::ChatLastMessage(_) => eprintln!("[HANDLE UPDATE]:
        // ChatLastMessage"), Update::ChatPosition(_) =>
        // eprintln!("[HANDLE UPDATE]: ChatPosition"),
        // Update::ChatReadInbox(_) => eprintln!("[HANDLE UPDATE]:
        // ChatReadInbox"), Update::ChatReadOutbox(_) =>
        // eprintln!("[HANDLE UPDATE]: ChatReadOutbox"),
        // Update::ChatUnreadMentionCount(_) => eprintln!("[HANDLE UPDATE]:
        // ChatUnreadMentionCount"), Update::MessageMentionRead(_) =>
        // eprintln!("[HANDLE UPDATE]: MessageMentionRead"),
        // Update::ChatReplyMarkup(_) => eprintln!("[HANDLE UPDATE]:
        // ChatReplyMarkup"), Update::ChatDraftMessage(_) =>
        // eprintln!("[HANDLE UPDATE]: ChatDraftMessage"),
        // Update::ChatPermissions(_) => eprintln!("[HANDLE UPDATE]:
        // ChatPermissions"), Update::ChatNotificationSettings(_) =>
        // eprintln!("[HANDLE UPDATE]: ChatNotificationSettings"),
        // Update::ChatDefaultDisableNotification(_) => eprintln!("[HANDLE
        // UPDATE]: ChatDefaultDisableNotification"),
        // Update::ChatIsMarkedAsUnread(_) => eprintln!("[HANDLE UPDATE]:
        // ChatIsMarkedAsUnread"), Update::ChatBlockList(_) =>
        // eprintln!("[HANDLE UPDATE]: ChatBlockList"),
        // Update::ChatHasScheduledMessages(_) => eprintln!("[HANDLE UPDATE]:
        // ChatHasScheduledMessages"), Update::UserFullInfo(_) =>
        // eprintln!("[HANDLE UPDATE]: UserFullInfo"),
        // Update::BasicGroupFullInfo(_) => eprintln!("[HANDLE UPDATE]:
        // BasicGroupFullInfo"), Update::SupergroupFullInfo(_) =>
        // eprintln!("[HANDLE UPDATE]: SupergroupFullInfo"),
        // Too much prints
        // _ => eprintln!("[HANDLE UPDATE]: {update:?}"),
        _ => (),
    }
}

async fn handle_authorization_state(
    client_id: i32,
    mut auth_rx: Receiver<AuthorizationState>,
    run_flag: Arc<AtomicBool>,
) -> Receiver<AuthorizationState> {
    while let Some(state) = auth_rx.recv().await {
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
                    env!("API_ID").parse().unwrap(),
                    env!("API_HASH").into(),
                    "en".into(),
                    "Desktop".into(),
                    String::new(),
                    env!("CARGO_PKG_VERSION").into(),
                    false,
                    true,
                    client_id,
                )
                .await;

                if let Err(error) = response {
                    println!("{}", error.message);
                }
            }
            AuthorizationState::WaitPhoneNumber => loop {
                let input = ask_user("Enter your phone number (include the country calling code):");
                let response = functions::set_authentication_phone_number(
                    input, None, client_id,
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
            AuthorizationState::WaitCode(_x) => loop {
                // x contains info about verification code
                let input = ask_user("Enter the verification code:");
                let response =
                    functions::check_authentication_code(input, client_id)
                        .await;
                match response {
                    Ok(_) => break,
                    Err(e) => println!("{}", e.message),
                }
            },
            AuthorizationState::WaitRegistration(_x) => {
                // x useless but contains the TOS if we want to show it
                let first_name = ask_user("Please enter your first name: ");
                let last_name = ask_user("Please enter your last name: ");
                functions::register_user(first_name, last_name, client_id)
                    .await
                    .unwrap();
            }
            AuthorizationState::WaitPassword(_x) => {
                let password = ask_user("Please enter password: ");
                functions::check_authentication_password(password, client_id)
                    .await
                    .unwrap();
            }
            AuthorizationState::Ready => {
                // Maybe block all until this state is reached
                break;
            }
            AuthorizationState::LoggingOut => {
                println!("[HANDLE AUTH]: Logging out");
            }
            AuthorizationState::Closing => {
                println!("[HANDLE AUTH]: Closing");
            }
            AuthorizationState::Closed => {
                println!("[HANDLE AUTH]: Closed");
                // Set the flag to false to stop receiving updates from the
                // spawned task
                run_flag.store(false, Ordering::Release);
                break;
            }
            _ => eprintln!(
                "[HANDLE AUTH] Unsupported authorization state: {state:?}"
            ),
        }
    }

    auth_rx
}

#[tokio::main]
async fn main() {
    // Create the client object
    let client_id = tdlib::create_client();

    // Create a mpsc channel for handling AuthorizationState updates separately
    // from the task
    let (auth_tx, auth_rx) = mpsc::channel(5);

    // Create a flag to make it possible to stop receiving updates
    let run_flag = Arc::new(AtomicBool::new(true));
    let run_flag_clone = run_flag.clone();

    // Spawn a task to receive updates/responses
    let handle = tokio::spawn(async move {
        while run_flag_clone.load(Ordering::Acquire) {
            if let Some((update, _client_id)) = tdlib::receive() {
                handle_update(update, &auth_tx).await;
            }
        }
    });

    // Set a fairly low verbosity level. We mainly do this because tdlib
    // requires to perform a random request with the client to start receiving
    // updates for it.
    functions::set_log_verbosity_level(2, client_id)
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
        functions::set_log_stream(LogStream::File(log_stream_file), client_id)
            .await
    {
        eprintln!("[ERROR] \"Write access to the current directory is required\": {error:?}")
    }

    // Test get_text_entities
    match functions::get_text_entities(
        "@telegram /test_command https://telegram.org telegram.me @gif @test"
            .into(),
        client_id,
    )
    .await
    {
        Err(error) => {
            eprintln!("[ERROR] \"functions::get_text_entitie\": {error:?}")
        }
        Ok(ok) => println!("[TEST]: {ok:?}"),
    }

    // Handle the authorization state to authenticate the client
    let auth_rx =
        handle_authorization_state(client_id, auth_rx, run_flag.clone()).await;

    // // Run the get_me() method to get user information
    // let User::User(me) = functions::get_me(client_id).await.unwrap();
    // println!("[MAIN]: {me:?}");

    let mut need_quit: bool = false;

    while !need_quit {
        need_quit = get_command(client_id).await;
    }

    // // Tell the client to close
    // functions::close(client_id).await.unwrap();

    // Handle the authorization state to wait for the "Closed" state
    handle_authorization_state(client_id, auth_rx, run_flag.clone()).await;

    // Wait for the previously spawned task to end the execution
    handle.await.unwrap();
}
