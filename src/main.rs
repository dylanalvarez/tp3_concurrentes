use std::io::{stdin, stdout, Write};
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::{env, thread};

use crate::blockchain_node::BlockchainNode;
use crate::logger::log;

mod acquire_message;
mod add_grade_message;
mod blockchain;
mod blockchain_message;
mod blockchain_node;
mod coordinator_state;
mod dist_mutex;
mod election_message;
mod ip_parser;
mod logger;
mod sender;

pub const BUFFER_SIZE: usize = 2;

#[allow(clippy::mutex_atomic)]
fn main() {
    let args: Vec<String> = env::args().collect();
    log(format!("Received args = {:?}", args));

    if args.len() - 1 < 2 {
        panic!(
            "Required args: port ip1:port1 ip2:port2. Try: cargo run 6060 127.0.0.1:6061 127.0.0.1:6062"
        );
    }

    let port = args[1].clone();
    let neighbor_addresses: Vec<String> = args
        .into_iter()
        .enumerate()
        .filter_map(|(i, e)| if i > 1 { Some(e) } else { None })
        .collect();
    log(format!("neighbor_addresses = {:?}", neighbor_addresses));

    start_node(&port, neighbor_addresses);
}

fn start_node(port: &str, neighbor_addresses: Vec<String>) {
    let numeric_port = port.parse::<usize>().unwrap();
    let node = Arc::new(Mutex::new(BlockchainNode::new(
        numeric_port,
        neighbor_addresses,
    )));
    let cloned_node = node.clone();

    thread::spawn(move || {
        BlockchainNode::listen(cloned_node);
    });

    BlockchainNode::ask_for_blockchain(node.clone());

    BlockchainNode::begin_election(node.clone());
    loop {
        prompt_loop(node.clone());
    }
}

fn prompt_loop(node: Arc<Mutex<BlockchainNode>>) {
    let mut command = String::new();
    print!("Enter command: ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut command)
        .expect("Ups! Didn't understand that :(");
    if let Some('\n') = command.chars().next_back() {
        command.pop();
    }
    if let Some('\r') = command.chars().next_back() {
        command.pop();
    }
    execute_command(command, node);
}

fn execute_command(raw_command: String, node: Arc<Mutex<BlockchainNode>>) {
    let parsed_command = raw_command.split(' ').collect::<Vec<&str>>();
    match parsed_command[0] {
        "add_grade" => {
            if parsed_command.len() != 3 {
                println!("Invalid command. add_grade <student name (without spaces)> <student grade (with dot notation. eg: 9.54)>");
                return;
            }
            let student_name = parsed_command[1].to_string();
            match parsed_command[2].parse() {
                Ok(grade) => {
                    log(format!(
                        "Received add_grade command with params: {:?} {:?}",
                        student_name, grade
                    ));
                    let _ = BlockchainNode::add_grade(node, student_name, grade);
                }
                Err(_error) => {
                    log("Invalid grade number for add_grade command".to_string());
                }
            };
        }
        "print" => {
            log("Received print command".to_string());
            match node.lock() {
                Ok(node) => node.print(),
                Err(error) => {
                    panic!("{}", error.to_string())
                }
            }
        }
        "quit" => {
            log("Received quit command".to_string());
            exit(0);
        }
        "ping" => {
            log("Received ping command".to_string());
            match node.lock() {
                Ok(node) => node.ping_neighbors(),
                Err(error) => {
                    panic!("{}", error.to_string())
                }
            }
        }
        "make_coordinator" => {
            log("Received make_coordinator command".to_string());
            match node.lock() {
                Ok(node) => node.make_coordinator(),
                Err(error) => {
                    panic!("{}", error.to_string())
                }
            }
        }

        "begin_election" => {
            log("Received begin_election command".to_string());
            BlockchainNode::begin_election(node);
        }

        "clear" => {
            print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        }
        _ => {
            log("Ups! Didn't understand that. Available commands: add_grade, print, quit, ping, make_coordinator, begin_election, clear".to_string());
        }
    }
}
