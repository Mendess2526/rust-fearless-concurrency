use super::server_type::ServerType;
use super::client::Client;

use std::sync::atomic::{AtomicUsize, Ordering};

static ID :AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct TopBid {
    id: u32,
    tp: ServerType,
    value: i32,
    owner: String,
}

impl TopBid {
    pub fn new(tp :ServerType, owner :&Client, value :i32) -> Self {
        TopBid {
            id: ID.fetch_add(1, Ordering::SeqCst) as u32,
            tp,
            value,
            owner: owner.email().into(),
        }
    }

    pub fn bid(&mut self, value :i32, owner :&Client) -> bool {
        if value > self.value {
            self.value = value;
            self.owner = owner.email().into();
            true
        } else {
            false
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }
}
