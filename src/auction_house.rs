pub mod item;
mod client;
mod droplet;
mod topbid;

use std::collections::HashMap;
use self::client::Client;
use self::droplet::Droplet;
use self::item::{Item, ServerType};
use self::topbid::TopBid;

#[derive(Debug)]
pub struct AuctionHouse {
    stock :HashMap<ServerType, Vec<Item>>,
    auctions :Vec<TopBid>,
    reserved :Vec<Droplet>,
    clients: HashMap<String, Client>,
}

#[derive(Debug)]
pub enum AuctionError {
    OutOfStock(ServerType),
    NotEnughFunds(i32, i32),
    InvalidClient(String),
}

#[derive(Debug)]
pub enum ClientError {
    EmailTaken(String),
}

impl AuctionHouse {
    pub fn new() -> Self{
        AuctionHouse {
            stock: HashMap::new(),
            auctions :vec![],
            reserved :vec![],
            clients :HashMap::new(),
        }
    }

    pub fn ls(&self) -> &HashMap<ServerType, Vec<Item>> {
        &self.stock
    }

    pub fn ls_m(&self, clt :&str) -> Vec<&Droplet> {
        self.reserved.iter().filter(|d| d.owner() == clt).collect()
    }

    pub fn buy(&mut self, sv_tp :ServerType, clt :&str) -> Result<(), AuctionError> {
        if !self.clients.contains_key(clt) {
            return Err(AuctionError::InvalidClient(clt.into()))
        };
        let client = self.clients.get_mut(clt).unwrap();
        if sv_tp.price() > client.funds() {
            return Err(AuctionError::NotEnughFunds(sv_tp.price(), client.funds()))
        }
        match self.stock.get_mut(&sv_tp) {
            None => Err(AuctionError::OutOfStock(sv_tp)),
            Some(v) => {
                if v.is_empty() {
                    Err(AuctionError::OutOfStock(sv_tp))
                }else{
                    client.spend(sv_tp.price());
                    self.reserved.push(Droplet::new(v.pop().unwrap(), client));
                    Ok(())
                }
            }
        }
    }
    pub fn add(&mut self, server_type :ServerType) {
        self.stock
            .entry(server_type)
            .and_modify(|v| v.push(Item::new(server_type)))
            .or_insert(vec![Item::new(server_type)]);
    }

    pub fn register(&mut self, email :&str, password :&str) -> Result<(), ClientError> {
        if self.clients.contains_key(email) {
            Err(ClientError::EmailTaken(email.to_string()))
        }else{
            self.clients.insert(
                email.to_string(),
                Client::new(email.to_string(), password.to_string())
            );
            Ok(())
        }
    }

    pub fn login(&self, email: &str, password :&str) -> bool {
        self.clients.get(email).map(|c| c.password() == password).unwrap_or(false)
    }
}
