use std::{thread, u8, usize};
use std::alloc::System;
use std::collections::VecDeque;
use std::mem::size_of;
use std::net::UdpSocket;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::acquire_message::AcquireMessage;
use crate::blockchain::Blockchain;
use crate::election_message::ElectionMessage;
use crate::ip_parser;
use crate::logger::log;

struct DistMutex {
    coordinator_addr: String,
    self_addr: String,
    socket_to_coordinator: UdpSocket,
    lock_taken: Arc<(Mutex<bool>)>,
    lock_owner_addr: Arc<(Mutex<String>)>,
    got_acquire_confirmation: Arc<(Mutex<bool>, Condvar)>,
    got_release_confirmation: Arc<(Mutex<bool>, Condvar)>,
    pending_locks: VecDeque<String>,
}

impl DistMutex {
    fn new(
        coordinator_addr: String,
        self_addr: String,
        socket_to_coordinator: UdpSocket,
    ) -> DistMutex {
        let lock_taken = Arc::new(Mutex::new(false));
        let lock_owner_addr = Arc::new(Mutex::new(String::new()));
        let got_acquire_confirmation = Arc::new((Mutex::new(false), Condvar::new()));
        let got_release_confirmation = Arc::new((Mutex::new(false), Condvar::new()));
        let pending_locks = VecDeque::new();

        let new_dist_mutex = DistMutex {
            coordinator_addr,
            self_addr,
            socket_to_coordinator,
            lock_taken,
            lock_owner_addr,
            got_acquire_confirmation,
            got_release_confirmation,
            pending_locks,
        };

        new_dist_mutex
    }

    fn clone(&self) -> DistMutex {
        DistMutex {
            coordinator_addr: self.coordinator_addr.clone(),
            self_addr: self.self_addr.clone(),
            socket_to_coordinator: self.socket_to_coordinator.try_clone().unwrap(),
            lock_taken: self.lock_taken.clone(),
            lock_owner_addr: self.lock_owner_addr.clone(),
            got_acquire_confirmation: self.got_acquire_confirmation.clone(),
            got_release_confirmation: self.got_release_confirmation.clone(),
            pending_locks: self.pending_locks.clone(),
        }
    }

    fn acquire(&mut self) {
        if *self.lock_taken.lock().unwrap() {
            // Lock already taken
            return;
        }
        // Lock not taken
        log(format!(
            "Sending ACQUIRE to coordinator: {:?}",
            self.coordinator_addr
        ));
        self.socket_to_coordinator
            .send_to("ACQUIRE".as_bytes(), &self.coordinator_addr)
            .unwrap();

        log(format!("Waiting for OK_ACQUIRE message"));
        const OK_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);
        let got_acquire_confirmation = self.got_acquire_confirmation.1.wait_timeout_while(
            self.got_acquire_confirmation.0.lock().unwrap(),
            OK_ACQUIRE_TIMEOUT,
            |dont_got_it| !*dont_got_it,
        );
        if !*got_acquire_confirmation.unwrap().0 {
            log(format!("Timeout waiting for OK_ACQUIRE message"));
            // TODO: retornar error, disparar leader_election y reintentar
        } else {
            log(format!("Got OK_ACQUIRE message"));
            *self.lock_taken.lock().unwrap() = true;
        }
    }

    fn release(&mut self) {
        if self.is_coordinator((*self.self_addr).to_string()) {
            if *self.lock_taken.lock().unwrap() {
                if *self.lock_owner_addr.lock().unwrap() == *self.self_addr {
                    log(format!(
                        "Sending RELEASE to coordinator with addr: {:?}",
                        self.coordinator_addr
                    ));
                    self.socket_to_coordinator
                        .send_to(
                            "RELEASE".as_bytes(),
                            ip_parser::local_address_with_port(&self.coordinator_addr),
                        )
                        .unwrap();
                    *self.lock_taken.lock().unwrap() = false;
                    *self.got_acquire_confirmation.0.lock().unwrap() = false;
                }
            }
        }
    }

    fn is_coordinator(&self, addr: String) -> bool {
        return addr == *self.coordinator_addr;
    }

    fn is_taken(&self) -> bool {
        return *self.lock_taken.lock().unwrap();
    }

    fn enqueue_requestor(&mut self, sender_addr: String) {
        self.pending_locks.push_back(sender_addr);
    }

    fn deque_requestor(&mut self) -> Option<String> {
        self.pending_locks.pop_front()
    }

    fn set_taken(&self, taken: bool) {
        *self.lock_taken.lock().unwrap() = taken;
    }

    fn set_lock_owner_addr(&self, lock_owner_addr: String) {
        *self.lock_owner_addr.lock().unwrap() = lock_owner_addr;
    }
}

