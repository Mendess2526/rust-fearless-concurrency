pub mod server_type;
mod client;
mod droplet;
mod topbid;

use self::client::Client;
use self::droplet::Droplet;
use self::server_type::ServerType;
use self::topbid::TopBid;

use std::sync::RwLock;
use std::collections::HashMap;

type Stock = HashMap<ServerType, u32>;
type Auctions = HashMap<String, TopBid>;
type Reservations = HashMap<u32, Droplet>;
type Clients = HashMap<String, Client>;

#[derive(Debug)]
pub struct AuctionHouse {
    stock :RwLock<Stock>,
    auctions :RwLock<Auctions>,
    reserved :RwLock<Reservations>,
    clients :RwLock<Clients>,
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
            auctions :RwLock::new(HashMap::new()),
            reserved :RwLock::new(HashMap::new()),
            clients :RwLock::new(HashMap::new()),
        }
    }

    pub fn ls(&self) -> Vec<(ServerType, u32)> {
        self.stock.read().unwrap()
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect()
    }

    pub fn ls_m(&self, clt :&str) -> Vec<Droplet> {
        self.reserved.read().unwrap().values().filter(|d| d.owner() == clt).cloned().collect()
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
                if *v == 0 {
                    Err(AuctionError::OutOfStock(sv_tp))
                }else{
                    client.spend(sv_tp.price());
                    let mut reserved = self.reserved.write().unwrap();
                    let new_drop = Droplet::new(sv_tp, client);
                    reserved.insert(new_drop.id(), new_drop);
                    *v -= 1;
                    Ok(())
                }
            }
        }
    }

    pub fn add(&self, server_type :ServerType) {
        self.stock
            .write().unwrap()
            .entry(server_type)
            .and_modify(|v| *v += 1)
            .or_insert(1);
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

    pub fn drop_server(&self, ctl :&str, id :u32) -> bool {
        let droplet =
        {
            let mut reserved = self.reserved.write().unwrap();
            if !reserved.contains_key(&id) || reserved[&id].owner() != ctl {
                return false
            } else {
                reserved.remove(&id).unwrap()
            }
        };
        self.stock.write().unwrap()
            .entry(droplet.server_type())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        true
    }
}
