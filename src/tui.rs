use {
  crate::{
    action::Action,
    components::{
      core_window::CoreWindow, status_bar::StatusBar, title_bar::TitleBar, ComponentName, SMALL_AREA_HEIGHT,
      SMALL_AREA_WIDTH,
    },
    traits::component::Component,
    tui_backend::Event,
  },
  ratatui::layout::{Constraint, Direction, Layout, Rect},
  std::{collections::HashMap, io},
  tokio::sync::mpsc,
};

pub struct Tui {
  command_tx: Option<mpsc::UnboundedSender<Action>>,
  components: HashMap<ComponentName, Box<dyn Component>>,
}

impl Tui {
  pub fn new() -> Self {
    let components_iter: Vec<(ComponentName, Box<dyn Component>)> = vec![
      (ComponentName::TitleBar, TitleBar::new().name("TG-TUI").new_boxed()),
      (
        ComponentName::CoreWindow,
        CoreWindow::new().name("CoreWindow").new_boxed(),
      ),
      (
        ComponentName::StatusBar,
        StatusBar::new().name("Status Bar").new_boxed(),
      ),
    ];

    let command_tx = None;
    let components: HashMap<ComponentName, Box<dyn Component>> = components_iter.into_iter().collect();

    Tui { command_tx, components }
  }

  pub fn register_action_handler(&mut self, tx: mpsc::UnboundedSender<Action>) -> io::Result<()> {
    self.command_tx = Some(tx.clone());
    self
      .components
      .iter_mut()
      .try_for_each(|(_, component)| component.register_action_handler(tx.clone()))?;
    Ok(())
  }

  pub fn handle_events(&mut self, event: Option<Event>) -> io::Result<Option<Action>> {
    // Handle focus
    self.components.iter_mut().try_fold(None, |acc, (_, component)| {
      match component.handle_events(event.clone()) {
        Ok(Some(action)) => Ok(Some(action)),
        Ok(None) => Ok(acc),
        Err(e) => Err(e),
      }
    })
  }

  pub fn update(&mut self, action: Action) -> io::Result<Option<Action>> {
    // Handle focus
    self
      .components
      .iter_mut()
      .try_fold(None, |acc, (_, component)| match component.update(action.clone()) {
        Ok(Some(action)) => Ok(Some(action)),
        Ok(None) => Ok(acc),
        Err(e) => Err(e),
      })
  }

  pub fn draw(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) -> io::Result<()> {
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
      .get_mut(&ComponentName::TitleBar)
      .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::TitleBar))
      .draw(frame, main_layout[0])?;

    self
      .components
      .get_mut(&ComponentName::CoreWindow)
      .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::CoreWindow))
      .draw(frame, main_layout[1])?;

    self
      .components
      .get_mut(&ComponentName::StatusBar)
      .unwrap_or_else(|| panic!("Failed to get component: {}", ComponentName::StatusBar))
      .draw(frame, main_layout[2])?;

    Ok(())
  }
}
