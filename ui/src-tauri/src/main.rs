// Aegis Guardian - Tauri Backend
// v1.2.0-Sprint-AI-Alerts

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod network;
mod file_integrity;
mod whitelist;
pub mod quarantine;
pub mod process_control;

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{State, Emitter};
use crate::network::{NetworkSentinel, ExposedPort};
use crate::file_integrity::check_file_integrity;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::fs::File;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;
use sysinfo::{System, Disks};
use std::sync::Mutex as StdMutex;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct IncidentStats {
    total_blocked: u32,
    sensitive_keys: u32,
    visual_alerts: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GuardianEvent {
    time: String,
    source: String,
    desc: String,
    _ts: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RealActivities {
    stats: IncidentStats,
    threats: Vec<GuardianEvent>,
    keys: Vec<GuardianEvent>,
    visual: Vec<GuardianEvent>,
}

/// AI 告警結構，emit 給前端的事件 payload
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AiAlert {
    time: String,
    module: String,      // AI_Brain_Clipboard | AI_Brain_Visual
    severity: String,
    message: String,
    preview: Option<String>,
    model: Option<String>,
}

fn parse_timestamp(ts: &str) -> (String, i64) {
    let mut ts_clean = ts;
    if let Some(idx) = ts.find('.') {
        ts_clean = &ts[..idx];
    }
    match chrono::NaiveDateTime::parse_from_str(ts_clean, "%Y-%m-%dT%H:%M:%S") {
        Ok(dt) => {
            let time_str = dt.format("%H:%M:%S").to_string();
            let ts_ms = dt.and_utc().timestamp_millis();
            (time_str, ts_ms)
        }
        Err(_) => {
            ("00:00:00".to_string(), 0)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ModulesConfig {
    visual: bool,
    clipboard: bool,
    network: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FileIntegrityConfig {
    custom_paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GuardianConfig {
    mode: String,
    modules: ModulesConfig,
    file_integrity: FileIntegrityConfig,
}

struct SharedData {
    config: GuardianConfig,
    stats: IncidentStats,
    whitelist_conn: Connection,
}

// ─── Tauri Commands ────────────────────────────────────────────────────────

#[tauri::command]
fn get_exposed_ports(state: State<Arc<Mutex<SharedData>>>) -> Vec<ExposedPort> {
    let data = state.lock().unwrap();
    let sentinel = NetworkSentinel::new();
    let ports = sentinel.scan_local_ports();

    // 取得當前所有正在監聽的 (port, pid) 清單
    let active_ports_and_pids: Vec<(u16, u32)> = ports.iter().map(|p| (p.port, p.pid)).collect();

    // 清理資料庫中已經失效的白名單（如果原本的服務已關閉，就把該 port 從白名單移除）
    let _ = whitelist::cleanup_stale_whitelist(&data.whitelist_conn, &active_ports_and_pids);

    // 從 DB 取得最新白名單 (port, pid)，過濾回傳
    let whitelisted = whitelist::get_whitelisted_ports(&data.whitelist_conn)
        .unwrap_or_default();

    ports
        .into_iter()
        .map(|mut p| {
            if whitelisted.contains(&(p.port, p.pid)) {
                p.ignored = true;
            }
            p
        })
        .collect()
}

#[tauri::command]
fn add_network_whitelist(
    state: State<Arc<Mutex<SharedData>>>,
    port: u16,
    pid: u32,
    process_name: String,
) -> Result<(), String> {
    let data = state.lock().unwrap();
    whitelist::add_whitelist(&data.whitelist_conn, port, pid, &process_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_network_whitelist(
    state: State<Arc<Mutex<SharedData>>>,
    port: u16,
) -> Result<(), String> {
    let data = state.lock().unwrap();
    whitelist::remove_whitelist(&data.whitelist_conn, port)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_network_whitelist(
    state: State<Arc<Mutex<SharedData>>>,
) -> Result<Vec<u16>, String> {
    let data = state.lock().unwrap();
    whitelist::get_whitelisted_ports(&data.whitelist_conn)
        .map(|ports| ports.into_iter().map(|(p, _)| p).collect())
        .map_err(|e| e.to_string())
}

// --- Sysinfo State ---
#[derive(Clone, Serialize)]
struct SystemResources {
    cpu: f32,
    ram: f32,
    disk: f32,
}

#[derive(Clone, Serialize)]
struct ThreatProcess {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory_mb: f64,
    status: String,
}

#[derive(Clone)]
struct SystemInfoCache {
    resources: SystemResources,
    threats: Vec<ThreatProcess>,
}

struct AppState {
    cache: Arc<StdMutex<SystemInfoCache>>,
}

#[tauri::command]
fn get_system_resources(state: State<'_, AppState>) -> Result<SystemResources, String> {
    let cache = state.cache.lock().unwrap();
    Ok(cache.resources.clone())
}

#[tauri::command]
fn get_threat_processes(state: State<'_, AppState>) -> Result<Vec<ThreatProcess>, String> {
    let cache = state.cache.lock().unwrap();
    Ok(cache.threats.clone())
}

#[tauri::command]
fn mitigate_process(pid: u32, action: String) -> Result<String, String> {
    match action.as_str() {
        "kill" => {
            let status = std::process::Command::new("kill").arg("-9").arg(pid.to_string()).status().map_err(|e| e.to_string())?;
            if status.success() { Ok(format!("Killed PID {}", pid)) } else { Err(format!("Failed to kill PID {}", pid)) }
        },
        "isolate" => {
            let status = std::process::Command::new("kill").arg("-STOP").arg(pid.to_string()).status().map_err(|e| e.to_string())?;
            if status.success() { Ok(format!("Isolated (Suspended) PID {}", pid)) } else { Err(format!("Failed to isolate PID {}", pid)) }
        },
        "resume" => {
            let status = std::process::Command::new("kill").arg("-CONT").arg(pid.to_string()).status().map_err(|e| e.to_string())?;
            if status.success() { Ok(format!("Resumed PID {}", pid)) } else { Err(format!("Failed to resume PID {}", pid)) }
        },
        _ => Err("Unknown action".to_string())
    }
}

#[tauri::command]
fn get_incident_stats(state: State<Arc<Mutex<SharedData>>>) -> IncidentStats {
    let data = state.lock().unwrap();
    data.stats.clone()
}

#[tauri::command]
fn get_real_activities() -> Result<RealActivities, String> {
    let mut stats = IncidentStats {
        total_blocked: 0,
        sensitive_keys: 0,
        visual_alerts: 0,
    };
    let mut threats = Vec::new();
    let mut keys = Vec::new();
    let mut visual = Vec::new();

    let path = PathBuf::from("../logs/incidents.json");
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().filter_map(|l| l.ok()) {
            if let Ok(v) = serde_json::from_str::<Value>(&line) {
                let module = v["module"].as_str().unwrap_or("Unknown").to_string();
                let severity = v["severity"].as_str().unwrap_or("INFO");
                let message = v["message"].as_str().unwrap_or("").to_string();
                let timestamp_str = v["timestamp"].as_str().unwrap_or("");
                
                let (time, _ts) = parse_timestamp(timestamp_str);
                
                // Skip info logs like "Log Initialized" to not clutter the UI
                if severity == "INFO" {
                    continue;
                }
                
                let event = GuardianEvent {
                    time,
                    source: module.clone(),
                    desc: message,
                    _ts,
                };

                if module == "NetworkMonitor" || module == "SystemFirewall" || severity == "CRITICAL" {
                    stats.total_blocked += 1;
                    if severity != "INFO" { threats.push(event); }
                } else if module == "ClipboardMonitor" || module == "TerminalFirewall" {
                    stats.sensitive_keys += 1;
                    keys.push(event);
                } else if module == "VisualSentry" {
                    stats.visual_alerts += 1;
                    visual.push(event);
                }
            }
        }
    }

    Ok(RealActivities {
        stats,
        threats,
        keys,
        visual,
    })
}

/// 供前端初始化時載入歷史 AI 告警
#[tauri::command]
fn get_ai_alerts() -> Result<Vec<AiAlert>, String> {
    let mut alerts = Vec::new();
    let path = PathBuf::from("../logs/incidents.json");
    if let Ok(file) = File::open(&path) {
        let reader = BufReader::new(file);
        for line in reader.lines().filter_map(|l| l.ok()) {
            if let Ok(v) = serde_json::from_str::<Value>(&line) {
                let module = v["module"].as_str().unwrap_or("").to_string();
                if !module.starts_with("AI_Brain") {
                    continue;
                }
                let timestamp_str = v["timestamp"].as_str().unwrap_or("");
                let (time, _) = parse_timestamp(timestamp_str);
                let metadata = &v["metadata"];
                alerts.push(AiAlert {
                    time,
                    module: module.clone(),
                    severity: v["severity"].as_str().unwrap_or("INFO").to_string(),
                    message: v["message"].as_str().unwrap_or("").to_string(),
                    preview: metadata["preview"].as_str().map(|s| s.to_string()),
                    model: metadata["model"].as_str().map(|s| s.to_string()),
                });
            }
        }
    }
    Ok(alerts)
}

#[tauri::command]
fn get_config(state: State<Arc<Mutex<SharedData>>>) -> Result<GuardianConfig, String> {
    let data = state.lock().unwrap();
    Ok(data.config.clone())
}

#[tauri::command]
fn update_config(state: State<Arc<Mutex<SharedData>>>, mode: String, modules: ModulesConfig, file_integrity: FileIntegrityConfig) -> Result<(), String> {
    let config_path = PathBuf::from("../config.yaml");
    
    {
        let mut data = state.lock().unwrap();
        data.config.mode = mode.clone();
        data.config.modules = modules.clone();
        data.config.file_integrity = file_integrity.clone();
    }

    let mut new_lines = Vec::new();
    new_lines.push(format!("mode: {}", mode));
    new_lines.push("modules:".to_string());
    new_lines.push(format!("  visual: {}", modules.visual));
    new_lines.push(format!("  clipboard: {}", modules.clipboard));
    new_lines.push(format!("  network: {}", modules.network));
    new_lines.push("".to_string());
    new_lines.push("file_integrity:".to_string());
    if file_integrity.custom_paths.is_empty() {
        new_lines.push("  custom_paths: []".to_string());
    } else {
        new_lines.push("  custom_paths:".to_string());
        for path in &file_integrity.custom_paths {
            new_lines.push(format!("    - \"{}\"", path));
        }
    }

    fs::write(config_path, new_lines.join("\n")).map_err(|e| e.to_string())?;
    let _ = fs::write("../.reload_config", "1");
    Ok(())
}

// ─── App Entry ─────────────────────────────────────────────────────────────

fn main() {
    let wl_conn = whitelist::init_db().expect("Failed to initialise whitelist DB");

    let shared_data = Arc::new(Mutex::new(SharedData {
        config: GuardianConfig {
            mode: "Silent Monitor".to_string(),
            modules: ModulesConfig {
                visual: true,
                clipboard: true,
                network: true,
            },
            file_integrity: FileIntegrityConfig {
                custom_paths: vec![],
            },
        },
        stats: IncidentStats {
            total_blocked: 0,
            sensitive_keys: 0,
            visual_alerts: 0,
        },
        whitelist_conn: wl_conn,
    }));

    // 初始化 Sysinfo 緩存 State 與背景採集執行緒
    let cache_mutex = Arc::new(StdMutex::new(SystemInfoCache {
        resources: SystemResources { cpu: 0.0, ram: 0.0, disk: 0.0 },
        threats: vec![],
    }));
    let app_state = AppState { cache: cache_mutex.clone() };

    thread::spawn(move || {
        let mut sys = System::new_all();
        // 第一次讓系統初始化差值所需狀態
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        
        loop {
            thread::sleep(Duration::from_millis(1500));
            
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

            let cpu_count = sys.cpus().len() as f32;
            let global_cpu = sys.global_cpu_usage();
            let cpu_usage = if cpu_count > 0.0 { global_cpu / cpu_count } else { global_cpu };

            let total_ram = sys.total_memory() as f32;
            let used_ram = sys.used_memory() as f32;
            let ram_usage = if total_ram > 0.0 { (used_ram / total_ram) * 100.0 } else { 0.0 };

            let disks = Disks::new_with_refreshed_list();
            let mut total_disk = 0.0;
            let mut available_disk = 0.0;
            for disk in disks.list() {
                total_disk += disk.total_space() as f32;
                available_disk += disk.available_space() as f32;
            }
            let used_disk = total_disk - available_disk;
            let disk_usage = if total_disk > 0.0 { (used_disk / total_disk) * 100.0 } else { 0.0 };

            let mut procs: Vec<ThreatProcess> = sys.processes().iter()
                .filter(|(_, p)| p.cpu_usage() > 3.0)
                .map(|(pid, p)| {
                    ThreatProcess {
                        pid: pid.as_u32(),
                        name: p.name().to_string_lossy().into_owned(),
                        cpu_usage: p.cpu_usage(),
                        memory_mb: (p.memory() as f64) / 1024.0 / 1024.0,
                        status: format!("{:?}", p.status()),
                    }
                })
                .collect();
            
            procs.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
            procs.truncate(8);

            if let Ok(mut cache) = cache_mutex.lock() {
                cache.resources = SystemResources { cpu: cpu_usage, ram: ram_usage, disk: disk_usage };
                cache.threats = procs;
            }
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(shared_data)
        .manage(app_state)
        .setup(|app| {
            let app_handle = app.handle().clone();

            // ── Sidecar: 啟動 Python 後端 daemon ─────────────────────────────
            let sidecar_cmd = app
                .shell()
                .sidecar("aegis-core-daemon")
                .expect("[Aegis] 找不到 sidecar: aegis-core-daemon");

            let (mut rx, _child) = sidecar_cmd
                .spawn()
                .expect("[Aegis] 無法啟動 Python 後端");

            // 將 sidecar 的 stdout/stderr 轉發至 Tauri 日誌
            tauri::async_runtime::spawn(async move {
                while let Some(event) = rx.recv().await {
                    match event {
                        CommandEvent::Stdout(line) => {
                            if let Ok(s) = String::from_utf8(line) {
                                log::info!("[daemon] {}", s.trim_end());
                            }
                        }
                        CommandEvent::Stderr(line) => {
                            if let Ok(s) = String::from_utf8(line) {
                                log::warn!("[daemon:err] {}", s.trim_end());
                            }
                        }
                        _ => {}
                    }
                }
            });

            // ── 背景執行緒：輪詢 incidents.json，即時推送 AI 告警 ──────────
            let app_handle = app_handle.clone();
            thread::spawn(move || {
                let path = PathBuf::from("../logs/incidents.json");
                let mut last_pos: u64 = 0;

                // 若檔案存在，先跳到尾端，避免重播歷史告警
                if let Ok(meta) = fs::metadata(&path) {
                    last_pos = meta.len();
                }

                loop {
                    thread::sleep(Duration::from_secs(1));

                    if let Ok(mut file) = File::open(&path) {
                        if let Ok(meta) = file.metadata() {
                            let current_len = meta.len();
                            if current_len > last_pos {
                                // 有新資料，從上次讀到的位置繼續
                                let _ = file.seek(SeekFrom::Start(last_pos));
                                let reader = BufReader::new(&file);
                                for line in reader.lines().filter_map(|l| l.ok()) {
                                    if let Ok(v) = serde_json::from_str::<Value>(&line) {
                                        let module = v["module"].as_str().unwrap_or("").to_string();
                                        if module.starts_with("AI_Brain") {
                                            let timestamp_str = v["timestamp"].as_str().unwrap_or("");
                                            let (time, _) = parse_timestamp(timestamp_str);
                                            let metadata = &v["metadata"];
                                            let alert = AiAlert {
                                                time,
                                                module: module.clone(),
                                                severity: v["severity"].as_str().unwrap_or("INFO").to_string(),
                                                message: v["message"].as_str().unwrap_or("").to_string(),
                                                preview: metadata["preview"].as_str().map(|s| s.to_string()),
                                                model: metadata["model"].as_str().map(|s| s.to_string()),
                                            };
                                            let _ = app_handle.emit("ai-alert", &alert);
                                        }
                                    }
                                }
                                last_pos = current_len;
                            }
                        }
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_config,
            get_incident_stats,
            get_real_activities,
            get_ai_alerts,
            get_exposed_ports,
            add_network_whitelist,
            remove_network_whitelist,
            get_network_whitelist,
            check_file_integrity,
            quarantine::move_to_quarantine,
            process_control::terminate_process,
            get_system_resources,
            get_threat_processes,
            mitigate_process
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}