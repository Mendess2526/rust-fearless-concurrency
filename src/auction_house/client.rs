#[derive(Debug, Clone)]
pub struct Client {
    email :String,
    password :String,
}

impl Client {
    pub fn new(email :String, password :String) -> Self {
        Client {
            email,
            password,
        }
    }

    pub fn email(&self) -> &str {
        &self.email
    }
    pub fn password(&self) -> &str {
        &self.password
    }
}
