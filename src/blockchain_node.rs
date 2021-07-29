use std::collections::HashMap;
use std::net::UdpSocket;
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
use std::{thread, usize};

use crate::acquire_message::AcquireMessage;
use crate::add_grade_message::AddGradeMessage;
use crate::blockchain::Blockchain;
use crate::blockchain_message::BlockchainMessage;
use crate::coordinator_state::CoordinatorState;
use crate::dist_mutex::DistMutex;
use crate::election_message::ElectionMessage;
use crate::ip_parser;
use crate::logger::log;

pub struct BlockchainNode {
    port: usize,
    socket: UdpSocket,
    leader_port: Arc<Mutex<Option<usize>>>,
    neighbor_addresses: Vec<String>,
    blockchain: Blockchain,
    got_ok: Arc<(Mutex<bool>, Condvar)>,
    is_in_election: Arc<(Mutex<bool>, Condvar)>,
    synchronization_done: Arc<(Mutex<bool>, Condvar)>,
    pub dist_mutex: DistMutex,
    pub coordinator_state: CoordinatorState,
}

impl BlockchainNode {
    #[allow(clippy::mutex_atomic)]
    pub(crate) fn new(port: usize, neighbor_addresses: Vec<String>) -> BlockchainNode {
        let self_addr = ip_parser::local_address_with_port(&port.to_string());
        let cloned_self_addr = self_addr.clone();
        log(format!(
            "Node address for neighbor messages: {:?}",
            self_addr
        ));
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

        let dist_mutex = DistMutex::new(cloned_self_addr, cloned_socket.try_clone().unwrap());
        let coordinator_state = CoordinatorState::new();

        BlockchainNode {
            port,
            socket,
            leader_port: Arc::new(Mutex::new(Some(port))),
            neighbor_addresses,
            blockchain: Blockchain::new(),
            got_ok: Arc::new((Mutex::new(false), Condvar::new())),
            is_in_election: Arc::new((Mutex::new(false), Condvar::new())),
            synchronization_done: Arc::new((Mutex::new(false), Condvar::new())),
            dist_mutex,
            coordinator_state,
        }
    }

    pub fn handle_incoming_message(
        arc_mutex_self: Arc<Mutex<BlockchainNode>>,
        message: &str,
        sender: &str,
    ) {
        let cloned_arc_mutex_self = arc_mutex_self.clone();
        if let Some(election_message) = ElectionMessage::from_bytes(message.as_bytes()) {
            return BlockchainNode::process_election_message(
                cloned_arc_mutex_self,
                election_message,
                sender,
            );
        }
        if let Some(acquire_message) = AcquireMessage::from_bytes(message.as_bytes()) {
            BlockchainNode::process_dist_mutex_message(arc_mutex_self, acquire_message, sender);
            return;
        }
        if let Some(add_grade_message) = AddGradeMessage::from_string(String::from(message)) {
            return BlockchainNode::process_add_grade_message(arc_mutex_self, add_grade_message);
        }
        if let Some(blockchain_message) = BlockchainMessage::from_string(String::from(message)) {
            return BlockchainNode::process_blockchain_message(
                arc_mutex_self,
                blockchain_message,
                sender,
            );
        }
        panic!("Unknown message: {}", message)
    }

    fn process_add_grade_message(
        arc_mutex_self: Arc<Mutex<BlockchainNode>>,
        add_grade_message: AddGradeMessage,
    ) {
        match add_grade_message {
            AddGradeMessage::FromCoordinator(blockchain_record) => {
                log(format!(
                    "Received add grade message from coordinator: {} {} {}",
                    blockchain_record.student_name, blockchain_record.grade, blockchain_record.hash
                ));
                arc_mutex_self
                    .lock()
                    .unwrap()
                    .blockchain
                    .add_record(blockchain_record.clone());
                log(format!(
                    "Processed add grade message from coordinator: {} {} {}",
                    blockchain_record.student_name, blockchain_record.grade, blockchain_record.hash
                ))
            }
            AddGradeMessage::ToCoordinator(student_name, grade) => {
                let mut _self = arc_mutex_self.lock().unwrap();
                _self.blockchain.add_grade(student_name.clone(), grade);
                for neighbor_addr in _self.neighbor_addresses.iter() {
                    let _ = _self.socket.send_to(
                        AddGradeMessage::FromCoordinator(
                            _self.blockchain.last_record().unwrap().clone(),
                        )
                        .as_string()
                        .as_bytes(),
                        neighbor_addr,
                    );
                }
                log(format!(
                    "Received add grade message to coordinator: {} {}",
                    student_name, grade
                ));
            }
        }
    }

