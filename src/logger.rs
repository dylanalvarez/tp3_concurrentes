use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;

pub fn log(formatted_string: String) {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string();
    println!("[{}] [{:?}] {}", timestamp, thread::current().id(), formatted_string);
}
