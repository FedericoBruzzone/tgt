use crate::{
    action::Action,
    app_context::AppContext,
    components::component_traits::{Component, HandleFocus},
};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use dirs;
use ratatui::widgets::FrameExt as _;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, HighlightSpacing, Paragraph},
    Frame,
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder, Input as ExplorerInput, Theme};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug, Eq, PartialEq)]
enum DownloadPhase {
    FileName,
    PickFolder,
    /// Destination path already exists; user must confirm overwrite.
    ConfirmOverwrite,
}

/// Popup: edit file name (default from the message), then pick a folder; Alt+Enter saves there.
pub struct FileDownloadExplorer {
    app_context: Arc<AppContext>,
    name: String,
    action_tx: Option<UnboundedSender<Action>>,
    focused: bool,
    visible: bool,
    explorer: FileExplorer,
    phase: DownloadPhase,
    message_id: Option<i64>,
    /// Single-line file name (base name only; path separators stripped when saving).
    filename_chars: Vec<char>,
    filename_cursor: usize,
    /// Full path pending when asking to replace an existing file.
    pending_overwrite_dest: Option<String>,
}

impl FileDownloadExplorer {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let theme = Self::build_explorer_theme(Arc::clone(&app_context));
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let explorer = FileExplorerBuilder::default()
            .working_dir(home_dir)
            .theme(theme)
            .build()
            .unwrap_or_else(|_| {
                FileExplorerBuilder::build_with_theme(Self::build_explorer_theme(Arc::clone(
                    &app_context,
                )))
                .unwrap()
            });