pub struct BlockchainNode {
    port: usize,
    socket: UdpSocket,
    leader_port: Arc<(Mutex<Option<usize>>, Condvar)>,
    neighbor_addresses: Vec<String>,
    blockchain: Blockchain,
    got_ok: Arc<(Mutex<bool>, Condvar)>,
    is_in_election: Arc<(Mutex<bool>, Condvar)>,
    dist_mutex: DistMutex,
}

impl BlockchainNode {
    pub(crate) fn new(port: usize, neighbor_addresses: Vec<String>) -> BlockchainNode {
        let self_addr = ip_parser::local_address_with_port(&port.to_string());
        let cloned_self_addr = self_addr.clone();
        log(format!("Node address for neighbor messages: {:?}", self_addr));
        let socket = match UdpSocket::bind(self_addr) {
            Ok(socket) => socket,
            Err(_error) => {
                panic!("Couldn't start to listen on listen port. Port in use?");
            }
        };

        // try_clone() devuelve referencia independiente apuntando al mismo socket
        let cloned_socket = match socket.try_clone() {
            Ok(socket) => socket,
            Err(error) => {
                panic!(
                    "Error trying to clone socket when creating BlockchainNode. Error: {:?}",
                    error.to_string()
                )
            }
        };
        // TODO: por ahora se pone al propio puerto como coordinador, luego se actualizara con la election
        let dist_mutex = DistMutex::new(
            cloned_self_addr,
            ip_parser::local_address_with_port(&port.to_string()),
            cloned_socket,
        );

        BlockchainNode {
            port,
            socket,
            leader_port: Arc::new((Mutex::new(Some(port)), Condvar::new())),
            neighbor_addresses,
            blockchain: Blockchain::new(),
            got_ok: Arc::new((Mutex::new(false), Condvar::new())),
            is_in_election: Arc::new((Mutex::new(false), Condvar::new())),
            dist_mutex,
        }
    }

    pub fn handle_incoming_message(arc_mutex_self: Arc<Mutex<BlockchainNode>>, message: &str, sender: &str) -> () {
        let cloned_arc_mutex_self = arc_mutex_self.clone();
        match ElectionMessage::from_bytes(message.as_bytes()) {
            Some(election_message) => BlockchainNode::process_election_message(cloned_arc_mutex_self, election_message, sender),
            None => {
                match arc_mutex_self.lock() {
                    Ok(mut _self) => {
                        _self.process_dist_mutex_message(message, sender)
                    }
                    Err(error) => { panic!(error.to_string()) }
                }
            }
        }
    }

    fn process_election_message(arc_mutex_self: Arc<Mutex<BlockchainNode>>, election_message: ElectionMessage, sender: &str) -> () {
        match election_message {
            ElectionMessage::Election => {
                let (self_port, socket) = {
                    let _self = arc_mutex_self.lock().unwrap();
                    (_self.port, _self.socket.try_clone().unwrap())
                };
                log(format!(
                    "Quieren hacer elecciones desde {:?} y yo soy {:?}!",
                    sender, self_port
                ));
                if let Some(port) = ip_parser::get_port_from_dir(sender) {
                    if self_port > port {
                        let message_to_send = ElectionMessage::OkElection.as_bytes();
                        socket.send_to(&message_to_send, sender).unwrap();
                        let __self = arc_mutex_self.clone();
                        thread::spawn(move || {
                            BlockchainNode::begin_election(__self);
                        });
                    }
                }
            }
            ElectionMessage::Coordinator => {
                let _self = arc_mutex_self.lock().unwrap();
                *_self.leader_port.0.lock().unwrap() =
                    Some(ip_parser::get_port_from_dir(sender).unwrap());
                *_self.is_in_election.0.lock().unwrap() = false;
                _self.is_in_election.1.notify_all();
                log(format!(
                    "Mi nuevo coordinador es {:?}",
                    *_self.leader_port.0.lock().unwrap()
                ));
            }
            ElectionMessage::OkElection => {
                log(format!("Recibi OkElection. No seré el coordinador."));
                let _self = arc_mutex_self.lock().unwrap();
                *_self.got_ok.0.lock().unwrap() = true;
                _self.got_ok.1.notify_all();
            }
        }
    }

