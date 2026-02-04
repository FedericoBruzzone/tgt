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

/// Maximum number of messages to keep in memory per chat to prevent unbounded growth.
/// When exceeded, messages outside the sliding window are evicted based on which direction
/// the user is scrolling. This prevents performance degradation when scrolling back months.
/// Using 500 allows for jump loading (~100 older + ~300 newer) plus incremental scroll batches.
const MAX_MESSAGES_IN_CACHE: usize = 500;
/// When cache is full, keep this many messages and evict the rest from the opposite end
const MIN_MESSAGES_AFTER_EVICTION: usize = 400;

impl OpenChatMessageStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert messages (backend/task only). Updates view window.
    /// If cache exceeds MAX_MESSAGES_IN_CACHE, evicts messages based on scroll direction.
    pub fn insert_messages(&mut self, messages: impl IntoIterator<Item = MessageEntry>) {
        let old_oldest = self.range.oldest_message_id;

        for entry in messages {
            let id = entry.id();
            self.range.extend_with(id);
            self.cache.insert(id, entry);
        }

        // Evict messages if cache is too large (sliding window)
        if self.cache.len() > MAX_MESSAGES_IN_CACHE {
            let to_remove = self.cache.len() - MIN_MESSAGES_AFTER_EVICTION;

            // Detect scroll direction by comparing old and new ranges
            let scrolling_backwards = match (old_oldest, self.range.oldest_message_id) {
                (Some(old), Some(new)) => new < old, // Loading older messages
                _ => false,
            };

            let keys_to_remove: Vec<i64> = if scrolling_backwards {
                // Scrolling backwards (loading older): evict NEWEST messages
                self.cache.keys().rev().take(to_remove).copied().collect()
            } else {
                // Scrolling forwards or receiving new: evict OLDEST messages
                self.cache.keys().take(to_remove).copied().collect()
            };

            for key in &keys_to_remove {
                self.cache.remove(key);
            }

            // Recompute range after eviction
            self.range = self.cache.keys().fold(LoadedRange::new(), |mut r, &k| {
                r.extend_with(k);
                r
            });

            tracing::debug!(
                removed = to_remove,
                remaining = self.cache.len(),
                direction = if scrolling_backwards {
                    "backwards"
                } else {
                    "forwards"
                },
                "Evicted messages from cache (sliding window)"
            );
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

    /// Ordered messages (oldest to newest). Single lock for a consistent snapshot; use in UI draw.
    pub fn ordered_messages(&self) -> Vec<MessageEntry> {
        self.cache.values().cloned().collect()
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

    #[test]
    fn cache_evicts_oldest_when_exceeding_max() {
        let mut store = OpenChatMessageStore::new();

        // Insert MAX_MESSAGES_IN_CACHE + 10 messages
        let extra = 10;
        let total = MAX_MESSAGES_IN_CACHE + extra;
        let messages: Vec<_> = (1..=total as i64).map(make_entry).collect();
        store.insert_messages(messages);

        // Should evict down to MIN_MESSAGES_AFTER_EVICTION
        assert_eq!(store.len(), MIN_MESSAGES_AFTER_EVICTION);

        // Some oldest messages should be evicted
        assert!(store.get_message(1).is_none());

        // Newest messages should still be present
        assert!(store.get_message(total as i64).is_some());
        assert!(store.get_message((total - 50) as i64).is_some());
    }

    #[test]
    fn cache_eviction_on_new_message() {
        let mut store = OpenChatMessageStore::new();

        // Fill cache to MAX_MESSAGES_IN_CACHE (IDs 1..2000)
        store.insert_messages((1..=MAX_MESSAGES_IN_CACHE as i64).map(make_entry));
        assert_eq!(store.len(), MAX_MESSAGES_IN_CACHE);

        // Add one more message (triggers eviction)
        store.insert_messages([make_entry((MAX_MESSAGES_IN_CACHE + 1) as i64)]);

        // Should evict down to MIN_MESSAGES_AFTER_EVICTION
        assert_eq!(store.len(), MIN_MESSAGES_AFTER_EVICTION);

        // Oldest message (ID=1) should be evicted
        assert!(store.get_message(1).is_none());

        // Newest message should be present
        assert!(store
            .get_message((MAX_MESSAGES_IN_CACHE + 1) as i64)
            .is_some());
    }
}