    #[allow(clippy::mutex_atomic)]
    fn process_election_message(
        arc_mutex_self: Arc<Mutex<BlockchainNode>>,
        election_message: ElectionMessage,
        sender: &str,
    ) {
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
                        thread::spawn(move || {
                            BlockchainNode::begin_election(arc_mutex_self);
                        });
                    }
                }
            }
            ElectionMessage::Coordinator => {
                let mut _self = arc_mutex_self.lock().unwrap();
                *_self.leader_port.lock().unwrap() =
                    Some(ip_parser::get_port_from_dir(sender).unwrap());
                *_self.is_in_election.0.lock().unwrap() = false;
                _self.is_in_election.1.notify_all();
                _self.dist_mutex.coordinator_addr = sender.to_string();
                log(format!(
                    "Mi nuevo coordinador es {:?}",
                    *_self.leader_port.lock().unwrap()
                ));
            }
            ElectionMessage::OkElection => {
                log("Recibi OkElection. No seré el coordinador.".to_string());
                let _self = arc_mutex_self.lock().unwrap();
                *_self.got_ok.0.lock().unwrap() = true;
                _self.got_ok.1.notify_all();
            }
        }
    }

    fn process_dist_mutex_message(
        arc_mutex_self: Arc<Mutex<BlockchainNode>>,
        message: AcquireMessage,
        sender: &str,
    ) {
        match message {
            AcquireMessage::Acquire => {
                BlockchainNode::process_acquire_message(arc_mutex_self, sender);
            }
            AcquireMessage::OkAcquire => {
                BlockchainNode::process_ok_acquire_message(arc_mutex_self);
            }
            AcquireMessage::Release => {
                BlockchainNode::process_release_message(arc_mutex_self);
            }
        }
    }

    fn process_blockchain_message(
        arc_mutex_self: Arc<Mutex<BlockchainNode>>,
        blockchain_message: BlockchainMessage,
        sender: &str,
    ) {
        match blockchain_message {
            BlockchainMessage::AskForBlockchain => {
                BlockchainNode::process_ask_for_blockchain_message(arc_mutex_self, sender);
            }
            BlockchainMessage::BlockchainResult(blockchain) => {
                BlockchainNode::process_blockchain_result_message(
                    arc_mutex_self,
                    sender,
                    blockchain,
                );
            }
        }
    }

    #[allow(clippy::mutex_atomic)]
    fn process_acquire_message(arc_mutex_self: Arc<Mutex<BlockchainNode>>, sender: &str) {
        log("Processing ACQUIRE message".to_string());
        let (is_coordinator, is_taken, socket) = {
            let _self = arc_mutex_self.lock().unwrap();
            (
                _self
                    .dist_mutex
                    .is_coordinator(ip_parser::local_address_with_port(&_self.port.to_string())),
                _self.coordinator_state.is_taken(),
                _self.socket.try_clone(),
            )
        };

        if is_coordinator {
            if is_taken {
                arc_mutex_self
                    .lock()
                    .unwrap()
                    .coordinator_state
                    .enqueue_requestor(sender.to_string());
            } else {
                {
                    let _self = arc_mutex_self.lock().unwrap();
                    _self.coordinator_state.set_taken(true);
                    _self
                        .coordinator_state
                        .set_lock_owner_addr(sender.to_string());
                }
                let ok_acquire_message = AcquireMessage::OkAcquire.as_bytes();
                socket
                    .unwrap()
                    .send_to(&ok_acquire_message, sender)
                    .unwrap();
                log(String::from("Sent OK_ACQUIRE"));

                const OK_RELEASE_TIMEOUT: Duration = Duration::from_secs(10);
                let _got_release_confirmation = {
                    arc_mutex_self
                        .lock()
                        .unwrap()
                        .coordinator_state
                        .got_release_confirmation
                        .clone()
                };
                let got_release_confirmation = _got_release_confirmation.1.wait_timeout_while(
                    _got_release_confirmation.0.lock().unwrap(),
                    OK_RELEASE_TIMEOUT,
                    |dont_got_it| !*dont_got_it,
                );
                {
                    let _self = arc_mutex_self.lock().unwrap();
                    _self.coordinator_state.set_lock_owner_addr(String::new());
                    _self.coordinator_state.set_taken(false);
                }
                if !*got_release_confirmation.unwrap().0 {
                    log("Timeout waiting for RELEASE message".to_string());
                    let requestor = {
                        arc_mutex_self
                            .lock()
                            .unwrap()
                            .coordinator_state
                            .deque_requestor()
                    };
                    match requestor {
                        None => {}
                        Some(requestor) => {
                            BlockchainNode::process_acquire_message(
                                arc_mutex_self,
                                requestor.as_str(),
                            );
                        }
                    }
                } else {
                    log("Successfully received RELEASE message".to_string());
                    *arc_mutex_self
                        .lock()
                        .unwrap()
                        .coordinator_state
                        .got_release_confirmation
                        .0
                        .lock()
                        .unwrap() = false;
                }
            }
        } else {
            log("Non-coordinator received ACQUIRE message".to_string())
        }
    }

    #[allow(clippy::mutex_atomic)]
    fn process_ok_acquire_message(arc_mutex_self: Arc<Mutex<BlockchainNode>>) {
        let _self = arc_mutex_self.lock().unwrap();
        *_self.dist_mutex.got_acquire_confirmation.0.lock().unwrap() = true;
        _self.dist_mutex.got_acquire_confirmation.1.notify_all();
    }

    #[allow(clippy::mutex_atomic)]
    fn process_release_message(arc_mutex_self: Arc<Mutex<BlockchainNode>>) {
        log("Processing RELEASE message".to_string());

        match arc_mutex_self.lock() {
            Ok(_self) => {
                if !_self
                    .dist_mutex
                    .is_coordinator(ip_parser::local_address_with_port(&_self.port.to_string()))
                {
                    return;
                }

                if !_self.coordinator_state.is_taken() {
                    return;
                }

                _self.coordinator_state.set_taken(false);
                _self.coordinator_state.set_lock_owner_addr(String::new());
                {
                    *_self
                        .coordinator_state
                        .got_release_confirmation
                        .0
                        .lock()
                        .unwrap() = true;
                }
                _self
                    .coordinator_state
                    .got_release_confirmation
                    .1
                    .notify_all();
            }
            Err(error) => {
                panic!("{}", error.to_string())
            }
        }

        loop {
            let enqueded_requestor = {
                let mut _self = arc_mutex_self.lock().unwrap();
                if _self.coordinator_state.waiting_nodes_queue.is_empty() {
                    break;
                }
                _self.coordinator_state.deque_requestor()
            };
            log(format!(
                "Dequeued pending requestor with addr: {:?}",
                enqueded_requestor
            ));
            BlockchainNode::process_acquire_message(
                arc_mutex_self.clone(),
                enqueded_requestor.unwrap().as_str(),
            );
        }
    }

    fn process_ask_for_blockchain_message(
        arc_mutex_self: Arc<Mutex<BlockchainNode>>,
        sender: &str,
    ) {
        let _self = arc_mutex_self.lock().unwrap();
        let blockchain_result_message =
            BlockchainMessage::BlockchainResult(_self.blockchain.clone()).as_string();
        log(format!(
            "Sending BlockchainResult {:?} to : {:?}",
            _self.blockchain, sender
        ));
        _self
            .socket
            .send_to(blockchain_result_message.as_bytes(), sender)
            .unwrap();
    }

    #[allow(clippy::mutex_atomic)]
    fn process_blockchain_result_message(
        arc_mutex_self: Arc<Mutex<BlockchainNode>>,
        sender: &str,
        blockchain: Blockchain,
    ) {
        let mut _self = arc_mutex_self.lock().unwrap();
        if *_self.synchronization_done.0.lock().unwrap() {
            log("I was already synchronized. Skipping..".to_string());
            return;
        }
        log(format!(
            "Processing BlockchainResult message from : {:?} content: {:?}",
            sender, blockchain
        ));
        _self.blockchain = blockchain;
        log(format!("Current blockchain is: {:?}", _self.blockchain));
        *_self.synchronization_done.0.lock().unwrap() = true;
        _self.synchronization_done.1.notify_all();
        log("Notifying synchronization_done condvar".to_string());
    }

    pub fn listen(arc_mutex_self: Arc<Mutex<BlockchainNode>>) {
        let port = { arc_mutex_self.lock().unwrap().port };
        log(format!("Starting to listen on port: {:?}", port));
        let socket = {
            let _self = arc_mutex_self.lock().unwrap();
            _self.socket.try_clone().unwrap()
        };

        let mut incoming_messages: HashMap<String, String>  = HashMap::new();

        loop {
            let mut buf = [0; 1000];
            match socket.recv_from(&mut buf) {
                Ok((size, from)) => {
                    let received = Vec::from(&buf[0..size]);
                    let str_received = String::from_utf8(received).unwrap();
                    log(format!(
                        "Received bytes {:?} from neighbor: {:?}: {}",
                        size, from, str_received
                    ));
                    let neighbor = from.to_string();
                    let clone = arc_mutex_self.clone();

                    let last_message : &mut String = incoming_messages.entry(neighbor.clone()).or_insert(String::from(&str_received));

                    let last_message : Option<&mut String> = incoming_messages.get_mut(&neighbor);
                    match last_message {
                        Some(last_message) => {
                            last_message.push_str(&str_received);
                        }

                        None => {
                            incoming_messages.insert(neighbor.clone(), String::from(&str_received));
                        }
                    };
                    let last_message: &String = incoming_messages.get(&neighbor).unwrap();
                    if  last_message.chars().last().unwrap() == '\n' {
                        let message_to_process: &mut String = incoming_messages.get_mut(&neighbor).unwrap();
                        message_to_process.pop();
                        thread::spawn(move || {
                            BlockchainNode::handle_incoming_message(clone, &message_to_process, &neighbor);
                        });
                    }
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
        log(format!(
            "Sending ping to neighbor with addr: {:?}",
            dest_addr
        ));
        self.socket.send_to("PING".as_bytes(), dest_addr).unwrap();
    }

    pub fn make_coordinator(&self) {
        log("Node received make_coordinator".to_string());
        match (*self).leader_port.lock() {
            Ok(mut leader_port) => {
                *leader_port = Option::from((*self).port);
            }
            Err(error) => {
                panic!("{}", error.to_string())
            }
        }
        log(format!("New coordinator: {:?}", self.leader_port));
    }

    pub fn add_grade(
        arc_mutex_self: Arc<Mutex<BlockchainNode>>,
        _name: String,
        _note: f64,
    ) -> Result<(), ()> {
        log("Node received add_grade".to_string());
        let result = DistMutex::acquire(arc_mutex_self.clone());
        match result {
            Ok(()) => {
                {
                    log(String::from("antes de enviar el TO COORDINATOR"));
                    let _self = arc_mutex_self.lock().unwrap();
                    _self
                        .socket
                        .send_to(
                            AddGradeMessage::ToCoordinator(_name, _note)
                                .as_string()
                                .as_bytes(),
                            _self.dist_mutex.coordinator_addr.clone(),
                        )
                        .unwrap();
                    log(String::from("despues de enviar el TO COORDINATOR"));
                }
                {
                    arc_mutex_self.lock().unwrap().dist_mutex.release();
                    let mut _self = arc_mutex_self.lock().unwrap();
                    if _self.port != _self.leader_port.lock().unwrap().unwrap() {
                        *_self.coordinator_state.lock_taken.lock().unwrap() = false;
                    }
                    Ok(())
                }
            }
            Err(()) => {
                log(String::from(
                    "No hubo respuesta del Coordinador. Comenzando proceso de eleccion de lider.",
                ));
                BlockchainNode::begin_election(arc_mutex_self.clone());
                BlockchainNode::add_grade(arc_mutex_self, _name, _note)
            }
        }
        // let result_acquire = DistMutex::acquire(arc_mutex_self.clone());
    }

    pub fn print(&self) {
        log("Print current blockchain".to_string());
        println!("{}", self.blockchain);
        if !self.blockchain.is_valid() {
            println!("Invalid blockchain!");
        }
    }

    #[allow(clippy::mutex_atomic)]
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
            Err(error) => {
                panic!("{}", error.to_string())
            }
        }

        log("Enviando mensaje ELECTION a vecinos. Esperando sus respuestas...".to_string());
        const OK_ELECTION_TIMEOUT: Duration = Duration::from_secs(1);
        let _got_ok = { arc_mutex_self.lock().unwrap().got_ok.clone() };
        let got_ok = _got_ok.1.wait_timeout_while(
            _got_ok.0.lock().unwrap(),
            OK_ELECTION_TIMEOUT,
            |got_it| !*got_it,
        );
        if !*got_ok.unwrap().0 {
            match arc_mutex_self.lock() {
                Ok(mut _self) => {
                    _self.make_leader();
                    *_self.is_in_election.0.lock().unwrap() = false;
                }
                Err(error) => {
                    panic!("{}", error.to_string())
                }
            }
        } else {
            let _is_in_election = { arc_mutex_self.lock().unwrap().is_in_election.clone() };
            let _is_in_election_condvar = _is_in_election
                .1
                .wait_while(_is_in_election.0.lock().unwrap(), |is_in_election| {
                    *is_in_election
                });
        }
    }

    fn make_leader(&mut self) {
        *self.leader_port.lock().unwrap() = Some(self.port);
        self.dist_mutex.coordinator_addr =
            ip_parser::local_address_with_port(&self.port.to_string());
        log(format!(
            "Soy el nuevo coordinador! Puerto {:?}",
            *self.leader_port.lock().unwrap()
        ));
        for neighbor in &self.neighbor_addresses {
            log(format!("\t\tEnviando mensaje COORDINATOR a {:?}", neighbor));
            let message_to_send = ElectionMessage::Coordinator.as_bytes();
            self.socket.send_to(&message_to_send, neighbor).unwrap();
        }
    }

    #[allow(clippy::mutex_atomic)]
    pub fn ask_for_blockchain(arc_mutex_self: Arc<Mutex<BlockchainNode>>) {
        let (neighbor_addresses, socket, synchronization_done) = {
            let _self = arc_mutex_self.lock().unwrap();
            (
                _self.neighbor_addresses.clone(),
                _self.socket.try_clone().unwrap(),
                _self.synchronization_done.clone(),
            )
        };
        for neighbor in &neighbor_addresses {
            log(format!(
                "\t\tEnviando mensaje AskForBlockchain a {:?}",
                neighbor
            ));
            let message_to_send = BlockchainMessage::AskForBlockchain.as_string();
            socket
                .send_to(&message_to_send.as_bytes(), neighbor)
                .unwrap();
        }

        const SYNCHRONIZATION_DONE_TIMEOUT: Duration = Duration::from_secs(1);
        let _synchronization_done = synchronization_done;
        log("Waiting for synchronization_done condvar".to_string());
        let _synchronization_done_condvar = _synchronization_done.1.wait_timeout_while(
            _synchronization_done.0.lock().unwrap(),
            SYNCHRONIZATION_DONE_TIMEOUT,
            |cannot_begin| !*cannot_begin,
        );
        log("Done waiting for synchronization_done condvar".to_string());
    }
}
