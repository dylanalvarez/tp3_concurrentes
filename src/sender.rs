use std::net::UdpSocket;

use crate::BUFFER_SIZE;

pub fn send(socket: UdpSocket, buffer: &[u8], address: &str) {
    for chunk in buffer.chunks(BUFFER_SIZE) {
        socket.send_to(chunk, address).unwrap();
    }
    socket.send_to("\n".as_bytes(), address).unwrap();
}
