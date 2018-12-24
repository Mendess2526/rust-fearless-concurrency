mod auction_house;

use crate::auction_house::{AuctionHouse, ClientError, item::ServerType};

use std::io;
use std::str::FromStr;

fn main() {
    let mut auction_house = AuctionHouse::new();
    auction_house.add(ServerType::Slow);
    auction_house.add(ServerType::Fast);
    auction_house.add(ServerType::Slow);
    auction_house.add(ServerType::Fast);
    println!("{:#?}", auction_house);
    let emails = ["pedro@email.com", "pedro@email.com"];
    for email in emails.iter() {
        match auction_house.register(email, "password") {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
    }

    // for _ in 0..3 {
    //     match auction_house.buy(ServerType::Slow, emails[0]) {
    //         Err(e) => println!("{:?}", e),
    //         _ => (),
    //     }
    // };
    // println!("{:#?}", auction_house);
    // println!("{}", auction_house.login(emails[0], "password"));
    // println!("{}", auction_house.login(emails[0], "wordpass"));
    // println!("{}", auction_house.login("unregitered@email", "passfrase"));
    use_loop(&mut auction_house);
}

fn use_loop(ah :&mut AuctionHouse) {
    let mut input = String::new();
    let mut user = None;
    loop {
        input.clear();
        if let Err(_) = io::stdin().read_line(&mut input) { break };
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
