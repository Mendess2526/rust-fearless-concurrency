pub mod server_type;
mod client;
mod droplet;
mod topbid;
pub mod transaction;

use self::client::Client;
use self::droplet::Droplet;
use self::server_type::ServerType;
use self::topbid::TopBid;
use crate::task::Task;

use std::sync::RwLock;
use std::sync::Arc;
use std::sync::Weak;
use std::collections::HashMap;

type Stock = HashMap<ServerType, u32>;
type Auctions = HashMap<u32, TopBid>;
type Reservations = HashMap<u32, Droplet>;
type Clients = HashMap<String, Client>;

#[derive(Debug)]
pub struct AuctionHouse {
    stock :Arc<RwLock<Stock>>,
    auctions :RwLock<Auctions>,
    reserved :Arc<RwLock<Reservations>>,
    clients :RwLock<Clients>,
}

#[derive(Debug)]
pub enum AHouseError {
    OutOfStock(ServerType),
    NotEnughFunds(i32, i32),
    InvalidClient(String),
    EmailTaken(String),
    LockError(String),
    InvalidAuction(u32),
}

impl<T> From<std::sync::PoisonError<T>> for AHouseError {
    fn from(error :std::sync::PoisonError<T>) -> Self {
        AHouseError::LockError(format!("{:?}", error))
    }
}

impl AuctionHouse {
    pub fn new() -> Self{
        AuctionHouse {
            stock: Arc::new(RwLock::new(HashMap::new())), // change to Arc Weak
            auctions :RwLock::new(HashMap::new()),
            reserved :Arc::new(RwLock::new(HashMap::new())),
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

    pub fn buy(&self, sv_tp :ServerType, clt :&str) -> Result<(), AHouseError> {
        let mut clients = self.clients.write().unwrap();
        if !clients.contains_key(clt) {
            return Err(AHouseError::InvalidClient(clt.into()))
        };
        let client = clients.get_mut(clt).unwrap();
        let mut stock = self.stock.write().unwrap();
        match stock.get_mut(&sv_tp) {
            None => Err(AHouseError::OutOfStock(sv_tp)),
            Some(v) => {
                if *v == 0 {
                    Err(AHouseError::OutOfStock(sv_tp))
                }else{
                    client.buy(sv_tp);
                    let mut reserved = self.reserved.write().unwrap();
                    let mut new_drop = Droplet::new(sv_tp, client);
                    if sv_tp == ServerType::Fast {
                        let id = new_drop.id();
                        let stock_ptr = Arc::downgrade(&self.stock);
                        let reserved_ptr = Arc::downgrade(&self.reserved);
                        new_drop.set_task(
                            Task::new(
                                move || {drop_server_unchecked(reserved_ptr, stock_ptr, id);},
                                10)
                            );
                    }
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

    pub fn register(&self, email :&str, password :&str) -> Result<(), AHouseError> {
        let mut clients = self.clients.write().unwrap();
        if clients.contains_key(email) {
            Err(AHouseError::EmailTaken(email.to_string()))
        }else{
            clients.insert( email.to_string(),
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

    pub fn drop_server(&self, ctl :&str, id :u32) -> bool { // TODO: make this transactional
        let droplet =
        {
            let mut reserved = self.reserved.write().unwrap();
            if !reserved.contains_key(&id) || reserved[&id].owner() != ctl {
                return false
            }
            reserved.remove(&id).unwrap()
        };
        self.stock.write().unwrap()
            .entry(droplet.server_type())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        true
    }

    pub fn start_bid(&self, ctl :&str, sv_tp :ServerType, value :i32) -> Result<u32,AHouseError> {
        let clients = self.clients.read()?;
        let mut auctions = self.auctions.write()?;
        let client = match clients.get(ctl) {
            None => return Err(AHouseError::InvalidClient(ctl.into())),
            Some(c) => c,
        };
        let auction = TopBid::new(sv_tp, client, value);
        let id = auction.id();
        auctions.insert(id, auction);
        Ok(id)
    }

    pub fn bid(&self, ctl :&str, id :u32, value :i32) -> Result<bool, AHouseError> {
        let clients = self.clients.read()?;
        let mut auctions = self.auctions.write()?;
        let client = match clients.get(ctl) {
            None => return Err(AHouseError::InvalidClient(ctl.into())),
            Some(c) => c,
        };
        auctions.get_mut(&id)
            .map(|b| Ok(b.bid(value, client)))
            .unwrap_or(Err(AHouseError::InvalidAuction(id)))
    }

}

fn drop_server_unchecked(
    reserved :Weak<RwLock<Reservations>>,
    stock :Weak<RwLock<Stock>>,
    id :u32) -> Option<()> {

    let droplet = {
        match reserved.upgrade()?.write().unwrap().remove(&id) {
            None => return None,
            Some(d) => d,
        }
    };
    stock.upgrade()?.write().unwrap()
        .entry(droplet.server_type())
        .and_modify(|c| *c += 1)
        .or_insert(1);
    Some(())
}

fn end_bid(
    auctions :Weak<RwLock<Reservations>>,
    stock :Weak<RwLock<Stock>>,
    clients :Weak<RwLock<Clients>>,
    id :u32) -> Option<()> {

    let auctions = auctions.upgrade()?;
    let stock    = stock   .upgrade()?;
    let clients  = clients .upgrade()?;
    let auctions = auctions.write().unwrap();
    let stock    = stock   .write().unwrap();
    let clients  = clients .write().unwrap();
    Some(())
}
