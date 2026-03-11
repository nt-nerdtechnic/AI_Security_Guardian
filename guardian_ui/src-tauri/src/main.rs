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
struct GuardianConfig {
    mode: String,
    modules: ModulesConfig,
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
fn update_config(state: State<Arc<Mutex<SharedData>>>, mode: String, modules: ModulesConfig) -> Result<(), String> {
    let config_path = PathBuf::from("../config.yaml");
    
    {
        let mut data = state.lock().unwrap();
        data.config.mode = mode.clone();
        data.config.modules = modules.clone();
    }

    let mut new_lines = Vec::new();
    new_lines.push(format!("mode: {}", mode));
    new_lines.push("modules:".to_string());
    new_lines.push(format!("  visual: {}", modules.visual));
    new_lines.push(format!("  clipboard: {}", modules.clipboard));
    new_lines.push(format!("  network: {}", modules.network));

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
        },
        stats: IncidentStats {
            total_blocked: 0,
            sensitive_keys: 0,
            visual_alerts: 0,
        },
        whitelist_conn: wl_conn,
    }));

    tauri::Builder::default()
        .manage(shared_data)
        .setup(|app| {
            let app_handle = app.handle().clone();
            // ── 背景執行緒：輪詢 incidents.json，即時推送 AI 告警 ──────────
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
            process_control::terminate_process
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}