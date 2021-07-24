pub enum ElectionMessage {
    Election,
    OkElection,
    Coordinator
}

impl ElectionMessage {
    pub fn as_bytes(self) -> [u8; 1] {
        match self {
            ElectionMessage::Election => b"E".clone(),
            ElectionMessage::OkElection => b"O".clone(),
            ElectionMessage::Coordinator => b"C".clone()
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<ElectionMessage> {
        match bytes {
            b"E" => Some(ElectionMessage::Election),
            b"O" => Some(ElectionMessage::OkElection),
            b"C" => Some(ElectionMessage::Coordinator),
            _ => None
        }
    }
}