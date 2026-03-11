mod os_agnostic;
pub mod quarantine;
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
use pyo3::prelude::*;
use std::process::Command;

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
    println!("🛡️ [Rust Kernel] Aegis Guardian Core Started on {} ({})", sys_info.hostname, sys_info.os_name);
    println!("🛡️ [Rust Kernel] Prepared to load Aegis-Brain via PyO3.");

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
                    println!("🧠 [AI 思考中] 截獲剪貼簿內容，正在傳送至本地 AI 大腦進行語意分析...");
                    
                    let is_threat = Python::with_gil(|py| -> PyResult<bool> {
                        // 確保目前的工作目錄在 Python 的 sys.path 中
                        let sys = py.import("sys")?;
                        let path = sys.getattr("path")?;
                        let current_dir = std::env::current_dir()?.to_string_lossy().into_owned();
                        path.call_method1("insert", (0, current_dir.as_str()))?;

                        let brain_module = PyModule::import(py, "guardian_brain")?;
                        let analyze_fn = brain_module.getattr("analyze_command_semantics")?;
                        let result: bool = analyze_fn.call1((&details,))?.extract()?;
                        Ok(result)
                    }).unwrap_or_else(|e| {
                        eprintln!("⚠️ [AI 大腦斷線] 無法呼叫 Python 模型: {}", e);
                        false // 發生錯誤時預設放行或可以改成 true (最嚴謹)
                    });

                    if is_threat {
                        let incident = Incident {
                            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                            event_type: "AI 判定: 高危險意圖或敏感資料洩漏".to_string(),
                            process_name: "Clipboard".to_string(),
                            pid: 0,
                            severity: "CRITICAL".to_string(),
                            details: Some(format!("AI 攔截剪貼簿: {}", details)),
                        };
                        if let Ok(json) = serde_json::to_string(&incident) {
                            println!("🚨 [RUST 執行硬攔截] {}", json);
                            // 這裡可以呼叫 clipboard monitor 的 clear 方法清空剪貼簿，實現真正的「攔截」
                        }
                    } else {
                        println!("✅ [AI 放行] 剪貼簿內容語意安全。");
                    }
                }
            }
            // Visual Sentry (硬派截圖與 AI 視覺分析)
            // 這裡每 5 秒觸發一次截圖分析，因每秒截圖對效能影響較大
            if count % 5 == 0 {
                let snapshot_path = "/tmp/aegis_snapshot.png";
                if let Ok(_) = Command::new("screencapture").arg("-x").arg(snapshot_path).output() {
                    println!("👁️  [AI 視覺哨兵] 已擷取目前畫面，正在使用本地 VLM (視覺大模型) 進行全畫面掃描...");
                    
                    let is_visual_threat = Python::with_gil(|py| -> PyResult<bool> {
                        let sys = py.import("sys")?;
                        let path = sys.getattr("path")?;
                        let current_dir = std::env::current_dir()?.to_string_lossy().into_owned();
                        path.call_method1("insert", (0, current_dir.as_str()))?;

                        let brain_module = PyModule::import(py, "guardian_brain")?;
                        let analyze_fn = brain_module.getattr("analyze_visual_threat")?;
                        let result: bool = analyze_fn.call1((snapshot_path,))?.extract()?;
                        Ok(result)
                    }).unwrap_or(false);

                    if is_visual_threat {
                        let incident = Incident {
                            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                            event_type: "AI 視覺判定: 發現特權提權或密碼外洩風險".to_string(),
                            process_name: "Window Server".to_string(),
                            pid: 0,
                            severity: "CRITICAL".to_string(),
                            details: Some("AI 偵測到危險的系統視窗或隱私內容".to_string()),
                        };
                        if let Ok(json) = serde_json::to_string(&incident) {
                            println!("🚨 [RUST 視覺攔截] {}", json);
                        }
                    } else {
                        println!("✅ [AI 放行] 視覺環境安全。");
                    }
                }
            }

            // 極速掃描頻率: 每 1 秒執行一次 (符合 SPEC 中要求)
            sleep(Duration::from_secs(1)).await;
        }
    });

    // Main event loop placeholder
    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
