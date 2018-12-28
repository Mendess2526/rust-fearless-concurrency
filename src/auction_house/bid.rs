use std::cmp::{Ordering};

#[derive(Debug, Clone)]
pub struct Bid {
    value: i32,
    owner: String,
}

impl PartialOrd for Bid {
    fn partial_cmp(&self, other :&Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Bid {
    fn cmp(&self, other :&Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialEq for Bid {
    fn eq(&self, other :&Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Bid {}

impl Bid {
    pub fn new(owner :&str, value :i32) -> Self {
        Bid {
            value,
            owner: owner.into(),
        }
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}