    fn process_dist_mutex_message(&mut self, message: &str, sender: &str) -> () {
        let acquire_message = AcquireMessage::from_bytes(message.as_bytes());
        match acquire_message {
            Some(AcquireMessage::Acquire) => {
                self.process_acquire_message(sender);
            }
            Some(AcquireMessage::OkAcquire) => {
                self.process_ok_acquire_message(sender);
            }
            Some(AcquireMessage::Release) => {
                self.process_release_message(sender);
            }
            None => {}
        }
    }

    fn process_acquire_message(&mut self, sender: &str) {
        log(format!("Processing ACQUIRE message"));
        if self
            .dist_mutex
            .is_coordinator(ip_parser::local_address_with_port(&self.port.to_string()))
        {
            if self.dist_mutex.is_taken() {
                self.dist_mutex.enqueue_requestor(sender.to_string());
            } else {
                self.dist_mutex.set_taken(true);
                self.dist_mutex.set_lock_owner_addr(sender.to_string());
                let ok_acquire_message = AcquireMessage::OkAcquire.as_bytes();
                self.socket.send_to(&ok_acquire_message, sender).unwrap();

                const OK_RELEASE_TIMEOUT: Duration = Duration::from_secs(10);
                let got_release_confirmation = self
                    .dist_mutex
                    .got_release_confirmation
                    .1
                    .wait_timeout_while(
                        self.dist_mutex.got_release_confirmation.0.lock().unwrap(),
                        OK_RELEASE_TIMEOUT,
                        |dont_got_it| !*dont_got_it,
                    );
                self.dist_mutex.set_lock_owner_addr(String::new());
                self.dist_mutex.set_taken(false);
                if !*got_release_confirmation.unwrap().0 {
                    log(format!("Timeout waiting for RELEASE message"));
                    let enqueded_requestor = self.dist_mutex.deque_requestor();
                    self.process_acquire_message(enqueded_requestor.unwrap().as_str());
                } else {
                    log(format!("Successfully received RELEASE message"));
                    *self.dist_mutex.got_release_confirmation.0.lock().unwrap() = false;
                }
            }
        } else {
            log(format!("Non-coordinator received ACQUIRE message"))
        }
    }

    fn process_ok_acquire_message(&mut self, sender: &str) {
        if !self
            .dist_mutex
            .is_coordinator(ip_parser::local_address_with_port(&self.port.to_string()))
        {
            // No soy el coordinador
            *self.dist_mutex.got_acquire_confirmation.0.lock().unwrap() = true;
            self.dist_mutex.got_acquire_confirmation.1.notify_all();
        }
    }

    fn process_release_message(&mut self, sender: &str) {
        log(format!("Processing RELEASE message"));
        if self
            .dist_mutex
            .is_coordinator(ip_parser::local_address_with_port(&self.port.to_string()))
        {
            // Soy el coordinador
            if self.dist_mutex.is_taken() {
                // TODO: enviar NUEVA_NOTA a todos?
                self.dist_mutex.set_taken(false);
                self.dist_mutex.set_lock_owner_addr(String::new());
                *self.dist_mutex.got_release_confirmation.0.lock().unwrap() = true;
                self.dist_mutex.got_release_confirmation.1.notify_all();
                while !self.dist_mutex.pending_locks.is_empty() {
                    let enqueded_requestor = self.dist_mutex.deque_requestor();
                    log(format!(
                        "Dequeued pending requestor with addr: {:?}",
                        enqueded_requestor
                    ));
                    self.process_acquire_message(enqueded_requestor.unwrap().as_str());
                }
            }
        }
    }

