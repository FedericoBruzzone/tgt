use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// `ComponentName` is an enum that represents the name of a component in the
/// user interface.
pub enum ComponentName {
    CoreWindow,
    ChatList,
    Chat,
    Prompt,

    TitleBar,
    StatusBar,
}

/// Implement the `Display` trait for the `ComponentName` enum.
impl Display for ComponentName {
    fn fmt(&self, f: &mut Formatter) -> Result {
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
