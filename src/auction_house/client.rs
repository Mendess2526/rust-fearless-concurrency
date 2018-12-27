use super::server_type::ServerType;
use super::transaction::Transaction;

#[derive(Debug, Clone)]
pub struct Client {
    email :String,
    password :String,
    transactions :Vec<Transaction>,
}

impl Client {
    pub fn new(email :String, password :String) -> Self {
        Client {
            email,
            password,
            transactions: vec![],
        }
    }

    pub fn email(&self) -> &str {
        &self.email
    }
    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    pub fn buy(&mut self, server_type :ServerType) {
        self.transactions.push(Transaction::new_purchase(server_type));
    }

    pub fn auction(&mut self, server_type :ServerType, value :i32){
        self.transactions.push(Transaction::new_auction(server_type, value));
    }
}
