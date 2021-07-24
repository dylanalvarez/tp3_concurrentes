/// Contiene los distintos tipos de mensajes durante el proceso de eleccion (algoritmo Bully)
/// - **Election**: El nodo que envia este mensaje desea empezar un proceso de eleccion de lider.
/// - **OkElection**: El nodo que envia este mensaje es porque recibio un mensaje Election previamente, tiene mayor ID que quien le envia Election
/// y continuará con el proceso de eleccion.
/// - **Coordinator**: Un nodo envia este mensaje cuando detecta que debe ser el lider. Los que reciben estos mensajes actualizan su referencia al nuevo Lider
///
/// |     Tipo      |  Representacion Binaria   |
/// |---------------|------------------------   |
/// |   Election    |            b'E'           |
/// |   OkElection  |            b'O'           |
/// |   Coordinator |            b'C'           |
#[derive(PartialEq, Debug)]
pub enum ElectionMessage {
    Election,
    OkElection,
    Coordinator,
}

impl ElectionMessage {
    /// Devuelve la representacion binaria del mensaje para enviar por un puerto.
    /// ```rust
    /// ElectionMessage::Election.as_bytes() // => b'E'
    /// ```
    pub fn as_bytes(self) -> [u8; 1] {
        match self {
            ElectionMessage::Election => b"E".clone(),
            ElectionMessage::OkElection => b"O".clone(),
            ElectionMessage::Coordinator => b"C".clone(),
        }
    }

    /// Recibe un caracter binario. Devuelve el tipo de mensaje que corresponde a esa representacion binaria.
    ///```rust
    ///ElectionMessage::from_bytes(b'C'); // => Some(ElectionMessage::Coordinator)
    ///ElectionMessage::from_bytes(b"Whatever"); // => None
    ///```

    pub fn from_bytes(bytes: &[u8]) -> Option<ElectionMessage> {
        match bytes {
            b"E" => Some(ElectionMessage::Election),
            b"O" => Some(ElectionMessage::OkElection),
            b"C" => Some(ElectionMessage::Coordinator),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_bytes() {
        assert_eq!([b'E'], ElectionMessage::Election.as_bytes());
        assert_eq!([b'O'], ElectionMessage::OkElection.as_bytes());
        assert_eq!([b'C'], ElectionMessage::Coordinator.as_bytes());
    }

    #[test]
    fn from_bytes() {
        assert_eq!(
            Some(ElectionMessage::Election),
            ElectionMessage::from_bytes(&[b'E'])
        );
        assert_eq!(
            Some(ElectionMessage::OkElection),
            ElectionMessage::from_bytes(&[b'O'])
        );
        assert_eq!(
            Some(ElectionMessage::Coordinator),
            ElectionMessage::from_bytes(&[b'C'])
        );
    }
}
