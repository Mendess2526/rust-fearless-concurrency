use super::client::Client;
use super::server_type::ServerType;

use std::sync::atomic::{AtomicUsize, Ordering};

static ID :AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct Droplet {
    id :u32,
    tp :ServerType,
    owner :String,
    value :i32,
}

impl Droplet {
    pub fn new_reserved(tp :ServerType, owner :&str) -> Self {
        Droplet {
            tp,
            id: ID.fetch_add(1, Ordering::SeqCst) as u32,
            owner: owner.to_string(),
            value: tp.price(),
        }
    }

    pub fn new_auctioned(tp :ServerType, owner :&str, value :i32) -> Self {
        Droplet {
            tp,
            id: ID.fetch_add(1, Ordering::SeqCst) as u32,
            owner: owner.to_string(),
            value: value,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn server_type(&self) -> ServerType {
        self.tp
    }

}
