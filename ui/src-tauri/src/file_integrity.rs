use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use sha2::{Digest, Sha256};
use tauri::State;
use crate::SharedData;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileIntegrityAlert {
    pub file_path: String,
    pub status: String,
    pub last_modified: String,
    pub message: String,
}

/// 基準 checksum 的儲存路徑：~/.aegis-guardian/file_checksums.json
fn baseline_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".aegis-guardian")
        .join("file_checksums.json")
}

/// 從磁碟讀取已知 checksum 基準表（path → sha256 hex）
fn load_baseline() -> HashMap<String, String> {
    let path = baseline_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// 將更新後的基準表寫回磁碟
fn save_baseline(baseline: &HashMap<String, String>) {
    let path = baseline_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    if let Ok(content) = serde_json::to_string_pretty(baseline) {
        fs::write(&path, content).ok();
    }
}

/// 計算單一路徑的 SHA-256
/// - 一般檔案：對檔案內容雜湊
/// - 目錄：對排序後的子項目名稱列表雜湊（偵測新增/刪除）
fn sha256_path(path: &str) -> Option<String> {
    let meta = fs::metadata(path).ok()?;
    let mut hasher = Sha256::new();

    if meta.is_dir() {
        let mut entries: Vec<String> = fs::read_dir(path)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        entries.sort();
        for entry in &entries {
            hasher.update(entry.as_bytes());
            hasher.update(b"\n");
        }
    } else {
        let mut file = fs::File::open(path).ok()?;
        let mut buf = [0u8; 65536];
        loop {
            let n = file.read(&mut buf).ok()?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
    }

    Some(hex::encode(hasher.finalize()))
}

#[tauri::command]
pub fn check_file_integrity(state: State<Arc<Mutex<SharedData>>>) -> Vec<FileIntegrityAlert> {
    let mut alerts = Vec::new();
    let mut sensitive_files = vec![
        "/etc/hosts".to_string(),
        "/etc/passwd".to_string(),
    ];

    // 如果能取得 HOME 目錄，則加入 SSH key、啟動項目及環境變數監控
    if let Ok(home) = std::env::var("HOME") {
        sensitive_files.push(format!("{}/.ssh/authorized_keys", home));
        sensitive_files.push(format!("{}/.bash_profile", home));
        sensitive_files.push(format!("{}/.zshrc", home));
        sensitive_files.push(format!("{}/Library/LaunchAgents", home));
    }
    sensitive_files.push("/Library/LaunchDaemons".to_string());

    // 讀取設定檔內的自訂路徑
    if let Ok(data) = state.lock() {
        for path in &data.config.file_integrity.custom_paths {
            if !sensitive_files.contains(path) {
                sensitive_files.push(path.clone());
            }
        }
    }

    let mut baseline = load_baseline();
    let mut baseline_updated = false;

    for file in sensitive_files {
        match fs::metadata(&file) {
            Ok(metadata) => {
                let modified = metadata
                    .modified()
                    .unwrap_or_else(|_| SystemTime::now())
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let (status, message) = match sha256_path(&file) {
                    Some(current_hash) => {
                        match baseline.get(&file) {
                            None => {
                                // 首次記錄：建立基準，不視為威脅
                                baseline.insert(file.clone(), current_hash.clone());
                                baseline_updated = true;
                                (
                                    "OK".to_string(),
                                    format!("Baseline recorded (sha256: {}...)", &current_hash[..12]),
                                )
                            }
                            Some(known_hash) if known_hash == &current_hash => {
                                (
                                    "OK".to_string(),
                                    format!("Checksum verified (sha256: {}...)", &current_hash[..12]),
                                )
                            }
                            Some(known_hash) => {
                                // checksum 不符 → 真實異動，觸發 WARNING
                                (
                                    "WARNING".to_string(),
                                    format!(
                                        "Checksum mismatch! expected {}... got {}...",
                                        &known_hash[..12],
                                        &current_hash[..12]
                                    ),
                                )
                            }
                        }
                    }
                    None => {
                        // 無法讀取內容（例如權限不足），僅以 mtime 輔助提示
                        (
                            "INFO".to_string(),
                            "Cannot compute checksum (permission denied)".to_string(),
                        )
                    }
                };

                alerts.push(FileIntegrityAlert {
                    file_path: file.clone(),
                    status,
                    last_modified: modified.to_string(),
                    message,
                });
            }
            Err(_) => {
                alerts.push(FileIntegrityAlert {
                    file_path: file.clone(),
                    status: "INFO".to_string(),
                    last_modified: "N/A".to_string(),
                    message: "File not found or inaccessible".to_string(),
                });
            }
        }
    }

    if baseline_updated {
        save_baseline(&baseline);
    }

    alerts
}
