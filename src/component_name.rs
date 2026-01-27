use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// `ComponentName` is an enum that represents the name of a component in the
/// user interface.
pub enum ComponentName {
    /// The core window.
    CoreWindow,
    /// The chat list.
    ChatList,
    /// The chat.
    Chat,
    /// The prompt.
    Prompt,
    /// The reply message window.
    ReplyMessage,
    /// The title bar.
    TitleBar,
    /// The status bar.
    StatusBar,
    /// The command guide popup.
    CommandGuide,
}

impl Display for ComponentName {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ComponentName::CoreWindow => write!(f, "Core Window"),
            ComponentName::ChatList => write!(f, "Chat List"),
            ComponentName::Chat => write!(f, "Chat"),
            ComponentName::Prompt => write!(f, "Prompt"),
            ComponentName::TitleBar => write!(f, "Title Bar"),
            ComponentName::StatusBar => write!(f, "Status Bar"),
            ComponentName::ReplyMessage => write!(f, "Reply Message"),
            ComponentName::CommandGuide => write!(f, "Command Guide"),
        }
    }
}
