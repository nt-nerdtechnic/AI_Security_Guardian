mod os_agnostic;
mod interceptor;

use std::time::Duration;
use tokio::time::sleep;
use sysinfo::System;
use os_agnostic::SystemInfo;
use serde::Serialize;
use serde_json;
use crate::interceptor::clipboard::ClipboardMonitor;

#[derive(Serialize)]
struct Incident {
    timestamp: u64,
    event_type: String,
    process_name: String,
    pid: u32,
    severity: String,
    details: Option<String>,
}

#[tokio::main]
async fn main() {
    let sys_info = SystemInfo::new();
    println!("🛡️ [Rust Kernel] Guardian Core Started on {} ({})", sys_info.hostname, sys_info.os_name);
    println!("🛡️ [Rust Kernel] Prepared to load Python-Brain via PyO3.");

    let mut sys = System::new_all();
    
    // Attempt to initialize Clipboard Monitor
    let clipboard_monitor = match ClipboardMonitor::new() {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("⚠️ [Clipboard] Failed to init: {}", e);
            None
        }
    };

    // Heartbeat logic with Process Monitoring and Clipboard Scanning
    tokio::spawn(async move {
        let mut count = 0;
        let memory_scanner = interceptor::memory::MemoryScanner::new();

        loop {
            count += 1;
            sys.refresh_all();
            
            let processes = sys.processes();
            let process_count = processes.len();
            
            // Log core status
            println!("💓 [Heartbeat] Rust Kernel Tick: {}. Monitoring {} processes.", count, process_count);

            // Clipboard Check
            if let Some(ref monitor) = clipboard_monitor {
                if let Some(details) = monitor.check_clipboard() {
                    let incident = Incident {
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                        event_type: "Sensitive Clipboard Data".to_string(),
                        process_name: "Clipboard".to_string(),
                        pid: 0,
                        severity: "CRITICAL".to_string(),
                        details: Some(details),
                    };
                    if let Ok(json) = serde_json::to_string(&incident) {
                        println!("🚨 [RUST ALERT] {}", json);
                    }
                }
            }

            if process_count > 0 {
                // Randomly select one process to scan
                let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or(std::time::Duration::from_secs(0));
                let random_index = now.subsec_nanos() as usize % process_count;
                
                if let Some((pid, _)) = processes.iter().nth(random_index) {
                    memory_scanner.scan_process(pid.as_u32(), b"");
                }
            }
            
            for (pid, process) in sys.processes() {
                let name = process.name().to_lowercase();
                if name.contains("nc") || name.contains("ncat") || name.contains("socat") {
                    let incident = Incident {
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                        event_type: "Suspicious Process".to_string(),
                        process_name: name.clone(),
                        pid: pid.as_u32(),
                        severity: "HIGH".to_string(),
                        details: None,
                    };
                    
                    if let Ok(json) = serde_json::to_string(&incident) {
                        println!("🚨 [RUST ALERT] {}", json);
                    }
                }
            }

            sleep(Duration::from_secs(60)).await;
        }
    });

    // Main event loop placeholder
    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
