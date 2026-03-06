mod os_agnostic;
mod interceptor;

use std::time::Duration;
use tokio::time::sleep;
use os_agnostic::SystemInfo;
use serde::Serialize;
use serde_json;
use crate::interceptor::clipboard::ClipboardMonitor;
use crate::interceptor::process::ProcessMonitor;
use crate::interceptor::file_watch::FileIntegrityMonitor;
use crate::interceptor::network::NetworkSentinel;

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
    let mut file_integrity_monitor = FileIntegrityMonitor::new();
    let network_sentinel = NetworkSentinel::new();

    // Baseline common critical files (R&D Sprint target)
    let critical_files = vec![
        "/etc/hosts", 
        "guardian_core/src/main.rs",
        "guardian.py",
        "config.yaml"
    ];
    for file in &critical_files {
        if let Err(e) = file_integrity_monitor.add_to_baseline(file) {
            eprintln!("⚠️ [Integrity] Failed to baseline {}: {}", file, e);
        }
    }

    // Baseline project root recursively
    let _ = file_integrity_monitor.scan_directory_recursive(".");

    // Heartbeat logic with Process Monitoring and Clipboard Scanning
    tokio::spawn(async move {
        let mut count = 0;
        let memory_scanner = interceptor::memory::MemoryScanner::new();
        let sys_info = SystemInfo::new();

        loop {
            count += 1;
            
            // Log core status
            println!("💓 [Heartbeat] Rust Kernel Tick: {}. Detected {} active threats. [OS: {}]", count, 0, sys_info.os_name);

            // Core logic update: using process_monitor
            let incidents = process_monitor.scan_processes();
            
            // File Integrity check
            let integrity_alerts = file_integrity_monitor.check_integrity();
            
            // Network check
            let network_alerts = network_sentinel.scan_local_ports();

            // Project root scan for new files (Recursive)
            if let Ok(new_files) = file_integrity_monitor.scan_directory_recursive(".") {
                for new_file in new_files {
                    let incident = Incident {
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                        event_type: "Unknown File Detected".to_string(),
                        process_name: "FS Watcher".to_string(),
                        pid: 0,
                        severity: "MEDIUM".to_string(),
                        details: Some(format!("New file detected: {}", new_file)),
                    };
                    if let Ok(json) = serde_json::to_string(&incident) {
                        println!("🚨 [RUST ALERT] {}", json);
                    }
                    // Auto-baseline new file for now (to avoid repeated alerts)
                    let _ = file_integrity_monitor.add_to_baseline(&new_file);
                }
            }

            // Log core status
            println!("💓 [Heartbeat] Rust Kernel Tick: {}. Detected {} active threats.", count, incidents.len() + integrity_alerts.len() + network_alerts.len());

            for alert in network_alerts {
                let incident = Incident {
                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                    event_type: "Network Exposure Detected".to_string(),
                    process_name: "Network Sentinel".to_string(),
                    pid: 0,
                    severity: "LOW".to_string(),
                    details: Some(alert),
                };
                if let Ok(json) = serde_json::to_string(&incident) {
                    println!("🚨 [RUST ALERT] {}", json);
                }
            }

            for alert in integrity_alerts {
                let incident = Incident {
                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                    event_type: "File Integrity Compromised".to_string(),
                    process_name: "FS Watcher".to_string(),
                    pid: 0,
                    severity: "HIGH".to_string(),
                    details: Some(alert),
                };
                if let Ok(json) = serde_json::to_string(&incident) {
                    println!("🚨 [RUST ALERT] {}", json);
                }
            }

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
