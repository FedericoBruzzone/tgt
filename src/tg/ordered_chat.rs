use {std::hash::Hash, tdlib::types::ChatPosition};

#[derive(Debug, Clone, PartialEq)]
pub struct OrderedChat {
    pub chat_id: i64,
    pub position: ChatPosition, // maybe can be changed with position.order
}

impl Hash for OrderedChat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.chat_id.hash(state);

        // self.position.hash(state);
        format!("{:?}", self.position.list).hash(state);
        self.position.order.hash(state);
        self.position.is_pinned.hash(state);
        format!("{:?}", self.position.source).hash(state);
    }
}

impl Eq for OrderedChat {}

impl PartialOrd for OrderedChat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedChat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.position.order != other.position.order {
            if self.position.order > other.position.order {
                return core::cmp::Ordering::Less;
            } else {
                return core::cmp::Ordering::Greater;
            }
        }
        if self.chat_id != other.chat_id {
            if self.chat_id > other.chat_id {
                return core::cmp::Ordering::Less;
            } else {
                return core::cmp::Ordering::Greater;
            }
        }
        core::cmp::Ordering::Equal
    }
}
