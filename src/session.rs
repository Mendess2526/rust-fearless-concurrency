use crate::auction_house::{AuctionHouse, AHouseError, bid::Bid, server_type::ServerType, client::Client};

use std::io::{Read, Write};
use std::str::FromStr;
use std::net::TcpStream;
use std::sync::{Arc, atomic::AtomicUsize, atomic::Ordering};
use std::ops::Add;

const LOGIN_REQUIRED :&str = "You must be logged in to use this!";
const ID :AtomicUsize = AtomicUsize::new(0);

pub struct Session {
    id :usize,
    user :Option<String>,
    ah :Arc<AuctionHouse>,
    stream: TcpStream,
}

enum Command {
    Register(Client),
    Login(Client),
    Ls(String),
    Buy(u32),
    Auction,
    Profile(String),
    DropServer,
}

struct CommandError(String);

impl From<AHouseError> for CommandError {
    fn from(e :AHouseError) -> Self {
        match e {
            AHouseError::OutOfStock(_) => CommandError("Out of stock".into()),
            AHouseError::EmailTaken(e) => CommandError("Email Taken: ".to_owned() + &e),
            AHouseError::LockError(_) => CommandError("500: Internal Server Error".into()),
            _ => unreachable!(),
        }
    }
}

impl From<&str> for CommandError {
    fn from(s :&str) -> CommandError {
        CommandError(s.to_owned())
    }
}

impl From<String> for CommandError {
    fn from(s :String) -> CommandError {
        CommandError(s)
    }
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f :&mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type CommandResult = Result<Command, CommandError>;

impl Session {
    pub fn new(ah :Arc<AuctionHouse>, stream :TcpStream) -> Self {
        Session {
            id: ID.fetch_add(1, Ordering::SeqCst),
            user: None,
            ah,
            stream,
        }

    }

    pub fn run(mut self) {
        let mut buf = [0; 1024];
        loop {
            let len = match self.stream.read(&mut buf) {
                Err(_) => break,
                Ok(0) => break,
                Ok(n) => n,
            };
            let input = String::from_utf8_lossy(&buf[..len]);
            let command = input
                .split(' ')
                .map(|s| s.trim())
                .filter(|s| s.len() > 0)
                .collect::<Vec<&str>>();
            if command.len() == 0 { continue }
            if command[0] == "quit" { break }
            let response = self.run_command(&command);
            if let Err(_) = self.stream.write_all(response.add("\n").as_bytes()) {
                break
            }
        }
    }

    fn run_command(&mut self, command :&[&str]) -> String {
        match command[0] {
            "register" => {
                match self.register(&command[1..]) {
                    Err(e) => format!("{}", e),
                    Ok(_) => {
                        self.user = Some(command[1].to_owned());
                        "Registered successfully!".into()
                    },
                }
            }
            "login" => {
                match self.login(&command[1..]) {
                    Err(e) => format!("{}", e),
                    Ok(_) => {
                        self.user = Some(command[1].to_owned());
                        "Logged in successfully!".into()
                    },
                }
            }
            "ls" => {
                match self.ls(&command[1..]) {
                    Err(e) => format!("{}", e),
                    Ok(Command::Ls(s)) => s,
                    Ok(_) => "".into(),
                }
            }
            "buy" => {
                match self.buy(&command[1..]) {
                    Err(e) => format!("{}", e),
                    Ok(_) => "Purchase successfull!".into(),
                }
            }
            "profile" => {
                match self.profile() {
                    Err(e) => format!("{}", e),
                    Ok(Command::Profile(s)) => s,
                    Ok(_) => unreachable!(),
                }
            }
            "drop" => {
                match self.drop_server(&command[1..]) {
                    Err(e) => format!("{}", e),
                    Ok(_) => "Server removed successfully".into(),
                }
            }
            "auction" => {
                match self.auction(&command[1..]) {
                    Err(e) => format!("{}", e),
                    Ok(_) => "Auction Started".into(),
                }
            },
            s => format!("Command not found: {}", s),
        }
    }

