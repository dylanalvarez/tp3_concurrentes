use crate::blockchain::Blockchain;
use crate::local_address_with_port;
use std::mem::size_of;
use std::net::UdpSocket;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

struct DistMutex {
    port_to_coordinator: usize,
    socket_to_coordinator: UdpSocket,
}

impl DistMutex {
    fn new(port_to_coordinator: usize, port_receive_from_coordinator: usize) -> DistMutex {
        let socket_to_coordinator = match UdpSocket::bind(local_address_with_port(
            &port_receive_from_coordinator.to_string(),
        )) {
            Ok(socket) => socket,
            Err(_error) => {
                panic!("Couldn't start to listen on listen port. Port in use?");
            }
        };

        let new_dist_mutex = DistMutex {
            port_to_coordinator,
            socket_to_coordinator,
        };

        let cloned_new_dist_mutex = new_dist_mutex.clone();
        thread::spawn(move || cloned_new_dist_mutex.listen());

        new_dist_mutex
    }

    fn clone(&self) -> DistMutex {
        DistMutex {
            socket_to_coordinator: self.socket_to_coordinator.try_clone().unwrap(),
            port_to_coordinator: self.port_to_coordinator.clone(),
        }
    }

    fn acquire(&mut self) {
        self.socket_to_coordinator
            .send_to(
                "ACQUIRE".as_bytes(),
                local_address_with_port(&self.port_to_coordinator.to_string()),
            )
            .unwrap();

        // TODO condvar(acquiring) que bloquee esta funcion y deje retornar solo cuando se haya obtenido respuesta del ACQUIRE
    }

    fn release(&mut self) {
        self.socket_to_coordinator
            .send_to(
                "RELEASE".as_bytes(),
                local_address_with_port(&self.port_to_coordinator.to_string()),
            )
            .unwrap();
    }

    fn listen(&self) {
        loop {
            let mut buf = [0; size_of::<usize>() + 1];
            let (size, from) = self.socket_to_coordinator.recv_from(&mut buf).unwrap();
            println!("Received bytes {:?} on DistMutex from: {:?}", size, from);
            // TODO: if ACQUIRE_RESPONSE: liberar condvar(acquiring) para indicar que se tiene el lock
        }
    }
}

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
