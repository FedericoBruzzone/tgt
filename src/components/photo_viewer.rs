use crate::{
    action::Action,
    app_context::AppContext,
    components::component_traits::{Component, HandleFocus},
    tg::message_entry::MessageContentType,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, Resize, StatefulImage};
use std::io::IsTerminal;
use std::{io, path::Path, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

/// Loading state for the photo viewer
#[allow(clippy::large_enum_variant)]
enum PhotoState {
    /// No photo selected
    None,
    /// Photo is being downloaded or decoded
    Loading { message_id: i64 },
    /// Photo is loaded and ready to display (dimensions stored to avoid keeping full image)
    Loaded {
        image_state: StatefulProtocol,
        width: u32,
        height: u32,
        /// Cached content area and image rect to avoid recalc every frame
        cached_rect: Option<(Rect, Rect)>,
    },
    /// Error loading photo
    Error { message: String },
}

/// `PhotoViewer` is a struct that represents a popup window for viewing photos.
pub struct PhotoViewer {
    /// The application context.
    app_context: Arc<AppContext>,
    /// The name of the `PhotoViewer`.
    name: String,
    /// An unbounded sender that send action for processing.
    action_tx: Option<UnboundedSender<Action>>,
    /// Indicates whether the `PhotoViewer` is focused or not.
    focused: bool,
    /// Indicates whether the photo viewer should be shown.
    visible: bool,
    /// The current photo state
    photo_state: PhotoState,
    /// Image picker for creating protocol handlers
    picker: Picker,
}

impl PhotoViewer {
    /// Create a new instance of the `PhotoViewer` struct.
    ///
    /// # Arguments
    /// * `app_context` - An Arc wrapped AppContext struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `PhotoViewer` struct.
    pub fn new(app_context: Arc<AppContext>) -> Self {
        // Create picker. Only query terminal when stdout is a TTY (real UI); in tests/CI
        // stdout is not a TTY and from_query_stdio() can block or timeout on Windows.
        let picker = if std::io::stdout().is_terminal() {
            Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks())
        } else {
            Picker::halfblocks()
        };

        PhotoViewer {
            app_context,
            name: "".to_string(),
            action_tx: None,
            focused: false,
            visible: false,
            photo_state: PhotoState::None,
            picker,
        }
    }

    /// Set the name of the `PhotoViewer`.
    ///
    /// # Arguments
    /// * `name` - The name of the `PhotoViewer`.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `PhotoViewer`.
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }

    /// Show the photo viewer and start loading a photo.
    pub fn show(&mut self, message_id: i64) {
        self.visible = true;
        self.photo_state = PhotoState::Loading { message_id };

        // Get the message and check if it's a photo
        if let Some(message) = self.app_context.tg_context().get_message(message_id) {
            if let MessageContentType::Photo {
                file_id: _,
                file_path,
            } = message.content_type()
            {
                // Check if file is already downloaded
                if !file_path.is_empty() && Path::new(file_path).exists() {
                    // Decode on background thread via run loop (avoids blocking main thread)
                    if let Some(tx) = self.action_tx.as_ref() {
                        let _ =
                            tx.send(Action::LoadPhotoFromPath(file_path.to_string(), message_id));
                    }
                }
                // If file doesn't exist, download will be handled by run.rs when it receives ViewPhotoMessage
                // Keep showing Loading state until PhotoDownloaded is received
            } else {
                self.photo_state = PhotoState::Error {
                    message: "Selected message is not a photo".to_string(),
                };
            }
        } else {
            self.photo_state = PhotoState::Error {
                message: "Message not found".to_string(),
            };
        }
    }

    /// Hide the photo viewer.
    pub fn hide(&mut self) {
        self.visible = false;
        self.photo_state = PhotoState::None;
    }

    /// Check if the photo viewer is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Apply decoded image from background (called when receiving PhotoDecoded).
    fn apply_decoded_photo(
        &mut self,
        message_id: i64,
        result: Result<image::DynamicImage, String>,
    ) {
        match result {
            Ok(img) => {
                let (width, height) = (img.width(), img.height());
                let image_state = self.picker.new_resize_protocol(img);
                self.photo_state = PhotoState::Loaded {
                    image_state,
                    width,
                    height,
                    cached_rect: None,
                };
            }
            Err(e) => {
                self.photo_state = PhotoState::Error { message: e };
            }
        }
        let _ = message_id;
    }

    /// Update photo state when download completes: request async decode (no blocking).
    pub fn on_photo_downloaded(&mut self, file_path: String) {
        if let PhotoState::Loading { message_id } = self.photo_state {
            if let Some(tx) = self.action_tx.as_ref() {
                let _ = tx.send(Action::LoadPhotoFromPath(file_path, message_id));
            }
        }
    }
}

