pub struct PeerCounter {
    num_peers: usize,
}

impl PeerCounter {
    pub fn new() -> Self {
        PeerCounter { num_peers: 0 }
    }

    pub fn increment(&mut self) {
        self.num_peers += 1;
    }

    pub fn decrement(&mut self) {
        self.num_peers -= 1;
    }

    pub fn get_num_peers(&self) -> usize {
        return self.num_peers;
    }
}