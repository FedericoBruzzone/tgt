use crate::component_name::ComponentName::Prompt;
use crate::{
    action::Action, app_context::AppContext, app_error::AppError,
    configs::custom::keymap_custom::ActionBinding, event::Event, tg::tg_backend::TgBackend,
    tui::Tui, tui_backend::TuiBackend,
};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use std::{collections::HashMap, io, sync::Arc, time::Instant};
use tdlib_rs::enums::ChatList;
use tokio::sync::mpsc::UnboundedSender;

/// Run the main event loop for the application.
/// This function will process events and actions for the tui and the backend.
///
/// # Arguments
/// * `app_context` - An Arc wrapped AppContext struct.
/// * `tui` - A mutable reference to the Tui struct.
/// * `tui_backend` - A mutable reference to the TuiBackend struct.
/// * `tg_backend` - A mutable reference to the TgBackend struct.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
pub async fn run_app(
    app_context: Arc<AppContext>,
    tui: &mut Tui,
    tui_backend: &mut TuiBackend,
    tg_backend: &mut TgBackend,
) -> Result<(), AppError<Action>> {
    tracing::info!("Starting run_app");

    // Clear the terminal and move the cursor to the top left corner
    io::Write::write_all(&mut io::stdout().lock(), b"\x1b[2J\x1b[1;1H").unwrap();

    tg_backend.start();
    tg_backend.set_logging().await;
    tg_backend.handle_authorization_state().await;
    tg_backend.use_quick_ack().await;
    tg_backend.get_me().await;
    tg_backend.load_chats(ChatList::Main, 30).await;

    match handle_cli(Arc::clone(&app_context), tg_backend).await {
        HandleCliOutcome::Quit => {
            futures::join!(quit_cli(tg_backend));
            return Ok(());
        }
        HandleCliOutcome::Logout => {
            futures::join!(log_out(tg_backend));
            return Ok(());
        }
        HandleCliOutcome::Continue => {}
    }

    tg_backend.online().await;
    tg_backend.disable_animated_emoji(true).await;

    tui_backend.enter()?;
    tui.register_action_handler(app_context.action_tx().clone())?;

    // Main loop
    while tg_backend.have_authorization {
        handle_tui_backend_events(Arc::clone(&app_context), tui, tui_backend).await?;
        handle_tg_backend_events(Arc::clone(&app_context), tg_backend).await?;
        handle_app_actions(Arc::clone(&app_context), tui, tui_backend, tg_backend).await?;

        if app_context.quit_acquire() {
            quit_tui(tg_backend, tui_backend).await;
            tracing::info!("Quitting");
            return Ok(());
        }
    }

    Ok(())
}
/// Handle incoming events from the Telegram backend and produce actions if
/// necessary.
///
/// # Arguments
/// * `app_context` - An Arc wrapped AppContext struct.
/// * `tg_backend` - A mutable reference to the TgBackend struct.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
async fn handle_tg_backend_events(
    app_context: Arc<AppContext>,
    tg_backend: &mut TgBackend,
) -> Result<(), AppError<Action>> {
    if let Some(event) = tg_backend.next().await {
        match event {
            Event::LoadChats(chat_list, limit) => {
                app_context
                    .action_tx()
                    .send(Action::LoadChats(chat_list, limit))?;
            }
            Event::SendMessage(message, reply_to) => {
                app_context
                    .action_tx()
                    .send(Action::SendMessage(message, reply_to))?;
            }
            Event::SendMessageEdited(message_id, message) => {
                app_context
                    .action_tx()
                    .send(Action::SendMessageEdited(message_id, message))?;
            }
            Event::GetChatHistory => {
                app_context.action_tx().send(Action::GetChatHistory)?;
            }
            Event::DeleteMessages(message_ids, revoke) => {
                app_context
                    .action_tx()
                    .send(Action::DeleteMessages(message_ids, revoke))?;
            }
            Event::EditMessage(message_id, message) => {
                // It is important to focus the prompt before editing the message.
                // Because the actions are sent to the focused component.
                app_context
                    .action_tx()
                    .send(Action::FocusComponent(Prompt))?;

                app_context
                    .action_tx()
                    .send(Action::EditMessage(message_id, message))?;
            }
            Event::ReplyMessage(message_id, message) => {
                app_context
                    .action_tx()
                    .send(Action::FocusComponent(Prompt))?;

                app_context
                    .action_tx()
                    .send(Action::ReplyMessage(message_id, message))?;
            }
            Event::ViewAllMessages => {
                app_context.action_tx().send(Action::ViewAllMessages)?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[allow(clippy::await_holding_lock)]
/// Handle incoming events from the TUI backend and produce actions if
/// necessary.
///
/// # Arguments
/// * `app_context` - An Arc wrapped AppContext struct.
/// * `tui` - A mutable reference to the Tui struct.
/// * `tui_backend` - A mutable reference to the TuiBackend struct.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
async fn handle_tui_backend_events(
    app_context: Arc<AppContext>,
    tui: &mut Tui,
    tui_backend: &mut TuiBackend,
) -> Result<(), AppError<Action>> {
    if let Some(event) = tui_backend.next().await {
        match event {
            Event::Render => app_context.action_tx().send(Action::Render)?,
            Event::Resize(width, height) => app_context
                .action_tx()
                .send(Action::Resize(width, height))?,
            Event::Key(key, modifiers) => {
                // Always send Key action first so components can check visibility state
                app_context
                    .action_tx()
                    .send(Action::from_key_event(key, modifiers))?;

                // Handle Esc and F1 specially - let them go through to components for command guide handling
                // This allows CoreWindow to check if guide is visible and close it if needed
                if key == KeyCode::Esc
                    || (key == KeyCode::F(1) && !modifiers.contains(KeyModifiers::ALT))
                {
                    // Esc/F1 (without Alt) is sent as Key action above, now let it go to Tui/CoreWindow
                    // Don't check keymap for Esc/F1, let components handle it
                    // CoreWindow will check visibility and close guide if visible
                    // Alt+F1 is handled by the keymap system
                } else {
                    // Handle core_window key bindings for other keys.
                    if let Some(action_binding) = app_context
                        .keymap_config()
                        .core_window
                        .get(&Event::Key(key, modifiers))
                    {
                        match action_binding {
                            ActionBinding::Single { action, .. } => {
                                app_context.action_tx().send(action.clone())?;
                                return Ok(());
                            }
                            ActionBinding::Multiple(map_event_action) => {
                                consume_until_single_action(
                                    &app_context.action_tx(),
                                    tui_backend,
                                    map_event_action.clone(),
                                )
                                .await;
                                // We need to return here to avoid sending the
                                // event to the tui. At the moment, the components
                                // are not able to handle multiple events.
                                return Ok(());
                            }
                        }
                    }
                }
            }
            Event::FocusLost => app_context.action_tx().send(Action::FocusLost)?,
            Event::FocusGained => app_context.action_tx().send(Action::FocusGained)?,
            Event::Paste(ref text) => app_context.action_tx().send(Action::Paste(text.clone()))?,
            _ => {}
        }

        // Note that sending the event to the tui it will send the event
        // directly to the `CoreWindow` component.
        if let Some(action) = tui.handle_events(Some(event.clone()))? {
            app_context.action_tx().send(action)?
        }
    }
    Ok(())
}
/// Consume events until a single action is produced.
/// This function is used to consume events until a single action is produced
/// from a map of events to actions.
/// This is useful for handling multiple key bindings that produce the same
/// action, or simply to consume events until a single action is produced.
/// The time limit for consuming events is 1 second.
///
/// # Arguments
/// * `action_tx` - An unbounded sender that can send actions.
/// * `tui_backend` - A mutable reference to the `TuiBackend` struct.
/// * `map_event_action` - A map of events to actions.
async fn consume_until_single_action(
    action_tx: &UnboundedSender<Action>,
    tui_backend: &mut TuiBackend,
    map_event_action: HashMap<Event, ActionBinding>,
) {
    let start = Instant::now();
    loop {
        if let Some(event) = tui_backend.next().await {
            if let Some(ActionBinding::Single { action, .. }) = map_event_action.get(&event) {
                action_tx.send(action.clone()).unwrap();
                break;
            }
        }
        if start.elapsed().as_secs() > 1 {
            break;
        }
    }
}
#[allow(clippy::await_holding_lock)]
/// Handle incoming actions from the application.
///
/// # Arguments
/// * `app_context` - An Arc wrapped AppContext struct.
/// * `tui` - A mutable reference to the Tui struct.
/// * `tui_backend` - A mutable reference to the TuiBackend struct.
/// * `tg_backend` - A mutable reference to the TgBackend struct.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
pub async fn handle_app_actions(
    app_context: Arc<AppContext>,
    tui: &mut Tui,
    tui_backend: &mut TuiBackend,
    tg_backend: &mut TgBackend,
) -> Result<(), AppError<Action>> {
    while let Ok(action) = app_context.action_rx().try_recv() {
        match action {
            Action::Render => {
                tui_backend.terminal.draw(|f| {
                    tui.draw(f, f.area()).unwrap();
                })?;
            }
            Action::Resize(width, height) => {
                tui_backend
                    .terminal
                    .resize(Rect::new(0, 0, width, height))?;
                tui_backend.terminal.draw(|f| {
                    tui.draw(f, f.area()).unwrap();
                })?;
            }
            Action::FocusLost => tui_backend.suspend()?,
            Action::FocusGained => tui_backend.resume()?,
            Action::Quit => {
                app_context.quit_store(true);
            }
            Action::LoadChats(chat_list, limit) => {
                tg_backend.load_chats(chat_list.into(), limit).await;
            }
            Action::SendMessage(ref message, ref reply_to) => {
                let _ = tg_backend
                    .send_message(
                        message.to_string(),
                        app_context.tg_context().open_chat_id(),
                        reply_to.clone(),
                    )
                    .await;
            }
            Action::SendMessageEdited(message_id, ref message) => {
                tg_backend
                    .send_message_edited(message_id, message.to_string())
                    .await;
            }
            Action::GetChatHistory => {
                tg_backend
                    .get_chat_history(app_context.tg_context().open_chat_id())
                    .await;
            }
            Action::DeleteMessages(ref message_ids, revoke) => {
                tg_backend
                    .delete_messages(
                        app_context.tg_context().open_chat_id(),
                        message_ids.to_vec(),
                        revoke,
                    )
                    .await;
            }
            Action::ReplyMessage(message_id, ref message) => {
                app_context
                    .tg_context()
                    .set_reply_message(message_id, message.to_string());
            }
            Action::ViewAllMessages => {
                tg_backend.view_all_messages().await;
            }
            _ => {}
        }

        tui.update(action.clone())
    }
    Ok(())
}

/// An enum to represent the outcome of the handle_cli function.
enum HandleCliOutcome {
    /// The application should quit.
    Quit,
    /// The application should continue.
    Continue,
    /// The user should be logged out.
    Logout,
}

#[allow(clippy::await_holding_lock)]
/// Handle the command line arguments.
/// This function will handle the command line arguments.
///
/// # Arguments
/// * `app_context` - An Arc wrapped AppContext struct.
/// * `tui_backend` - A mutable reference to the TuiBackend struct.
/// * `tg_backend` - A mutable reference to the TgBackend struct.
async fn handle_cli(app_context: Arc<AppContext>, tg_backend: &mut TgBackend) -> HandleCliOutcome {
    if app_context.cli_args().telegram_cli().logout() {
        return HandleCliOutcome::Logout;
    }
    if let Some(chat) = app_context.cli_args().telegram_cli().send_message() {
        futures::join!(tg_backend.load_all_chats());

        let [chat_name, message_text] = chat.as_slice() else {
            tracing::error!("Invalid number of arguments for send message");
            println!("Invalid number of arguments for send message");
            return HandleCliOutcome::Quit;
        };
        match tg_backend.search_chats(chat_name.to_string()).await {
            Ok(chat) => {
                tracing::info!("Chat found: {:?}", chat);
                let total_chats = chat.total_count;
                let chats_vec = chat.chat_ids;
                if total_chats == 0 {
                    tracing::error!("No chat found with the name: {}", chat_name);
                    println!("No chat found with the name: {chat_name}");
                    return HandleCliOutcome::Quit;
                }
                if total_chats > 1 {
                    tracing::error!("Multiple chats found with the name: {}", chat_name);
                    println!("Multiple chats found with the name: {chat_name}");
                    return HandleCliOutcome::Quit;
                }
                let chat_id = chats_vec[0];
                let msg = tg_backend
                    .send_message(message_text.to_string(), chat_id, None)
                    .await;
                match msg {
                    Ok(msg) => {
                        let message_id = msg.id;
                        while app_context.tg_context().last_acknowledged_message_id() != message_id
                        {
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error sending message: {e:?}");
                        println!("Error sending message: {e:?}");
                        return HandleCliOutcome::Quit;
                    }
                }

                tracing::info!(
                    "Sent message {} to chat_name {} ({})",
                    message_text,
                    chat_name,
                    chat_id
                );
                return HandleCliOutcome::Quit;
            }
            Err(e) => {
                tracing::error!("Error searching for chat: {e:?}");
                println!("Error searching for chat: {e:?}");
                return HandleCliOutcome::Quit;
            }
        }
    }
    HandleCliOutcome::Continue
}

/// Quit the tui.
///
/// # Arguments
/// * `tg_backend` - A mutable reference to the TgBackend struct.
/// * `tui_backend` - A mutable reference to the TuiBackend struct.
async fn quit_tui(tg_backend: &mut TgBackend, tui_backend: &mut TuiBackend) {
    futures::join!(tg_backend.offline());
    tg_backend.have_authorization = false;
    tg_backend.close().await;
    tui_backend.exit();
    tg_backend.handle_authorization_state().await;

    // Clear the terminal and move the cursor to the top left corner
    io::Write::write_all(&mut io::stdout().lock(), b"\x1b[2J\x1b[1;1H").unwrap();
}

/// Quit the cli.
///
/// # Arguments
/// * `tg_backend` - A mutable reference to the TgBackend struct.
async fn quit_cli(tg_backend: &mut TgBackend) {
    tg_backend.have_authorization = false;
    tg_backend.close().await;
    tg_backend.handle_authorization_state().await;

    // Clear the terminal and move the cursor to the top left corner
    io::Write::write_all(&mut io::stdout().lock(), b"\x1b[2J\x1b[1;1H").unwrap();
}

/// Logout the user from the Telegram backend.
///
/// # Arguments
/// * `tg_backend` - A mutable reference to the TgBackend struct.
async fn log_out(tg_backend: &mut TgBackend) {
    tg_backend.log_out().await;
    tg_backend.handle_authorization_state().await;

    // Clear the terminal and move the cursor to the top left corner
    io::Write::write_all(&mut io::stdout().lock(), b"\x1b[2J\x1b[1;1H").unwrap();
}