        FileDownloadExplorer {
            app_context,
            name: String::new(),
            action_tx: None,
            focused: false,
            visible: false,
            explorer,
            phase: DownloadPhase::FileName,
            message_id: None,
            filename_chars: Vec::new(),
            filename_cursor: 0,
            pending_overwrite_dest: None,
        }
    }

    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }

    fn build_explorer_theme(app_context: Arc<AppContext>) -> Theme {
        let style_chat = app_context.style_chat();
        let border = app_context.style_border_component_focused();
        let item_file = app_context.style_chat_message_other_content();
        let dir_style = app_context.style_item_reply_target();
        let highlight_item = app_context.style_item_selected();
        let highlight_dir = highlight_item.add_modifier(Modifier::BOLD);

        let ac_title = Arc::clone(&app_context);
        let ac_footer = Arc::clone(&app_context);

        Theme::new()
            .with_block(
                Block::new()
                    .borders(Borders::ALL)
                    .border_style(border)
                    .style(style_chat),
            )
            .with_title_top(move |fe: &FileExplorer| {
                Line::from(vec![Span::styled(
                    fe.cwd().display().to_string(),
                    ac_title.style_timestamp(),
                )])
            })
            .with_title_bottom(move |fe: &FileExplorer| {
                let n = fe.files().len();
                Line::from(vec![Span::styled(
                    format!("[{n} entries]"),
                    ac_footer.style_timestamp(),
                )])
                .alignment(Alignment::Right)
            })
            .with_style(style_chat)
            .with_item_style(item_file)
            .with_dir_style(dir_style)
            .with_highlight_item_style(highlight_item)
            .with_highlight_dir_style(highlight_dir)
            .with_highlight_symbol("> ")
            .with_highlight_spacing(HighlightSpacing::Always)
    }

    fn reset_for_message(&mut self, message_id: i64) {
        self.message_id = Some(message_id);
        self.phase = DownloadPhase::FileName;
        self.filename_chars.clear();
        self.filename_cursor = 0;

        if let Some(msg) = self.app_context.tg_context().get_message(message_id) {
            if let Some((_file_id, default_name)) = msg.save_as_candidate() {
                self.filename_chars = default_name.chars().collect();
                self.filename_cursor = self.filename_chars.len();
            }
        }

        self.explorer
            .set_theme(Self::build_explorer_theme(Arc::clone(&self.app_context)));
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let _ = self.explorer.set_cwd(home_dir);
    }

    fn sanitized_filename(&self) -> Option<String> {
        let s: String = self.filename_chars.iter().collect();
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        Path::new(trimmed)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_string())
            .filter(|n| !n.is_empty())
    }

    fn hide(&mut self) {
        self.visible = false;
        self.phase = DownloadPhase::FileName;
        self.message_id = None;
        self.filename_chars.clear();
        self.filename_cursor = 0;
        self.pending_overwrite_dest = None;
    }

    fn folder_save_target_label(&self) -> String {
        format!("Save into: {}", self.explorer.cwd().display())
    }

    fn handle_filename_key(&mut self, key_code: KeyCode, modifiers: KeyModifiers) {
        match key_code {
            KeyCode::Esc => {
                if let Some(tx) = self.action_tx.as_ref() {
                    let _ = tx.send(Action::HideFileDownloadExplorer);
                }
            }
            KeyCode::Enter => {
                if self.sanitized_filename().is_some() {
                    self.phase = DownloadPhase::PickFolder;
                    self.explorer
                        .set_theme(Self::build_explorer_theme(Arc::clone(&self.app_context)));
                    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                    let _ = self.explorer.set_cwd(home_dir);
                }
            }
            KeyCode::Left if !modifiers.contains(KeyModifiers::ALT) => {
                self.filename_cursor = self.filename_cursor.saturating_sub(1);
            }
            KeyCode::Right => {
                if self.filename_cursor < self.filename_chars.len() {
                    self.filename_cursor += 1;
                }
            }
            KeyCode::Home => {
                self.filename_cursor = 0;
            }
            KeyCode::End => {
                self.filename_cursor = self.filename_chars.len();
            }
            KeyCode::Backspace => {
                if self.filename_cursor > 0 {
                    self.filename_cursor -= 1;
                    self.filename_chars.remove(self.filename_cursor);
                }
            }
            KeyCode::Delete => {
                if self.filename_cursor < self.filename_chars.len() {
                    self.filename_chars.remove(self.filename_cursor);
                }
            }
            KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                if !c.is_control() {
                    self.filename_chars.insert(self.filename_cursor, c);
                    self.filename_cursor += 1;
                }
            }
            _ => {}
        }
    }

    fn send_save_and_close(&mut self, message_id: i64, dest_path: String) {
        if let Some(tx) = self.action_tx.as_ref() {
            let _ = tx.send(Action::SaveChatFileAs {
                message_id,
                dest_path,
            });
            let _ = tx.send(Action::HideFileDownloadExplorer);
        }
    }

    fn confirm_save_to_folder(&mut self) {
        let Some(message_id) = self.message_id else {
            return;
        };
        let Some(base) = self.sanitized_filename() else {
            return;
        };

        // Always use the explorer's working directory — not the list selection (`..`, empty dirs, etc.).
        let folder = self.explorer.cwd().to_path_buf();
        let dest_path = folder.join(&base);
        let dest_string = match dest_path.to_str() {
            Some(s) => s.to_string(),
            None => {
                if let Some(tx) = self.action_tx.as_ref() {
                    let _ = tx.send(Action::StatusMessage(
                        "Save path contains invalid UTF-8.".into(),
                    ));
                }
                return;
            }
        };

        if Path::new(&dest_string).exists() {
            self.pending_overwrite_dest = Some(dest_string);
            self.phase = DownloadPhase::ConfirmOverwrite;
            return;
        }

        self.send_save_and_close(message_id, dest_string);
    }

    fn confirm_overwrite_proceed(&mut self) {
        let Some(message_id) = self.message_id else {
            return;
        };
        let Some(dest) = self.pending_overwrite_dest.take() else {
            self.phase = DownloadPhase::PickFolder;
            return;
        };
        self.send_save_and_close(message_id, dest);
    }

    fn confirm_overwrite_cancel(&mut self) {
        self.pending_overwrite_dest = None;
        self.phase = DownloadPhase::PickFolder;
    }

    fn handle_confirm_overwrite_key(&mut self, key_code: KeyCode, _modifiers: KeyModifiers) {
        match key_code {
            KeyCode::Esc => self.confirm_overwrite_cancel(),
            KeyCode::Enter => self.confirm_overwrite_proceed(),
            KeyCode::Char('y' | 'Y') => self.confirm_overwrite_proceed(),
            KeyCode::Char('n' | 'N') => self.confirm_overwrite_cancel(),
            _ => {}
        }
    }

    fn ellipsis_path(s: &str, max_chars: usize) -> String {
        let max_chars = max_chars.max(8);
        let count = s.chars().count();
        if count <= max_chars {
            return s.to_string();
        }
        let keep = max_chars.saturating_sub(3);
        format!("{}...", s.chars().take(keep).collect::<String>())
    }

    fn handle_folder_key(&mut self, key_code: KeyCode, modifiers: KeyModifiers) {
        if key_code == KeyCode::Esc {
            if let Some(tx) = self.action_tx.as_ref() {
                let _ = tx.send(Action::HideFileDownloadExplorer);
            }
            return;
        }

        if key_code == KeyCode::Enter && modifiers.contains(KeyModifiers::ALT) {
            self.confirm_save_to_folder();
            return;
        }

        if key_code == KeyCode::Enter && !modifiers.contains(KeyModifiers::ALT) {
            let current = self.explorer.current();
            if current.is_dir {
                let _ = self.explorer.handle(ExplorerInput::Right);
            }
            return;
        }

        let key_event = KeyEvent::new(key_code, modifiers);
        let ev = CrosstermEvent::Key(key_event);
        let _ = self.explorer.handle(&ev);
    }
}

