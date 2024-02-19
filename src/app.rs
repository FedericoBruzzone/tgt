use crate::tui;
use crossterm::event;
use ratatui::{
  layout,
  symbols::{border, line},
  widgets::{block, block::title, Borders},
};
use std::io;
use std::time;
use tokio::sync::mpsc;

#[derive(Default)]
pub struct App {}

impl App {
  pub fn new() -> Self {
    Self::default()
  }

  pub async fn run(&mut self) -> Result<(), io::Error> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();
    let mut tui = tui::Tui::new()?.mouse(true).paste(true);
    tui.enter()?;

    loop {
      if let Some(event) = tui.next().await {
        match event {}
      }
    }

    let mut size: u16 = 20;

    let mut should_quit = false;
    while !should_quit {
      tui.terminal.draw(|f| Self::ui(size, f))?;
      size = ((size as i16) + Self::handle_events_size()?) as u16;
      should_quit = Self::handle_events_quit()?;
    }

    tui.exit()?;

    Ok(())
  }

  fn handle_events_size() -> io::Result<i16> {
    if event::poll(std::time::Duration::from_millis(50))? {
      if let event::Event::Key(key) = event::read()? {
        if key.kind == event::KeyEventKind::Press {
          match key.code {
            event::KeyCode::Char('1') => return Ok(1),
            event::KeyCode::Char('2') => return Ok(-1),
            _ => {},
          }
        }
      }
    }
    Ok(0)
  }

  fn handle_events_quit() -> io::Result<bool> {
    if event::poll(time::Duration::from_millis(50))? {
      if let event::Event::Key(key) = event::read()? {
        if key.kind == event::KeyEventKind::Press && key.code == event::KeyCode::Char('q') {
          return Ok(true);
        }
      }
    }
    Ok(false)
  }

  fn ui(size: u16, frame: &mut ratatui::Frame) {
    let main_layout = layout::Layout::new(
      layout::Direction::Vertical,
      [layout::Constraint::Length(5), layout::Constraint::Min(0), layout::Constraint::Length(5)],
    )
    .split(frame.size());
    frame.render_widget(block::Block::new().borders(Borders::TOP).title("Title Bar"), main_layout[0]);
    frame.render_widget(
      block::Block::new()
        .borders(Borders::BOTTOM)
        .title(title::Title::from("Status Bar").position(title::Position::Bottom)),
      main_layout[2],
    );

    let layout = layout::Layout::default()
      .direction(layout::Direction::Horizontal)
      .constraints([layout::Constraint::Percentage(size), layout::Constraint::Percentage(100 - size)])
      .split(main_layout[1]);

    frame.render_widget(
      block::Block::new()
        .border_set(border::PLAIN)
        .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
        .title("Left Block"),
      layout[0],
    );

    let top_right_border_set =
      border::Set { top_left: line::NORMAL.horizontal_down, bottom_left: line::NORMAL.horizontal_up, ..border::PLAIN };
    frame.render_widget(
      block::Block::new().border_set(top_right_border_set).borders(Borders::ALL).title("Top Right Block"),
      layout[1],
    );
  }
}
