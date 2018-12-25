use super::client::Client;
use super::server_type::ServerType;

use std::sync::atomic::{AtomicUsize, Ordering};

static ID :AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
pub struct Droplet {
    id :u32,
    tp :ServerType,
    owner :String,
}

impl Droplet {
    pub fn new(tp :ServerType, owner :&Client) -> Self {
        Droplet {
            tp,
            id : ID.fetch_add(1, Ordering::SeqCst) as u32,
            owner :owner.email().to_string(),
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
