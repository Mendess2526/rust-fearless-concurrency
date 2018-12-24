# SD

## Base Structure

### Main
- auctionHouse : `AuctionHouse`
- clients :`List<Client>`
- socketServer :`SocketServer`


### AuctionHouse
- stock :`HashMap<String, (Item, int)>`
- auctions :`List<TopBid>`
- reserved :`List<Droplet>`

### Client
- email :`String`
- passowrd :`String`
- socket :`Socket @Nullable`

### Item
- type :`String`
- price :`int`

### Droplet
- type :`String`
- clientEmail :`String`

### TopBid
- type :`String`
- amount :`int`

### Client-Con (In Thread)
- auctionHouse :`AuctionHouse`
- Socket :`Socket`
- clientEmail :`String`


## Extra points
Reutilizar workers (Client-Con)
