use {
    super::component_name::ComponentName,
    crate::app_error::AppError,
    crossterm::event::{KeyCode, KeyModifiers},
    ratatui::layout::Rect,
    std::str::FromStr,
};

#[derive(Debug, Clone, Eq, PartialEq)]
// Action` is an enum that represents an action that can be handled by the
/// main application loop and the components of the user interface.
pub enum Action {
    /// Unknown action.
    Unknown,
    /// Init action.
    Init,
    /// Quit action.
    Quit,
    /// TryQuit action, it is used to try to quit the application.
    /// It asks the the core window to confirm the quit action.
    /// If the prompt is not focused, we can quit.
    TryQuit,
    /// Render action.
    Render,
    /// Resize action with width and height.
    Resize(u16, u16),
    /// Paste action with a `String`.
    Paste(String),

    /// Focus action with a `ComponentName`.
    FocusComponent(ComponentName),
    /// Unfocus action.
    UnfocusComponent,
    /// Increase ChatList size action.
    IncreaseChatListSize,
    /// Decrease ChatList size action.
    DecreaseChatListSize,
    /// Increase Prompt size action.
    IncreasePromptSize,
    /// Decrease Prompt size action.
    DecreasePromptSize,
    /// Key action with a key code.
    Key(KeyCode, KeyModifiers),
    /// Update area action with a rectangular area.
    UpdateArea(Rect),

    /// ChatListNext action.
    ChatListNext,
    /// ChatListPrevious action.
    ChatListPrevious,
    /// ChatListSelect action.
    ChatListUnselect,

    /// MessageListNext action.
    MessageListNext,
    /// MessageListPrevious action.
    MessageListPrevious,
    /// MessageListSelect action.
    MessageListUnselect,
}

/// Implement the `FromStr` trait for `Action`.
impl FromStr for Action {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "quit" => Ok(Action::Quit),
            "try_quit" => Ok(Action::TryQuit),
            "render" => Ok(Action::Render),
            "focus_chat_list" => Ok(Action::FocusComponent(ComponentName::ChatList)),
            "focus_chat" => Ok(Action::FocusComponent(ComponentName::Chat)),
            "focus_prompt" => Ok(Action::FocusComponent(ComponentName::Prompt)),
            "unfocus_component" => Ok(Action::UnfocusComponent),
            "increase_chat_list_size" => Ok(Action::IncreaseChatListSize),
            "decrease_chat_list_size" => Ok(Action::DecreaseChatListSize),
            "increase_prompt_size" => Ok(Action::IncreasePromptSize),
            "decrease_prompt_size" => Ok(Action::DecreasePromptSize),
            "chat_list_next" => Ok(Action::ChatListNext),
            "chat_list_previous" => Ok(Action::ChatListPrevious),
            "chat_list_unselect" => Ok(Action::ChatListUnselect),
            "message_list_next" => Ok(Action::MessageListNext),
            "message_list_previous" => Ok(Action::MessageListPrevious),
            "message_list_unselect" => Ok(Action::MessageListUnselect),
            _ => Err(AppError::InvalidAction(s.to_string())),
        }
    }
}
