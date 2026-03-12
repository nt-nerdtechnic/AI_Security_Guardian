use serde::{Deserialize, Serialize};
use std::fs;
use std::time::SystemTime;
use tauri::State;
use std::sync::{Arc, Mutex};
use crate::SharedData;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileIntegrityAlert {
    pub file_path: String,
    pub status: String,
    pub last_modified: String,
    pub message: String,
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

    for file in sensitive_files {
        match fs::metadata(&file) {
            Ok(metadata) => {
                let modified = metadata
                    .modified()
                    .unwrap_or_else(|_| SystemTime::now())
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let current_time = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                // 簡單判斷：如果在最近 24 小時（86400秒）內被修改過
                let status = if current_time > modified && current_time - modified < 86400 {
                    "WARNING".to_string()
                } else {
                    "OK".to_string()
                };

                let message = if status == "WARNING" {
                    "Recent modifications detected".to_string()
                } else {
                    "No recent changes".to_string()
                };

                alerts.push(FileIntegrityAlert {
                    file_path: file.clone(),
                    status,
                    last_modified: modified.to_string(),
                    message,
                });
            }
            Err(_) => {
                // 若找不到檔案，或是沒有權限
                alerts.push(FileIntegrityAlert {
                    file_path: file.clone(),
                    status: "INFO".to_string(),
                    last_modified: "N/A".to_string(),
                    message: "File not found or inaccessible".to_string(),
                });
            }
        }
    }

    alerts
}
