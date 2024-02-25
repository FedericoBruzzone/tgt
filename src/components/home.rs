use std::io;

use super::Component;
use crate::action::Action;
use ratatui::{
  layout::{self, Alignment},
  symbols::{border, line},
  widgets::{
    block::{self, title, Position, Title},
    Borders,
  },
};
use tokio::sync::mpsc;

pub struct Home {
  command_tx: Option<mpsc::UnboundedSender<Action>>,
}

impl Home {
  pub fn new() -> Self {
    let command_tx = None;
    Home { command_tx }
  }
}

impl Component for Home {
  fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn update(&mut self, action: Action) -> io::Result<Option<Action>> {
    Ok(None)
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: layout::Rect) -> io::Result<()> {
    let small_area = area.width < 50;
    let size_chats = if small_area { 0 } else { 20 };
    let size_prompt = 3;

    let main_layout = layout::Layout::new(
      layout::Direction::Vertical,
      [
        layout::Constraint::Length(1),
        layout::Constraint::Min(20),
        layout::Constraint::Length(1),
      ],
    )
    .split(area);

    frame.render_widget(
      block::Block::new().borders(Borders::TOP).title(
        Title::from("TG-TUI")
          .position(Position::Top)
          .alignment(Alignment::Center),
      ),
      main_layout[0],
    );
    frame.render_widget(
      block::Block::new()
        .borders(Borders::BOTTOM)
        .title(title::Title::from("Status").position(title::Position::Bottom))
        .title(
          title::Title::from(area.width.to_string() + "x" + area.height.to_string().as_str())
            .position(title::Position::Bottom)
            .alignment(layout::Alignment::Center),
        ),
      main_layout[2],
    );

    let layout = layout::Layout::default()
      .direction(layout::Direction::Horizontal)
      .constraints([
        layout::Constraint::Percentage(size_chats),
        layout::Constraint::Percentage(100 - size_chats),
      ])
      .split(main_layout[1]);

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
