mod blockchain;

use std::{env, thread};
use crate::blockchain::Blockchain;
use std::net::{UdpSocket};
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Received args = {:?}", args);

    if args.len() - 1 < 2 {
        panic!(
            "Required args: port ip1:port1 ip2:port2. Try: cargo run 6060 127.0.0.1:6061 127.0.0.1:6062"
        );
    }

    let port = args[1].clone();
    let numeric_port = args[1].clone().parse::<i32>().unwrap();
    let listen_thread_handle = thread::spawn(move || { start_listen_udp(&port.to_owned()) });

    let neighbor_addresses: Vec<String> = args.into_iter()
        .enumerate()
        .filter_map(|(i, e)| if i > 1 { Some(e) } else { None })
        .collect();
    println!("neighbor_addresses = {:?}", neighbor_addresses);

    let mut neighbor_handles = vec!();
    let mut i = 1;
    for neighbor_addr in neighbor_addresses.iter() {
        let addr = neighbor_addr.clone();
        println!("Ping to neighbor with addr: {:?}", addr);
        let port_for_neighbor_response = numeric_port + i * 1000;
        neighbor_handles.push(thread::spawn(move || { ping_neighbor(addr, port_for_neighbor_response) }));
        i += 1;
    }

    neighbor_handles.into_iter().for_each(|h| { h.join(); });
    listen_thread_handle.join();

    let mut blockchain = Blockchain::new();
    blockchain.add_grade(String::from("Dylan"), 10.0);
    blockchain.add_grade(String::from("Gustavo"), 7.99);
    println!("is valid? {}", blockchain.is_valid());
}

fn local_address_with_port(port: &String) -> String { "127.0.0.1:".to_owned() + port }

fn start_listen_udp(port: &String) {
    match UdpSocket::bind(local_address_with_port(port)) {
        Ok(socket) => {
            println!("Starting to listen on port {:?}", port);
            loop {
                let mut buf = [0; 10];
                let (size, from) = socket.recv_from(&mut buf).unwrap();
                println!("Received bytes {:?} from: {:?}", size, from);
                socket.send_to("PONG".as_bytes(), from).unwrap();
            }
        }
        Err(_error) => {
            panic!(
                "Couldn't start to listen on assigned port. Port in use?"
            );
        }
    }
}

fn ping_neighbor(dest_addr: String, response_port: i32) {
    // TODO: reusar el socket de escucha para hacer el send_to a los vecinos
    let neighbor_response_addr = local_address_with_port(&response_port.to_string());
    println!("Addr for neighbor responses: {:?}", neighbor_response_addr);
    match UdpSocket::bind(neighbor_response_addr) {
        Ok(socket) => {
            thread::sleep(Duration::from_millis(10000));
            socket.send_to("PING".as_bytes(), dest_addr).unwrap();
            println!("Sent PING to neighbor");
            let mut buf = [0; 10];
            loop {
                println!("Starting to listen for neighbor response");
                socket.set_read_timeout(None);
                let (size, from) = socket.recv_from(&mut buf).unwrap();
                println!("Received bytes {:?} from neighbor: {:?}", size, from);
            }
        }
        Err(_error) => {
            panic!(
                "Couldn't start to listen on listen port. Port in use?"
            );
        }
    }
}