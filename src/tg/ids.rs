//! Type-safe ID wrappers to avoid sentinel values like -1.
//!
//! These types wrap i64 but provide type-safe "none" values, allowing us to model
//! "no value" in the type system while still using AtomicI64 for thread-safe access.
//! The sentinel value is encapsulated within the type, so callers don't need to know
//! that -1 or 0 is used internally.

use std::sync::atomic::{AtomicI64, Ordering};

/// Chat ID: wraps i64, with NONE = 0 (TDLib chat IDs are positive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChatId(i64);

impl ChatId {
    /// No chat selected (sentinel value: 0).
    pub const NONE: ChatId = ChatId(0);

    /// Create a ChatId from an i64. Use NONE for "no chat".
    pub fn new(id: i64) -> Self {
        ChatId(id)
    }

    /// Returns true if this is NONE.
    pub fn is_none(self) -> bool {
        self.0 == 0
    }

    /// Returns true if this is a valid chat ID (not NONE).
    pub fn is_some(self) -> bool {
        self.0 != 0
    }

    /// Get the inner i64 value.
    pub fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for ChatId {
    fn from(id: i64) -> Self {
        ChatId(id)
    }
}

impl From<ChatId> for i64 {
    fn from(id: ChatId) -> Self {
        id.0
    }
}

/// Message ID: wraps i64, with NONE = -1 (TDLib message IDs are positive).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageId(i64);

impl MessageId {
    /// No message selected (sentinel value: -1).
    pub const NONE: MessageId = MessageId(-1);

    /// Create a MessageId from an i64. Use NONE for "no message".
    pub fn new(id: i64) -> Self {
        MessageId(id)
    }

    /// Returns true if this is NONE.
    pub fn is_none(self) -> bool {
        self.0 == -1
    }

    /// Returns true if this is a valid message ID (not NONE).
    pub fn is_some(self) -> bool {
        self.0 != -1
    }

    /// Get the inner i64 value.
    pub fn as_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for MessageId {
    fn from(id: i64) -> Self {
        MessageId(id)
    }
}

impl From<MessageId> for i64 {
    fn from(id: MessageId) -> Self {
        id.0
    }
}

/// Thread-safe wrapper around AtomicI64 for ChatId.
/// Stores ChatId internally as i64, so no indirect memory access needed.
#[derive(Debug)]
pub struct AtomicChatId(AtomicI64);

impl AtomicChatId {
    pub const fn new(id: ChatId) -> Self {
        AtomicChatId(AtomicI64::new(id.0))
    }

    pub fn load(&self, order: Ordering) -> ChatId {
        ChatId(self.0.load(order))
    }

    pub fn store(&self, id: ChatId, order: Ordering) {
        self.0.store(id.0, order);
    }

    pub fn compare_exchange(
        &self,
        current: ChatId,
        new: ChatId,
        success: Ordering,
        failure: Ordering,
    ) -> Result<ChatId, ChatId> {
        self.0
            .compare_exchange(current.0, new.0, success, failure)
            .map(ChatId)
            .map_err(ChatId)
    }
}

/// Thread-safe wrapper around AtomicI64 for MessageId.
/// Stores MessageId internally as i64, so no indirect memory access needed.
#[derive(Debug)]
pub struct AtomicMessageId(AtomicI64);

impl AtomicMessageId {
    pub const fn new(id: MessageId) -> Self {
        AtomicMessageId(AtomicI64::new(id.0))
    }

    pub fn load(&self, order: Ordering) -> MessageId {
        MessageId(self.0.load(order))
    }

    pub fn store(&self, id: MessageId, order: Ordering) {
        self.0.store(id.0, order);
    }

    pub fn compare_exchange(
        &self,
        current: MessageId,
        new: MessageId,
        success: Ordering,
        failure: Ordering,
    ) -> Result<MessageId, MessageId> {
        self.0
            .compare_exchange(current.0, new.0, success, failure)
            .map(MessageId)
            .map_err(MessageId)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_id_none() {
        assert!(ChatId::NONE.is_none());
        assert!(!ChatId::NONE.is_some());
        assert_eq!(ChatId::NONE.as_i64(), 0);
    }

    #[test]
    fn chat_id_some() {
        let id = ChatId::new(123);
        assert!(!id.is_none());
        assert!(id.is_some());
        assert_eq!(id.as_i64(), 123);
    }

    #[test]
    fn message_id_none() {
        assert!(MessageId::NONE.is_none());
        assert!(!MessageId::NONE.is_some());
        assert_eq!(MessageId::NONE.as_i64(), -1);
    }

    #[test]
    fn message_id_some() {
        let id = MessageId::new(456);
        assert!(!id.is_none());
        assert!(id.is_some());
        assert_eq!(id.as_i64(), 456);
    }

    #[test]
    fn atomic_chat_id() {
        let atomic = AtomicChatId::new(ChatId::NONE);
        assert_eq!(atomic.load(Ordering::Relaxed), ChatId::NONE);
        
        atomic.store(ChatId::new(789), Ordering::Relaxed);
        assert_eq!(atomic.load(Ordering::Relaxed).as_i64(), 789);
    }

    #[test]
    fn atomic_message_id() {
        let atomic = AtomicMessageId::new(MessageId::NONE);
        assert_eq!(atomic.load(Ordering::Relaxed), MessageId::NONE);
        
        atomic.store(MessageId::new(999), Ordering::Relaxed);
        assert_eq!(atomic.load(Ordering::Relaxed).as_i64(), 999);
    }
}
