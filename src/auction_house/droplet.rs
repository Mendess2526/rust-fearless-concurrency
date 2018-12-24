use super::client::Client;
use super::item::Item;

#[derive(Debug)]
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
}
