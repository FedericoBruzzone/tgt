use crate::{
    action::Action,
    app_context::AppContext,
    components::component::{Component, HandleFocus, HandleSmallArea},
    event::Event,
};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    layout::Rect,
    symbols::{
        border::{Set, PLAIN},
        line::NORMAL,
    },
    widgets::{block::Block, Borders, Paragraph},
    Frame,
};
use std::{io, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

enum InputMode {
    Normal,
    Input,
}

struct Input {
    text: Vec<Vec<char>>,
    cursor: (usize, usize),
    area_input: Rect,
    command_tx: Option<UnboundedSender<Action>>,
}

impl Input {
    fn set_command_tx(&mut self, command_tx: UnboundedSender<Action>) {
        self.command_tx = Some(command_tx);
    }

    fn cursor_x(&self) -> usize {
        self.cursor.0
    }

    fn cursor_y(&self) -> usize {
        self.cursor.1
    }

    fn text(&mut self) -> String {
        self.text
            .iter()
            .map(|line| line.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn insert(&mut self, c: char) {
        // The -2 is to account the cursor at the end of the line.
        if self.cursor.0 + 1 == (self.area_input.width - 2) as usize {
            self.insert_newline();
        }
        self.text[self.cursor.1].insert(self.cursor.0, c);
        self.cursor.0 += 1;
    }

    fn insert_newline(&mut self) {
        let line = &mut self.text[self.cursor.1];
        let right = line[self.cursor.0..].to_vec();
        line.truncate(self.cursor.0);
        self.text.insert(self.cursor.1 + 1, right);
        self.cursor.0 = 0;
        self.cursor.1 += 1;
        if let Some(tx) = self.command_tx.as_ref() {
            tx.send(Action::IncreasePromptSize).unwrap()
        }
    }

    fn backspace(&mut self) {
        if self.text[self.cursor.1].is_empty() && self.cursor.1 > 0 {
            if let Some(tx) = self.command_tx.as_ref() {
                tx.send(Action::DecreasePromptSize).unwrap();
            }
        }
        if self.cursor.0 == 0 {
            if self.cursor.1 > 0 {
                let line = self.text.remove(self.cursor.1);
                self.cursor.1 -= 1;
                self.cursor.0 = self.text[self.cursor.1].len();
                self.text[self.cursor.1].extend(line);
            }
        } else {
            self.text[self.cursor.1].remove(self.cursor.0 - 1);
            self.cursor.0 -= 1;
        }
    }

    fn delete(&mut self) {
        if self.cursor.0 == self.text[self.cursor.1].len() {
            if self.cursor.1 < self.text.len() - 1 {
                let line = self.text.remove(self.cursor.1 + 1);
                self.text[self.cursor.1].extend(line);
            }
        } else {
            self.text[self.cursor.1].remove(self.cursor.0);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor.0 < self.text[self.cursor.1].len() {
            self.cursor.0 += 1;
        }
    }

    fn move_cursor_up(&mut self) {
        if self.cursor.1 > 0 {
            // Handle moving cursor to the left if the current line is shorter
            // than the previous line.
            if self.cursor.0 > self.text[self.cursor.1 - 1].len() {
                self.cursor.0 = self.text[self.cursor.1 - 1].len();
            }
            self.cursor.1 -= 1;
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor.1 < self.text.len() - 1 {
            // Handle moving cursor to the left if the current line is shorter
            // than the next line.
            if self.cursor.0 > self.text[self.cursor.1 + 1].len() {
                self.cursor.0 = self.text[self.cursor.1 + 1].len();
            }
            self.cursor.1 += 1;
        }
    }

    fn move_cursor_to_previous_word(&mut self) {
        let line = &self.text[self.cursor.1];
        let mut i = self.cursor.0;
        while i > 0 && line[i - 1].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !line[i - 1].is_whitespace() {
            i -= 1;
        }
        self.cursor.0 = i;
    }

    fn move_cursor_to_next_word(&mut self) {
        let line = &self.text[self.cursor.1];
        let mut i = self.cursor.0;
        while i < line.len() && line[i].is_whitespace() {
            i += 1;
        }
        while i < line.len() && !line[i].is_whitespace() {
            i += 1;
        }
        self.cursor.0 = i;
    }

    fn move_cursor_to_end(&mut self) {
        self.cursor.0 = self.text[self.cursor.1].len();
    }

    fn move_cursor_to_start(&mut self) {
        self.cursor.0 = 0;
    }

    fn delete_previous_word(&mut self) {
        let line = &mut self.text[self.cursor.1];
        let mut i = self.cursor.0;
        while i > 0 && line[i - 1].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !line[i - 1].is_whitespace() {
            i -= 1;
        }
        line.drain(i..self.cursor.0);
        self.cursor.0 = i;
    }

    fn paste(&mut self, text: String) {
        for c in text.chars() {
            self.insert(c);
        }
    }
}
/// Implement the `Default` trait for the `Input` struct.
impl Default for Input {
    fn default() -> Self {
        Self {
            text: vec![vec![]],
            cursor: (0, 0),
            area_input: Rect::default(),
            command_tx: None,
        }
    }
}

/// `PromptWindow` is a struct that represents a window for displaying a prompt.
/// It is responsible for managing the layout and rendering of the prompt
/// window.
pub struct PromptWindow {
    /// The application context.
    app_context: Arc<AppContext>,
    /// The name of the `PromptWindow`.
    name: String,
    /// An unbounded sender that send action for processing.
    action_tx: Option<UnboundedSender<Action>>,
    /// A flag indicating whether the `PromptWindow` should be displayed as a
    /// smaller version of itself.
    small_area: bool,
    /// Indicates whether the `PromptWindow` is focused or not.
    focused: bool,
    /// The key that allows the `PromptWindow` to be focused.
    focused_key: String,
    /// The current input mode of the `PromptWindow`.
    /// Usually, when the `PromptWindow` is focused, the input mode is set to
    /// `Input`. Otherwise, it is set to `Normal`.
    input_mode: InputMode,
    /// The current input of the `PromptWindow`.
    input: Input,
}
/// Implement the `PromptWindow` struct.
impl PromptWindow {
    /// Create a new instance of the `PromptWindow` struct.
    ///
    /// # Arguments
    /// * `app_context` - An Arc wrapped AppContext struct.
    ///
    /// # Returns
    /// * `Self` - The new instance of the `PromptWindow` struct.
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let name = "".to_string();
        let command_tx = None;
        let small_area = false;
        let focused = false;
        let focused_key = "".to_string();
        let input_mode = InputMode::Normal;
        let input = Input::default();

        PromptWindow {
            app_context,
            name,
            action_tx: command_tx,
            small_area,
            focused,
            focused_key,
            input_mode,
            input,
        }
    }
    /// Set the name of the `PromptWindow`.
    ///
    /// # Arguments
    /// * `name` - The name of the `PromptWindow`
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `PromptWindow`.
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = name.as_ref().to_string();
        self
    }
    /// Set the focused key of the `PromptWindow`.
    /// It is the key that allows the `PromptWindow` to be focused.
    ///
    /// # Arguments
    /// * `event` - An optional event that contains the focused key.
    ///
    /// # Returns
    /// * `Self` - The modified instance of the `PromptWindow`.
    pub fn with_focused_key(mut self, event: Option<&Event>) -> Self {
        if let Some(event) = event {
            self.focused_key = event.to_string();
        }
        self
    }
    /// Update the input area of the `PromptWindow`.
    /// It is used to update the input area of the `PromptWindow` when a new
    /// line is inserted or deleted.
    ///
    /// # Arguments
    /// * `area_input` - The new input area of the `PromptWindow`.
    pub fn update_input(&mut self, area_input: Rect) {
        if self.input.area_input != area_input {
            self.input.area_input = area_input;
        }
    }
}

/// Implement the `HandleFocus` trait for the `PromptWindow` struct.
/// This trait allows the `PromptWindow` to be focused or unfocused.
impl HandleFocus for PromptWindow {
    /// Set the `focused` flag for the `PromptWindow`.
    fn focus(&mut self) {
        self.focused = true;
        self.input_mode = InputMode::Input;
    }
    /// Set the `focused` flag for the `PromptWindow`.
    fn unfocus(&mut self) {
        self.focused = false;
        self.input_mode = InputMode::Normal;
    }
}

/// Implement the `HandleSmallArea` trait for the `PromptWindow` struct.
/// This trait allows the `PromptWindow` to display a smaller version of itself
/// if necessary.
impl HandleSmallArea for PromptWindow {
    /// Set the `small_area` flag for the `PromptWindow`.
    ///
    /// # Arguments
    /// * `small_area` - A boolean flag indicating whether the `PromptWindow`
    ///   should be displayed as a smaller version of itself.
    fn with_small_area(&mut self, small_area: bool) {
        self.small_area = small_area;
    }
}

/// Implement the `Component` trait for the `ChatListWindow` struct.
impl Component for PromptWindow {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> std::io::Result<()> {
        self.action_tx = Some(tx.clone());
        self.input.set_command_tx(tx);
        Ok(())
    }

    fn update(&mut self, action: Action) {
        match action {
            Action::Key(key_code, key_modifiers) => match (key_code, key_modifiers) {
                (KeyCode::Left, KeyModifiers::ALT) => {
                    self.input.move_cursor_to_previous_word();
                }
                (KeyCode::Right, KeyModifiers::ALT) => {
                    self.input.move_cursor_to_next_word();
                }
                (KeyCode::Backspace, KeyModifiers::ALT) => {
                    self.input.delete_previous_word();
                }
                (KeyCode::Left, KeyModifiers::SHIFT) => {
                    self.input.move_cursor_to_start();
                }
                (KeyCode::Right, KeyModifiers::SHIFT) => {
                    self.input.move_cursor_to_end();
                }
                (KeyCode::Char(c), _) => {
                    self.input.insert(c);
                }
                (KeyCode::Backspace, KeyModifiers::NONE) => {
                    self.input.backspace();
                }
                (KeyCode::Delete, KeyModifiers::NONE) => {
                    self.input.delete();
                }
                (KeyCode::Enter, KeyModifiers::NONE) => {
                    self.input.insert_newline();
                }
                (KeyCode::Left, KeyModifiers::NONE) => {
                    self.input.move_cursor_left();
                }
                (KeyCode::Right, KeyModifiers::NONE) => {
                    self.input.move_cursor_right();
                }
                (KeyCode::Up, KeyModifiers::NONE) => {
                    self.input.move_cursor_up();
                }
                (KeyCode::Down, KeyModifiers::NONE) => {
                    self.input.move_cursor_down();
                }
                _ => {}
            },
            Action::Paste(text) => {
                self.input.paste(text);
            }
            _ => {}
        }
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> io::Result<()> {
        self.update_input(area);

        let collapsed_top_and_left_border_set = Set {
            top_left: NORMAL.vertical_right,
            top_right: NORMAL.vertical_left,
            bottom_left: if self.small_area {
                NORMAL.bottom_left
            } else {
                NORMAL.horizontal_up
            },
            ..PLAIN
        };
        let (text, style_text, style_border_focused) = if self.focused {
            (
                self.input.text().clone(),
                self.app_context.style_prompt(),
                self.app_context.style_border_component_focused(),
            )
        } else {
            (
                format!("Press {} to send a message", self.focused_key),
                self.app_context.style_prompt_message_preview_text(),
                self.app_context.style_prompt(),
            )
        };

        let block = Block::new()
            .border_set(collapsed_top_and_left_border_set)
            .border_style(style_border_focused)
            .borders(Borders::ALL)
            .title(self.name.as_str());

        let input = Paragraph::new(text).style(style_text).block(block);

        frame.render_widget(input, area);

        if let InputMode::Input = self.input_mode {
            frame.set_cursor(
                area.x + self.input.cursor_x() as u16 + 1,
                area.y + self.input.cursor_y() as u16 + 1,
            );
        }
        Ok(())
    }
}