    pub fn listen(arc_mutex_self: Arc<Mutex<BlockchainNode>>) {
        let port = {
            arc_mutex_self.lock().unwrap().port
        };
        log(format!("Starting to listen on port: {:?}", port));
        loop {
            let mut buf = [0; size_of::<usize>() + 1];
            let socket = {
                arc_mutex_self.lock().unwrap().socket.try_clone()
            };
            match socket.unwrap().recv_from(&mut buf) {
                Ok((size, from)) => {
                    log(format!("Received bytes {:?} from neighbor: {:?}", size, from));
                    let received = Vec::from(&buf[0..size]);
                    let str_received = String::from_utf8(received).unwrap();
                    let neighbor = from.to_string();
                    BlockchainNode::handle_incoming_message(arc_mutex_self.clone(), &str_received, &neighbor);
                }
                Err(error) => print!("Error while listening on port: {:?}", error),
            }
        }
    }

    pub fn ping_neighbors(&self) {
        for neighbor_addr in self.neighbor_addresses.iter() {
            self.ping_neighbor(neighbor_addr.clone());
        }
    }

    pub fn ping_neighbor(&self, dest_addr: String) {
        log(format!("Sending ping to neighbor with addr: {:?}", dest_addr));
        self.socket.send_to("PING".as_bytes(), dest_addr).unwrap();
    }

    pub fn make_coordinator(&self) {
        log(format!("Node received make_coordinator"));
        match (*self).leader_port.0.lock() {
            Ok(mut leader_port) => {
                *leader_port = Option::from((*self).port);
            }
            Err(error) => {
                panic!("{}", error.to_string())
            }
        }
        log(format!("New coordinator: {:?}", self.leader_port));
    }

    pub fn add_grade(&self, _name: String, _note: f64) {
        log(format!("Node received add_grade"));
        // TODO
    }

    pub fn print(&self) {
        log(format!("Print current blockchain"));
        if self.blockchain.is_valid() {
            // self.blockchain.print();
        }
    }

    /// Comienza el proceso de eleccion de lider.
    /// Al finalizar, el nodo con número de puerto mas grande es quien queda como coordinador.
    pub fn begin_election(arc_mutex_self: Arc<Mutex<BlockchainNode>>) {
        match arc_mutex_self.lock() {
            Ok(_self) => {
                if *_self.is_in_election.0.lock().unwrap() {
                    return;
                }

                *_self.got_ok.0.lock().unwrap() = false;
                *_self.is_in_election.0.lock().unwrap() = true;

                for neighbor in &_self.neighbor_addresses {
                    match ip_parser::get_port_from_dir(neighbor) {
                        Some(port) => {
                            if port < _self.port {
                                continue;
                            }
                            log(format!("\t\tSending ELECTION to {:?}", neighbor));
                            let message_to_send = ElectionMessage::Election.as_bytes();
                            _self.socket.send_to(&message_to_send, neighbor).unwrap();
                        }

                        None => {
                            panic!("There is an intruder!")
                        }
                    }
                }
            }
            Err(error) => { panic!(error.to_string()) }
        }

        log(format!("Enviando mensaje ELECTION a vecinos. Esperando sus respuestas..."));
        const TIMEOUT: Duration = Duration::from_secs(3);
        let _got_ok = {
            arc_mutex_self.lock().unwrap().got_ok.clone()
        };
        let got_ok =
            _got_ok
                .1
                .wait_timeout_while(_got_ok.0.lock().unwrap(), TIMEOUT, |got_it| !*got_it);
        if !*got_ok.unwrap().0 {
            match arc_mutex_self.lock() {
                Ok(_self) => {
                    _self.make_leader();
                    *_self.is_in_election.0.lock().unwrap() = false;
                }
                Err(error) => { panic!(error.to_string()) }
            }
        } else {
            let _is_in_election = {
                arc_mutex_self.lock().unwrap().is_in_election.clone()
            };
            let _ = _is_in_election
                .1
                .wait_while(_is_in_election.0.lock().unwrap(), |is_in_election| {
                    *is_in_election
                });
        }
    }

    fn make_leader(&self) {
        *self.leader_port.0.lock().unwrap() = Some(self.port);
        log(format!("{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().to_string()));
        log(format!(
            "Soy el nuevo coordinador! Puerto {:?}",
            *self.leader_port.0.lock().unwrap()
        ));
        for neighbor in &self.neighbor_addresses {
            log(format!("\t\tEnviando mensaje COORDINATOR a {:?}", neighbor));
            let message_to_send = ElectionMessage::Coordinator.as_bytes();
            self.socket.send_to(&message_to_send, neighbor).unwrap();
        }
    }
}
