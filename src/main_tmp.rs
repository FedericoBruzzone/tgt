// pub mod tui;
// pub mod utils;

use std::io::{self, stdout};

use crossterm::{
  event::{self, Event, KeyCode},
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
  ExecutableCommand,
};
use ratatui::{
  prelude::*,
  widgets::{block::*, *},
};

fn main() -> io::Result<()> {
  enable_raw_mode()?;
  stdout().execute(EnterAlternateScreen)?;
  let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
  let mut size: u16 = 20;

  let mut should_quit = false;
  while !should_quit {
    terminal.draw(move |frame: &mut Frame| ui(size, frame))?;
    size = ((size as i16) + handle_events_size()?) as u16;
    should_quit = handle_events()?;
  }

  disable_raw_mode()?;
  stdout().execute(LeaveAlternateScreen)?;
  Ok(())
}

fn handle_events_size() -> io::Result<i16> {
  if event::poll(std::time::Duration::from_millis(50))? {
    if let Event::Key(key) = event::read()? {
      if key.kind == event::KeyEventKind::Press {
        match key.code {
          KeyCode::Char('1') => return Ok(1),
          KeyCode::Char('2') => return Ok(-1),
          _ => {},
        }
      }
    }
  }
  Ok(0)
}

fn handle_events() -> io::Result<bool> {
  if event::poll(std::time::Duration::from_millis(50))? {
    if let Event::Key(key) = event::read()? {
      if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
        return Ok(true);
      }
    }
  }
  Ok(false)
}

fn ui(size: u16, frame: &mut Frame) {
  let main_layout =
    Layout::new(Direction::Vertical, [Constraint::Length(5), Constraint::Min(0), Constraint::Length(5)])
      .split(frame.size());
  frame.render_widget(Block::new().borders(Borders::TOP).title("Title Bar"), main_layout[0]);
  frame.render_widget(
    Block::new().borders(Borders::BOTTOM).title(Title::from("Status Bar").position(Position::Bottom)),
    main_layout[2],
  );

  let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(size), Constraint::Percentage(100 - size)])
    .split(main_layout[1]);

  frame.render_widget(
    Block::new()
      .border_set(symbols::border::PLAIN)
      .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
      .title("Left Block"),
    layout[0],
  );

  let top_right_border_set = symbols::border::Set {
    top_left: symbols::line::NORMAL.horizontal_down,
    bottom_left: symbols::line::NORMAL.horizontal_up,
    ..symbols::border::PLAIN
  };
  frame.render_widget(
    Block::new().border_set(top_right_border_set).borders(Borders::ALL).title("Top Right Block"),
    layout[1],
  );
}
