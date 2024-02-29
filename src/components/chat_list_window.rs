use crate::action::Action;
use crate::traits::{component::Component, handle_small_area::HandleSmallArea};
use ratatui::{
  layout,
  style::{Color, Modifier, Style},
  symbols::border,
  widgets::{
    block::{Block, Title},
    Borders, List, ListDirection,
  },
};
use std::io;
use tokio::sync::mpsc;

pub const CHAT_LIST: &str = "chat_list_window";

pub struct ChatListWindow {
  name: String,
  command_tx: Option<mpsc::UnboundedSender<Action>>,
  small_area: bool,
  chat_list: Vec<String>, // TODO: Use generic of Vec of Contact type
}

impl ChatListWindow {
  pub fn new() -> Self {
    let name = "".to_string();
    let command_tx = None;
    let small_area = false;
    let chat_list = vec![
      "Chat 1".to_string(),
      "Chat 2".to_string(),
      "Chat 2".to_string(),
      "Chat 2".to_string(),
      "Chat 2".to_string(),
      "Chat 2".to_string(),
    ];

    ChatListWindow {
      name,
      command_tx,
      small_area,
      chat_list,
    }
  }

  pub fn name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    self
  }
}

impl HandleSmallArea for ChatListWindow {
  fn small_area(&mut self, small: bool) {
    self.small_area = small;
  }
}

impl Component for ChatListWindow {
  fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
    self.command_tx = Some(tx.clone());
    Ok(())
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: layout::Rect) -> io::Result<()> {
    let list = List::new(self.chat_list.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
      .block(
        Block::default()
          .title("List")
          .border_set(border::PLAIN)
          .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
          .title(Title::from(self.name.as_str())),
      )
      .style(Style::default().fg(Color::White))
      .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
      .highlight_symbol(">>")
      .repeat_highlight_symbol(true)
      .direction(ListDirection::BottomToTop);
    frame.render_widget(list, area);
    Ok(())
  }
}
