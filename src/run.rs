use {
    crate::{
        app_context::AppContext,
        app_error::AppError,
        enums::{action::Action, event::Event},
        tui_backend::TuiBackend,
    },
    ratatui::layout::Rect,
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
            // Event::Quit => action_tx.send(Action::Quit)?,
            // Event::Key(key) => action_tx.send(Action::Key(key))?,
            Event::Render => {
                app_context.action_tx_ref().send(Action::Render)?
            }
            Event::Resize(width, height) => app_context
                .action_tx_ref()
                .send(Action::Resize(width, height))?,
            Event::Mouse(mouse) => {
                app_context.action_tx_ref().send(Action::Mouse(mouse))?
            }
            _ =>
                /* Event::Init */
                {}
        }
        if let Some(action) = app_context
            .tui_mut_ref()
            .handle_events(Some(event.clone()))?
        {
            app_context.action_tx_ref().send(action)?;
        }
    }
    Ok(())
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
                    // TODO: handle with AppError
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
