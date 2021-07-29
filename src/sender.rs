use std::net::UdpSocket;

pub fn send(socket: UdpSocket, buffer: &[u8], address: &str) -> usize {
    let mut message = String::from_utf8(Vec::from(buffer)).unwrap();
    message.push_str("\n");
    socket
        .send_to(
            message.as_bytes(),
            address,
        )
        .unwrap()
}