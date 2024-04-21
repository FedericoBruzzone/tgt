use crate::{
    action::Action, app_context::AppContext, app_error::AppError,
    configs::custom::keymap_custom::ActionBinding, event::Event, tg::tg_backend::TgBackend,
    tui::Tui, tui_backend::TuiBackend,
};
use ratatui::layout::Rect;
use std::{collections::HashMap, sync::Arc, time::Instant};
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
) -> Result<(), AppError> {
    tracing::info!("Starting run_app");

    // Clear the terminal
    std::io::Write::write_all(&mut std::io::stdout().lock(), b"\x1b[2J").unwrap();
    // Move the cursor to the top left corner
    std::io::Write::write_all(&mut std::io::stdout().lock(), b"\x1b[1;1H").unwrap();

    tg_backend.start();
    tg_backend.set_logging().await;
    tg_backend.handle_authorization_state().await;

    tui_backend.enter()?;
    tui.register_action_handler(app_context.action_tx().clone())?;

    // Main loop
    loop {
        while tg_backend.have_authorization {
            handle_tui_backend_events(Arc::clone(&app_context), tui, tui_backend).await?;
            handle_app_actions(Arc::clone(&app_context), tui, tui_backend)?;

            if app_context.quit_acquire() {
                tg_backend.need_quit = true;
                tg_backend.have_authorization = false;
                match tdlib::functions::close(tg_backend.client_id).await {
                    Ok(me) => tracing::info!("TDLib client closed: {:?}", me),
                    Err(error) => tracing::error!("Error closing TDLib client: {:?}", error),
                }
                if let Err(e) = tui_backend.exit() {
                    tracing::error!("Error exiting tui backend: {}", e);
                }
                tg_backend.handle_authorization_state().await;

                // Clear the terminal
                std::io::Write::write_all(&mut std::io::stdout().lock(), b"\x1b[2J").unwrap();
                // Move the cursor to the top left corner
                std::io::Write::write_all(&mut std::io::stdout().lock(), b"\x1b[1;1H").unwrap();

                return Ok(());
            }
        }
    }
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
) -> Result<(), AppError> {
    if let Some(event) = tui_backend.next().await {
        match event {
            Event::Render => app_context.action_tx().send(Action::Render)?,
            Event::Resize(width, height) => app_context
                .action_tx()
                .send(Action::Resize(width, height))?,
            Event::Key(key, modifiers) => {
                app_context.action_tx().send(Action::Key(key, modifiers))?;

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
/// Handle incoming actions from the application.
///
/// # Arguments
/// * `app_context` - An Arc wrapped AppContext struct.
/// * `tui` - A mutable reference to the Tui struct.
/// * `tui_backend` - A mutable reference to the TuiBackend struct.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
pub fn handle_app_actions(
    app_context: Arc<AppContext>,
    tui: &mut Tui,
    tui_backend: &mut TuiBackend,
) -> Result<(), AppError> {
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
            _ => {}
        }

        tui.update(action.clone())
    }
    Ok(())
}
