use crate::{
    app_error::AppError,
    configs::{
        self,
        config_file::ConfigFile,
        config_type::ConfigType,
        raw::keymap_raw::{KeymapEntry, KeymapRaw},
    },
    enums::{action::Action, component_name::ComponentName, event::Event},
};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    path::Path,
    str::FromStr,
};

#[derive(Clone, Debug, PartialEq, Eq)]
/// The action binding.
pub enum ActionBinding {
    /// A single action binding.
    /// It binds a single key binding to an action.
    /// In tgt a key plus a modifier is considered a single key binding.
    Single {
        action: Action,
        description: Option<String>,
    },
    /// A multiple action binding.
    /// It is used to bind multiple keys to an action.
    Multiple(HashMap<Event, ActionBinding>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// The kind of keymap.
/// It is used to check for conflicts in the keymaps.
/// If a keymap entry is present in multiple keymaps, it is considered a
/// conflict
enum KeymapKind {
    CoreWindow,
    ChatList,
    Chat,
    Prompt,
}

#[derive(Clone, Debug)]
/// The keymap configuration.
pub struct KeymapConfig {
    /// The default keymap configuration.
    /// They can be used in any component.
    pub core_window: HashMap<Event, ActionBinding>,
    /// The keymap configuration for the chats list component.
    pub chat_list: HashMap<Event, ActionBinding>,
    /// The keymap configuration for the chat component.
    pub chat: HashMap<Event, ActionBinding>,
    /// The keymap configuration for the prompt component.
    pub prompt: HashMap<Event, ActionBinding>,
}
/// The keymap configuration implementation.
impl KeymapConfig {
    /// Get the default keymap configuration.
    ///
    /// # Returns
    /// The default keymap configuration.
    pub fn default_result() -> Result<Self, AppError> {
        configs::deserialize_to_config_into::<KeymapRaw, Self>(Path::new(
            &configs::custom::default_config_keymap_file_path()?,
        ))
    }
    /// Get the key of a single action.
    ///
    /// # Arguments
    /// * `map` - A hashmap of event and action binding.
    /// * `value` - An action.
    ///
    /// # Returns
    /// The key of the single action.
    pub fn get_key_of_single_action(
        &self,
        component_name: ComponentName,
        value: Action,
    ) -> Option<&Event> {
        let map = self.get_map_of(Some(component_name));
        for (k, v) in map.iter() {
            match v {
                ActionBinding::Single { action, .. } if *action == value => return Some(k),
                _ => {}
            }
        }
        None
    }
    /// Print the configuration file error.
    /// It is used to print the error when the configuration file is not
    /// correct. It prints the unrecognized settings.
    ///
    /// # Arguments
    /// * `v` - A vector of strings that represents the unrecognized settings.
    fn print_config_file_error(s: &str, v: Vec<String>) {
        eprintln!(
            "\n\
         [TGT] ConfigFileError: Some setting were not recognized, the {} filed is {:?}\n    \
         Please check the {} configuration file in the config directory or\n    \
         the default config file in the GitHub repository.",
            s,
            v,
            ConfigType::Keymap.as_default_filename()
        );
    }
    /// Convert a vector of keymap entries to a hashmap of event and action
    /// binding.
    /// When the keymap entries are read from the default configuration file,
    /// they are stored in a vector. This function converts the vector to a
    /// hashmap.
    ///
    /// # Arguments
    /// * `keymaps` - A vector of keymap entries.
    /// * `kind` - The kind of keymap.
    ///
    /// # Returns
    /// A hashmap of event and action binding.
    fn keymaps_vec_to_map(
        keymaps: Vec<KeymapEntry>,
        kind: KeymapKind,
    ) -> HashMap<Event, ActionBinding> {
        let mut hashmap = HashMap::new();

        for keymap in keymaps {
            let event: Vec<Event> = keymap
                .keys
                .iter()
                .map(|k| match Event::from_str(k) {
                    Ok(e) => e,
                    Err(e) => {
                        if let AppError::InvalidEvent(err) = e {
                            tracing::warn!(err);
                        }
                        Event::Unknown
                    }
                })
                .filter(|e| *e != Event::Unknown)
                .collect();
            if keymap.keys.len() != event.len() {
                tracing::warn!(
                    "Some events were not recognized in {:?} section for key: {:?}",
                    kind,
                    keymap.keys
                );
                Self::print_config_file_error("keys", keymap.keys);
                std::process::exit(1);
            }
            let action: Action = match Action::from_str(&keymap.command) {
                Ok(a) => a,
                Err(e) => {
                    if let AppError::InvalidAction(err) = e {
                        tracing::warn!(err);
                    }
                    Action::Unknown
                }
            };
            if action == Action::Unknown {
                tracing::warn!(
                    "Some actions were not recognized in {:?} section for command: {:?}",
                    kind,
                    keymap.command
                );
                Self::print_config_file_error("command", Vec::from([keymap.command]));
                std::process::exit(1);
            }

            let description = keymap.description.clone();

            if let Err(AppError::AlreadyBound) =
                Self::insert_keymap(&mut hashmap, event, action, description, kind.clone())
            {
                tracing::warn!(
                    "Keymap entry {:?} is already present in the {:?} section",
                    keymap
                        .keys
                        .iter()
                        .map(|k| k.to_string())
                        .collect::<Vec<String>>(),
                    kind
                );
            }
        }
        hashmap
    }

    /// Insert a keymap entry into the keymap hashmap.
    /// It is used to insert a keymap entry into the keymap hashmap. It is
    /// recursive and it is used to insert a keymap entry with multiple events.
    /// It returns an error if the key is already bound to a command.
    /// Note that can not exist two keymap entries that start with the same
    /// key (event), for example "q" and ["q", "q"]. If the keymap entry
    /// already exists, it returns an error.
    ///
    /// # Arguments
    /// * `keymap` - A mutable reference to the keymap hashmap.
    /// * `event` - A vector of events.
    /// * `action` - An action.
    /// * `description` - An optional description.
    /// * `kind` - The kind of keymap.
    ///
    /// # Returns
    /// An error if the key is already bound to a command.
    fn insert_keymap(
        keymap: &mut HashMap<Event, ActionBinding>,
        event: Vec<Event>,
        action: Action,
        description: Option<String>,
        kind: KeymapKind,
    ) -> Result<(), AppError> {
        let num_events = event.len();
        match num_events {
            0 => Ok(()),
            1 => {
                match keymap.entry(event[0].clone()) {
                    Entry::Occupied(_) => {
                        tracing::error!(
                            "Key {:?} already bound to a command in {:?} section",
                            event[0].to_string(),
                            kind
                        );
                        return Err(AppError::AlreadyBound);
                    }
                    Entry::Vacant(e) => {
                        e.insert(ActionBinding::Single {
                            action,
                            description,
                        });
                    }
                }
                Ok(())
            }
            _ => match keymap.entry(event[0].clone()) {
                Entry::Occupied(mut entry) => match entry.get_mut() {
                    ActionBinding::Multiple(ref mut map) => {
                        Self::insert_keymap(map, event[1..].to_vec(), action, description, kind)
                    }
                    _ => {
                        tracing::error!(
                            "Key {:?} already bound to a command in {:?} section",
                            event[0].to_string(),
                            kind
                        );
                        Err(AppError::AlreadyBound)
                    }
                },
                Entry::Vacant(entry) => {
                    let mut map = HashMap::new();
                    let res = Self::insert_keymap(
                        &mut map,
                        event[1..].to_vec(),
                        action,
                        description,
                        kind,
                    );
                    if res.is_ok() {
                        entry.insert(ActionBinding::Multiple(map));
                    }
                    res
                }
            },
        }
    }

    /// Check for duplicates in the keymaps.
    /// It is used to check for duplicates in the keymaps. If a keymap entry is
    /// present in multiple keymaps, it is considered a conflict.
    ///
    /// # Arguments
    /// * `default` - The default keymap.
    /// * `chat_list` - The chat list keymap.
    /// * `chat` - The chat keymap.
    /// * `prompt` - The prompt keymap.
    fn check_duplicates(
        default: &HashMap<Event, ActionBinding>,
        chat_list: &HashMap<Event, ActionBinding>,
        chat: &HashMap<Event, ActionBinding>,
        prompt: &HashMap<Event, ActionBinding>,
    ) {
        let mut all: Vec<&Event> = vec![];
        all.extend(default.keys());
        all.extend(chat_list.keys());
        all.extend(chat.keys());
        all.extend(prompt.keys());

        let mut duplicates = HashSet::new();
        for k in all {
            if !duplicates.insert(k) {
                tracing::warn!(
                    "Keymap entry {:?} is already present in another keymap",
                    k.to_string(),
                );
            }
        }
    }

    /// Get the keymap configuration of a component.
    /// It is used to get the keymap configuration of a component.
    ///
    /// # Arguments
    /// * `component_name` - The name of the component.
    ///
    /// # Returns
    /// The keymap configuration of the component.
    pub fn get_map_of(
        &self,
        component_name: Option<ComponentName>,
    ) -> &HashMap<Event, ActionBinding> {
        match component_name {
            Some(componnt) => match componnt {
                ComponentName::ChatList => &self.chat_list,
                ComponentName::Chat => &self.chat,
                ComponentName::Prompt => &self.prompt,
                _ => &self.core_window,
            },
            None => &self.core_window,
        }
    }
}

/// The implementation of the configuration file for the keymap.
impl ConfigFile for KeymapConfig {
    type Raw = KeymapRaw;

    fn get_type() -> ConfigType {
        ConfigType::Keymap
    }

    fn override_fields() -> bool {
        true
    }

    fn merge(&mut self, other: Option<Self::Raw>) -> Self {
        match other {
            None => self.clone(),
            Some(other) => {
                tracing::info!("Merging keymap config");
                // It is important that the default keymap is merged first.
                // The other keymaps can override the default keymap, but the
                // default keymap can not override the other keymaps.
                if let Some(default) = other.core_window {
                    for (k, v) in Self::keymaps_vec_to_map(default.keymap, KeymapKind::CoreWindow) {
                        if self.core_window.insert(k.clone(), v).is_some() {
                            tracing::warn!(
                                    "Keymap entry {:?} is already present in the default section, you are overriding it",
                                    k.to_string()
                                );
                        }
                    }
                }
                if let Some(chat_list) = other.chat_list {
                    for (k, v) in Self::keymaps_vec_to_map(chat_list.keymap, KeymapKind::ChatList) {
                        if self.chat_list.insert(k.clone(), v).is_some() {
                            tracing::warn!(
                                    "Keymap entry {:?} is already present in the chat list section, you are overriding it",
                                    k.to_string()
                                );
                        }
                    }
                }
                if let Some(chat) = other.chat {
                    for (k, v) in Self::keymaps_vec_to_map(chat.keymap, KeymapKind::Chat) {
                        if self.chat.insert(k.clone(), v).is_some() {
                            tracing::warn!(
                                    "Keymap entry {:?} is already present in the chat section, you are overriding it",
                                    k.to_string()
                                );
                        }
                    }
                }
                if let Some(prompt) = other.prompt {
                    for (k, v) in Self::keymaps_vec_to_map(prompt.keymap, KeymapKind::Prompt) {
                        if self.prompt.insert(k.clone(), v).is_some() {
                            tracing::warn!(
                                    "Keymap entry {:?} is already present in the prompt section, you are overriding it",
                                    k.to_string()
                                );
                        }
                    }
                }
                Self::check_duplicates(
                    &self.core_window,
                    &self.chat_list,
                    &self.chat,
                    &self.prompt,
                );
                self.clone()
            }
        }
    }
}
/// The default keymap configuration.
impl Default for KeymapConfig {
    fn default() -> Self {
        Self::default_result().unwrap()
    }
}
/// The conversion from the raw keymap configuration to the keymap
/// configuration.
impl From<KeymapRaw> for KeymapConfig {
    fn from(raw: KeymapRaw) -> Self {
        let core_window =
            Self::keymaps_vec_to_map(raw.core_window.unwrap().keymap, KeymapKind::CoreWindow);
        let chat_list =
            Self::keymaps_vec_to_map(raw.chat_list.unwrap().keymap, KeymapKind::ChatList);
        let chat = Self::keymaps_vec_to_map(raw.chat.unwrap().keymap, KeymapKind::Chat);
        let prompt = Self::keymaps_vec_to_map(raw.prompt.unwrap().keymap, KeymapKind::Prompt);
        Self::check_duplicates(&core_window, &chat_list, &chat, &prompt);
        Self {
            core_window,
            chat_list,
            chat,
            prompt,
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::{
            configs::{
                config_file::ConfigFile,
                custom::keymap_custom::{ActionBinding, KeymapConfig},
                raw::keymap_raw::{KeymapEntry, KeymapMode, KeymapRaw},
            },
            enums::{action::Action, event::Event},
        },
        std::str::FromStr,
    };

    #[test]
    fn test_keymap_config_default() {
        let keymap_config = KeymapConfig::default();
        assert_eq!(keymap_config.core_window.len(), 10);
        assert_eq!(keymap_config.chat_list.len(), 3);
        assert_eq!(keymap_config.chat.len(), 3);
        assert_eq!(keymap_config.prompt.len(), 0);
    }

    #[test]
    fn test_keymap_config_from_raw_empty() {
        let keymap_raw = KeymapRaw {
            core_window: Some(KeymapMode { keymap: vec![] }),
            chat_list: Some(KeymapMode { keymap: vec![] }),
            chat: Some(KeymapMode { keymap: vec![] }),
            prompt: Some(KeymapMode { keymap: vec![] }),
        };
        let keymap_config = KeymapConfig::from(keymap_raw);
        assert_eq!(keymap_config.core_window.len(), 0);
        assert_eq!(keymap_config.chat_list.len(), 0);
        assert_eq!(keymap_config.chat.len(), 0);
        assert_eq!(keymap_config.prompt.len(), 0);
    }

    #[test]
    fn test_keymap_config_from_raw() {
        let keymap_raw = KeymapRaw {
            core_window: Some(KeymapMode {
                keymap: vec![KeymapEntry {
                    keys: vec!["q".to_string()],
                    command: "quit".to_string(),
                    description: None,
                }],
            }),
            chat_list: Some(KeymapMode { keymap: vec![] }),
            chat: Some(KeymapMode { keymap: vec![] }),
            prompt: Some(KeymapMode { keymap: vec![] }),
        };
        let keymap_config = KeymapConfig::from(keymap_raw);
        assert_eq!(keymap_config.core_window.len(), 1);
        assert_eq!(keymap_config.chat_list.len(), 0);
        assert_eq!(keymap_config.chat.len(), 0);
        assert_eq!(keymap_config.prompt.len(), 0);
    }

    #[test]
    fn test_keymap_config_merge() {
        let keymap_raw = KeymapRaw {
            core_window: Some(KeymapMode {
                keymap: vec![KeymapEntry {
                    keys: vec!["q".to_string()],
                    command: "quit".to_string(),
                    description: None,
                }],
            }),
            chat_list: Some(KeymapMode { keymap: vec![] }),
            chat: Some(KeymapMode { keymap: vec![] }),
            prompt: Some(KeymapMode { keymap: vec![] }),
        };
        let mut keymap_config = KeymapConfig::from(keymap_raw);
        let keymap_raw = KeymapRaw {
            core_window: Some(KeymapMode {
                keymap: vec![KeymapEntry {
                    keys: vec!["q".to_string()],
                    command: "render".to_string(),
                    description: None,
                }],
            }),
            chat_list: Some(KeymapMode { keymap: vec![] }),
            chat: Some(KeymapMode { keymap: vec![] }),
            prompt: Some(KeymapMode { keymap: vec![] }),
        };
        keymap_config = keymap_config.merge(Some(keymap_raw));
        assert_eq!(keymap_config.core_window.len(), 1);
        assert_eq!(keymap_config.chat_list.len(), 0);
        assert_eq!(keymap_config.chat.len(), 0);
        assert_eq!(keymap_config.prompt.len(), 0);
        assert_eq!(
            keymap_config
                .core_window
                .get(&Event::from_str("q").unwrap())
                .unwrap()
                .clone(),
            ActionBinding::Single {
                action: Action::from_str("render").unwrap(),
                description: None
            }
        );
    }

    #[test]
    fn test_keymap_config_override_fields() {
        assert!(KeymapConfig::override_fields());
    }

    #[test]
    fn test_merge_all_fields() {
        let mut keymap_config = KeymapConfig::default();
        let keymap_raw = KeymapRaw {
            core_window: Some(KeymapMode { keymap: vec![] }),
            chat_list: Some(KeymapMode { keymap: vec![] }),
            chat: Some(KeymapMode { keymap: vec![] }),
            prompt: Some(KeymapMode { keymap: vec![] }),
        };
        keymap_config = keymap_config.merge(Some(keymap_raw));
        assert_eq!(keymap_config.core_window.len(), 10);
        assert_eq!(keymap_config.chat_list.len(), 3);
        assert_eq!(keymap_config.chat.len(), 3);
        assert_eq!(keymap_config.prompt.len(), 0);
    }

    #[test]
    fn test_get_type() {
        assert_eq!(
            KeymapConfig::get_type(),
            crate::configs::config_type::ConfigType::Keymap
        );
    }
}
