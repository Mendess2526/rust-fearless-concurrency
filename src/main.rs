mod auction_house;

use crate::auction_house::AuctionHouse;
use crate::auction_house::item::ServerType;

fn main() {
    let mut auction_house = AuctionHouse::new();
    auction_house.add(ServerType::Slow);
    auction_house.add(ServerType::Fast);
    auction_house.add(ServerType::Slow);
    auction_house.add(ServerType::Fast);
    println!("{:#?}", auction_house);
    let emails = ["pedro@email.com", "pedro@email.com"];
    for email in emails.iter() {
        match auction_house.register(email.to_string(), "password".into()) {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
    }
    for _ in 0..3 {
        match auction_house.buy(ServerType::Slow, emails[0]) {
            Err(e) => println!("{:?}", e),
            _ => (),
        }
    };
    println!("{:#?}", auction_house);
}
