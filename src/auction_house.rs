pub mod server_type;
pub mod client;
mod droplet;
pub mod bid;
mod auction;

use self::client::Client;
use self::droplet::Droplet;
use self::server_type::ServerType;
use self::bid::Bid;
use self::auction::Auction;

use std::sync::RwLock;
use std::sync::Arc;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

#[derive(Debug)]
pub enum AHouseError {
    OutOfStock(ServerType),
    InvalidClient(String),
    EmailTaken(String),
    LockError(String),
    BidTooLow,
}

impl<T> From<std::sync::PoisonError<T>> for AHouseError {
    fn from(error :std::sync::PoisonError<T>) -> Self {
        AHouseError::LockError(format!("{:?}", error))
    }
}

impl From<self::auction::BidError> for AHouseError {
    fn from(_error :self::auction::BidError) -> Self {
        AHouseError::BidTooLow
    }
}
#[derive(Debug)]
pub struct AuctionHouse {
    stock           :RwLock<HashMap<ServerType, u32>>,
    auctions        :RwLock<HashMap<ServerType, Auction>>,
    reserved_a      :RwLock<HashMap<u32,        Droplet>>,
    reserved_d      :RwLock<HashMap<u32,        Droplet>>,
    clients         :RwLock<HashMap<String,     Client>>,
    dropped_servers :RwLock<HashMap<String,     AtomicUsize>>,
}

impl AuctionHouse {
    pub fn new() -> Self{
        AuctionHouse {
            stock :RwLock::new(HashMap::new()),
            auctions :RwLock::new(HashMap::new()),
            reserved_a :RwLock::new(HashMap::new()),
            reserved_d :RwLock::new(HashMap::new()),
            clients :RwLock::new(HashMap::new()),
            dropped_servers :RwLock::new(HashMap::new()),
        }
    }

    pub fn ls(&self) -> Vec<(ServerType, u32)> {
        self.stock.read().unwrap()
            .iter()
            .map(|(k, v)| (*k, *v))
            .collect()
    }

    pub fn ls_m(&self, clt :&str) -> Vec<Droplet> {
        self.reserved_d.read().unwrap().values().filter(|d| d.owner() == clt).cloned().collect()
    }

    pub fn buy(ah :Arc<AuctionHouse>, sv_tp :ServerType, clt :&str) -> Result<u32, AHouseError> {
        if !ah.clients.read()?.contains_key(clt) {
            return Err(AHouseError::InvalidClient(clt.into()))
        };
        let mut stock = ah.stock.write().unwrap();
        match stock.get_mut(&sv_tp) {
            None => Err(AHouseError::OutOfStock(sv_tp)),
            Some(v) => {
                if *v == 0 {
                    Err(AHouseError::OutOfStock(sv_tp))
                }else{
                    let mut reserved = ah.reserved_d.write().unwrap();
                    let new_drop = Droplet::new_reserved(sv_tp, clt);
                    let id = new_drop.id();
                    reserved.insert(id, new_drop);
                    *v -= 1;
                    Ok(id)
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

    pub fn register(&self, email :&str, password :&str) -> Result<Client, AHouseError> {
        let mut clients = self.clients.write().unwrap();
        if clients.contains_key(email) {
            Err(AHouseError::EmailTaken(email.to_string()))
        }else{
            let client = Client::new(email.to_string(), password.to_string());
            clients.insert(email.to_string(), client.clone());
            Ok(client)
        }
    }

    pub fn login(&self, email: &str, password :&str) -> Result<Client, AHouseError> {
        self.clients.read()?
            .get(email)
            .cloned()
            .map(|c| Ok(c))
            .unwrap_or(Err(AHouseError::InvalidClient(email.into())))
            .and_then(|c|
                      if c.password() == password {
                          Ok(c)
                      } else {
                          Err(AHouseError::InvalidClient(email.into()))
                      }
                     )
    }

    pub fn profile(&self, ctl :&str) -> Option<Client> {
        self.clients.read().unwrap().get(ctl).cloned()
    }

    pub fn drop_server(&self, ctl :&str, id :u32) -> bool { // TODO: make this transactional
        let droplet =
        {
            let mut reserved = self.reserved_d.write().unwrap();
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

    pub fn auction(
        ah :Arc<AuctionHouse>,
        server_type :ServerType,
        bid :Bid,
        id :usize) -> Result<(),AHouseError> {

        let mut auctions = ah.auctions.write()?;
        match auctions.get_mut(&server_type) {
            Some(a) => a.bid(bid).map_err(AHouseError::from),
            None => {
                let ah_arc = Arc::clone(&ah);
                auctions.insert(
                    server_type,
                    Auction::new(
                        server_type,
                        bid,
                        move |bid| {buy_auctioned(ah_arc, server_type, bid);}
                        )
                    );
                Ok(())
            },
        }
    }
}

fn buy_auctioned(
    ah :Arc<AuctionHouse>,
    server_type :ServerType,
    bid :Bid) -> Result<(), AHouseError> {

    match ah.stock.write()?.get_mut(&server_type) {
        None => unreachable!(),
        Some(0) => return Err(AHouseError::OutOfStock(server_type)),
        Some(i) => {
            *i -= 1;
            let droplet = Droplet::new_auctioned(server_type, bid.owner(), bid.value());
            ah.reserved_a.write()?.insert(droplet.id(), droplet);
            Ok(())
        },
    }
}
