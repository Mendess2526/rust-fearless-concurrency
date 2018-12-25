pub mod item;
mod client;
mod droplet;
mod topbid;

use std::collections::HashMap;
use self::client::Client;
use self::droplet::Droplet;
use self::item::{Item, ServerType};
use self::topbid::TopBid;

use std::sync::RwLock;

#[derive(Debug)]
pub struct AuctionHouse {
    stock :RwLock<HashMap<ServerType, Vec<Item>>>,
    auctions :RwLock<Vec<TopBid>>,
    reserved :RwLock<Vec<Droplet>>,
    clients :RwLock<HashMap<String, Client>>,
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
            stock: RwLock::new(HashMap::new()),
            auctions :RwLock::new(vec![]),
            reserved :RwLock::new(vec![]),
            clients :RwLock::new(HashMap::new()),
        }
    }

    pub fn ls(&self) -> Vec<(ServerType, usize)> {
        self.stock.read().unwrap()
            .iter()
            .map(|(k, v)| (*k, v.len()))
            .collect()
    }

    pub fn ls_m(&self, clt :&str) -> Vec<Droplet> {
        self.reserved.read().unwrap().iter().filter(|d| d.owner() == clt).cloned().collect()
    }

    pub fn buy(&self, sv_tp :ServerType, clt :&str) -> Result<(), AuctionError> {
        let mut clients = self.clients.write().unwrap();
        if !clients.contains_key(clt) {
            return Err(AuctionError::InvalidClient(clt.into()))
        };
        let client = clients.get_mut(clt).unwrap();
        if sv_tp.price() > client.funds() {
            return Err(AuctionError::NotEnughFunds(sv_tp.price(), client.funds()))
        }
        let mut stock = self.stock.write().unwrap();
        match stock.get_mut(&sv_tp) {
            None => Err(AuctionError::OutOfStock(sv_tp)),
            Some(v) => {
                if v.is_empty() {
                    Err(AuctionError::OutOfStock(sv_tp))
                }else{
                    client.spend(sv_tp.price());
                    let mut reserved = self.reserved.write().unwrap();
                    reserved.push(Droplet::new(v.pop().unwrap(), client));
                    Ok(())
                }
            }
        }
    }
    pub fn add(&self, server_type :ServerType) {
        self.stock
            .write().unwrap()
            .entry(server_type)
            .and_modify(|v| v.push(Item::new(server_type)))
            .or_insert(vec![Item::new(server_type)]);
    }

    pub fn register(&self, email :&str, password :&str) -> Result<(), ClientError> {
        let mut clients = self.clients.write().unwrap();
        if clients.contains_key(email) {
            Err(ClientError::EmailTaken(email.to_string()))
        }else{
            clients.insert(
                email.to_string(),
                Client::new(email.to_string(), password.to_string())
            );
            Ok(())
        }
    }

    pub fn login(&self, email: &str, password :&str) -> bool {
        self.clients.read().unwrap().get(email).map(|c| c.password() == password).unwrap_or(false)
    }

    pub fn profile(&self, ctl :&str) -> Option<Client> {
        self.clients.read().unwrap().get(ctl).cloned()
    }
}
