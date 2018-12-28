mod auction_house;
mod task;
mod session;

use crate::auction_house::{AuctionHouse, server_type::ServerType};
use crate::session::Session;

use std::io::Result;
use std::thread;
use std::net::TcpListener;
use std::sync::Arc;

fn main() -> Result<()> {
    let auction_house = AuctionHouse::new();
    for _ in 0..30 { auction_house.add(ServerType::Slow); }
    for _ in 0..4 { auction_house.add(ServerType::Fast); }
    let ah_arc = Arc::new(auction_house);
    let server = TcpListener::bind("127.0.0.1:12345")?;
    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                let ah_instance = Arc::clone(&ah_arc);
                thread::spawn(move || Session::new(ah_instance, stream).run());
            },
            Err(e) => eprintln!("{:?}", e),
        }
    }
    Ok(())
}
