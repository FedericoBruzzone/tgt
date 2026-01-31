//! Open-chat message cache and view window for PR1 data layer.
//! No `ratatui` or `components` dependencies; UI reads only via read-only API.

use std::collections::BTreeMap;

use super::message_entry::MessageEntry;

/// View window: the loaded range of message IDs (oldest = earliest, newest = latest).
#[derive(Debug, Default, Clone)]
pub struct LoadedRange {
    /// Oldest (earliest) message ID in the loaded buffer; None if empty.
    pub oldest_message_id: Option<i64>,
    /// Newest (latest) message ID in the loaded buffer; None if empty.
    pub newest_message_id: Option<i64>,
}

impl LoadedRange {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update range with a new message id (e.g. when appending history).
    pub fn extend_with(&mut self, message_id: i64) {
        match (self.oldest_message_id, self.newest_message_id) {
            (None, None) => {
                self.oldest_message_id = Some(message_id);
                self.newest_message_id = Some(message_id);
            }
            (Some(old), Some(new)) => {
                self.oldest_message_id = Some(old.min(message_id));
                self.newest_message_id = Some(new.max(message_id));
            }
            _ => {}
        }
    }

    /// Reset after clear.
    pub fn clear(&mut self) {
        self.oldest_message_id = None;
        self.newest_message_id = None;
    }
}

/// Message cache keyed by ID with stable order (BTreeMap) and view window.
#[derive(Debug, Default)]
pub struct OpenChatMessageStore {
    cache: BTreeMap<i64, MessageEntry>,
    range: LoadedRange,
}

impl OpenChatMessageStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert messages (backend/task only). Updates view window.
    pub fn insert_messages(&mut self, messages: impl IntoIterator<Item = MessageEntry>) {
        for entry in messages {
            let id = entry.id();
            self.range.extend_with(id);
            self.cache.insert(id, entry);
        }
    }

    /// Clear all messages and view window (e.g. when switching chat).
    pub fn clear(&mut self) {
        self.cache.clear();
        self.range.clear();
    }

    /// Ordered message IDs (oldest to newest by id; BTreeMap iterates in key order).
    /// UI uses this to build the list.
    pub fn ordered_message_ids(&self) -> Vec<i64> {
        self.cache.keys().copied().collect()
    }

    /// Get a message by ID (clone). UI read-only.
    pub fn get_message(&self, id: i64) -> Option<MessageEntry> {
        self.cache.get(&id).cloned()
    }

    /// Get mutable reference to a message (backend only, e.g. for editing content).
    pub fn get_message_mut(&mut self, id: i64) -> Option<&mut MessageEntry> {
        self.cache.get_mut(&id)
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Oldest loaded message ID (read-only for UI).
    pub fn oldest_message_id(&self) -> Option<i64> {
        self.range.oldest_message_id
    }

    /// Newest loaded message ID (read-only for UI).
    pub fn newest_message_id(&self) -> Option<i64> {
        self.range.newest_message_id
    }

    /// For backend: set from_message_id for next "load older" request.
    /// Should be the current oldest message id, or 0 if empty.
    pub fn from_message_id_for_load_older(&self) -> i64 {
        self.range.oldest_message_id.unwrap_or(0)
    }

    /// Remove a single message (e.g. delete_message). Recomputes view window from remaining entries.
    pub fn remove_message(&mut self, id: i64) {
        self.cache.remove(&id);
        self.range = self.cache.keys().fold(LoadedRange::new(), |mut r, &k| {
            r.extend_with(k);
            r
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: i64) -> MessageEntry {
        MessageEntry::test_entry(id)
    }

    #[test]
    fn empty_store() {
        let store = OpenChatMessageStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
        assert!(store.ordered_message_ids().is_empty());
        assert!(store.get_message(1).is_none());
        assert!(store.oldest_message_id().is_none());
        assert!(store.newest_message_id().is_none());
        assert_eq!(store.from_message_id_for_load_older(), 0);
    }

    #[test]
    fn insert_and_get() {
        let mut store = OpenChatMessageStore::new();
        let e1 = make_entry(10);
        let e2 = make_entry(20);
        store.insert_messages([e1.clone(), e2.clone()]);
        assert_eq!(store.len(), 2);
        assert_eq!(store.get_message(10).map(|e| e.id()), Some(10));
        assert_eq!(store.get_message(20).map(|e| e.id()), Some(20));
        assert!(store.get_message(15).is_none());
    }

    #[test]
    fn ordered_ids_ascending() {
        let mut store = OpenChatMessageStore::new();
        store.insert_messages([make_entry(30), make_entry(10), make_entry(20)]);
        assert_eq!(store.ordered_message_ids(), [10, 20, 30]);
    }

    #[test]
    fn view_window_oldest_newest() {
        let mut store = OpenChatMessageStore::new();
        assert!(store.oldest_message_id().is_none());
        assert!(store.newest_message_id().is_none());
        store.insert_messages([make_entry(100), make_entry(50)]);
        assert_eq!(store.oldest_message_id(), Some(50));
        assert_eq!(store.newest_message_id(), Some(100));
        store.insert_messages([make_entry(25)]);
        assert_eq!(store.oldest_message_id(), Some(25));
        assert_eq!(store.newest_message_id(), Some(100));
    }

    #[test]
    fn clear_resets_all() {
        let mut store = OpenChatMessageStore::new();
        store.insert_messages([make_entry(1), make_entry(2)]);
        store.clear();
        assert!(store.is_empty());
        assert!(store.ordered_message_ids().is_empty());
        assert!(store.get_message(1).is_none());
        assert!(store.oldest_message_id().is_none());
        assert!(store.newest_message_id().is_none());
    }

    #[test]
    fn from_message_id_for_load_older() {
        let mut store = OpenChatMessageStore::new();
        assert_eq!(store.from_message_id_for_load_older(), 0);
        store.insert_messages([make_entry(100), make_entry(50)]);
        assert_eq!(store.from_message_id_for_load_older(), 50);
    }
}
