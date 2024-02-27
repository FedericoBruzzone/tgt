use super::Component;
use crate::action::Action;
use crate::components::{chats::Chats, status_bar::StatusBar, title_bar::TitleBar};
use ratatui::{
  layout::{self, Alignment},
  symbols::{border, line},
  widgets::{
    block::{self, title, Position, Title},
    Borders,
  },
};
use std::io;
use tokio::sync::mpsc;

pub struct Home {
  command_tx: Option<mpsc::UnboundedSender<Action>>,
  components: Vec<Box<dyn Component>>,
}

impl Home {
  pub fn new() -> Self {
    let command_tx = None;
    let components: Vec<Box<dyn Component>> = vec![
      TitleBar::new().name("TG-TUI".to_string()).new_boxed(),
      Chats::new().name("Chats".to_string()).new_boxed(),
      StatusBar::new().name("Status Bar".to_string()).new_boxed(),
    ];
    Home { command_tx, components }
  }
}

impl Component for Home {
  fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: layout::Rect) -> io::Result<()> {
    let small_area = area.width < 50;
    let size_chats = if small_area { 0 } else { 20 };
    let size_prompt = 3;

    let home_layout = layout::Layout::new(
      layout::Direction::Vertical,
      [
        layout::Constraint::Length(1),
        layout::Constraint::Min(20),
        layout::Constraint::Length(1),
      ],
    )
    .split(area);

    self.components[0].draw(frame, home_layout[0])?;
    self.components[1].draw(frame, home_layout[2])?;

    let layout = layout::Layout::default()
      .direction(layout::Direction::Horizontal)
      .constraints([
        layout::Constraint::Percentage(size_chats),
        layout::Constraint::Percentage(100 - size_chats),
      ])
      .split(home_layout[1]);

    let sub_layout = layout::Layout::default()
      .direction(layout::Direction::Vertical)
      .constraints([layout::Constraint::Fill(1), layout::Constraint::Length(size_prompt)])
      .split(layout[1]);

    frame.render_widget(
      block::Block::new()
        .border_set(border::PLAIN)
        .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
        .title("Chats"),
      layout[0],
    );

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
      sub_layout[0],
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
        .borders(Borders::ALL),
      // .title("Bottom Right Block"),
      sub_layout[1],
    );
    Ok(())
  }
}
