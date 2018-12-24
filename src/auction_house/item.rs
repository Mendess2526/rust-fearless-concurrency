use std::sync::atomic::{AtomicUsize, Ordering};

static ID :AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct Item {
    id :u32,
    tp :ServerType,
}

impl Item {
    pub fn new(tp :ServerType) -> Self {
        Item {
            id : ID.fetch_add(1, Ordering::SeqCst) as u32,
            tp,
        }
    }

    pub fn server_type(&self) -> ServerType {
        self.tp
    }

    pub fn price(&self) -> i32 {
        self.tp.price()
    }
}

#[derive(Debug,Copy,Clone,PartialEq,PartialOrd,Eq,Ord,Hash)]
pub enum ServerType {
    Slow,
    Fast,
}

impl ServerType {
    pub fn price(&self) -> i32 {
        match self {
            ServerType::Slow => 20,
            ServerType::Fast => 40,
        }
    }
}
