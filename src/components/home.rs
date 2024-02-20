use std::io;

use super::Component;
use crate::action::Action;
use ratatui::{
  layout,
  symbols::{border, line},
  widgets::{block, block::title, Borders},
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
  fn register_action_handler(
    &mut self,
    tx: mpsc::UnboundedSender<Action>,
  ) -> io::Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn update(&mut self, action: Action) -> io::Result<Option<Action>> {
    Ok(None)
  }

  fn draw(
    &mut self,
    frame: &mut ratatui::Frame<'_>,
    area: layout::Rect,
  ) -> io::Result<()> {
    let size = 20;

    let main_layout = layout::Layout::new(
      layout::Direction::Vertical,
      [
        layout::Constraint::Length(1),
        layout::Constraint::Min(0),
        layout::Constraint::Length(1),
      ],
    )
    .split(frame.size());
    frame.render_widget(
      block::Block::new().borders(Borders::TOP).title("Title Bar"),
      main_layout[0],
    );
    frame.render_widget(
      block::Block::new().borders(Borders::BOTTOM).title(
        title::Title::from("Status Bar").position(title::Position::Bottom),
      ),
      main_layout[2],
    );

    let layout = layout::Layout::default()
      .direction(layout::Direction::Horizontal)
      .constraints([
        layout::Constraint::Percentage(size),
        layout::Constraint::Percentage(100 - size),
      ])
      .split(main_layout[1]);

    frame.render_widget(
      block::Block::new()
        .border_set(border::PLAIN)
        .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
        .title("Left Block"),
      layout[0],
    );

    let top_right_border_set = border::Set {
      top_left: line::NORMAL.horizontal_down,
      bottom_left: line::NORMAL.horizontal_up,
      ..border::PLAIN
    };
    frame.render_widget(
      block::Block::new()
        .border_set(top_right_border_set)
        .borders(Borders::ALL)
        .title("Top Right Block"),
      layout[1],
    );
    Ok(())
  }
}
