///
/// |     Tipo      |  Representacion Binaria   |
/// |---------------|------------------------   |
/// |   Acquire     |            b'ACQUI'       |
/// |   OkAcquire   |            b'OKACQ'       |
/// |   Release     |            b'RELEA'       |
#[derive(PartialEq, Debug)]
pub enum AcquireMessage {
    Acquire,
    OkAcquire,
    Release,
}

impl AcquireMessage {
    /// Devuelve la representacion binaria del mensaje para enviar por un puerto.
    /// ```rust
    /// AcquireMessage::Acquire.as_bytes() // => b'ACQUI'
    /// ```
    pub fn as_bytes(self) -> [u8; 5] {
        match self {
            AcquireMessage::Acquire => b"ACQUI".clone(),
            AcquireMessage::OkAcquire => b"OKACQ".clone(),
            AcquireMessage::Release => b"RELEA".clone(),
        }
    }

    /// Recibe un caracter binario. Devuelve el tipo de mensaje que corresponde a esa representacion binaria.
    ///```rust
    ///AcquireMessage::from_bytes(b'ACQUI'); // => Some(AcquireMessage::Acquire)
    ///AcquireMessage::from_bytes(b"Whatever"); // => None
    ///```

    pub fn from_bytes(bytes: &[u8]) -> Option<AcquireMessage> {
        match bytes {
            b"ACQUI" => Some(AcquireMessage::Acquire),
            b"OKACQ" => Some(AcquireMessage::OkAcquire),
            b"RELEA" => Some(AcquireMessage::Release),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_bytes() {
        assert_eq!(*b"ACQUI", AcquireMessage::Acquire.as_bytes());
        assert_eq!(*b"OKACQ", AcquireMessage::OkAcquire.as_bytes());
        assert_eq!(*b"RELEA", AcquireMessage::Release.as_bytes());
    }

    #[test]
    fn from_bytes() {
        assert_eq!(
            Some(AcquireMessage::Acquire),
            AcquireMessage::from_bytes("ACQUI".as_bytes())
        );
        assert_eq!(
            Some(AcquireMessage::OkAcquire),
            AcquireMessage::from_bytes("OKACQ".as_bytes())
        );
        assert_eq!(
            Some(AcquireMessage::Release),
            AcquireMessage::from_bytes("RELEA".as_bytes())
        );
    }
}
