use chrono::{NaiveDateTime};

#[derive(Debug)]
pub struct Auction {
    pub transactions: Vec<Transaction>
}
#[derive(Debug)]
pub struct Transaction {
    pub user: String,
    pub current_bidder: String,
    pub bidding_price: f32,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub active: bool,
    pub subscription: bool
}

impl Auction {
    pub fn new(&self){}
    pub fn subscribe(&self, tx: Transaction){}
    pub fn create_auction(&mut self, tx: Transaction){
        self.transactions.push(tx);
    }
    pub fn create_bid(&mut self, id: u32, user: String, credits: f32 , bidding_price: f32) -> f32{
        // validate if user has enough credits.
        if self.transactions[id as usize].active && bidding_price <= credits {
            self.transactions[id as usize].current_bidder = user;
            self.transactions[id as usize].bidding_price = bidding_price;
            println!("Bid created!!\n");
            credits-bidding_price

        }else {
            println!("Auction is not active or not enough credits!!\n");
            credits
        }


    }
    pub fn list_auctions(&self){
        for tx in self.transactions.iter(){
            println!("user:{} current_bidder:{} bidding_price:{} start_time:{} end_time:{} active:{} subscription:{}",
                tx.user, tx.current_bidder, tx.bidding_price, tx.start_time, tx.end_time, tx.active, tx.subscription)
        }
    }

}
