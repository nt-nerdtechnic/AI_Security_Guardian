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
            let exe_path = process.exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            
            // 1. Name-based matching
            if name == "nc" || name == "ncat" || name == "socat" || 
               name.contains("wireshark") || name.contains("tcpdump") || name.contains("metasploit") ||
               name.contains("keylogger") || name.contains("spyware") || name == "aircrack-ng" ||
               name.contains("mimikatz") || name.contains("hydra") || name == "nmap" ||
               name == "john" || name == "hashcat" || name == "gobuster" || name == "dirb" ||
               name == "nikto" || name == "maltego" || name == "beef-xss" || name == "msfconsole" {
                
                incidents.push(super::super::Incident {
                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                    event_type: "Suspicious Process Detected".to_string(),
                    process_name: name.clone(),
                    pid: pid.as_u32(),
                    severity: "HIGH".to_string(),
                    details: Some(format!("Detected potentially malicious or monitoring tool: {}", process.name())),
                });
            }

            // 2. Location-based matching (e.g., running from /tmp or /var/tmp)
            if exe_path.contains("/tmp/") || exe_path.contains("/var/tmp/") || exe_path.contains("/private/tmp/") {
                 incidents.push(super::super::Incident {
                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                    event_type: "Suspicious Execution Path".to_string(),
                    process_name: name,
                    pid: pid.as_u32(),
                    severity: "CRITICAL".to_string(),
                    details: Some(format!("Process running from temporary directory: {}", exe_path)),
                });
            }
        }
        incidents
    }
}
