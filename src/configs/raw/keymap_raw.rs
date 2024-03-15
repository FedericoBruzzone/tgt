use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
/// The command keymap configuration.
pub struct KeyMapEntry {
    /// The key combination.
    /// It must be a valid key combination.
    pub keys: Vec<String>, // Event
    /// The command to execute.
    /// It must be a valid command.
    pub command: String, // Action
    /// The description of the command.
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
/// The keymap configuration.
pub struct KeyMapMode {
    #[serde(default)]
    /// The keymap entries.
    pub keymap: Vec<KeyMapEntry>,
}

#[derive(Clone, Debug, Deserialize)]
/// The raw keymap configuration.
pub struct KeyMapRaw {
    /// The keymap for the default mode.
    pub default: Option<KeyMapMode>,
    /// The keymap for the chat list mode.
    pub chats_list: Option<KeyMapMode>,
    /// The keymap for the chat mode.
    pub chat: Option<KeyMapMode>,
    /// The keymap for the chat edit mode.
    pub prompt: Option<KeyMapMode>,
}
