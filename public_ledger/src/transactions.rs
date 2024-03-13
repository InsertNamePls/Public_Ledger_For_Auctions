use std::collections::HashMap;

#[derive(Debug)]
struct Transaction {
    tx_content: String,
    tx_signature: String,
}

impl Transaction {
    fn new(tx_content: String, tx_signature: String) -> Self {
        Transaction {
            tx_content,
            tx_signature,
        }
    }
}

fn tx_mapper(tx: Transaction, tx_type: String) -> HashMap<String, Transaction> {
    let tx_map: HashMap<String, Transaction> = HashMap::new();
    tx_map
}
fn main() {
    println!("hello");

    let tx = Transaction::new(
        String::from("uid: 1, credits: 10, pub_key: 21asndajsndasndajda"),
        "asndasjkndajksndakjsd".to_string(),
    );
    let tx2 = Transaction::new(
        String::from("uid: 1, credits: 200, pub_key: 21asndajsndasndajda"),
        "asndasjkndajksndakjsd".to_string(),
    );
    println!("{:?}", tx);

    let mut tx_map: HashMap<String, Transaction> = HashMap::new();
    tx_map.insert("user_creation".to_string(), tx);
    tx_map.insert("credits_update".to_string(), tx2);
    println!("{:?}", tx_map);
}
