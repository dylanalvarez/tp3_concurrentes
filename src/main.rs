mod blockchain;
use crate::blockchain::Blockchain;

fn main() {
    let mut blockchain = Blockchain::new();
    blockchain.add_grade(String::from("Dylan"), 10.0);
    blockchain.add_grade(String::from("Gustavo"), 7.99);
    println!("is valid? {}", blockchain.is_valid());
}
