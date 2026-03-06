use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExposedPort {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub is_risky: bool,
}

pub struct NetworkSentinel {
    monitored_ports: Vec<u16>,
}

impl NetworkSentinel {
    pub fn new() -> Self {
        Self {
            monitored_ports: vec![22, 80, 443, 1433, 3306, 5432, 6379, 8080, 27017],
        }
    }

    pub fn scan_local_ports(&self) -> Vec<ExposedPort> {
        let mut exposed_ports = Vec::new();
        
        let output = Command::new("lsof")
            .args(["-iTCP", "-sTCP:LISTEN", "-n", "-P", "-Fpcn"])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut current_pid: u32 = 0;
            let mut current_process_name = String::new();

            for line in output_str.lines() {
                if line.starts_with('p') {
                    if let Ok(pid) = line[1..].parse::<u32>() {
                        current_pid = pid;
                    }
                } else if line.starts_with('c') {
                    current_process_name = line[1..].to_string();
                } else if line.starts_with('n') {
                    if let Some(colon_idx) = line.rfind(':') {
                        if let Ok(port) = line[colon_idx + 1..].parse::<u16>() {
                            let is_risky = self.monitored_ports.contains(&port);
                            let exposed = ExposedPort {
                                port,
                                pid: current_pid,
                                process_name: current_process_name.clone(),
                                is_risky,
                            };
                            if !exposed_ports.contains(&exposed) {
                                exposed_ports.push(exposed);
                            }
                        }
                    }
                }
            }
        }
        
        exposed_ports
    }
}