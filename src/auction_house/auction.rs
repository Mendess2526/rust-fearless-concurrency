use super::server_type::ServerType;
use super::bid::Bid;
use crate::task::Task;

use std::collections::BinaryHeap;
use std::sync::{RwLock, Arc};

#[derive(Debug)]
pub struct Auction {
    server_type :ServerType,
    bids :Arc<RwLock<BinaryHeap<Bid>>>,
    callback :Task
}

pub enum BidError {
    BidTooLow(i32),
    LockError,
}

impl<T> From<std::sync::PoisonError<T>> for BidError {
    fn from(e :std::sync::PoisonError<T>) -> Self {
        BidError::LockError(format!("{:?}", e))
    }
}

impl Auction {
    pub fn new<T>(server_type :ServerType, bid :Bid, f :T) -> Auction
        where
        T: FnOnce(Bid) -> (),
        T: std::marker::Send + 'static
        {
            let bids = Arc::new(RwLock::new({
                let mut bids = BinaryHeap::new();
                bids.push(bid);
                bids
            }));
            let bids_arc = Arc::clone(&bids);
            Auction {
                server_type,
                bids: bids,
                callback: Task::new(|| f(Auction::highest_bid(bids_arc)), 10)
            }
        }

    pub fn bid(&mut self, bid :Bid) -> Result<(), BidError> {
        let top_bid = self.bids.read()?.peek().unwrap();
        if top_bid > &bid {
            Err(BidError::BidTooLow(top_bid.value()))
        } else {
            self.bids.write()?.push(bid);
            Ok(())
        }
    }

    fn highest_bid(bids :Arc<RwLock<BinaryHeap<Bid>>>) -> Bid {
        bids.read().unwrap().peek().unwrap().clone()
    }
}
