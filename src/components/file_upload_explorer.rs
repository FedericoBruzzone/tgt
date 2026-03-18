use crate::{
    action::Action,
    app_context::AppContext,
    components::component_traits::{Component, HandleFocus},
};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use dirs;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use ratatui::widgets::FrameExt as _;
use ratatui_explorer::{FileExplorer, FileExplorerBuilder, Input as ExplorerInput, Theme};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

/// Popup component that lets the user browse the filesystem and upload the selected file.
pub struct FileUploadExplorer {
    app_context: Arc<AppContext>,
    name: String,
    action_tx: Option<UnboundedSender<Action>>,
    focused: bool,
    visible: bool,
    explorer: FileExplorer,
}

impl FileUploadExplorer {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let theme = Self::build_explorer_theme(&app_context);

        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        let explorer = FileExplorerBuilder::default()
            .working_dir(home_dir)
            .theme(theme)
            .build()
            .unwrap_or_else(|_| FileExplorerBuilder::build_with_theme(Self::build_explorer_theme(&app_context)).unwrap());

        FileUploadExplorer {
            app_context,
            name: "".to_string(),
            action_tx: None,
            focused: false,
            visible: false,
            explorer,
        }
    }

    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }

    fn build_explorer_theme(app_context: &AppContext) -> Theme {
        let style_chat = app_context.style_chat();
        let style_item_selected = app_context.style_item_selected();
        let style_dir = app_context.style_item_reply_target();

        Theme::default()
            .with_style(style_chat)
            .with_item_style(style_chat)
            .with_dir_style(style_dir)
            .with_highlight_item_style(style_item_selected)
            .with_highlight_dir_style(style_item_selected)
            .with_highlight_symbol("> ".into())
    }

    fn current_selected_label(&self) -> String {
        let f = self.explorer.current();
        if f.is_dir {
            format!("Selected: {} (dir)", f.name)
        } else {
            format!("Selected: {} (file)", f.name)
        }
    }
}

impl HandleFocus for FileUploadExplorer {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl Component for FileUploadExplorer {
    fn register_action_handler(
        &mut self,
        tx: UnboundedSender<Action>,
    ) -> std::io::Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::ShowFileUploadExplorer => {
                self.visible = true;
                self.focused = true;

                // Reset browsing location when the popup is opened.
                let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                let _ = self.explorer.set_cwd(home_dir);
            }
            Action::HideFileUploadExplorer => {
                self.visible = false;
            }
            Action::Key(key_code, modifiers) => {
                if !self.visible || !self.focused {
                    return;
                }

                // Ratatui-explorer doesn't treat Enter as "select", so we map it here:
                // - If current selection is a directory: enter it.
                // - If current selection is a file: upload it.
                match key_code {
                    KeyCode::Enter => {
                        let current = self.explorer.current();
                        if current.is_dir {
                            let _ = self.explorer.handle(ExplorerInput::Right);
                        } else {
                            let path = current.path.display().to_string();
                            let filename = current.name.clone();

                            // Best-effort quick validation before handing it to TDLib.
                            if current.path.is_file() {
                                if let Some(tx) = self.action_tx.as_ref() {
                                    let _ = tx.send(Action::StatusMessage(format!(
                                        "Uploading: {}",
                                        filename
                                    )));
                                    let _ = tx.send(Action::UploadFile(path));
                                    let _ = tx.send(Action::HideFileUploadExplorer);
                                }
                            } else if let Some(tx) = self.action_tx.as_ref() {
                                let _ = tx.send(Action::StatusMessage(format!(
                                    "Selected path is not a file: {}",
                                    filename
                                )));
                            }
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        if let Some(tx) = self.action_tx.as_ref() {
                            let _ = tx.send(Action::HideFileUploadExplorer);
                        }
                    }
                    _ => {
                        let key_event = KeyEvent::new(key_code, KeyModifiers::from(modifiers));
                        let ev = CrosstermEvent::Key(key_event);
                        let _ = self.explorer.handle(&ev);
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> std::io::Result<()> {
        if !self.visible {
            return Ok(());
        }

        // Centered overlay.
        let popup_w = area.width.min(100);
        let popup_h = area.height.min(28);
        let popup_x = area.x + area.width.saturating_sub(popup_w) / 2;
        let popup_y = area.y + area.height.saturating_sub(popup_h) / 2;
        let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

        frame.render_widget(Clear, popup_area);

        let block = Block::new()
            .borders(Borders::ALL)
            .title("File Upload")
            .title_alignment(Alignment::Center)
            .border_style(self.app_context.style_border_component_focused())
            .style(self.app_context.style_chat());

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Split: explorer + instructions footer.
        let instructions_height = 3;
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(instructions_height)])
            .split(inner);

        frame.render_widget_ref(self.explorer.widget(), layout[0]);

        let selected_label = self.current_selected_label();
        let hint = vec![Line::from(vec![
            Span::styled(selected_label, self.app_context.style_timestamp()),
            Span::raw("  "),
            Span::styled("Enter=upload  Esc/q=cancel", self.app_context.style_timestamp()),
        ])];

        let instructions_area = layout[1];
        frame.render_widget(
            Paragraph::new(hint)
                .style(self.app_context.style_chat())
                .alignment(Alignment::Center),
            instructions_area,
        );

        Ok(())
    }
}

