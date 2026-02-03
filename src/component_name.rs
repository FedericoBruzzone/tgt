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
    /// The theme selector popup.
    ThemeSelector,
    /// The search overlay (server-side chat message search).
    SearchOverlay,
    /// The photo viewer popup.
    PhotoViewer,
}

impl ComponentName {
    /// Returns true if this component is a popup (overlay) component.
    /// Popup components are modal overlays that should hide other popups when shown.
    pub fn is_popup(&self) -> bool {
        matches!(
            self,
            ComponentName::CommandGuide
                | ComponentName::ThemeSelector
                | ComponentName::SearchOverlay
                | ComponentName::PhotoViewer
        )
    }
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
            ComponentName::ThemeSelector => write!(f, "Theme Selector"),
            ComponentName::SearchOverlay => write!(f, "Search Overlay"),
            ComponentName::PhotoViewer => write!(f, "Photo Viewer"),
        }
    }
}
