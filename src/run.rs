use crate::component_name::ComponentName;
use crate::component_name::ComponentName::Prompt;
use crate::{
    action::Action, app_context::AppContext, app_error::AppError,
    configs::custom::keymap_custom::ActionBinding, event::Event, tg::tg_backend::TgBackend,
    tui::Tui, tui_backend::TuiBackend,
};
use ratatui::layout::Rect;
use std::{collections::HashMap, io, sync::Arc, time::Duration, time::Instant};
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
    app_context.mark_dirty();
    // Notify ChatList to populate visible_chats from initial load (it only rebuilds on LoadChats/ChatHistoryAppended/Resize).
    let _ = app_context
        .action_tx()
        .send(Action::LoadChats(ChatList::Main.into(), 30));

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
            Event::GetChatHistoryNewer => {
                app_context.action_tx().send(Action::GetChatHistoryNewer)?;
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
                // Reply flow is now handled by ChatWindow sending FocusComponent(Prompt) + ReplyMessage
                // directly to action_tx when user presses R. This branch is kept for any other caller.
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
            Event::ChatMessageAdded(message_id) => {
                app_context
                    .tg_context()
                    .set_jump_target_message_id_i64(message_id);
                app_context.action_tx().send(Action::ChatHistoryAppended)?;
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
    // Short timeout so we can process ChatHistoryAppended (and other actions) when
    // the background history task completes, without requiring a key press.
    let poll = tokio::time::timeout(Duration::from_millis(150), tui_backend.next()).await;
    let Some(event) = (match poll {
        Ok(Some(ev)) => Some(ev),
        Ok(None) | Err(_) => None,
    }) else {
        return Ok(());
    };
    match event {
        Event::Render => app_context.mark_dirty(),
        Event::Resize(width, height) => {
            app_context.mark_dirty();
            app_context
                .action_tx()
                .send(Action::Resize(width, height))?;
        }
        Event::Key(key, modifiers) => {
            let focused = app_context.focused_component();
            let keymap_config = app_context.keymap_config();
            let key_event = Event::Key(key, modifiers);

            // Check if key is explicitly bound in the component-specific keymap (not merged).
            // If not explicitly bound in component keymap, skip keymap lookup to allow typing.
            // This allows users to type keys that are only bound in core_window by not
            // binding them in the component-specific keymap.
            let component_keymap = match focused {
                Some(ComponentName::ChatList) => &keymap_config.chat_list,
                Some(ComponentName::Chat) => &keymap_config.chat,
                Some(ComponentName::Prompt) => &keymap_config.prompt,
                Some(ComponentName::CommandGuide) => &keymap_config.command_guide,
                Some(ComponentName::ThemeSelector) => &keymap_config.theme_selector,
                Some(ComponentName::SearchOverlay) => &keymap_config.search_overlay,
                _ => &keymap_config.core_window,
            };

            // Only check merged keymap if key is explicitly bound in component-specific keymap
            // or if no component is focused (use core_window)
            let should_check_keymap =
                focused.is_none() || component_keymap.contains_key(&key_event);

            if should_check_keymap {
                // Check if key is bound in the merged keymap for the focused component
                let keymap = keymap_config.get_map_of(focused);
                if let Some(action_binding) = keymap.get(&key_event) {
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
                            return Ok(());
                        }
                    }
                }
            }
            // Key not bound in keymap (or not explicitly bound in component keymap): pass through to components
            // This allows components to handle keys directly (e.g. typing characters in prompt)
            app_context
                .action_tx()
                .send(Action::from_key_event(key, modifiers))?;
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
/// Returns true for actions that change UI-visible state and should trigger a render.
fn action_changes_ui(action: &Action) -> bool {
    matches!(
        action,
        Action::FocusComponent(_)
            | Action::ChatListNext
            | Action::ChatListPrevious
            | Action::ChatListSearch
            | Action::ChatListOpen
            | Action::ChatListSortWithString(_)
            | Action::ChatListRestoreSort
            | Action::ChatWindowNext
            | Action::ChatWindowPrevious
            | Action::ChatWindowSearch
            | Action::ChatWindowSortWithString(_)
            | Action::ChatWindowRestoreSort
            | Action::SearchChatMessages(_)
            | Action::SearchResults(_)
            | Action::JumpToMessage(_)
            | Action::JumpCompleted(_)
            | Action::CloseSearchOverlay
            | Action::ShowSearchOverlay
            | Action::SwitchTheme
            | Action::SwitchThemeTo(_)
            | Action::ShowThemeSelector
            | Action::HideThemeSelector
            | Action::ToggleChatList
            | Action::IncreaseChatListSize
            | Action::DecreaseChatListSize
            | Action::IncreasePromptSize
            | Action::DecreasePromptSize
            | Action::ShowChatWindowReply
            | Action::HideChatWindowReply
            | Action::ReplyMessage(_, _)
            | Action::ShowCommandGuide
            | Action::HideCommandGuide
            | Action::StatusMessage(_)
            | Action::UpdateArea(_)
            | Action::GetChatHistoryNewer
            | Action::ChatHistoryAppended
    )
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
        match &action {
            Action::Render => {
                // Actual draw happens at end of loop when should_render()
            }
            Action::Resize(width, height) => {
                tui_backend
                    .terminal
                    .resize(Rect::new(0, 0, *width, *height))?;
                app_context.mark_dirty();
            }
            Action::FocusLost => {
                tui_backend.suspend()?;
                app_context.mark_dirty();
            }
            Action::FocusGained => {
                tui_backend.resume()?;
                app_context.mark_dirty();
            }
            Action::Quit => {
                app_context.quit_store(true);
            }
            Action::LoadChats(chat_list, limit) => {
                app_context.mark_dirty();
                tg_backend.load_chats((*chat_list).into(), *limit).await;
            }
            Action::SendMessage(ref message, ref reply_to) => {
                let _ = tg_backend
                    .send_message(
                        message.to_string(),
                        app_context.tg_context().open_chat_id().as_i64(),
                        reply_to.clone(),
                    )
                    .await;
            }
            Action::SendMessageEdited(message_id, ref message) => {
                tg_backend
                    .send_message_edited(*message_id, message.to_string())
                    .await;
            }
            Action::GetChatHistory => {
                if !app_context.tg_context().is_history_loading() {
                    app_context.tg_context().set_history_loading(true);
                    let chat_id = app_context.tg_context().open_chat_id().as_i64();
                    const INITIAL_LOAD_TARGET: usize = 100;
                    let start_len = app_context.tg_context().open_chat_messages().len();
                    loop {
                        // Only insert for the chat that is still open (guard against stale load).
                        if app_context.tg_context().open_chat_id().as_i64() != chat_id {
                            break;
                        }
                        let from_message_id = app_context.tg_context().from_message_id();
                        let (entries, _has_more) = tg_backend
                            .get_chat_history_one_batch(chat_id, from_message_id)
                            .await;
                        if entries.is_empty() {
                            break;
                        }
                        if app_context.tg_context().open_chat_id().as_i64() != chat_id {
                            break;
                        }
                        let tg = app_context.tg_context();
                        tg.open_chat_messages().insert_messages(entries.clone());
                        if let Some(last) = entries.last() {
                            tg.set_from_message_id(last.id());
                        }
                        let _ = app_context.action_tx().send(Action::ChatHistoryAppended);
                        let current_len = app_context.tg_context().open_chat_messages().len();
                        if current_len >= start_len + INITIAL_LOAD_TARGET {
                            break;
                        }
                    }
                    app_context.tg_context().set_history_loading(false);
                    let _ = app_context.action_tx().send(Action::ChatHistoryAppended);
                }
            }
            Action::GetChatHistoryNewer => {
                if !app_context.tg_context().is_history_loading() {
                    let chat_id = app_context.tg_context().open_chat_id().as_i64();
                    let newest = app_context
                        .tg_context()
                        .open_chat_messages()
                        .newest_message_id();
                    if let Some(from_id) = newest {
                        app_context.tg_context().set_history_loading(true);
                        // TDLib: offset -N = from_message_id + N newer messages; limit >= -offset
                        const NEWER_BATCH: i32 = 50;
                        let (entries, _) = tg_backend
                            .get_chat_history_batch(
                                chat_id,
                                from_id,
                                -(NEWER_BATCH - 1),
                                NEWER_BATCH,
                            )
                            .await;
                        if app_context.tg_context().open_chat_id().as_i64() == chat_id
                            && !entries.is_empty()
                        {
                            app_context
                                .tg_context()
                                .open_chat_messages()
                                .insert_messages(entries);
                        }
                        app_context.tg_context().set_history_loading(false);
                        let _ = app_context.action_tx().send(Action::ChatHistoryAppended);
                    }
                }
            }
            Action::SearchChatMessages(ref query) => {
                let chat_id = app_context.tg_context().open_chat_id().as_i64();
                if chat_id == 0 {
                    continue;
                }
                match tg_backend
                    .search_chat_messages(chat_id, query.clone(), 50)
                    .await
                {
                    Ok(entries) => {
                        let _ = app_context.action_tx().send(Action::SearchResults(entries));
                    }
                    Err(_) => {
                        let _ = app_context.action_tx().send(Action::SearchResults(vec![]));
                    }
                }
            }
            Action::JumpToMessage(message_id) => {
                let chat_id = app_context.tg_context().open_chat_id().as_i64();
                if chat_id == 0 {
                    continue;
                }
                // If message already loaded, just select it; don't clear (avoids wiping chat).
                if app_context
                    .tg_context()
                    .open_chat_messages()
                    .get_message(*message_id)
                    .is_some()
                {
                    app_context
                        .tg_context()
                        .set_jump_target_message_id_i64(*message_id);
                    let _ = app_context
                        .action_tx()
                        .send(Action::JumpCompleted(*message_id));
                    let _ = app_context.action_tx().send(Action::ChatHistoryAppended);
                    continue;
                }
                // Fetch first; only clear when we have messages to insert (avoids "empty response" overwrite).
                app_context.tg_context().set_history_loading(true);
                app_context
                    .tg_context()
                    .set_jump_target_message_id_i64(*message_id);
                const OLDER_LIMIT: i32 = 31; // target + 30 older
                const NEWER_LIMIT: i32 = 16; // target + 15 newer (limit must be >= -offset)
                let (entries_older, _) = tg_backend
                    .get_chat_history_batch(chat_id, *message_id, 0, OLDER_LIMIT)
                    .await;
                let (entries_newer, _) = tg_backend
                    .get_chat_history_batch(chat_id, *message_id, -15, NEWER_LIMIT)
                    .await;
                if !entries_older.is_empty() || !entries_newer.is_empty() {
                    //app_context.tg_context().clear_open_chat_messages();
                    app_context
                        .tg_context()
                        .open_chat_messages()
                        .insert_messages(entries_older);
                    app_context
                        .tg_context()
                        .open_chat_messages()
                        .insert_messages(entries_newer);
                    let oldest = app_context
                        .tg_context()
                        .open_chat_messages()
                        .oldest_message_id();
                    if let Some(oldest_id) = oldest {
                        app_context.tg_context().set_from_message_id(oldest_id);
                    }
                }
                app_context.tg_context().set_history_loading(false);
                let _ = app_context
                    .action_tx()
                    .send(Action::JumpCompleted(*message_id));
                let _ = app_context.action_tx().send(Action::ChatHistoryAppended);
            }
            Action::DeleteMessages(ref message_ids, revoke) => {
                tg_backend
                    .delete_messages(
                        app_context.tg_context().open_chat_id().as_i64(),
                        message_ids.to_vec(),
                        *revoke,
                    )
                    .await;
            }
            Action::ReplyMessage(message_id, ref message) => {
                app_context
                    .tg_context()
                    .set_reply_message_i64(*message_id, message.to_string());
            }
            Action::ViewAllMessages => {
                tg_backend.view_all_messages().await;
            }
            _ => {}
        }

        if action_changes_ui(&action) {
            app_context.mark_dirty();
        }
        tui.update(action.clone())
    }

    if app_context.should_render() {
        tui_backend.terminal.draw(|f| {
            tui.draw(f, f.area()).unwrap();
        })?;
        app_context.clear_render_flag();
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
