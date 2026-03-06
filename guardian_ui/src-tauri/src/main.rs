// Aegis Guardian - Tauri Backend
// v1.0.7-Sprint-NetworkSentinel

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod network;
mod file_integrity;
pub mod quarantine;
pub mod process_control;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::State;
use crate::network::{NetworkSentinel, ExposedPort};
use crate::file_integrity::check_file_integrity;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct IncidentStats {
    total_blocked: u32,
    sensitive_keys: u32,
    visual_alerts: u32,
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
}

#[tauri::command]
fn get_exposed_ports() -> Vec<ExposedPort> {
    let sentinel = NetworkSentinel::new();
    sentinel.scan_local_ports()
}

#[tauri::command]
fn get_incident_stats(state: State<Arc<Mutex<SharedData>>>) -> IncidentStats {
    let data = state.lock().unwrap();
    data.stats.clone()
}

#[tauri::command]
fn get_config(state: State<Arc<Mutex<SharedData>>>) -> Result<GuardianConfig, String> {
    let data = state.lock().unwrap();
    Ok(data.config.clone())
}

#[tauri::command]
fn update_config(state: State<Arc<Mutex<SharedData>>>, mode: String, modules: ModulesConfig) -> Result<(), String> {
    let config_path = PathBuf::from("../config.yaml");
    
    // Update the shared state first
    {
        let mut data = state.lock().unwrap();
        data.config.mode = mode.clone();
        data.config.modules = modules.clone();
    }

    // Then write to file
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

fn main() {
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
    }));

    // Start a background thread to simulate gathering stats
    let data_clone = shared_data.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));
            let mut data = data_clone.lock().unwrap();
            data.stats.total_blocked += 1;
            if data.stats.total_blocked % 5 == 0 {
                data.stats.sensitive_keys += 1;
            }
        }
    });

    tauri::Builder::default()
        .manage(shared_data)
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_config,
            get_incident_stats,
            get_exposed_ports, // This was already here, but the example showed network::get_exposed_ports
            check_file_integrity, // This was already here, but the example showed file_integrity::check_file_integrity
            quarantine::move_to_quarantine,
            process_control::terminate_process
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}