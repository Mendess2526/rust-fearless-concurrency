
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

    pub fn from_str(s :&str) -> Option<Self> {
        match s {
            "Fast" => Some(ServerType::Fast),
            "Slow" => Some(ServerType::Slow),
            &_ => None,
        }
    }
}
