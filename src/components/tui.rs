use crate::action::Action;
use crate::components::{
  core_window::{CoreWindow, CORE_WINDOW},
  status_bar::{StatusBar, STATUS_BAR},
  title_bar::{TitleBar, TITLE_BAR},
  SMALL_AREA_HEIGHT, SMALL_AREA_WIDTH,
};
use crate::traits::{component::Component, handle_small_area::HandleSmallArea};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::collections::HashMap;
use std::io;
use tokio::sync::mpsc;

pub struct Tui {
  command_tx: Option<mpsc::UnboundedSender<Action>>,
  components: HashMap<String, Box<dyn Component>>,
}

impl Tui {
  pub fn new() -> Self {
    let components_iter: Vec<(&str, Box<dyn Component>)> = vec![
      (TITLE_BAR, TitleBar::new().name("TG-TUI").new_boxed()),
      (CORE_WINDOW, CoreWindow::new().name("CoreWindow").new_boxed()),
      (STATUS_BAR, StatusBar::new().name("Status Bar").new_boxed()),
    ];

    let command_tx = None;
    let components: HashMap<String, Box<dyn Component>> = components_iter
      .into_iter()
      .map(|(name, component)| (name.to_string(), component))
      .collect();

    Tui { command_tx, components }
  }
}

impl HandleSmallArea for Tui {}

// TODO: MainWindow should not implement Component
impl Component for Tui {
  fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
    self.command_tx = Some(tx.clone());
    for (_, component) in self.components.iter_mut() {
      component.register_action_handler(tx.clone())?;
    }
    Ok(())
  }

  fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> io::Result<()> {
    if area.width < SMALL_AREA_WIDTH {
      self.components.iter_mut().for_each(|(_, component)| {
        component.small_area(true);
      });
    } else {
      self.components.iter_mut().for_each(|(_, component)| {
        component.small_area(false);
      });
    }

    let main_layout = Layout::new(
      Direction::Vertical,
      [
        Constraint::Length(1),
        Constraint::Min(SMALL_AREA_HEIGHT),
        Constraint::Length(1),
      ],
    )
    .split(area);

    self
      .components
      .get_mut(TITLE_BAR)
      .unwrap_or_else(|| panic!("Failed to get component: {}", TITLE_BAR))
      .draw(frame, main_layout[0])?;

    self
      .components
      .get_mut(CORE_WINDOW)
      .unwrap_or_else(|| panic!("Failed to get component: {}", CORE_WINDOW))
      .draw(frame, main_layout[1])?;

    self
      .components
      .get_mut(STATUS_BAR)
      .unwrap_or_else(|| panic!("Failed to get component: {}", STATUS_BAR))
      .draw(frame, main_layout[2])?;

    Ok(())
  }
}
