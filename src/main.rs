mod auction_house;

use crate::auction_house::{AuctionHouse, ClientError, item::ServerType};

use std::io;
use std::io::Read;
use std::str::FromStr;
use std::thread;
use std::net::{TcpListener, TcpStream};

fn main() -> io::Result<()> {
    let mut auction_house = AuctionHouse::new();
    for _ in 0..30 { auction_house.add(ServerType::Slow); }
    for _ in 0..4 { auction_house.add(ServerType::Fast); }
    let server = TcpListener::bind("127.0.0.1:12345")?;
    let (stream, _addr) = server.accept()?;
    let thread = thread::spawn(move || use_loop(&mut auction_house, &mut stream));
    thread.join();
    Ok(())
}

fn use_loop(ah :&mut AuctionHouse, stream :&mut TcpStream) {
    let mut user = None;
    let mut socket_input = [u8;1024];
    loop {
        input.clear();
        if let Err(_) = stream.read_to_end(&mut socket_input) { break };
        let mut input = String::from_utf8_lossy(socket_input);
        if input.trim() == "quit" { break }
        let command = input.split_whitespace().map(|s| s.trim()).collect::<Vec<&str>>();
        if command.len() == 0 { continue }
        match command[0] {
            "register" => {
                match Command::register(&command[1..], ah) {
                    Err(e) => eprintln!("{}", e),
                    Ok(_) => {
                        user = Some(command[1].to_owned());
                        println!("Registered successfully!")
                    },
                }
            }
            "login" => {
                match Command::login(&command[1..], ah) {
                    Err(e) => eprintln!("{}", e),
                    Ok(_) => {
                        user = Some(command[1].to_owned());
                        println!("Logged in successfully!")
                    },
                }
            }
            "ls" => {
                match Command::ls(&command[1..], ah, &user) {
                    Err(e) => eprintln!("{}", e),
                    Ok(Command::Ls(s)) => println!("{}", s),
                    Ok(_) => (),
                }
            }
            "buy" => {
                match Command::buy(&command[1..], ah, &user) {
                    Err(e) => eprintln!("{}", e),
                    Ok(_) => println!("Purchase successfull!"),
                }
            }
            &_ => eprintln!("Command not found: {}", input),
        }
    }
}

enum Command {
    Register,
    Login,
    Ls(String),
    Buy,
}

struct CommandError(String);

impl std::fmt::Display for CommandError {
    fn fmt(&self, f :&mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Command {
    fn register(args :&[&str], ah :&mut AuctionHouse) -> Result<Command, CommandError> {
        if args.len() < 2 {
            Err(CommandError("Usage: register <email> <password>".into()))
        } else {
            match ah.register(args[0], args[1]){
                Ok(()) => Ok(Command::Register),
                Err(ClientError::EmailTaken(s)) => Err(CommandError(format!("Email Taken: {}", s))),
            }
        }
    }

    fn login(args :&[&str], ah :&mut AuctionHouse) -> Result<Command, CommandError> {
        if args.len() < 2 {
            Err(CommandError("Usage: login <email> <password>".into()))
        } else if ah.login(args[0], args[1]) {
            Ok(Command::Login)
        } else {
            Err(CommandError("Invalid Credentials".into()))
        }
    }

    fn ls(args :&[&str], ah :&AuctionHouse, user :&Option<String>)
        -> Result<Command, CommandError>
        {
            if args.len() == 0 {
                let stock = ah.ls();
                let mut result = String::from_str("Type\tAmount in stock\n=======================\n")
                    .unwrap();
                for (k, v) in stock.iter() {
                    result += &format!("{:?}\t{}\n", k, v.len());
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

    fn buy(args :&[&str], ah :&mut AuctionHouse, user :&Option<String>)
        -> Result<Command, CommandError>
        {
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
}
