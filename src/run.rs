use crate::{
    action::Action, app_context::AppContext, app_error::AppError,
    configs::custom::keymap_custom::ActionBinding, event::Event, tg::tg_backend::TgBackend,
    tui::Tui, tui_backend::TuiBackend,
};
use ratatui::layout::Rect;
use std::{collections::HashMap, io, sync::Arc, time::Instant};
use tdlib::{enums::ChatList, functions};
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
    tg_backend.load_chats(ChatList::Main, 30).await;

    tui_backend.enter()?;
    tui.register_action_handler(app_context.action_tx().clone())?;

    // Main loop
    loop {
        while tg_backend.have_authorization {
            handle_tui_backend_events(Arc::clone(&app_context), tui, tui_backend).await?;
            handle_tg_backend_events(Arc::clone(&app_context), tg_backend).await?;
            handle_app_actions(Arc::clone(&app_context), tui, tui_backend, tg_backend).await?;

            if app_context.quit_acquire() {
                tg_backend.need_quit = true;
                tg_backend.have_authorization = false;
                match functions::close(tg_backend.client_id).await {
                    Ok(me) => tracing::info!("TDLib client closed: {:?}", me),
                    Err(error) => tracing::error!("Error closing TDLib client: {:?}", error),
                }
                match tui_backend.exit() {
                    Ok(_) => tracing::info!("Tui backend exited"),
                    Err(e) => tracing::error!("Error exiting tui backend: {}", e),
                }
                tg_backend.handle_authorization_state().await;

                // Clear the terminal and move the cursor to the top left corner
                io::Write::write_all(&mut io::stdout().lock(), b"\x1b[2J\x1b[1;1H").unwrap();

                return Ok(());
            }
        }
    }
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
    if let Some(Event::LoadChats(chat_list, limit)) = tg_backend.next().await {
        app_context
            .action_tx()
            .send(Action::LoadChats(chat_list, limit))?;
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
                app_context
                    .action_tx()
                    .send(Action::from_key_event(key, modifiers))?;

                // Handle core_window key bindings.
                if let Some(action_binding) = app_context
                    .keymap_config()
                    .core_window
                    .get(&Event::Key(key, modifiers))
                {
                    match action_binding {
                        ActionBinding::Single { action, .. } => {
                            app_context.action_tx().send(action.clone())?
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
                    tui.draw(f, f.size()).unwrap();
                })?;
            }
            Action::Resize(width, height) => {
                tui_backend
                    .terminal
                    .resize(Rect::new(0, 0, width, height))?;
                tui_backend.terminal.draw(|f| {
                    tui.draw(f, f.size()).unwrap();
                })?;
            }
            Action::Quit => {
                app_context.quit_store(true);
            }
            Action::LoadChats(chat_list, limit) => {
                tg_backend.load_chats(chat_list.into(), limit).await;
            }
            _ => {}
        }

        tui.update(action.clone())
    }
    Ok(())
}