impl HandleFocus for PhotoViewer {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl Component for PhotoViewer {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> io::Result<()> {
        self.action_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::ViewPhotoMessage(message_id) => {
                // Load and show the photo for this message
                self.show(message_id);
            }
            Action::ShowPhotoViewer => {
                // This should not reach PhotoViewer directly.
                // ChatWindow handles ShowPhotoViewer and sends ViewPhotoMessage with message_id.
            }
            Action::HidePhotoViewer => {
                self.hide();
            }
            Action::PhotoDownloaded(file_path) => {
                self.on_photo_downloaded(file_path);
            }
            Action::PhotoDecoded(message_id) => {
                if let PhotoState::Loading {
                    message_id: loading_id,
                } = self.photo_state
                {
                    if message_id == loading_id {
                        if let Some(result) =
                            self.app_context.take_pending_photo_decoded(message_id)
                        {
                            self.apply_decoded_photo(message_id, result);
                        }
                    }
                }
            }
            Action::PhotoViewerPrevious | Action::PhotoViewerNext => {
                // These actions are handled by CoreWindow, which forwards them to ChatWindow
                // No action needed here
            }
            Action::Key(key_code, _modifiers) => {
                if self.visible {
                    match key_code {
                        crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Char('q') => {
                            self.hide();
                            if let Some(tx) = self.action_tx.as_ref() {
                                tx.send(Action::HidePhotoViewer).unwrap_or(());
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> io::Result<()> {
        if !self.visible {
            return Ok(());
        }

        // Calculate popup size (80% width, 80% height, centered)
        let popup_width = (area.width as f32 * 0.8) as u16;
        let popup_height = (area.height as f32 * 0.8) as u16;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect::new(
            area.x + popup_x,
            area.y + popup_y,
            popup_width,
            popup_height,
        );

        // Clear the popup area first
        frame.render_widget(Clear, popup_area);

        let block = Block::new()
            .borders(Borders::ALL)
            .title("Photo Viewer")
            .title_alignment(Alignment::Center)
            .border_style(self.app_context.style_border_component_focused())
            .style(self.app_context.style_chat());

        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // Split inner area: main content area + instructions at bottom
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Main content area
                Constraint::Length(2), // Default keymap guide
            ])
            .split(inner_area);

        let content_area = layout[0];
        let instructions_area = layout[1];

        // Render based on state
        match &mut self.photo_state {
            PhotoState::None => {
                let text = vec![Line::from(Span::styled(
                    "No photo selected",
                    self.app_context.style_timestamp(),
                ))];
                let paragraph = Paragraph::new(text)
                    .style(self.app_context.style_chat())
                    .alignment(Alignment::Center);
                frame.render_widget(paragraph, content_area);
            }
            PhotoState::Loading { .. } => {
                let text = vec![Line::from(Span::styled(
                    "Loading photo...",
                    self.app_context.style_timestamp(),
                ))];
                let paragraph = Paragraph::new(text)
                    .style(self.app_context.style_chat())
                    .alignment(Alignment::Center);
                frame.render_widget(paragraph, content_area);
            }
            PhotoState::Loaded {
                image_state,
                width,
                height,
                cached_rect,
            } => {
                // Reuse cached rect when content area and dimensions unchanged to save CPU
                let image_rect = match cached_rect {
                    Some((ref cached_content, ref cached_image))
                        if *cached_content == content_area
                            && cached_image.width > 0
                            && cached_image.height > 0 =>
                    {
                        *cached_image
                    }
                    _ => {
                        let font_size = self.picker.font_size();
                        let font_width = font_size.0 as f32;
                        let font_height = font_size.1 as f32;
                        let (img_width, img_height) = (*width as f32, *height as f32);

                        let available_width_px = content_area.width as f32 * font_width;
                        let available_height_px = content_area.height as f32 * font_height;

                        let scale_x = available_width_px / img_width;
                        let scale_y = available_height_px / img_height;
                        let scale = scale_x.min(scale_y);

                        let display_width_px = img_width * scale;
                        let display_height_px = img_height * scale;

                        let display_width_cells = (display_width_px / font_width).ceil() as u16;
                        let display_height_cells = (display_height_px / font_height).ceil() as u16;

                        let x_offset = (content_area.width.saturating_sub(display_width_cells)) / 2;
                        let y_offset =
                            (content_area.height.saturating_sub(display_height_cells)) / 2;

                        let rect = Rect::new(
                            content_area.x + x_offset,
                            content_area.y + y_offset,
                            display_width_cells,
                            display_height_cells,
                        );
                        *cached_rect = Some((content_area, rect));
                        rect
                    }
                };

                let image_widget = StatefulImage::new().resize(Resize::Fit(None));
                frame.render_stateful_widget(image_widget, image_rect, image_state);
            }
            PhotoState::Error { message } => {
                let text = vec![
                    Line::from(Span::styled(
                        "Error",
                        self.app_context.style_border_component_focused(),
                    )),
                    Line::from(Span::styled(
                        message.clone(),
                        self.app_context.style_timestamp(),
                    )),
                ];
                let paragraph = Paragraph::new(text)
                    .style(self.app_context.style_chat())
                    .alignment(Alignment::Center);
                frame.render_widget(paragraph, content_area);
            }
        }

        // Draw default keymap guide at the bottom (matches config/keymap.toml [photo_viewer])
        let style_help = self.app_context.style_timestamp();
        let instructions = vec![
            Line::from(vec![
                Span::styled("Esc", style_help),
                Span::styled("Close", style_help),
            ]),
            Line::from(vec![
                Span::styled("Up, k ", style_help),
                Span::styled("Prev  ·  ", style_help),
                Span::styled("Down, j ", style_help),
                Span::styled("Next", style_help),
            ]),
        ];

        let instructions_paragraph = Paragraph::new(instructions)
            .style(self.app_context.style_chat())
            .alignment(Alignment::Center);
        frame.render_widget(instructions_paragraph, instructions_area);

        Ok(())
    }
}
