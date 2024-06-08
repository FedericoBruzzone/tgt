use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tdlib_rs::{
    enums::{AuthorizationState, Update, User},
    functions,
};
use tokio::sync::mpsc::{self, Receiver, Sender};

fn ask_user(string: &str) -> String {
    println!("{}", string);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

async fn handle_update(update: Update, auth_tx: &Sender<AuthorizationState>) {
    if let Update::AuthorizationState(update) = update {
        auth_tx.send(update.authorization_state).await.unwrap();
    }
}

async fn handle_authorization_state(
    client_id: i32,
    mut auth_rx: Receiver<AuthorizationState>,
    run_flag: Arc<AtomicBool>,
) -> Option<Receiver<AuthorizationState>> {
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
                    api_id,
                    api_hash.clone(),
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
                let response =
                    functions::set_authentication_phone_number(input, None, client_id).await;
                match response {
                    Ok(_) => break,
                    Err(e) => println!("{}", e.message),
                }
            },
            AuthorizationState::WaitCode(_) => loop {
                let input = ask_user("Enter the verification code:");
                let response = functions::check_authentication_code(input, client_id).await;
                match response {
                    Ok(_) => break,
                    Err(e) => println!("{}", e.message),
                }
            },
            AuthorizationState::Ready => {
                break;
            }
            AuthorizationState::Closed => {
                // Set the flag to false to stop receiving updates from the
                // spawned task
                run_flag.store(false, Ordering::Release);
                return None;
                // break;
            }
            _ => (),
        }
    }

    Some(auth_rx)
}

#[tokio::main]
async fn main() {
    // Create the client object
    let client_id = tdlib_rs::create_client();

    // Create a mpsc channel for handling AuthorizationState updates separately
    // from the task
    let (auth_tx, auth_rx) = mpsc::channel(5);

    // Create a flag to make it possible to stop receiving updates
    let run_flag = Arc::new(AtomicBool::new(true));
    let run_flag_clone = run_flag.clone();

    // Spawn a task to receive updates/responses
    let handle = tokio::spawn(async move {
        loop {
            if !run_flag_clone.load(Ordering::Acquire) {
                break;
            }

            if let Some((update, _client_id)) = tdlib_rs::receive() {
                handle_update(update, &auth_tx).await;
            }
        }
    });

    // Set a fairly low verbosity level. We mainly do this because tdlib_rs
    // requires to perform a random request with the client to start receiving
    // updates for it.
    functions::set_log_verbosity_level(2, client_id)
        .await
        .unwrap();

    // Handle the authorization state to authenticate the client
    let auth_rx = handle_authorization_state(client_id, auth_rx, run_flag.clone())
        .await
        .unwrap();

    // Run the get_me() method to get user information
    let User::User(me) = functions::get_me(client_id).await.unwrap();
    println!("Hi, I'm {}", me.first_name);

    // Tell the client to close
    functions::close(client_id).await.unwrap();

    // Handle the authorization state to wait for the "Closed" state
    match handle_authorization_state(client_id, auth_rx, run_flag.clone()).await {
        None => std::process::exit(0),
        Some(_) => (),
    }

    println!("BEFORE");
    // Wait for the previously spawned task to end the execution
    match handle.await {
        Ok(_) => (),
        Err(e) => println!("Error: {:?}", e),
    }

    println!("AFTER");
}
