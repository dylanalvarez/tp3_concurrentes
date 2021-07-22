mod blockchain;
mod blockchain_node;

use crate::blockchain::Blockchain;
use crate::blockchain_node::BlockchainNode;
use std::io::{stdin, stdout, Write};
use std::process::exit;
use std::{env, thread};

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Received args = {:?}", args);

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
    println!("neighbor_addresses = {:?}", neighbor_addresses);

    let listen_thread_handle =
        thread::spawn(move || start_node(&port.to_owned(), neighbor_addresses));
    listen_thread_handle.join();

    let mut blockchain = Blockchain::new();
    blockchain.add_grade(String::from("Dylan"), 10.0);
    blockchain.add_grade(String::from("Gustavo"), 7.99);
    println!("is valid? {}", blockchain.is_valid());
}

fn local_address_with_port(port: &String) -> String {
    "127.0.0.1:".to_owned() + port
}

fn start_node(port: &String, neighbor_addresses: Vec<String>) {
    let numeric_port = port.clone().parse::<usize>().unwrap();
    let node = BlockchainNode::new(numeric_port, neighbor_addresses.clone());
    loop {
        prompt_loop(node.clone());
    }
}

fn prompt_loop(node: BlockchainNode) {
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
    execute_command(command, node.clone());
}

fn execute_command(raw_command: String, node: BlockchainNode) {
    let parsed_command = raw_command.split(" ").collect::<Vec<&str>>();
    match parsed_command[0] {
        "add_grade" => {
            let student_name = parsed_command[1].to_string();
            match parsed_command[2].parse() {
                Ok(grade) => {
                    println!(
                        "Received add_grade command with params: {:?} {:?}",
                        student_name, grade
                    );
                    node.add_grade(student_name, grade);
                }
                Err(_error) => {
                    println!("Invalid grade number for add_grade command");
                }
            };
        }
        "print" => {
            println!("Received print command");
            node.print();
        }
        "quit" => {
            println!("Received quit command");
            exit(0);
        }
        "ping" => {
            println!("Received ping command");
            node.ping_neighbors();
        }
        "make_coordinator" => {
            println!("Received make_coordinator command");
            node.make_coordinator();
        }
        _ => {
            println!("Ups! Didn't understand that. Available commands: add_grade, print, quit, ping, make_coordinator");
        }
    }
}
