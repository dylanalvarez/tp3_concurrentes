mod blockchain;

use crate::blockchain::Blockchain;
use std::io::{stdin, stdout, Write};
use std::mem::size_of;
use std::net::UdpSocket;
use std::process::exit;
use std::sync::{Arc, Condvar, Mutex};
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

struct Node {
    id: usize,
    socket: UdpSocket,
    leader_port: Arc<(Mutex<Option<usize>>, Condvar)>,
    neighbor_addresses: Vec<String>,
    blockchain: Blockchain,
}

impl Node {
    fn new(port: usize, neighbor_addresses: Vec<String>) -> Node {
        let self_addr = local_address_with_port(&port.to_string());
        println!("Node address for neighbor messages: {:?}", self_addr);
        let socket = match UdpSocket::bind(self_addr) {
            Ok(socket) => socket,
            Err(_error) => {
                panic!("Couldn't start to listen on listen port. Port in use?");
            }
        };

        let new_node = Node {
            id: port,
            socket,
            leader_port: Arc::new((Mutex::new(Some(port)), Condvar::new())),
            neighbor_addresses,
            blockchain: Blockchain::new(),
        };

        println!("Starting to listen on port: {:?}", port);
        let clone = new_node.clone();
        thread::spawn(move || clone.listen());

        println!(
            "Starting to ping all neighbors: {:?}",
            new_node.neighbor_addresses
        );
        new_node.clone().ping_neighbors();

        // TODO: start leader election
        // new_node.find_new();
        new_node
    }

    fn clone(&self) -> Node {
        Node {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            leader_port: self.leader_port.clone(),
            neighbor_addresses: self.neighbor_addresses.clone(),
            blockchain: self.blockchain.clone(),
        }
    }

    fn listen(&self) {
        loop {
            let mut buf = [0; size_of::<usize>() + 1];
            let (size, from) = self.socket.recv_from(&mut buf).unwrap();
            println!("Received bytes {:?} from neighbor: {:?}", size, from);
        }
    }

    fn ping_neighbors(&self) {
        let mut neighbor_handles = vec![];
        for neighbor_addr in self.neighbor_addresses.iter() {
            let addr = neighbor_addr.clone();
            let me = self.clone();
            neighbor_handles.push(thread::spawn(move || me.ping_neighbor(addr)));
        }
        neighbor_handles.into_iter().for_each(|h| {
            h.join();
        });
    }

    fn ping_neighbor(&self, dest_addr: String) {
        println!("Sending ping to neighbor with addr: {:?}", dest_addr);
        self.socket.send_to("PING".as_bytes(), dest_addr).unwrap();
    }

    fn add_grade(&self, _name: String, _note: f64) {
        println!("Node received add_grade");
        // TODO
    }

    fn print(&self) {
        println!("Print current blockchain");
        if self.blockchain.is_valid() {
            // self.blockchain.print();
        }
    }
}

fn local_address_with_port(port: &String) -> String {
    "127.0.0.1:".to_owned() + port
}

fn start_node(port: &String, neighbor_addresses: Vec<String>) {
    let numeric_port = port.clone().parse::<usize>().unwrap();
    let node = Node::new(numeric_port, neighbor_addresses.clone());
    loop {
        prompt_loop(node.clone());
    }
}

fn prompt_loop(node: Node) {
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

fn execute_command(raw_command: String, node: Node) {
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
        "ping" => {
            println!("Received ping command");
            node.ping_neighbors();
        }
        "quit" => {
            println!("Received quit command");
            exit(0);
        }
        _ => {
            println!("Ups! Didn't understand that. Available commands: quit, add_grade, print");
        }
    }
}
