pub fn local_address_with_port(port: &str) -> String {
    "127.0.0.1:".to_owned() + port
}

pub fn get_port_from_dir(dir: &str) -> Option<usize> {
    dir.split(':')
        .last()
        .map(|port| port.parse::<usize>().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ip() {
        assert_eq!(Some(8080), get_port_from_dir("127.0.0.1:8080"))
    }
}
