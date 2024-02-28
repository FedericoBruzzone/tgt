use crate::action::Action;
use crate::components::chats_window::{ChatsWindow, CHATS};
use crate::traits::component::Component;
use ratatui::{
  layout,
  symbols::{border, line},
  widgets::{block, Borders},
};
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
    let components_iter: Vec<(&str, Box<dyn Component>)> =
      vec![(CHATS, ChatsWindow::new().name("Chats").new_boxed())];

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
    let small_area = area.width < 50;
    let size_chats = if small_area { 0 } else { 20 };
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
      .get_mut(CHATS)
      .unwrap_or_else(|| panic!("Failed to get component: {}", CHATS))
      .draw(frame, core_layout[0])?;

    let sub_core_layout = layout::Layout::default()
      .direction(layout::Direction::Vertical)
      .constraints([
        layout::Constraint::Fill(1),
        layout::Constraint::Length(size_prompt),
      ])
      .split(core_layout[1]);

    let top_right_border_set = if small_area {
      border::PLAIN
    } else {
      border::Set {
        top_left: line::NORMAL.horizontal_down,
        bottom_left: line::NORMAL.horizontal_up,
        ..border::PLAIN
      }
    };
    frame.render_widget(
      block::Block::new()
        .border_set(top_right_border_set)
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .title("Name"),
      sub_core_layout[0],
    );

    let collapsed_top_and_left_border_set = border::Set {
      top_left: line::NORMAL.vertical_right,
      top_right: line::NORMAL.vertical_left,
      bottom_left: if small_area {
        line::NORMAL.bottom_left
      } else {
        line::NORMAL.horizontal_up
      },
      ..border::PLAIN
    };
    frame.render_widget(
      block::Block::new()
        .border_set(collapsed_top_and_left_border_set)
        .borders(Borders::ALL)
        .title("Prompt Window"),
      sub_core_layout[1],
    );

    Ok(())
  }
}
