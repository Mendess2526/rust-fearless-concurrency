use super::client::Client;
use super::item::{Item, ServerType};

#[derive(Debug, Clone)]
pub struct Droplet {
    item :Item,
    owner :String,
}

impl Droplet {
    pub fn new(item :Item, owner :&Client) -> Self {
        Droplet {
            item,
            owner :owner.email().to_string(),
        }
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn server_type(&self) -> ServerType {
        self.item.server_type()
    }
}
