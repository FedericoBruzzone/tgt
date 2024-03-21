use {
    crate::{
        app_context::AppContext,
        app_error::AppError,
        configs::custom::keymap_custom::ActionBinding,
        enums::{action::Action, event::Event},
        tui_backend::TuiBackend,
    },
    ratatui::layout::Rect,
    std::{collections::HashMap, time::Instant},
};

/// Run the main event loop for the application.
/// This function will process events and actions for the user interface and
/// the backend.
///
/// # Arguments
/// * `app_context` - A mutable reference to the `AppContext` struct.
/// * `tui_backend` - A mutable reference to the `TuiBackend` struct.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
pub async fn run_app(
    app_context: &mut AppContext,
    tui_backend: &mut TuiBackend,
) -> Result<(), AppError> {
    tracing::info!("Starting run_app");
    let action_tx = app_context.action_tx_clone();

    tui_backend.enter()?;
    app_context
        .tui_mut_ref()
        .register_action_handler(action_tx)?;

    loop {
        if app_context.quit {
            // TODO: tui.stop()?
            break;
        }
        handle_tui_backend_events(app_context, tui_backend).await?;
        handle_app_actions(app_context, tui_backend)?;
    }

    tui_backend.exit()?;

    Ok(())
}
/// Handle incoming events from the TUI backend and produce actions if
/// necessary.
///
/// # Arguments
/// * `app_context` - A mutable reference to the `AppContext` struct.
/// * `tui_backend` - A mutable reference to the `TuiBackend` struct.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
async fn handle_tui_backend_events(
    app_context: &mut AppContext,
    tui_backend: &mut TuiBackend,
) -> Result<(), AppError> {
    if let Some(event) = tui_backend.next().await {
        match event {
            Event::Render => {
                app_context.action_tx_ref().send(Action::Render)?
            }
            Event::Resize(width, height) => app_context
                .action_tx_ref()
                .send(Action::Resize(width, height))?,
            Event::Key(key, modifiers) => {
                match app_context
                    .keymap_config_ref()
                    .default
                    .get(&Event::Key(key, modifiers))
                    .unwrap()
                {
                    ActionBinding::Single { action, .. } => {
                        app_context.action_tx_ref().send(action.clone())?
                    }
                    ActionBinding::Multiple(map_event_action) => {
                        consume_until_single_action(
                            app_context,
                            tui_backend,
                            map_event_action.clone(),
                        )
                        .await;
                    }
                }
            }
            _ =>
                /* Event::Quit */
            /* Event::Mouse(mouse) */
            /* Event::Init */
                {}
        }

        // [TODO] Remove this if block
        if let Some(action) = app_context
            .tui_mut_ref()
            .handle_events(Some(event.clone()))?
        {
            app_context.action_tx_ref().send(action)?
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
/// * `app_context` - A mutable reference to the `AppContext` struct.
/// * `tui_backend` - A mutable reference to the `TuiBackend` struct.
/// * `map_event_action` - A map of events to actions.
async fn consume_until_single_action(
    app_context: &mut AppContext,
    tui_backend: &mut TuiBackend,
    map_event_action: HashMap<Event, ActionBinding>,
) {
    let start = Instant::now();
    loop {
        if let Some(event) = tui_backend.next().await {
            if let Some(ActionBinding::Single { action, .. }) =
                map_event_action.get(&event)
            {
                app_context.action_tx_ref().send(action.clone()).unwrap();
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
/// * `app_context` - A mutable reference to the `AppContext` struct.
/// * `tui_backend` - A mutable reference to the `TuiBackend` struct.
///
/// # Returns
/// * `Result<(), AppError>` - An Ok result or an error.
pub fn handle_app_actions(
    app_context: &mut AppContext,
    tui_backend: &mut TuiBackend,
) -> Result<(), AppError> {
    while let Ok(action) = app_context.action_rx_mut_ref().try_recv() {
        match action {
            Action::Render => {
                tui_backend.terminal.draw(|f| {
                    app_context.tui_mut_ref().draw(f, f.size()).unwrap();
                })?;
            }
            Action::Resize(width, height) => {
                tui_backend
                    .terminal
                    .resize(Rect::new(0, 0, width, height))?;
                tui_backend.terminal.draw(|f| {
                    app_context.tui_mut_ref().draw(f, f.size()).unwrap();
                })?;
            }
            Action::Quit => {
                app_context.quit = true;
            }
            _ => {}
        }

        if let Some(action) =
            app_context.tui_mut_ref().update(action.clone())?
        {
            app_context.action_tx_ref().send(action)?;
        }
    }
    Ok(())
}

// let mut size: u16 = 20;
// let mut should_quit = false;
// while !should_quit {
//   tui.terminal.draw(|f| Self::ui(size, f))?;
//   size = ((size as i16) + Self::handle_events_size()?) as u16;
//   should_quit = Self::handle_events_quit()?;
// }