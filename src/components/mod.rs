use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentName {
  CoreWindow,
  ChatList,
  Chat,
  Prompt,

  TitleBar,
  StatusBar,
}

impl fmt::Display for ComponentName {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // write!(f, "{:?}", self)
    match self {
      ComponentName::CoreWindow => write!(f, "CoreWindow"),
      ComponentName::ChatList => write!(f, "ChatList"),
      ComponentName::Chat => write!(f, "Chat"),
      ComponentName::Prompt => write!(f, "Prompt"),
      ComponentName::TitleBar => write!(f, "TitleBar"),
      ComponentName::StatusBar => write!(f, "StatusBar"),
    }
  }
}

pub const SMALL_AREA_WIDTH: u16 = 100;
pub const SMALL_AREA_HEIGHT: u16 = 30;

pub mod chat_list_window;
pub mod chat_window;
pub mod core_window;
pub mod prompt_window;
pub mod status_bar;
pub mod title_bar;
