use crate::{
    action::{Action, Modifiers},
    app_context::AppContext,
    components::component::{Component, HandleFocus, HandleSmallArea},
    event::Event,
};
use arboard::Clipboard;
use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    symbols::{
        border::{Set, PLAIN},
        line::NORMAL,
    },
    text::{Line, Span},
    widgets::{block::Block, Borders, Paragraph},
    Frame,
};
use std::{io, sync::Arc};
use tokio::sync::mpsc::UnboundedSender;

enum InputMode {
    Normal,
    Input,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct InputCell {
    c: char,
    selected: bool,
}

struct Input {
    text: Vec<Vec<InputCell>>,
    cursor: (usize, usize),
    area_input: Rect,
    command_tx: Option<UnboundedSender<Action>>,
    is_selecting: bool,
    correct_prompt_size: usize,
    is_restored: bool,
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

    fn text(&mut self) -> Vec<Vec<InputCell>> {
        self.text.clone()
    }

    fn insert(&mut self, c: char) {
        // The -2 is to account the cursor at the end of the line.
        if self.cursor.0 + 1 == (self.area_input.width - 2) as usize {
            self.insert_newline();
        }
        self.text[self.cursor.1].insert(self.cursor.0, InputCell { c, selected: false });
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
            self.correct_prompt_size += 1;
            tx.send(Action::IncreasePromptSize).unwrap()
        }
    }

    fn restore_prompt_size(&mut self) {
        if !self.is_restored {
            if let Some(tx) = self.command_tx.as_ref() {
                for _ in 0..self.correct_prompt_size {
                    tx.send(Action::IncreasePromptSize).unwrap();
                }
            }
            self.is_restored = true;
        }
    }

    fn set_prompt_size_to_one(&mut self) {
        if let Some(tx) = self.command_tx.as_ref() {
            for _ in 0..self.correct_prompt_size {
                tx.send(Action::DecreasePromptSize).unwrap();
            }
        }
        self.is_restored = false;
    }

    fn copy_selected(&self) {
        let mut clipboard = Clipboard::new().unwrap();
        let mut text = String::new();
        for (i, line) in self.text.iter().enumerate() {
            for cell in line {
                if cell.selected {
                    text.push(cell.c);
                }
            }
            if i < self.text.len() - 1 {
                text.push('\n');
            }
        }
        clipboard.set_text(text).unwrap();
    }

