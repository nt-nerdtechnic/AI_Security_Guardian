use sysinfo::{System};

pub struct ProcessMonitor {
    sys: System,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
        }
    }

    pub fn scan_processes(&mut self) -> Vec<super::super::Incident> {
        self.sys.refresh_all();
        let mut incidents = Vec::new();
        
        for (pid, process) in self.sys.processes() {
            let name = process.name().to_lowercase();
            // Expanded suspicion list
            if name == "nc" || name == "ncat" || name == "socat" || 
               name.contains("wireshark") || name.contains("tcpdump") || name.contains("metasploit") {
                
                incidents.push(super::super::Incident {
                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                    event_type: "Suspicious Process Detected".to_string(),
                    process_name: name,
                    pid: pid.as_u32(),
                    severity: "HIGH".to_string(),
                    details: Some(format!("Detected potentially malicious or monitoring tool: {}", process.name())),
                });
            }
        }
        incidents
    }
}
