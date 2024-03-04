use {
  crate::{
    components::{chat_list_window::ChatListWindow, chat_window::ChatWindow, prompt_window::PromptWindow},
    enums::{action::Action, component_name::ComponentName},
    traits::{component::Component, handle_small_area::HandleSmallArea},
  },
  ratatui::layout,
  std::{collections::HashMap, io},
  tokio::sync::mpsc,
};

pub struct CoreWindow {
  name: String,
  command_tx: Option<mpsc::UnboundedSender<Action>>,
  components: HashMap<ComponentName, Box<dyn Component>>,
  small_area: bool,
  #[allow(dead_code)]
  focused: ComponentName,
}

impl CoreWindow {
  pub fn new() -> Self {
    let components_iter: Vec<(ComponentName, Box<dyn Component>)> = vec![
      (ComponentName::ChatList, ChatListWindow::new().name("Chats").new_boxed()),
      (ComponentName::Chat, ChatWindow::new().name("Name").new_boxed()),
      (ComponentName::Prompt, PromptWindow::new().name("Prompt").new_boxed()),
    ];

    let name = "".to_string();
    let command_tx = None;
    let components: HashMap<ComponentName, Box<dyn Component>> = components_iter.into_iter().collect();
    let small_area = false;
    let focused = ComponentName::ChatList;

    CoreWindow {
      name,
      command_tx,
      components,
      small_area,
      focused,
    }
  }

  pub fn name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    self
  }
}

impl HandleSmallArea for CoreWindow {
  fn small_area(&mut self, small: bool) {
    self.small_area = small;
    for (_, component) in self.components.iter_mut() {
      component.small_area(small);
    }
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
    // let size_chats = if area.width < SMALL_AREA_WIDTH { 0 } else { 20 };
    let size_chats = if self.small_area { 0 } else { 20 };
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
      .get_mut(&ComponentName::ChatList)
      .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::ChatList))
      .draw(frame, core_layout[0])?;

    let sub_core_layout = layout::Layout::default()
      .direction(layout::Direction::Vertical)
      .constraints([layout::Constraint::Fill(1), layout::Constraint::Length(size_prompt)])
      .split(core_layout[1]);

    self
      .components
      .get_mut(&ComponentName::Chat)
      .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::Chat))
      .draw(frame, sub_core_layout[0])?;

    self
      .components
      .get_mut(&ComponentName::Prompt)
      .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::Prompt))
      .draw(frame, sub_core_layout[1])?;

    Ok(())
  }
}
