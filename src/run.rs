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
                Some(ComponentName::PhotoViewer) => &keymap_config.photo_viewer,
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
            | Action::ShowPhotoViewer
            | Action::HidePhotoViewer
            | Action::ViewPhotoMessage(_)
            | Action::PhotoDownloaded(_)
            | Action::PhotoDecoded(_)
            | Action::ToggleChatList
            | Action::IncreaseChatListSize
            | Action::DecreaseChatListSize
            | Action::IncreasePromptSize
            | Action::DecreasePromptSize
            | Action::ShowChatWindowReply
            | Action::HideChatWindowReply
            | Action::EditMessage(_, _)
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
                    let cache_len_before = app_context.tg_context().open_chat_messages().len();

                    // If cache is empty (initial load), load 100 messages to fill the window
                    // Otherwise, load just ONE batch (50 messages) for incremental scrolling
                    let target_load = if cache_len_before == 0 { 100 } else { 50 };
                    let mut loaded_count = 0;

                    while loaded_count < target_load {
                        if app_context.tg_context().open_chat_id().as_i64() != chat_id {
                            break;
                        }

                        // Always use the oldest message in cache (or 0 if empty) to ensure continuity
                        let from_message_id =
                            app_context.tg_context().oldest_message_id().unwrap_or(0);
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
                        loaded_count += entries.len();
                        tg.open_chat_messages().insert_messages(entries.clone());
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
                        // Use offset -49 to get from_id (which we already have) + 49 newer, then filter out from_id
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
                            // Skip the first message if it's the boundary message we already have
                            let new_messages = if entries.first().map(|e| e.id()) == Some(from_id) {
                                &entries[1..]
                            } else {
                                &entries[..]
                            };
                            if !new_messages.is_empty() {
                                app_context
                                    .tg_context()
                                    .open_chat_messages()
                                    .insert_messages(new_messages.iter().cloned());
                            }
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

                // Clear chat history BEFORE loading new messages to ensure clean slate
                app_context.tg_context().clear_open_chat_messages();
                app_context.tg_context().set_history_loading(true);

                // Load symmetric window around target message
                // TDLib limitation: offset must be > -100, so we can't load 250 newer in one call
                // Strategy: Load target + older messages, then load newer in multiple batches
                const OLDER_LIMIT: i32 = 100; // target + 99 older messages
                const NEWER_BATCH_SIZE: i32 = 99; // max we can use with offset -98
                const NEWER_BATCHES: usize = 3; // 3 batches of ~99 = ~297 newer messages

                // Get target + 99 older messages (offset 0 = include from_message_id)
                let (mut all_entries, _) = tg_backend
                    .get_chat_history_batch(chat_id, *message_id, 0, OLDER_LIMIT)
                    .await;

                // Load newer messages in multiple batches (TDLib offset limit is -100)
                let mut current_from_id = *message_id;
                for batch_num in 0..NEWER_BATCHES {
                    let (entries_newer, _) = tg_backend
                        .get_chat_history_batch(
                            chat_id,
                            current_from_id,
                            -(NEWER_BATCH_SIZE - 1),
                            NEWER_BATCH_SIZE,
                        )
                        .await;

                    if entries_newer.is_empty() {
                        break;
                    }

                    // Filter out the boundary message (already have it from previous batch or older)
                    let new_msgs: Vec<_> = entries_newer
                        .into_iter()
                        .filter(|e| e.id() != current_from_id)
                        .collect();

                    if new_msgs.is_empty() {
                        break;
                    }

                    // Update boundary for next batch
                    if let Some(last) = new_msgs.last() {
                        current_from_id = last.id();
                    }

                    all_entries.extend(new_msgs);

                    tracing::debug!(
                        batch_num,
                        loaded = all_entries.len(),
                        "JumpToMessage: loaded newer batch"
                    );
                }

                tracing::debug!(
                    target_id = message_id,
                    total_count = all_entries.len(),
                    first = all_entries.first().map(|e| e.id()),
                    last = all_entries.last().map(|e| e.id()),
                    "JumpToMessage: loaded all batches"
                );

                if !all_entries.is_empty() {
                    // Insert messages into the fresh (already cleared) cache
                    app_context
                        .tg_context()
                        .open_chat_messages()
                        .insert_messages(all_entries);

                    let final_oldest = app_context.tg_context().oldest_message_id();
                    let final_newest = app_context.tg_context().newest_message_id();
                    let has_target = app_context.tg_context().get_message(*message_id).is_some();
                    tracing::debug!(
                        target_id = message_id,
                        has_target,
                        final_oldest,
                        final_newest,
                        total_cached = app_context.tg_context().open_chat_messages().len(),
                        "JumpToMessage: cache after insert"
                    );
                    // Only set jump target AFTER messages are inserted, so the message is in cache
                    app_context
                        .tg_context()
                        .set_jump_target_message_id_i64(*message_id);
                } else {
                    tracing::warn!(
                        target_id = message_id,
                        "JumpToMessage: failed to load any messages, cache is empty"
                    );
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
            Action::ViewPhotoMessage(message_id) => {
                // Get the message and check if it's a photo that needs downloading
                if let Some(message) = app_context.tg_context().get_message(*message_id) {
                    if let crate::tg::message_entry::MessageContentType::Photo {
                        file_id,
                        file_path,
                    } = message.content_type()
                    {
                        // Check if file needs to be downloaded
                        if file_path.is_empty() || !std::path::Path::new(file_path).exists() {
                            // Download the file
                            match tg_backend.download_file(*file_id, 32).await {
                                Ok(downloaded_path) => {
                                    // Notify PhotoViewer that the photo is ready (viewer will send LoadPhotoFromPath)
                                    app_context
                                        .action_tx()
                                        .send(Action::PhotoDownloaded(downloaded_path))?;
                                }
                                Err(e) => {
                                    tracing::error!("Failed to download photo: {:?}", e);
                                }
                            }
                        }
                    }
                }
            }
            Action::LoadPhotoFromPath(path, message_id) => {
                // Decode image on a blocking thread to avoid stalling the main loop
                let action_tx = app_context.action_tx().clone();
                let app_ctx = Arc::clone(&app_context);
                let path = path.clone();
                let msg_id = *message_id;
                tokio::spawn(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        image::open(&path).map_err(|e| e.to_string())
                    })
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()));
                    // Optionally downscale to reduce memory and protocol work (max 1920 on longer side)
                    let result = result.and_then(|img| {
                        const MAX_DIM: u32 = 1920;
                        let (w, h) = (img.width(), img.height());
                        let scaled = if w.max(h) > MAX_DIM {
                            if w >= h {
                                img.thumbnail(MAX_DIM, (h * MAX_DIM / w).max(1))
                            } else {
                                img.thumbnail((w * MAX_DIM / h).max(1), MAX_DIM)
                            }
                        } else {
                            img
                        };
                        Ok(scaled)
                    });
                    app_ctx.set_pending_photo_decoded(msg_id, result);
                    let _ = action_tx.send(Action::PhotoDecoded(msg_id));
                });
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
