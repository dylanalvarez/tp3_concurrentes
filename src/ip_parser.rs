pub fn local_address_with_port(port: &String) -> String {
    "127.0.0.1:".to_owned() + port
}

pub fn get_port_from_dir(dir: &str) -> Option<usize> {
    match dir.split(':').last() {
        Some(port) => Some(port.parse::<usize>().unwrap()),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ip() {
        assert_eq!(Some(8080), get_port_from_dir("127.0.0.1:8080"))
    }
}