impl HandleFocus for FileDownloadExplorer {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl Component for FileDownloadExplorer {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> std::io::Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::ShowFileDownloadExplorer(message_id) => {
                self.visible = true;
                self.focused = true;
                self.reset_for_message(message_id);
            }
            Action::HideFileDownloadExplorer => {
                self.hide();
            }
            Action::Key(key_code, modifiers) => {
                if !self.visible || !self.focused {
                    return;
                }
                let km: KeyModifiers = modifiers.into();
                match self.phase {
                    DownloadPhase::FileName => self.handle_filename_key(key_code, km),
                    DownloadPhase::PickFolder => self.handle_folder_key(key_code, km),
                    DownloadPhase::ConfirmOverwrite => {
                        self.handle_confirm_overwrite_key(key_code, km)
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

        self.explorer
            .set_theme(Self::build_explorer_theme(Arc::clone(&self.app_context)));

        let popup_w = area.width.min(100);
        let popup_h = area.height.min(28);
        let popup_x = area.x + area.width.saturating_sub(popup_w) / 2;
        let popup_y = area.y + area.height.saturating_sub(popup_h) / 2;
        let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

        frame.render_widget(Clear, popup_area);

        let title = match self.phase {
            DownloadPhase::ConfirmOverwrite => "Replace file?",
            _ => "Save file",
        };
        let block = Block::new()
            .borders(Borders::ALL)
            .title(title)
            .title_alignment(Alignment::Center)
            .border_style(self.app_context.style_border_component_focused())
            .style(self.app_context.style_chat());

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let instructions_height = 3;
        let name_block_height = if matches!(self.phase, DownloadPhase::FileName) {
            5u16
        } else {
            0u16
        };

        let constraints = if matches!(self.phase, DownloadPhase::FileName) {
            vec![
                Constraint::Length(name_block_height),
                Constraint::Min(3),
                Constraint::Length(instructions_height),
            ]
        } else {
            vec![Constraint::Min(3), Constraint::Length(instructions_height)]
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        if matches!(self.phase, DownloadPhase::FileName) {
            let name_area = layout[0];
            let hint = Paragraph::new(Line::from(vec![Span::styled(
                "File name (Enter = choose folder, Esc = cancel)",
                self.app_context.style_timestamp(),
            )]))
            .style(self.app_context.style_chat())
            .alignment(Alignment::Left);

            let (before, at_cursor, after): (String, char, String) =
                if self.filename_cursor >= self.filename_chars.len() {
                    (self.filename_chars.iter().collect(), ' ', String::new())
                } else {
                    let c = self.filename_chars[self.filename_cursor];
                    let b: String = self.filename_chars[..self.filename_cursor].iter().collect();
                    let a: String = self.filename_chars[self.filename_cursor + 1..]
                        .iter()
                        .collect();
                    (b, c, a)
                };

            let mut spans = vec![Span::styled(before, self.app_context.style_chat())];
            let cursor_style = self
                .app_context
                .style_chat()
                .add_modifier(Modifier::REVERSED);
            spans.push(Span::styled(at_cursor.to_string(), cursor_style));
            spans.push(Span::styled(after, self.app_context.style_chat()));

            let name_line = Paragraph::new(Line::from(spans)).style(self.app_context.style_chat());

            let inner_name = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(1)])
                .split(name_area);
            frame.render_widget(hint, inner_name[0]);
            frame.render_widget(name_line, inner_name[1]);
        }

        let (explorer_area, footer_idx) = if matches!(self.phase, DownloadPhase::FileName) {
            (layout[1], 2)
        } else {
            (layout[0], 1)
        };

        if matches!(self.phase, DownloadPhase::PickFolder) {
            frame.render_widget_ref(self.explorer.widget(), explorer_area);
        }

        if matches!(self.phase, DownloadPhase::ConfirmOverwrite) {
            let path_display = self.pending_overwrite_dest.as_deref().unwrap_or("");
            let budget = (popup_w as usize).saturating_sub(6).max(8);
            let body = vec![
                Line::from(vec![Span::styled(
                    "A file already exists at:",
                    self.app_context.style_timestamp(),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    Self::ellipsis_path(path_display, budget),
                    self.app_context.style_chat(),
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "Overwrite it?",
                    self.app_context.style_timestamp(),
                )]),
            ];
            frame.render_widget(
                Paragraph::new(body)
                    .style(self.app_context.style_chat())
                    .alignment(Alignment::Center),
                explorer_area,
            );
        }

        let footer = layout[footer_idx];
        let (label, keys) = match self.phase {
            DownloadPhase::FileName => (String::new(), "Enter=next  Esc=cancel"),
            DownloadPhase::PickFolder => (
                self.folder_save_target_label(),
                "Alt+Enter=save here  Enter=open folder  Esc=cancel",
            ),
            DownloadPhase::ConfirmOverwrite => (String::new(), "Y/Enter=overwrite  N/Esc=go back"),
        };

        let hint_line = if label.is_empty() {
            Line::from(vec![Span::styled(keys, self.app_context.style_timestamp())])
        } else {
            Line::from(vec![
                Span::styled(label, self.app_context.style_timestamp()),
                Span::raw("  "),
                Span::styled(keys, self.app_context.style_timestamp()),
            ])
        };

        frame.render_widget(
            Paragraph::new(hint_line)
                .style(self.app_context.style_chat())
                .alignment(Alignment::Center),
            footer,
        );

        Ok(())
    }
}