    fn register(&self, args :&[&str]) -> CommandResult {
        if args.len() < 2 {
            Err("Usage: register <email> <password>")?
        } else {
            self.ah.register(args[0], args[1]).map(|c| Ok(Command::Register(c)))?
        }
    }

    fn login(&self, args :&[&str]) -> CommandResult {
        if args.len() < 2 {
            Err("Usage: login <email> <password>")?
        } else {
            self.ah.login(args[0], args[1]).map(|c| Ok(Command::Login(c)))?
        }
    }

    fn ls(&self, args :&[&str]) -> CommandResult {

        if args.len() == 0 {
            let stock = self.ah.ls();
            let mut result = String::from_str("Type\tAmount in stock\n=======================\n")
                .unwrap();
            for (k, v) in stock.iter() {
                result += &format!("{:?}\t{}\n", k, v);
            }
            Ok(Command::Ls(result))
        } else if args[0] == "-m" {
            if self.user.is_none() {
                Err(LOGIN_REQUIRED)?
            } else {
                Ok(Command::Ls("ID\tType\n=========================\n".to_string()
                               + &self.ah.ls_m(self.user.as_ref().unwrap())
                               .iter()
                               .map(|d| format!("{}\t{:?}\n", d.id(), d.server_type()))
                               .fold(String::new(), |x, acc| acc + &x)
                              ))
            }
        } else {
            Err("Usage: ls [-m]\n\t-m show my droplets")?
        }
    }

    fn buy(&self, args :&[&str]) -> CommandResult {
        if self.user.is_none() {
            Err(LOGIN_REQUIRED)?
        } else if args.len() < 1 {
            Err("Usage: buy <Fast,Slow>")?
        } else {
            let st = match ServerType::from_str(args[0]) {
                None => Err("Invalid server type!")?,
                Some(s) => s,
            };
            AuctionHouse::buy(Arc::clone(&self.ah), st, &self.user.as_ref().unwrap())
                .map(|id| Command::Buy(id))
                .map_err(|e| e.into())
        }
    }

    fn profile(&self) -> CommandResult {
        match self.user.as_ref() {
            None => Err(LOGIN_REQUIRED)?,
            Some(ctl) => {
                let c = self.ah.profile(&ctl).unwrap();
                Ok(Command::Profile(format!("email: {}", c.email())))
            }
        }
    }

    fn drop_server(&self, args :&[&str]) -> CommandResult {
        if self.user.is_none() { Err(LOGIN_REQUIRED)? }
        if args.len() < 1 { Err("Usage: drop <id>")? }
        args[0].parse::<u32>()
            .map_err(|_| "Invalid id: {}".to_owned() + args[0])
            .and_then(|id|
                      if self.ah.drop_server(self.user.as_ref().unwrap(), id) {
                          Ok(Command::DropServer)
                      } else {
                          Err("Invalid Server id".to_owned())
                      }
                     )
            .map_err(|s| s.into())
    }

    fn auction(&self, args :&[&str]) -> CommandResult {
        if self.user.is_none() { Err(LOGIN_REQUIRED)? };
        if args.len() < 2 { Err("Usage: start-bid <Fast|Slow> <amount>")? };
        let sv_tp = match ServerType::from_str(args[0]) {
            None => Err("Invalid server type!")?,
            Some(sv_tp) => sv_tp,
        };
        args[1].parse::<i32>()
            .map_err(|_| CommandError("Invalid amount".into()))
            .and_then(|amount|
                      AuctionHouse::auction(
                          Arc::clone(&self.ah),
                          sv_tp,
                          Bid::new(self.user.as_ref().unwrap(), amount),
                          self.id
                          )
                      .map(|_| Command::Auction).map_err(|e| e.into()))
    }
}
