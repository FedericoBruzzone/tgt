use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentName {
  CoreWindow,
  ChatList,
  Chat,
  Prompt,

  TitleBar,
  StatusBar,
}

impl Display for ComponentName {
  fn fmt(&self, f: &mut Formatter) -> Result {
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
