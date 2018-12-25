mod auction_house;

use crate::auction_house::{AuctionHouse, ClientError, server_type::ServerType};

use std::io::{self, Read, Write};
use std::str::FromStr;
use std::thread;
use std::net::{TcpListener, TcpStream};
use std::ops::Add;
use std::sync::Arc;

fn main() -> io::Result<()> {
    let auction_house = AuctionHouse::new();
    for _ in 0..30 { auction_house.add(ServerType::Slow); }
    for _ in 0..4 { auction_house.add(ServerType::Fast); }
    let ah_arc = Arc::new(auction_house);
    let server = TcpListener::bind("127.0.0.1:12345")?;
    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                let ah_instance = Arc::clone(&ah_arc);
                thread::spawn(move || use_loop(ah_instance, stream));
            },
            Err(e) => eprintln!("{:?}", e),
        }
    }
    Ok(())
}

fn use_loop(ah :Arc<AuctionHouse>, mut stream :TcpStream) {
    let mut buf = [0; 1024];
    let mut runner = Runner{ user: None, auction_house: ah };
    loop {
        let len = match stream.read(&mut buf) {
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
        let response = runner.run_command(&command);
        if let Err(_) = stream.write_all(response.add("\n").as_bytes()) {
            break
        }
    }
}

struct Runner {
    user :Option<String>,
    auction_house :Arc<AuctionHouse>,
}

impl Runner {
    fn run_command(&mut self, command :&[&str]) -> String {
        match command[0] {
            "register" => {
                match Command::register(&command[1..], &self.auction_house) {
                    Err(e) => format!("{}", e),
                    Ok(_) => {
                        self.user = Some(command[1].to_owned());
                        "Registered successfully!".into()
                    },
                }
            }
            "login" => {
                match Command::login(&command[1..], &self.auction_house) {
                    Err(e) => format!("{}", e),
                    Ok(_) => {
                        self.user = Some(command[1].to_owned());
                        "Logged in successfully!".into()
                    },
                }
            }
            "ls" => {
                match Command::ls(&command[1..], &self.auction_house, &self.user) {
                    Err(e) => format!("{}", e),
                    Ok(Command::Ls(s)) => s,
                    Ok(_) => "".into(),
                }
            }
            "buy" => {
                match Command::buy(&command[1..], &self.auction_house, &self.user) {
                    Err(e) => format!("{}", e),
                    Ok(_) => "Purchase successfull!".into(),
                }
            }
            "profile" => {
                match Command::profile(&self.auction_house, &self.user) {
                    Err(e) => format!("{}", e),
                    Ok(Command::Profile(s)) => s,
                    Ok(_) => unreachable!(),
                }
            }
            s => format!("Command not found: {}", s),
        }
    }

}


enum Command {
    Register,
    Login,
    Ls(String),
    Buy,
    Profile(String),
    DropServer,
}

struct CommandError(String);

impl std::fmt::Display for CommandError {
    fn fmt(&self, f :&mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

type CommandResult = Result<Command, CommandError>;

impl Command {
    fn register(args :&[&str], ah :&Arc<AuctionHouse>) -> CommandResult {
        if args.len() < 2 {
            Err(CommandError("Usage: register <email> <password>".into()))
        } else {
            match ah.register(args[0], args[1]){
                Ok(()) => Ok(Command::Register),
                Err(ClientError::EmailTaken(s)) => Err(CommandError(format!("Email Taken: {}", s))),
            }
        }
    }

    fn login(args :&[&str], ah :&Arc<AuctionHouse>) -> CommandResult {
        if args.len() < 2 {
            Err(CommandError("Usage: login <email> <password>".into()))
        } else if ah.login(args[0], args[1]) {
            Ok(Command::Login)
        } else {
            Err(CommandError("Invalid Credentials".into()))
        }
    }

    fn ls(args :&[&str],
          ah :&Arc<AuctionHouse>,
          user :&Option<String>) -> CommandResult {

        if args.len() == 0 {
            let stock = ah.ls();
            let mut result = String::from_str("Type\tAmount in stock\n=======================\n")
                .unwrap();
            for (k, v) in stock.iter() {
                result += &format!("{:?}\t{}\n", k, v);
            }
            Ok(Command::Ls(result))
        } else if args[0] == "-m" {
            if user.is_none() {
                Err(CommandError("You must be logged in to use this!".into()))
            } else {
                Ok(Command::Ls(format!("{:?}", ah.ls_m(user.as_ref().unwrap())
                                       .iter()
                                       .map(|d| d.server_type())
                                       .collect::<Vec<ServerType>>())
                              ))
            }
        } else {
            Err(CommandError("Usage: ls [-m]\n\t-m show my droplets".into()))
        }
    }

    fn buy(args :&[&str],
           ah :&Arc<AuctionHouse>,
           user :&Option<String>) -> CommandResult {

        if user.is_none() {
            Err(CommandError("You must be logged in to use this!".into()))
        } else if args.len() < 1 {
            Err(CommandError("Usage: buy <Fast,Slow>".into()))
        } else {
            let maybe_st = ServerType::from_str(args[0]);
            if maybe_st.is_none() {
                return Err(CommandError("Invalid server type!".into()));
            }
            let st = maybe_st.unwrap();
            use crate::auction_house::AuctionError::*;
            match ah.buy(st, &user.as_ref().unwrap()) {
                Err(InvalidClient(_)) => unreachable!(),
                Err(OutOfStock(_)) => Err(CommandError("Out of stock".into())),
                Err(NotEnughFunds(p,f)) =>
                    Err(CommandError(format!("not enough funds: Price {}, Funds: {}", p, f))),
                Ok(()) => Ok(Command::Buy),
            }
        }
    }

    fn profile(ah :&Arc<AuctionHouse>, user :&Option<String>) -> CommandResult {
        match user.as_ref() {
            None => Err(CommandError("You must be logged in to use this!".into())),
            Some(ctl) => match ah.profile(&ctl) {
                None => unreachable!(),
                Some(c) =>
                    Ok(Command::Profile(format!("email: {}, funds: {}", c.email(), c.funds()))),
            }
        }
    }

    fn drop(ah :&Arc<AuctionHouse>, user :&Option<String>, id :u32) -> CommandResult {
        if user.is_none() { Err(CommandError("You must be logged in to use this!".into())) }
        else {
            if ah.drop(user.as_ref().unwrap(), id) {
                Ok(Command::DropServer)
            } else {
                Err(CommandError("Invalid Server id".into()))
            }
        }
    }
}
