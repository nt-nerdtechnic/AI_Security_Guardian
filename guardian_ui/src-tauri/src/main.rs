// AI Security Guardian - Tauri Backend
// v1.0.7-Sprint-NetworkSentinel

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod network;
mod file_integrity;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use serde_json::Value;
use crate::network::{NetworkSentinel, ExposedPort};
use crate::file_integrity::check_file_integrity;

#[derive(Serialize, Deserialize, Debug)]
struct IncidentStats {
    total_blocked: u32,
    sensitive_keys: u32,
    visual_alerts: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ModulesConfig {
    visual: bool,
    clipboard: bool,
    network: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct GuardianConfig {
    mode: String,
    modules: ModulesConfig,
}

#[tauri::command]
fn get_exposed_ports() -> Vec<ExposedPort> {
    let sentinel = NetworkSentinel::new();
    sentinel.scan_local_ports()
}

#[tauri::command]
fn get_incident_stats() -> IncidentStats {
    let log_path = PathBuf::from("../logs/incidents.json");
    if !log_path.exists() {
        return IncidentStats { total_blocked: 0, sensitive_keys: 0, visual_alerts: 0 };
    }

    let content = fs::read_to_string(log_path).unwrap_or_else(|_| "".to_string());
    let mut total_blocked = 0;
    let mut sensitive_keys = 0;
    let mut visual_alerts = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        
        if line.starts_with('[') && line.ends_with(']') {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                if let Some(arr) = v.as_array() {
                    for item in arr {
                        let event = item.get("event").and_then(|e| e.as_str()).unwrap_or("");
                        if event.contains("blocked") {
                            total_blocked += 1;
                        }
                    }
                }
            }
            continue;
        }

        if line.starts_with('{') {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                let module = v.get("module").and_then(|m| m.as_str()).unwrap_or("");
                match module {
                    "NetworkMonitor" => total_blocked += 1,
                    "ClipboardMonitor" => sensitive_keys += 1,
                    "VisualSentry" => visual_alerts += 1,
                    "TerminalFirewall" => total_blocked += 1,
                    _ => {}
                }
            }
        }
    }

    IncidentStats { total_blocked, sensitive_keys, visual_alerts }
}

#[tauri::command]
fn get_config() -> Result<GuardianConfig, String> {
    let config_path = PathBuf::from("../config.yaml");
    let content = fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    
    let mut mode = "Silent Monitor".to_string();
    let mut visual = true;
    let mut clipboard = true;
    let mut network = true;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("mode:") {
            mode = line.replace("mode:", "").trim().to_string();
        } else if line.starts_with("visual:") {
            visual = line.replace("visual:", "").trim() == "true";
        } else if line.starts_with("clipboard:") {
            clipboard = line.replace("clipboard:", "").trim() == "true";
        } else if line.starts_with("network:") {
            network = line.replace("network:", "").trim() == "true";
        }
    }

    Ok(GuardianConfig {
        mode,
        modules: ModulesConfig { visual, clipboard, network }
    })
}

#[tauri::command]
fn update_config(mode: String, modules: ModulesConfig) -> Result<(), String> {
    let config_path = PathBuf::from("../config.yaml");
    let existing_content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    
    let mut new_lines = Vec::new();
    let mut in_modules = false;

    for line in existing_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("mode:") {
            new_lines.push(format!("mode: {}", mode));
        } else if trimmed.starts_with("modules:") {
            new_lines.push(line.to_string());
            in_modules = true;
        } else if in_modules && trimmed.starts_with("visual:") {
            new_lines.push(format!("  visual: {}", modules.visual));
        } else if in_modules && trimmed.starts_with("clipboard:") {
            new_lines.push(format!("  clipboard: {}", modules.clipboard));
        } else if in_modules && trimmed.starts_with("network:") {
            new_lines.push(format!("  network: {}", modules.network));
            in_modules = false; 
        } else {
            new_lines.push(line.to_string());
        }
    }

    fs::write(config_path, new_lines.join("\n")).map_err(|e| e.to_string())?;
    let _ = fs::write("../.reload_config", "1");
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_incident_stats,
            get_config,
            update_config,
            get_exposed_ports,
            check_file_integrity
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}