//! Safe merge of user config with bundled default so new keys/sections are added without overwriting user customizations.
//! Merge rule: user values take precedence; any command/key present in default but missing in user is added.

use crate::configs::raw::keymap_raw::{KeymapMode, KeymapRaw};
use std::collections::HashSet;

/// Merge keymap sections: user entries first; then add any default entry whose `command` is not bound in user.
/// Order of lines in the config does not affect the result (merge is by command name).
fn merge_keymap_mode(user: Option<KeymapMode>, default: Option<KeymapMode>) -> Option<KeymapMode> {
    let (user_entries, default_entries) = match (user, default) {
        (None, None) => return None,
        (Some(u), None) => return Some(u),
        (None, Some(d)) => return Some(d),
        (Some(u), Some(d)) => (u.keymap, d.keymap),
    };
    let user_commands: HashSet<String> = user_entries.iter().map(|e| e.command.clone()).collect();
    let mut merged = user_entries;
    for e in default_entries {
        if !user_commands.contains(&e.command) {
            merged.push(e);
        }
    }
    Some(KeymapMode { keymap: merged })
}

/// Merge default keymap raw (bundled) with user keymap raw. User bindings take precedence;
/// any command present in default but missing in user is added so new keycodes get merged in.
pub fn merge_keymap_raw(default: KeymapRaw, user: Option<KeymapRaw>) -> KeymapRaw {
    let user = user.unwrap_or(KeymapRaw {
        core_window: None,
        chat_list: None,
        chat: None,
        prompt: None,
        command_guide: None,
        theme_selector: None,
        search_overlay: None,
        photo_viewer: None,
        file_upload_explorer: None,
        file_download_explorer: None,
        pinned_messages_popup: None,
    });
    KeymapRaw {
        core_window: merge_keymap_mode(user.core_window, default.core_window),
        chat_list: merge_keymap_mode(user.chat_list, default.chat_list),
        chat: merge_keymap_mode(user.chat, default.chat),
        prompt: merge_keymap_mode(user.prompt, default.prompt),
        command_guide: merge_keymap_mode(user.command_guide, default.command_guide),
        theme_selector: merge_keymap_mode(user.theme_selector, default.theme_selector),
        search_overlay: merge_keymap_mode(user.search_overlay, default.search_overlay),
        photo_viewer: merge_keymap_mode(user.photo_viewer, default.photo_viewer),
        file_upload_explorer: merge_keymap_mode(
            user.file_upload_explorer,
            default.file_upload_explorer,
        ),
        file_download_explorer: merge_keymap_mode(
            user.file_download_explorer,
            default.file_download_explorer,
        ),
        pinned_messages_popup: merge_keymap_mode(
            user.pinned_messages_popup,
            default.pinned_messages_popup,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::configs::raw::keymap_raw::KeymapEntry;

    fn entry(keys: &[&str], command: &str) -> KeymapEntry {
        KeymapEntry {
            keys: keys.iter().map(|s| (*s).to_string()).collect(),
            command: command.to_string(),
            description: None,
        }
    }

    fn mode(entries: Vec<KeymapEntry>) -> KeymapMode {
        KeymapMode { keymap: entries }
    }

    #[test]
    fn merge_adds_missing_command_from_default() {
        let default = KeymapRaw {
            core_window: Some(mode(vec![
                entry(&["q"], "try_quit"),
                entry(&["alt+f1"], "show_command_guide"),
            ])),
            chat_list: Some(mode(vec![])),
            chat: Some(mode(vec![])),
            prompt: Some(mode(vec![])),
            command_guide: None,
            theme_selector: None,
            search_overlay: None,
            photo_viewer: None,
            file_upload_explorer: None,
            file_download_explorer: None,
            pinned_messages_popup: None,
        };
        let user = KeymapRaw {
            core_window: Some(mode(vec![entry(&["q"], "try_quit")])),
            chat_list: Some(mode(vec![])),
            chat: Some(mode(vec![])),
            prompt: Some(mode(vec![])),
            command_guide: None,
            theme_selector: None,
            search_overlay: None,
            photo_viewer: None,
            file_upload_explorer: None,
            file_download_explorer: None,
            pinned_messages_popup: None,
        };
        let merged = merge_keymap_raw(default, Some(user));
        let core = merged.core_window.unwrap();
        assert_eq!(
            core.keymap.len(),
            2,
            "should have try_quit and show_command_guide"
        );
        let commands: Vec<&str> = core.keymap.iter().map(|e| e.command.as_str()).collect();
        assert!(commands.contains(&"try_quit"));
        assert!(commands.contains(&"show_command_guide"));
    }

    #[test]
    fn merge_user_binding_takes_precedence() {
        let default = KeymapRaw {
            core_window: Some(mode(vec![entry(&["alt+f1"], "show_command_guide")])),
            chat_list: Some(mode(vec![])),
            chat: Some(mode(vec![])),
            prompt: Some(mode(vec![])),
            command_guide: None,
            theme_selector: None,
            search_overlay: None,
            photo_viewer: None,
            file_upload_explorer: None,
            file_download_explorer: None,
            pinned_messages_popup: None,
        };
        let user = KeymapRaw {
            core_window: Some(mode(vec![entry(&["F1"], "show_command_guide")])),
            chat_list: Some(mode(vec![])),
            chat: Some(mode(vec![])),
            prompt: Some(mode(vec![])),
            command_guide: None,
            theme_selector: None,
            search_overlay: None,
            photo_viewer: None,
            file_upload_explorer: None,
            file_download_explorer: None,
            pinned_messages_popup: None,
        };
        let merged = merge_keymap_raw(default, Some(user));
        let core = merged.core_window.unwrap();
        assert_eq!(core.keymap.len(), 1);
        assert_eq!(core.keymap[0].command, "show_command_guide");
        assert_eq!(core.keymap[0].keys, vec!["F1"]);
    }

    #[test]
    fn merge_order_independent_user_first_then_missing_defaults() {
        let default = KeymapRaw {
            core_window: Some(mode(vec![
                entry(&["a"], "cmd_a"),
                entry(&["b"], "cmd_b"),
                entry(&["c"], "cmd_c"),
            ])),
            chat_list: Some(mode(vec![])),
            chat: Some(mode(vec![])),
            prompt: Some(mode(vec![])),
            command_guide: None,
            theme_selector: None,
            search_overlay: None,
            photo_viewer: None,
            file_upload_explorer: None,
            file_download_explorer: None,
            pinned_messages_popup: None,
        };
        let user = KeymapRaw {
            core_window: Some(mode(vec![entry(&["b"], "cmd_b"), entry(&["a"], "cmd_a")])),
            chat_list: Some(mode(vec![])),
            chat: Some(mode(vec![])),
            prompt: Some(mode(vec![])),
            command_guide: None,
            theme_selector: None,
            search_overlay: None,
            photo_viewer: None,
            file_upload_explorer: None,
            file_download_explorer: None,
            pinned_messages_popup: None,
        };
        let merged = merge_keymap_raw(default, Some(user));
        let core = merged.core_window.unwrap();
        assert_eq!(core.keymap.len(), 3);
        let commands: Vec<&str> = core.keymap.iter().map(|e| e.command.as_str()).collect();
        assert!(commands.contains(&"cmd_a"));
        assert!(commands.contains(&"cmd_b"));
        assert!(commands.contains(&"cmd_c"));
        assert_eq!(core.keymap[0].command, "cmd_b");
        assert_eq!(core.keymap[1].command, "cmd_a");
        assert_eq!(core.keymap[2].command, "cmd_c");
    }

    #[test]
    fn merge_preserves_user_only_commands() {
        let default = KeymapRaw {
            core_window: Some(mode(vec![entry(&["q"], "try_quit")])),
            chat_list: Some(mode(vec![])),
            chat: Some(mode(vec![])),
            prompt: Some(mode(vec![])),
            command_guide: None,
            theme_selector: None,
            search_overlay: None,
            photo_viewer: None,
            file_upload_explorer: None,
            file_download_explorer: None,
            pinned_messages_popup: None,
        };
        let user = KeymapRaw {
            core_window: Some(mode(vec![
                entry(&["x"], "try_quit"),
                entry(&["F2"], "my_custom_action"),
            ])),
            chat_list: Some(mode(vec![])),
            chat: Some(mode(vec![])),
            prompt: Some(mode(vec![])),
            command_guide: None,
            theme_selector: None,
            search_overlay: None,
            photo_viewer: None,
            file_upload_explorer: None,
            file_download_explorer: None,
            pinned_messages_popup: None,
        };
        let merged = merge_keymap_raw(default, Some(user));
        let core = merged.core_window.unwrap();
        assert_eq!(
            core.keymap.len(),
            2,
            "user's two bindings only (default try_quit not added, user overrode)"
        );
        let commands: Vec<&str> = core.keymap.iter().map(|e| e.command.as_str()).collect();
        assert!(commands.contains(&"try_quit"));
        assert!(commands.contains(&"my_custom_action"));
        assert_eq!(core.keymap[0].keys, vec!["x"]);
        assert_eq!(core.keymap[1].keys, vec!["F2"]);
    }

    #[test]
    fn merge_none_user_uses_default_entirely() {
        let default = KeymapRaw {
            core_window: Some(mode(vec![entry(&["q"], "try_quit")])),
            chat_list: Some(mode(vec![])),
            chat: Some(mode(vec![])),
            prompt: Some(mode(vec![])),
            command_guide: None,
            theme_selector: None,
            search_overlay: None,
            photo_viewer: None,
            file_upload_explorer: None,
            file_download_explorer: None,
            pinned_messages_popup: None,
        };
        let merged = merge_keymap_raw(default, None);
        let core = merged.core_window.unwrap();
        assert_eq!(core.keymap.len(), 1);
        assert_eq!(core.keymap[0].command, "try_quit");
    }
}
