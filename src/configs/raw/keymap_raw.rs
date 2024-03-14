use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
/// The command keymap configuration.
pub struct KeymapEntry {
    pub keys: Vec<String>,     // Event
    pub commands: Vec<String>, // Action
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
/// The keymap configuration.
pub struct KeyMappingMode {
    pub keymap: Option<Vec<KeymapEntry>>,
}

#[derive(Clone, Debug, Deserialize)]
/// The raw keymap configuration.
pub struct KeyMappingRaw {
    pub default: Option<KeyMappingMode>,
    pub chats_list: Option<KeyMappingMode>,
    pub chat: Option<KeyMappingMode>,
    pub prompt: Option<KeyMappingMode>,
}
