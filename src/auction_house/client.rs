
#[derive(Debug, Clone)]
pub struct Client {
    email :String,
    password :String,
    funds: i32,
}

impl Client {
    pub fn new(email :String, password :String) -> Self {
        Client {
            email,
            password,
            funds: 100,
        }
    }

    pub fn email(&self) -> &str {
        &self.email
    }
    pub fn password(&self) -> &str {
        &self.password
    }
    pub fn funds(&self) -> i32 {
        self.funds
    }

    pub fn spend(&mut self, funds :i32) {
        self.funds -= funds;
    }
}
