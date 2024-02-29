use crate::action::Action;
use crate::components::{
  chat_list_window::{ChatListWindow, CHAT_LIST},
  chat_window::{ChatWindow, CHAT},
  prompt_window::{PromptWindow, PROMPT},
  SMALL_AREA_WIDTH,
};
use crate::traits::component::Component;
use ratatui::layout;
use std::{collections::HashMap, io};
use tokio::sync::mpsc;

pub const CORE_WINDOW: &str = "core_window";

pub struct CoreWindow {
  name: String,
  command_tx: Option<mpsc::UnboundedSender<Action>>,
  components: HashMap<String, Box<dyn Component>>,
}

impl CoreWindow {
  pub fn new() -> Self {
    let components_iter: Vec<(&str, Box<dyn Component>)> = vec![
      (CHAT_LIST, ChatListWindow::new().name("Chats").new_boxed()),
      (CHAT, ChatWindow::new().name("Name").new_boxed()),
      (PROMPT, PromptWindow::new().name("Prompt").new_boxed()),
    ];

    let name = "".to_string();
    let command_tx = None;
    let components: HashMap<String, Box<dyn Component>> = components_iter
      .into_iter()
      .map(|(name, component)| (name.to_string(), component))
      .collect();

    CoreWindow {
      name,
      command_tx,
      components,
    }
  }

  pub fn name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    self
  }
}

impl Component for CoreWindow {
  fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
    self.command_tx = Some(tx.clone());
    for (_, component) in self.components.iter_mut() {
      component.register_action_handler(tx.clone())?;
    }
    Ok(())
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: layout::Rect) -> io::Result<()> {
    let size_chats = if area.width < SMALL_AREA_WIDTH { 0 } else { 20 };
    let size_prompt = 3;

    let core_layout = layout::Layout::default()
      .direction(layout::Direction::Horizontal)
      .constraints([
        layout::Constraint::Percentage(size_chats),
        layout::Constraint::Percentage(100 - size_chats),
      ])
      .split(area);

    self
      .components
      .get_mut(CHAT_LIST)
      .unwrap_or_else(|| panic!("Failed to get component: {}", CHAT_LIST))
      .draw(frame, core_layout[0])?;

    let sub_core_layout = layout::Layout::default()
      .direction(layout::Direction::Vertical)
      .constraints([
        layout::Constraint::Fill(1),
        layout::Constraint::Length(size_prompt),
      ])
      .split(core_layout[1]);

    self
      .components
      .get_mut(CHAT)
      .unwrap_or_else(|| panic!("Failed to get component: {}", CHAT))
      .draw(frame, sub_core_layout[0])?;

    self
      .components
      .get_mut(PROMPT)
      .unwrap_or_else(|| panic!("Failed to get component: {}", PROMPT))
      .draw(frame, sub_core_layout[1])?;

    Ok(())
  }
}
