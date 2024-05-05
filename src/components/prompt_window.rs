use crate::{
    action::{Action, Modifiers},
    app_context::AppContext,
    components::component_traits::{Component, HandleFocus, HandleSmallArea},
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

/// `DirSelection` is an enum that represents the direction of the selection.
/// It is used to keep track of the direction of the selection when the user
/// is selecting text.
#[derive(Debug, Clone, Eq, PartialEq)]
enum DirSelection {
    /// The user is not selecting text.
    Empty,
    /// The user is selecting text to the left.
    Left,
    /// The user is selecting text to the right.
    Right,
    /// The user is selecting text up.
    Up,
    /// The user is selecting text down.
    Down,
}
/// `InputCell` is a struct that represents a cell of the input.
/// It is responsible for managing the input cell of the prompt.
#[derive(Debug, Clone, Eq, PartialEq)]
struct InputCell {
    /// The character of the input cell.
    c: char,
    /// A flag indicating whether the input cell is selected or not.
    selected: bool,
}
/// `Input` is a struct that represents the input of a prompt.
/// It is responsible for managing the input of the prompt.
struct Input {
    /// The text of the input.
    text: Vec<Vec<InputCell>>,
    /// The cursor position of the input.
    cursor: (usize, usize),
    /// The area of the component where the input is displayed.
    area_input: Rect,
    /// An unbounded sender that send action for processing.
    /// It is used to send actions to the main event loop for processing.
    /// Basically, it is used to send `IncreasePromptSize` and `DecreasePromptSize`
    /// actions.
    command_tx: Option<UnboundedSender<Action>>,
    /// An enum that represents the direction of the selection.
    /// Implicitly, it is used to keep track whether the user is selecting text
    /// or not.
    dir_selection: DirSelection,
    /// The correct prompt size.
    /// It is used to keep track of the correct prompt size when the prompt
    /// window is focused or unfocused.
    correct_prompt_size: usize,
    /// A flag indicating whether the prompt size is restored or not.
    /// Basically, it is used to keep track of whether the prompt size is
    /// equal to the correct prompt size or not.
    is_restored: bool,
}
/// Implement the `Input` struct.
impl Input {
    /// Set the command sender of the `Input` struct.
    ///
    /// # Arguments
    /// * `command_tx` - An unbounded sender that send action for processing.
    fn set_command_tx(&mut self, command_tx: UnboundedSender<Action>) {
        self.command_tx = Some(command_tx);
    }
    /// Get the cursor x position of the `Input` struct.
    fn cursor_x(&self) -> usize {
        self.cursor.0
    }
    /// Get the cursor y position of the `Input` struct.
    fn cursor_y(&self) -> usize {
        self.cursor.1
    }
    /// Get the text of the `Input` struct.
    fn text(&mut self) -> &Vec<Vec<InputCell>> {
        &self.text
    }
    /// Insert a character into the `Input` struct.
    /// The character is inserted at the current cursor position.
    /// If the cursor is at the end of the line, a new line is inserted.
    ///
    /// # Arguments
    /// * `c` - The character to insert.
    fn insert(&mut self, c: char) {
        // The -2 is to account the cursor at the end of the line.
        if self.cursor.0 + 1 == (self.area_input.width - 2) as usize {
            self.insert_newline();
        }
        self.text[self.cursor.1].insert(self.cursor.0, InputCell { c, selected: false });
        self.cursor.0 += 1;
    }
    /// Insert a newline into the `Input` struct.
    /// A newline is inserted at the current cursor position.
    /// The text after the cursor position is moved to the next line.
    fn insert_newline(&mut self) {
        self.insert('\n');
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
    /// Delete the character before the cursor position.
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
    /// Delete the character next to the cursor position.
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
    /// Move the cursor to the left.
    fn move_cursor_left(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
        }
    }
    /// Move the cursor to the right.
    fn move_cursor_right(&mut self) {
        if self.cursor.0 < self.text[self.cursor.1].len() {
            self.cursor.0 += 1;
        }
    }
    /// Move the cursor up.
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
    /// Move the cursor down.
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
    /// Move the cursor to the previous word.
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
    /// Move the cursor to the next word.
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
    /// Move the cursor to the start of the line.
    fn move_cursor_to_end(&mut self) {
        self.cursor.0 = self.text[self.cursor.1].len();
    }
    /// Move the cursor to the end of the line.
    fn move_cursor_to_start(&mut self) {
        self.cursor.0 = 0;
    }
    /// Delete the previous word.
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
    /// Restore the prompt size of the `Input` struct.
    /// It is used to restore the prompt size to the correct prompt size when
    /// the prompt window is focused.
    /// It sends `IncreasePromptSize` actions to the main event loop to increase
    /// the prompt size.
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
    /// Set the prompt size to one.
    /// It is used to set the prompt size to one.
    fn set_prompt_size_to_one(&mut self) {
        if let Some(tx) = self.command_tx.as_ref() {
            for _ in 0..self.correct_prompt_size {
                tx.send(Action::DecreasePromptSize).unwrap();
            }
        }
    }
    /// Set the prompt size to one.
    /// It is used to set the prompt size to one when the prompt window is
    /// focused.
    /// Simply, it set the correct prompt size to zero and the cursor to the
    /// start of the line.
    fn set_prompt_size_to_one_focused(&mut self) {
        self.set_prompt_size_to_one();
        self.correct_prompt_size = 0;
        self.cursor = (0, 0);
    }
    /// Set the prompt size to one.
    /// It is used to set the prompt size to one when the prompt window is
    /// unfocused.
    /// It means that when the prompt window will return to focus, the prompt
    /// size will be restored to the correct prompt size.
    fn set_prompt_size_to_one_unfocused(&mut self) {
        self.set_prompt_size_to_one();
        self.is_restored = false;
    }
    /// Move the cursor to the left and toggle the selection.
    fn move_cursor_left_and_toggle_selection(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.text[self.cursor.1][self.cursor.0].selected =
                !self.text[self.cursor.1][self.cursor.0].selected;
        }
        self.dir_selection = DirSelection::Left;
    }
    /// Move the cursor to the right and toggle the selection.
    fn move_cursor_right_and_toggle_selection(&mut self) {
        if self.cursor.0 < self.text[self.cursor.1].len() {
            self.text[self.cursor.1][self.cursor.0].selected =
                !self.text[self.cursor.1][self.cursor.0].selected;
            self.cursor.0 += 1;
        }
        self.dir_selection = DirSelection::Right;
    }
    // Move the cursor to the previous word and toggle the selection.
    fn move_cursor_to_previous_word_and_toggle_selection(&mut self) {
        let line = &self.text[self.cursor.1];
        let mut i = self.cursor.0;
        while i > 0 && line[i - 1].c.is_whitespace() {
            i -= 1;
        }
        while i > 0 && !line[i - 1].c.is_whitespace() {
            i -= 1;
        }
        for cell in &mut self.text[self.cursor.1][i..self.cursor.0] {
            cell.selected = !cell.selected;
        }
        self.cursor.0 = i;
        self.dir_selection = DirSelection::Left;
    }
    /// Move the cursor to the next word and toggle the selection.
    fn move_cursor_to_next_word_and_toggle_selection(&mut self) {
        let line = &self.text[self.cursor.1];
        let mut i = self.cursor.0;
        while i < line.len() && line[i].c.is_whitespace() {
            i += 1;
        }
        while i < line.len() && !line[i].c.is_whitespace() {
            i += 1;
        }
        for cell in &mut self.text[self.cursor.1][self.cursor.0..i] {
            cell.selected = !cell.selected;
        }
        self.cursor.0 = i;
        self.dir_selection = DirSelection::Right;
    }
    /// Move the cursor up and toggle the selection.
    fn move_cursor_up_and_toggle_selection(&mut self) {
        if self.cursor.1 > 0 {
            self.toggle_selection_from_cursor_to_start_of_line();
            self.move_cursor_up();
            self.toggle_selection_from_end_of_line_to_cursor();
            self.dir_selection = DirSelection::Up;
        }
    }
    /// Move the cursor down and toggle the selection.
    fn move_cursor_down_and_toggle_selection(&mut self) {
        if self.cursor.1 < self.text.len() - 1 {
            self.toggle_selection_from_cursor_to_end_of_line();
            self.move_cursor_down();
            self.toggle_selection_from_start_of_line_to_cursor();
            self.dir_selection = DirSelection::Down;
        }
    }
    /// Toggle the selection from the cursor to the start of the line.
    fn toggle_selection_from_cursor_to_start_of_line(&mut self) {
        for cell in &mut self.text[self.cursor.1][..self.cursor.0] {
            cell.selected = !cell.selected;
        }
    }
    /// Toggle the selection from the cursor to the end of the line.
    fn toggle_selection_from_cursor_to_end_of_line(&mut self) {
        for cell in &mut self.text[self.cursor.1][self.cursor.0..] {
            cell.selected = !cell.selected;
        }
    }
    /// Toggle the selection from the start of the line to the cursor.
    fn toggle_selection_from_start_of_line_to_cursor(&mut self) {
        for cell in &mut self.text[self.cursor.1][..self.cursor.0] {
            cell.selected = !cell.selected;
        }
    }
    /// Toggle the selection from the end of the line to the cursor.
    fn toggle_selection_from_end_of_line_to_cursor(&mut self) {
        for cell in &mut self.text[self.cursor.1][self.cursor.0..] {
            cell.selected = !cell.selected;
        }
    }
    /// Unselect all the text.
    fn unselect_all(&mut self) {
        if self.dir_selection != DirSelection::Empty {
            for line in &mut self.text {
                for cell in line {
                    cell.selected = false;
                }
            }
            self.dir_selection = DirSelection::Empty;
        }
    }
    /// Copy the selected text of the `Input` struct.
    /// The selected text is copied to the clipboard.
    fn copy_selected(&self) {
        if let Ok(mut clipboard) = Clipboard::new() {
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
    }
    /// Paste text into the `Input` struct.
    /// The text is pasted at the current cursor position.
    fn paste(&mut self, text: String) {
        for c in text.chars() {
            if c == '\n' {
                self.insert_newline();
            } else {
                self.insert(c);
            }
        }
    }
    /// Send a message.
    /// The message is sent to the main event loop for processing.
    ///
    /// # Arguments
    /// * `app_context` - An Arc wrapped AppContext struct.
    fn send_message(&mut self, app_context: Arc<AppContext>) {
        if let Some(event_tx) = app_context.tg_context().event_tx().as_ref() {
            event_tx
                .send(Event::SendMessage(self.text_to_string()))
                .unwrap();
            self.text = vec![vec![]];
            self.set_prompt_size_to_one_focused();
        }
    }
    /// Convert the text of the `Input` struct to a string.
    fn text_to_string(&mut self) -> String {
        // TODO: Parse into markdown
        let mut message = String::new();
        self.text.iter().for_each(|e| {
            if e.is_empty() {
                message.push('\n');
            } else {
                e.iter().for_each(|e| message.push(e.c))
            }
        });
        message
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
            dir_selection: DirSelection::Empty,
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
        let action_tx = None;
        let small_area = false;
        let focused = false;
        let focused_key = "".to_string();
        let input = Input::default();

        PromptWindow {
            app_context,
            name,
            action_tx,
            small_area,
            focused,
            focused_key,
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
    }
    /// Set the `focused` flag for the `PromptWindow`.
    fn unfocus(&mut self) {
        self.focused = false;
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

                (
                    KeyCode::Left,
                    Modifiers {
                        shift: true,
                        control: true,
                        ..
                    },
                ) => {
                    self.input
                        .move_cursor_to_previous_word_and_toggle_selection();
                }

                (
                    KeyCode::Right,
                    Modifiers {
                        shift: true,
                        control: true,
                        ..
                    },
                ) => {
                    self.input.move_cursor_to_next_word_and_toggle_selection();
                }

                (KeyCode::Left, Modifiers { shift: true, .. }) => {
                    self.input.move_cursor_left_and_toggle_selection();
                }

                (KeyCode::Right, Modifiers { shift: true, .. }) => {
                    self.input.move_cursor_right_and_toggle_selection();
                }

                (KeyCode::Up, Modifiers { shift: true, .. }) => {
                    self.input.move_cursor_up_and_toggle_selection();
                }

                (KeyCode::Down, Modifiers { shift: true, .. }) => {
                    self.input.move_cursor_down_and_toggle_selection();
                }

                (KeyCode::Char('c'), Modifiers { control: true, .. }) => {
                    self.input.copy_selected();
                    self.input.unselect_all();
                }

                (KeyCode::Char('v'), Modifiers { control: true, .. }) => {
                    if let Ok(mut clipboard) = Clipboard::new() {
                        if let Ok(text) = clipboard.get_text() {
                            self.input.paste(text);
                        }
                    }
                }

                (KeyCode::Left | KeyCode::Char('b'), Modifiers { alt: true, .. }) => {
                    self.input.unselect_all();
                    self.input.move_cursor_to_previous_word();
                }

                (KeyCode::Right | KeyCode::Char('f'), Modifiers { alt: true, .. }) => {
                    self.input.unselect_all();
                    self.input.move_cursor_to_next_word();
                }

                (KeyCode::Enter, Modifiers { alt: true, .. }) => {
                    self.input.unselect_all();
                    self.input.send_message(Arc::clone(&self.app_context));
                }

                (KeyCode::Backspace, Modifiers { alt: true, .. })
                | (KeyCode::Char('w'), Modifiers { control: true, .. }) => {
                    self.input.unselect_all();
                    self.input.delete_previous_word();
                }

                (
                    KeyCode::Char(c),
                    Modifiers {
                        alt: false,
                        control: false,
                        meta: false,
                        super_: false,
                        hyper: false,
                        ..
                    },
                ) => {
                    self.input.unselect_all();
                    self.input.insert(c);
                }

                (KeyCode::Backspace, ..) => {
                    self.input.unselect_all();
                    self.input.backspace();
                }

                (KeyCode::Delete, ..) => {
                    self.input.unselect_all();
                    self.input.delete();
                }

                (KeyCode::Enter, ..) => {
                    self.input.unselect_all();
                    self.input.insert_newline();
                }

                (KeyCode::Left, ..) => {
                    self.input.unselect_all();
                    self.input.move_cursor_left();
                }

                (KeyCode::Right, ..) => {
                    self.input.unselect_all();
                    self.input.move_cursor_right();
                }

                (KeyCode::Up, ..) => {
                    self.input.unselect_all();
                    self.input.move_cursor_up();
                }

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
                                    self.app_context.style_prompt_message_text_selected(),
                                )
                            } else {
                                Span::styled(
                                    cell.c.to_string(),
                                    self.app_context.style_prompt_message_text(),
                                )
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
            self.input.set_prompt_size_to_one_unfocused();
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

        if self.focused {
            frame.set_cursor(
                area.x + self.input.cursor_x() as u16 + 1,
                area.y + self.input.cursor_y() as u16 + 1,
            );
        }
        Ok(())
    }
}
