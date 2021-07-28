use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};
pub struct CoordinatorState {
    pub lock_taken: Arc<Mutex<bool>>,
    pub lock_owner_addr: Arc<Mutex<String>>,
    pub got_release_confirmation: Arc<(Mutex<bool>, Condvar)>,
    pub waiting_nodes_queue: VecDeque<String>,
}

impl CoordinatorState {
    #[allow(clippy::mutex_atomic)]
    pub fn new() -> CoordinatorState {
        let lock_taken = Arc::new(Mutex::new(false));
        let lock_owner_addr = Arc::new(Mutex::new(String::new()));
        let got_release_confirmation = Arc::new((Mutex::new(false), Condvar::new()));
        let waiting_nodes_queue = VecDeque::new();
        CoordinatorState {
            lock_taken,
            lock_owner_addr,
            got_release_confirmation,
            waiting_nodes_queue,
        }
    }

    pub fn is_taken(&self) -> bool {
        *self.lock_taken.lock().unwrap()
    }

    pub fn enqueue_requestor(&mut self, sender_addr: String) {
        self.waiting_nodes_queue.push_back(sender_addr);
    }

    pub fn deque_requestor(&mut self) -> Option<String> {
        self.waiting_nodes_queue.pop_front()
    }

    pub fn set_taken(&self, taken: bool) {
        *self.lock_taken.lock().unwrap() = taken;
    }

    pub fn set_lock_owner_addr(&self, lock_owner_addr: String) {
        *self.lock_owner_addr.lock().unwrap() = lock_owner_addr;
    }
}
