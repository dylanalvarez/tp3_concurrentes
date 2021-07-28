use std::{
    net::UdpSocket,
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

use crate::{acquire_message::AcquireMessage, blockchain_node::BlockchainNode, logger::log};

pub struct DistMutex {
    pub coordinator_addr: String,
    pub socket_to_coordinator: UdpSocket,
    pub got_acquire_confirmation: Arc<(Mutex<bool>, Condvar)>,
}

impl DistMutex {
    #[allow(clippy::mutex_atomic)]
    pub fn new(coordinator_addr: String, socket_to_coordinator: UdpSocket) -> DistMutex {
        let got_acquire_confirmation = Arc::new((Mutex::new(false), Condvar::new()));
        DistMutex {
            coordinator_addr,
            socket_to_coordinator,
            got_acquire_confirmation,
        }
    }

    #[allow(clippy::mutex_atomic)]
    pub fn acquire(blockchain_node: Arc<Mutex<BlockchainNode>>) -> Result<(), ()> {
        {
            let node = blockchain_node.lock().unwrap();
            log(format!(
                "Sending ACQUIRE to coordinator: {:?}",
                node.dist_mutex.coordinator_addr
            ));
            node.dist_mutex
                .socket_to_coordinator
                .send_to(
                    &AcquireMessage::Acquire.as_bytes(),
                    &node.dist_mutex.coordinator_addr,
                )
                .unwrap();

            log("Waiting for OK_ACQUIRE message".to_string());
        }
        const OK_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

        let _got_acquire_confirmation = {
            blockchain_node
                .lock()
                .unwrap()
                .dist_mutex
                .got_acquire_confirmation
                .clone()
        };

        let got_acquire_confirmation = _got_acquire_confirmation.1.wait_timeout_while(
            _got_acquire_confirmation.0.lock().unwrap(),
            OK_ACQUIRE_TIMEOUT,
            |dont_got_it| !*dont_got_it,
        );
        if *got_acquire_confirmation.unwrap().0 {
            log("Got OK_ACQUIRE message".to_string());
            let node = blockchain_node.lock().unwrap();
            *node.dist_mutex.got_acquire_confirmation.0.lock().unwrap() = false;
            Ok(())
        } else {
            log("Timeout waiting for OK_ACQUIRE message".to_string());
            Err(())
        }
        // Lock not taken
    }

    pub fn release(&mut self) {
        log(format!(
            "Sending RELEASE to coordinator with addr: {:?}",
            self.coordinator_addr
        ));
        self.socket_to_coordinator
            .send_to(&AcquireMessage::Release.as_bytes(), &self.coordinator_addr)
            .unwrap();
    }

    pub fn is_coordinator(&self, addr: String) -> bool {
        addr == *self.coordinator_addr
    }
}
