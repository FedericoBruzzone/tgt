use {
    crate::{
        app_error::AppError,
        configs::{self, raw::keymap_raw::KeyMappingRaw},
    },
    std::{collections::HashMap, path::Path},
};

use crate::enums::{action::Action, event::Event};

type KeyMappingMode = HashMap<Event, Action>;

#[derive(Debug)]
pub struct KeyMappingConfig {
    pub default: KeyMappingMode,
    pub chats_list: KeyMappingMode,
    pub chat: KeyMappingMode,
    pub prompt: KeyMappingMode,
}

impl KeyMappingConfig {
    pub fn default_result() -> Result<Self, AppError> {
        configs::deserialize_to_config_into::<KeyMappingRaw, Self>(Path::new(
            &configs::custom::default_config_keymap_file_path()?,
        ))
    }

    fn keymaps_vec_to_map(_keymaps: Vec<crate::configs::raw::keymap_raw::KeymapEntry>) -> KeyMappingMode {
        // [TODO] Implement this function.
        KeyMappingMode::default()
    }
}

impl Default for KeyMappingConfig {
    fn default() -> Self {
        Self::default_result().unwrap()
    }
}

impl From<KeyMappingRaw> for KeyMappingConfig {
    fn from(raw: KeyMappingRaw) -> Self {
        Self {
            default: Self::keymaps_vec_to_map(raw.default.unwrap().keymap.unwrap()),
            chats_list: Self::keymaps_vec_to_map(raw.chats_list.unwrap().keymap.unwrap()),
            chat: Self::keymaps_vec_to_map(raw.chat.unwrap().keymap.unwrap()),
            prompt: Self::keymaps_vec_to_map(raw.prompt.unwrap().keymap.unwrap()),
        }
    }
}
