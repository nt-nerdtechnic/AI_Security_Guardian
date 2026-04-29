use crate::SharedData;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tauri::State;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileIntegrityAlert {
    pub file_path: String,
    pub status: String,
    pub last_modified: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BaselineActionResult {
    pub success: bool,
    pub message: String,
    pub affected: usize,
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
    let path = baseline_storage_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// 將更新後的基準表寫回磁碟
fn save_baseline(baseline: &HashMap<String, String>) {
    let path = baseline_storage_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    if let Ok(content) = serde_json::to_string_pretty(baseline) {
        fs::write(&path, content).ok();
    }
}

fn baseline_storage_path() -> PathBuf {
    if let Ok(path) = std::env::var("AEGIS_FILE_BASELINE_PATH") {
        return PathBuf::from(path);
    }
    baseline_path()
}

/// 計算單一路徑的 SHA-256
/// - 一般檔案：對檔案內容雜湊
/// - 目錄：遞迴納入相對檔名、大小與內容雜湊（偵測新增、刪除、改名與內容變更）
fn sha256_path(path: &str) -> Option<String> {
    let root = Path::new(path);
    let meta = fs::metadata(root).ok()?;
    let mut hasher = Sha256::new();

    if meta.is_dir() {
        let mut entries = Vec::new();
        collect_dir_entries(root, root, &mut entries).ok()?;
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        for (relative_path, size, digest) in entries {
            hasher.update(relative_path.as_bytes());
            hasher.update(b"\0");
            hasher.update(size.to_string().as_bytes());
            hasher.update(b"\0");
            hasher.update(digest.as_bytes());
            hasher.update(b"\n");
        }
    } else {
        hash_file_into(root, &mut hasher).ok()?;
    }

    Some(hex::encode(hasher.finalize()))
}

fn hash_file_into(path: &Path, hasher: &mut Sha256) -> std::io::Result<()> {
    let mut file = fs::File::open(path)?;
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(())
}

fn hash_file_hex(path: &Path) -> std::io::Result<String> {
    let mut hasher = Sha256::new();
    hash_file_into(path, &mut hasher)?;
    Ok(hex::encode(hasher.finalize()))
}

fn collect_dir_entries(
    root: &Path,
    current: &Path,
    out: &mut Vec<(String, u64, String)>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            collect_dir_entries(root, &path, out)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((relative, metadata.len(), hash_file_hex(&path)?));
        }
    }
    Ok(())
}

fn monitored_paths(state: &State<Arc<Mutex<SharedData>>>) -> Vec<String> {
    let mut sensitive_files = vec!["/etc/hosts".to_string(), "/etc/passwd".to_string()];

    if let Some(home) = dirs::home_dir() {
        let home = home.to_string_lossy();
        sensitive_files.push(format!("{}/.ssh/authorized_keys", home));
        sensitive_files.push(format!("{}/.bash_profile", home));
        sensitive_files.push(format!("{}/.zshrc", home));
        sensitive_files.push(format!("{}/Library/LaunchAgents", home));
    }
    sensitive_files.push("/Library/LaunchDaemons".to_string());

    if let Ok(data) = state.lock() {
        for path in &data.config.file_integrity.custom_paths {
            if !sensitive_files.contains(path) {
                sensitive_files.push(path.clone());
            }
        }
    }

    sensitive_files
}

fn compute_existing_hashes(paths: &[String]) -> HashMap<String, String> {
    paths
        .iter()
        .filter_map(|path| sha256_path(path).map(|hash| (path.clone(), hash)))
        .collect()
}

#[tauri::command]
pub fn check_file_integrity(state: State<Arc<Mutex<SharedData>>>) -> Vec<FileIntegrityAlert> {
    let mut alerts = Vec::new();
    let sensitive_files = monitored_paths(&state);

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
                                    format!(
                                        "Baseline recorded (sha256: {}...)",
                                        &current_hash[..12]
                                    ),
                                )
                            }
                            Some(known_hash) if known_hash == &current_hash => (
                                "OK".to_string(),
                                format!("Checksum verified (sha256: {}...)", &current_hash[..12]),
                            ),
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

#[tauri::command]
pub fn rebuild_file_integrity_baseline(
    state: State<Arc<Mutex<SharedData>>>,
) -> BaselineActionResult {
    let paths = monitored_paths(&state);
    let baseline = compute_existing_hashes(&paths);
    let affected = baseline.len();
    save_baseline(&baseline);

    BaselineActionResult {
        success: true,
        message: "File integrity baseline rebuilt.".to_string(),
        affected,
    }
}

#[tauri::command]
pub fn accept_file_integrity_change(file_path: String) -> BaselineActionResult {
    let Some(hash) = sha256_path(&file_path) else {
        return BaselineActionResult {
            success: false,
            message: "Cannot compute checksum for selected path.".to_string(),
            affected: 0,
        };
    };

    let mut baseline = load_baseline();
    baseline.insert(file_path, hash);
    save_baseline(&baseline);

    BaselineActionResult {
        success: true,
        message: "Selected file integrity change accepted.".to_string(),
        affected: 1,
    }
}

#[tauri::command]
pub fn export_file_integrity_report(state: State<Arc<Mutex<SharedData>>>) -> BaselineActionResult {
    let alerts = check_file_integrity(state);
    let report_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".aegis-guardian")
        .join("file_integrity_report.json");

    if let Some(parent) = report_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            return BaselineActionResult {
                success: false,
                message: format!("Failed to create report directory: {}", e),
                affected: 0,
            };
        }
    }

    match serde_json::to_string_pretty(&alerts)
        .map_err(|e| e.to_string())
        .and_then(|content| fs::write(&report_path, content).map_err(|e| e.to_string()))
    {
        Ok(_) => BaselineActionResult {
            success: true,
            message: report_path.to_string_lossy().into_owned(),
            affected: alerts.len(),
        },
        Err(e) => BaselineActionResult {
            success: false,
            message: e,
            affected: 0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::sha256_path;
    use std::fs;

    #[test]
    fn directory_hash_changes_when_nested_file_content_changes() {
        let root = std::env::temp_dir().join(format!(
            "aegis-fi-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        let file = nested.join("config.txt");
        fs::write(&file, "alpha").unwrap();

        let before = sha256_path(root.to_str().unwrap()).unwrap();
        fs::write(&file, "beta").unwrap();
        let after = sha256_path(root.to_str().unwrap()).unwrap();

        let _ = fs::remove_dir_all(&root);
        assert_ne!(before, after);
    }
}
