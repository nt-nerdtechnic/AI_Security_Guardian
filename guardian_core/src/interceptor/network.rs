use std::net::{TcpStream, SocketAddr};
use std::time::Duration;

pub struct NetworkSentinel {
    monitored_ports: Vec<u16>,
}

impl NetworkSentinel {
    pub fn new() -> Self {
        Self {
            monitored_ports: vec![22, 80, 443, 3306, 5432, 6379, 8080, 27017],
        }
    }

    pub fn scan_local_ports(&self) -> Vec<String> {
        let mut open_ports = Vec::new();
        for &port in &self.monitored_ports {
            let addr = format!("127.0.0.1:{}", port).parse::<SocketAddr>().unwrap();
            if TcpStream::connect_timeout(&addr, Duration::from_millis(50)).is_ok() {
                open_ports.push(format!("Port {} is OPEN on localhost", port));
            }
        }
        open_ports
    }
}