    fn backspace(&mut self) {
        if self.text[self.cursor.1].is_empty() && self.cursor.1 > 0 {
            if let Some(tx) = self.command_tx.as_ref() {
                self.correct_prompt_size -= 1;
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

    fn move_cursor_left_and_toggle_selection(&mut self) {
        self.is_selecting = true;
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.text[self.cursor.1][self.cursor.0].selected =
                !self.text[self.cursor.1][self.cursor.0].selected;
        }
    }

    fn move_cursor_right_and_toggle_selection(&mut self) {
        self.is_selecting = true;
        if self.cursor.0 < self.text[self.cursor.1].len() {
            self.text[self.cursor.1][self.cursor.0].selected =
                !self.text[self.cursor.1][self.cursor.0].selected;
            self.cursor.0 += 1;
        }
    }

    fn select_line(&mut self) {
        for cell in &mut self.text[self.cursor.1] {
            cell.selected = true;
        }
    }

    fn select_until_end_of_line(&mut self) {
        for cell in &mut self.text[self.cursor.1][self.cursor.0..] {
            cell.selected = true;
        }
    }

    fn select_until_start_of_line(&mut self) {
        for cell in &mut self.text[self.cursor.1][..self.cursor.0] {
            cell.selected = true;
        }
    }

    fn move_cursor_up_and_select_or_unselect(&mut self) {
        self.is_selecting = true;
        if self.cursor.1 > 0 {
            self.select_until_start_of_line();
            self.move_cursor_up();
            self.select_line();
        }
    }

    fn move_cursor_down_and_select_or_unselect(&mut self) {
        self.is_selecting = true;
        if self.cursor.1 < self.text.len() - 1 {
            self.select_until_end_of_line();
            self.move_cursor_down();
            self.select_line();
        }
    }

    fn unselect_all(&mut self) {
        if self.is_selecting {
            for line in &mut self.text {
                for cell in line {
                    cell.selected = false;
                }
            }
            self.is_selecting = false;
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
        while i > 0 && line[i - 1].c.is_whitespace() {
            i -= 1;
        }
        while i > 0 && !line[i - 1].c.is_whitespace() {
            i -= 1;
        }
        self.cursor.0 = i;
    }

    fn move_cursor_to_next_word(&mut self) {
        let line = &self.text[self.cursor.1];
        let mut i = self.cursor.0;
        while i < line.len() && line[i].c.is_whitespace() {
            i += 1;
        }
        while i < line.len() && !line[i].c.is_whitespace() {
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
        while i > 0 && line[i - 1].c.is_whitespace() {
            i -= 1;
        }
        while i > 0 && !line[i - 1].c.is_whitespace() {
            i -= 1;
        }
        line.drain(i..self.cursor.0);
        self.cursor.0 = i;
    }

    fn paste(&mut self, text: String) {
        for c in text.chars() {
            if c == '\n' {
                self.insert_newline();
            } else {
                self.insert(c);
            }
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
            is_selecting: false,
            correct_prompt_size: 0,
            is_restored: true,
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
            Action::Key(key_code, modifiers) => match (key_code, modifiers) {
                // Move cursor to the start of the line.
                (KeyCode::Home, ..)
                | (
                    KeyCode::Left | KeyCode::Char('b'),
                    Modifiers {
                        shift: true,
                        super_: true,
                        ..
                    },
                )
                | (
                    KeyCode::Left | KeyCode::Char('b'),
                    Modifiers {
                        control: true,
                        alt: true,
                        ..
                    },
                )
                | (KeyCode::Char('a'), Modifiers { control: true, .. }) => {
                    self.input.unselect_all();
                    self.input.move_cursor_to_start();
                }

                // Move cursor to the end of the line.
                (KeyCode::End, ..)
                | (
                    KeyCode::Right | KeyCode::Char('f'),
                    Modifiers {
                        shift: true,
                        super_: true,
                        ..
                    },
                )
                | (
                    KeyCode::Right | KeyCode::Char('f'),
                    Modifiers {
                        control: true,
                        alt: true,
                        ..
                    },
                )
                | (KeyCode::Char('e'), Modifiers { control: true, .. }) => {
                    self.input.unselect_all();
                    self.input.move_cursor_to_end();
                }
                // Select previous character.
                (KeyCode::Left, Modifiers { shift: true, .. }) => {
                    self.input.move_cursor_left_and_toggle_selection();
                }

                // Select next character.
                (KeyCode::Right, Modifiers { shift: true, .. }) => {
                    self.input.move_cursor_right_and_toggle_selection();
                }

                // Select previous line.
                (KeyCode::Up, Modifiers { shift: true, .. }) => {
                    self.input.move_cursor_up_and_select_or_unselect();
                }

                // Select next line.
                (KeyCode::Down, Modifiers { shift: true, .. }) => {
                    self.input.move_cursor_down_and_select_or_unselect();
                }

                // Copy selected text.
                (KeyCode::Char('c'), Modifiers { control: true, .. }) => {
                    self.input.copy_selected();
                    self.input.unselect_all();
                }

                // Paste text.
                (KeyCode::Char('v'), Modifiers { control: true, .. }) => {
                    let mut clipboard = Clipboard::new().unwrap();
                    if let Ok(text) = clipboard.get_text() {
                        self.input.paste(text);
                    }
                }

                // Move cursor to the previous word.
                (KeyCode::Left | KeyCode::Char('b'), Modifiers { alt: true, .. }) => {
                    self.input.unselect_all();
                    self.input.move_cursor_to_previous_word();
                }

                // Move cursor to the next word.
                (KeyCode::Right | KeyCode::Char('f'), Modifiers { alt: true, .. }) => {
                    self.input.unselect_all();
                    self.input.move_cursor_to_next_word();
                }

                // Delete the previous word.
                (KeyCode::Backspace, Modifiers { alt: true, .. }) => {
                    self.input.unselect_all();
                    self.input.delete_previous_word();
                }

                // Insert a character.
                (
                    KeyCode::Char(c),
                    Modifiers {
                        alt: false,
                        control: false,
                        meta: false,
                        shift: false,
                        super_: false,
                        hyper: false,
                    },
                ) => {
                    self.input.unselect_all();
                    self.input.insert(c);
                }

                // Delete a character.
                (KeyCode::Backspace, ..) => {
                    self.input.unselect_all();
                    self.input.backspace();
                }

                // Delete a character.
                (KeyCode::Delete, ..) => {
                    self.input.unselect_all();
                    self.input.delete();
                }

                // Insert a newline.
                (KeyCode::Enter, ..) => {
                    self.input.unselect_all();
                    self.input.insert_newline();
                }

                // Move cursor to the left.
                (KeyCode::Left, ..) => {
                    self.input.unselect_all();
                    self.input.move_cursor_left();
                }

                // Move cursor to the right.
                (KeyCode::Right, ..) => {
                    self.input.unselect_all();
                    self.input.move_cursor_right();
                }

                // Move cursor up.
                (KeyCode::Up, ..) => {
                    self.input.unselect_all();
                    self.input.move_cursor_up();
                }

                // Move cursor down.
                (KeyCode::Down, ..) => {
                    self.input.unselect_all();
                    self.input.move_cursor_down();
                }
                _ => {}
            },
            Action::Paste(text) => {
                self.input.unselect_all();
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
            bottom_left: NORMAL.horizontal_up,
            ..PLAIN
        };
        let text = self
            .input
            .text()
            .iter()
            .map(|line| {
                Line::from(
                    line.iter()
                        .map(|cell| {
                            if cell.selected {
                                Span::styled(
                                    cell.c.to_string(),
                                    self.app_context.style_item_selected(),
                                )
                            } else {
                                Span::raw(cell.c.to_string())
                            }
                        })
                        .collect::<Vec<Span>>(),
                )
            })
            .collect::<Vec<Line>>();

        let (text, style_text, style_border_focused) = if self.focused {
            self.input.restore_prompt_size();
            (
                text,
                self.app_context.style_prompt(),
                self.app_context.style_border_component_focused(),
            )
        } else {
            self.input.set_prompt_size_to_one();
            (
                vec![Line::from(format!(
                    "Press {} to send a message",
                    self.focused_key
                ))],
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
