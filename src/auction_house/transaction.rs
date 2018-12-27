use chrono::offset::Utc;
use chrono::DateTime;

use super::server_type::ServerType;

#[derive(Debug,Clone)]
pub struct Transaction {
    date :DateTime<Utc>,
    server_type :ServerType,
    value :i32,
    auction :bool,
}

impl Transaction {
    pub fn new_purchase(server_type :ServerType) -> Self {
        Transaction {
            date :Utc::now(),
            server_type,
            value :server_type.price(),
            auction :false,
        }
    }

    pub fn new_auction(server_type :ServerType, value :i32) -> Self {
        Transaction {
            date :Utc::now(),
            server_type,
            value,
            auction :true,
        }
    }
}

impl std::fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}] | {:?} | {:5} | {}",
               self.date,
               self.server_type,
               self.value,
               if self.auction { "auctioned" }else{ "bought" }
        )
    }
}
