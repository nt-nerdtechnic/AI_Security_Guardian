mod os_agnostic;

use std::time::Duration;
use tokio::time::sleep;
use sysinfo::{System, SystemExt, ProcessExt};
use os_agnostic::SystemInfo;
use serde::Serialize;
use serde_json;

#[derive(Serialize)]
struct Incident {
    timestamp: u64,
    event_type: String,
    process_name: String,
    pid: u32,
    severity: String,
}

#[tokio::main]
async fn main() {
    let sys_info = SystemInfo::new();
    println!("🛡️ [Rust Kernel] Guardian Core Started on {} ({})", sys_info.hostname, sys_info.os_name);
    println!("🛡️ [Rust Kernel] Prepared to load Python-Brain via PyO3.");

    let mut sys = System::new_all();

    // Heartbeat logic with Process Monitoring
    tokio::spawn(async move {
        let mut count = 0;
        loop {
            count += 1;
            sys.refresh_all();
            
            // Log core status
            println!("💓 [Heartbeat] Rust Kernel Tick: {}. Monitoring {} processes.", count, sys.processes().len());
            
            // Simplified check for suspicious processes (Phase R&D)
            for (pid, process) in sys.processes() {
                let name = process.name().to_lowercase();
                if name.contains("nc") || name.contains("ncat") || name.contains("socat") {
                    let incident = Incident {
                        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                        event_type: "Suspicious Process".to_string(),
                        process_name: name.clone(),
                        pid: pid.as_u32(),
                        severity: "HIGH".to_string(),
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
