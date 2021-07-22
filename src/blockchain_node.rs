use crate::blockchain::Blockchain;
use crate::local_address_with_port;
use std::mem::size_of;
use std::net::UdpSocket;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

pub struct BlockchainNode {
    port: usize,
    socket: UdpSocket,
    leader_port: Arc<(Mutex<Option<usize>>, Condvar)>,
    neighbor_addresses: Vec<String>,
    blockchain: Blockchain,
}

impl BlockchainNode {
    pub(crate) fn new(port: usize, neighbor_addresses: Vec<String>) -> BlockchainNode {
        let self_addr = local_address_with_port(&port.to_string());
        println!("Node address for neighbor messages: {:?}", self_addr);
        let socket = match UdpSocket::bind(self_addr) {
            Ok(socket) => socket,
            Err(_error) => {
                panic!("Couldn't start to listen on listen port. Port in use?");
            }
        };

        let new_node = BlockchainNode {
            port,
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

    pub fn clone(&self) -> BlockchainNode {
        BlockchainNode {
            port: self.port,
            socket: self.socket.try_clone().unwrap(),
            leader_port: self.leader_port.clone(),
            neighbor_addresses: self.neighbor_addresses.clone(),
            blockchain: self.blockchain.clone(),
        }
    }

    pub fn listen(&self) {
        loop {
            let mut buf = [0; size_of::<usize>() + 1];
            let (size, from) = self.socket.recv_from(&mut buf).unwrap();
            println!("Received bytes {:?} from neighbor: {:?}", size, from);
        }
    }

    pub fn ping_neighbors(&self) {
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

    pub fn ping_neighbor(&self, dest_addr: String) {
        println!("Sending ping to neighbor with addr: {:?}", dest_addr);
        self.socket.send_to("PING".as_bytes(), dest_addr).unwrap();
    }

    pub fn make_coordinator(&self) {
        println!("Node received make_coordinator");
        match (*self).leader_port.0.lock() {
            Ok(mut leader_port) => {
                *leader_port = Option::from((*self).port);
            }
            Err(error) => {
                panic!("{}", error.to_string())
            }
        }
        println!("New coordinator: {:?}", self.leader_port);
    }

    pub fn add_grade(&self, _name: String, _note: f64) {
        println!("Node received add_grade");
        // TODO
    }

    pub fn print(&self) {
        println!("Print current blockchain");
        if self.blockchain.is_valid() {
            // self.blockchain.print();
        }
    }
}
