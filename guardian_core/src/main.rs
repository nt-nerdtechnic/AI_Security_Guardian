mod os_agnostic;
mod interceptor;

use std::time::Duration;
use tokio::time::sleep;
use os_agnostic::SystemInfo;
use serde::Serialize;
use serde_json;
use crate::interceptor::clipboard::ClipboardMonitor;
use crate::interceptor::process::ProcessMonitor;

#[derive(Serialize, Clone)]
pub struct Incident {
    pub timestamp: u64,
    pub event_type: String,
    pub process_name: String,
    pub pid: u32,
    pub severity: String,
    pub details: Option<String>,
}

#[tokio::main]
async fn main() {
    let sys_info = SystemInfo::new();
    println!("🛡️ [Rust Kernel] Guardian Core Started on {} ({})", sys_info.hostname, sys_info.os_name);
    println!("🛡️ [Rust Kernel] Prepared to load Python-Brain via PyO3.");

    // Attempt to initialize Clipboard Monitor
    let clipboard_monitor = match ClipboardMonitor::new() {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("⚠️ [Clipboard] Failed to init: {}", e);
            None
        }
    };

    let mut process_monitor = ProcessMonitor::new();

    // Heartbeat logic with Process Monitoring and Clipboard Scanning
    tokio::spawn(async move {
        let mut count = 0;
        let memory_scanner = interceptor::memory::MemoryScanner::new();

        loop {
            count += 1;
            
            // Core logic update: using process_monitor
            let incidents = process_monitor.scan_processes();
            
            // Log core status
            println!("💓 [Heartbeat] Rust Kernel Tick: {}. Detected {} active threats.", count, incidents.len());

            for incident in incidents {
                if let Ok(json) = serde_json::to_string(&incident) {
                    println!("🚨 [RUST ALERT] {}", json);
                }
                
                // Trigger memory scan for suspicious process
                memory_scanner.scan_process(incident.pid, b"");
            }

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

            sleep(Duration::from_secs(60)).await;
        }
    });

    // Main event loop placeholder
    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
