use {
    super::component_name::ComponentName,
    crate::{app_error::AppError, tg::td_enums::TdChatList},
    crossterm::event::{KeyCode, KeyModifiers},
    ratatui::layout::Rect,
    std::str::FromStr,
};

#[derive(Debug, Clone, Eq, PartialEq)]
/// `Modifiers` is a struct that represents the modifiers of a key event.
/// It is used to determine the state of the modifiers when a key event is
/// generated.
pub struct Modifiers {
    /// A boolean that represents the shift modifier.
    pub shift: bool,
    /// A boolean that represents the control modifier.
    pub control: bool,
    /// A boolean that represents the alt modifier.
    pub alt: bool,
    /// A boolean that represents the super modifier.
    pub super_: bool,
    /// A boolean that represents the hyper modifier.
    pub hyper: bool,
    /// A boolean that represents the meta modifier.
    pub meta: bool,
}
/// Implement the `From` trait for `KeyModifiers`
impl From<KeyModifiers> for Modifiers {
    fn from(modifiers: KeyModifiers) -> Self {
        Modifiers {
            shift: modifiers.contains(KeyModifiers::SHIFT),
            control: modifiers.contains(KeyModifiers::CONTROL),
            alt: modifiers.contains(KeyModifiers::ALT),
            super_: modifiers.contains(KeyModifiers::SUPER),
            hyper: modifiers.contains(KeyModifiers::HYPER),
            meta: modifiers.contains(KeyModifiers::META),
        }
    }
}
/// Implement the `From` trait for `Modifiers`
impl From<Modifiers> for KeyModifiers {
    fn from(modifiers: Modifiers) -> Self {
        let mut key_modifiers = KeyModifiers::empty();
        if modifiers.shift {
            key_modifiers.insert(KeyModifiers::SHIFT);
        }
        if modifiers.control {
            key_modifiers.insert(KeyModifiers::CONTROL);
        }
        if modifiers.alt {
            key_modifiers.insert(KeyModifiers::ALT);
        }
        if modifiers.super_ {
            key_modifiers.insert(KeyModifiers::SUPER);
        }
        if modifiers.hyper {
            key_modifiers.insert(KeyModifiers::HYPER);
        }
        if modifiers.meta {
            key_modifiers.insert(KeyModifiers::META);
        }
        key_modifiers
    }
}

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

    /// GetMe action.
    GetMe,
    /// LoadChats action with a `ChatList` and a limit.
    LoadChats(TdChatList, i32),
    /// SendMessage action with a `String`.
    SendMessage(String),
    /// SendMessageEdited action with a `i64` and a `String`.
    /// The first parameter is the `message_id` and the second parameter is the `text`.
    SendMessageEdited(i64, String),
    /// Prepare chat history action.
    /// We need to call this function because telegram the first time we want to
    /// get the chat history send only one message.
    PrepareChatHistory,
    /// GetChatHistory action.
    /// The first parameter is the `from_message_id`, the second parameter
    /// is the `offset` and the third parameter is the `limit`.
    GetChatHistory(i64, i32, i32),
    /// DeleteMessages action.
    /// The first parameter is the `message_ids` and the second parameter is the `revoke`.
    /// If `revoke` is true, the message will be deleted for everyone.
    /// If `revoke` is false, the message will be deleted only for the current user.
    DeleteMessages(Vec<i64>, bool),
    /// ViewAllMessages action.
    ViewAllMessages,

    /// Focus action with a `ComponentName`.
    FocusComponent(ComponentName),
    /// Unfocus action.
    UnfocusComponent,
    /// Toggle ChatList action.
    ToggleChatList,
    /// Increase ChatList size action.
    IncreaseChatListSize,
    /// Decrease ChatList size action.
    DecreaseChatListSize,
    /// Increase Prompt size action.
    IncreasePromptSize,
    /// Decrease Prompt size action.
    DecreasePromptSize,
    /// Key action with a key code.
    Key(KeyCode, Modifiers),
    /// Update area action with a rectangular area.
    UpdateArea(Rect),

    /// ChatListNext action.
    ChatListNext,
    /// ChatListPrevious action.
    ChatListPrevious,
    /// ChatListSelect action.
    ChatListUnselect,
    /// ChatListOpen action.
    ChatListOpen,

    /// ChatWindowNext action.
    ChatWindowNext,
    /// ChatWindowPrevious action.
    ChatWindowPrevious,
    /// ChatWindowUnselect action.
    ChatWindowUnselect,
    /// ChatWindowDeleteForEveryone action.
    /// It is used to delete a message for everyone.
    ChatWindowDeleteForEveryone,
    /// ChatWindowDeleteForMe action.
    /// It is used to delete a message only for the current user.
    ChatWindowDeleteForMe,
    /// ChatWindowCopy action.
    ChatWindowCopy,
    /// ChatWindowEdit action.
    ChatWindowEdit,

    /// EditMessage action with a `String`.
    /// This action is used to edit a message.
    /// The first parameter is the `message_id` and the second parameter is the `text`.
    EditMessage(i64, String),
}
/// Implement the `Action` enum.
impl Action {
    /// Create an action from a key event.
    ///
    /// # Arguments
    /// * `key` - A `KeyCode` that represents the key code.
    /// * `modifiers` - A `KeyModifiers` struct that represents the modifiers.
    ///
    /// # Returns
    /// * `Action` - An action.
    pub fn from_key_event(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Action::Key(key, Modifiers::from(modifiers))
    }
}

/// Implement the `FromStr` trait for `Action`.
impl FromStr for Action {
    type Err = AppError<()>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "quit" => Ok(Action::Quit),
            "try_quit" => Ok(Action::TryQuit),
            "render" => Ok(Action::Render),
            "focus_chat_list" => Ok(Action::FocusComponent(ComponentName::ChatList)),
            "focus_chat" => Ok(Action::FocusComponent(ComponentName::Chat)),
            "focus_prompt" => Ok(Action::FocusComponent(ComponentName::Prompt)),
            "unfocus_component" => Ok(Action::UnfocusComponent),
            "toggle_chat_list" => Ok(Action::ToggleChatList),
            "increase_chat_list_size" => Ok(Action::IncreaseChatListSize),
            "decrease_chat_list_size" => Ok(Action::DecreaseChatListSize),
            "increase_prompt_size" => Ok(Action::IncreasePromptSize),
            "decrease_prompt_size" => Ok(Action::DecreasePromptSize),
            "chat_list_next" => Ok(Action::ChatListNext),
            "chat_list_previous" => Ok(Action::ChatListPrevious),
            "chat_list_unselect" => Ok(Action::ChatListUnselect),
            "chat_list_open" => Ok(Action::ChatListOpen),
            "chat_window_next" => Ok(Action::ChatWindowNext),
            "chat_window_previous" => Ok(Action::ChatWindowPrevious),
            "chat_window_unselect" => Ok(Action::ChatWindowUnselect),
            "chat_window_delete_for_everyone" => Ok(Action::ChatWindowDeleteForEveryone),
            "chat_window_delete_for_me" => Ok(Action::ChatWindowDeleteForMe),
            "chat_window_copy" => Ok(Action::ChatWindowCopy),
            "chat_window_edit" => Ok(Action::ChatWindowEdit),
            _ => Err(AppError::InvalidAction(s.to_string())),
        }
    }
}
