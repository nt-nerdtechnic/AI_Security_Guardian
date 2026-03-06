use std::fs;
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct QuarantineResult {
    pub success: bool,
    pub original_path: String,
    pub quarantine_path: Option<String>,
    pub message: String,
}

#[tauri::command]
pub fn move_to_quarantine(file_path: String) -> QuarantineResult {
    let source_path = Path::new(&file_path);
    if !source_path.exists() {
        return QuarantineResult {
            success: false,
            original_path: file_path.clone(),
            quarantine_path: None,
            message: "File does not exist.".to_string(),
        };
    }

    // Resolve user's home directory to find the quarantine folder
    let home_dir = match std::env::var("HOME") {
        Ok(val) => val,
        Err(_) => {
            return QuarantineResult {
                success: false,
                original_path: file_path.clone(),
                quarantine_path: None,
                message: "Could not determine HOME directory.".to_string(),
            };
        }
    };

    let quarantine_dir = PathBuf::from(home_dir).join("Desktop/AI_Security_Guardian/quarantine");

    // Ensure quarantine directory exists
    if !quarantine_dir.exists() {
         if let Err(e) = fs::create_dir_all(&quarantine_dir) {
            return QuarantineResult {
                success: false,
                original_path: file_path.clone(),
                quarantine_path: None,
                message: format!("Failed to create quarantine directory: {}", e),
            };
         }
    }

    let file_name = match source_path.file_name() {
        Some(name) => name,
        None => {
             return QuarantineResult {
                success: false,
                original_path: file_path.clone(),
                quarantine_path: None,
                message: "Invalid file path (no filename).".to_string(),
            };
        }
    };

    // Construct destination. Handle potential collisions by appending a timestamp
    let mut dest_path = quarantine_dir.join(file_name);
    if dest_path.exists() {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let new_file_name = format!("{}_{}", timestamp, file_name.to_string_lossy());
        dest_path = quarantine_dir.join(new_file_name);
    }

    // Perform the safe move
    match fs::rename(&source_path, &dest_path) {
        Ok(_) => QuarantineResult {
            success: true,
            original_path: file_path,
            quarantine_path: Some(dest_path.to_string_lossy().into_owned()),
            message: "File successfully moved to quarantine.".to_string(),
        },
        Err(e) => {
            // Also try a copy-and-remove fallback if they are on different mount points
             if let Ok(_) = fs::copy(&source_path, &dest_path) {
                if let Ok(_) = fs::remove_file(&source_path) {
                     return QuarantineResult {
                        success: true,
                        original_path: file_path,
                        quarantine_path: Some(dest_path.to_string_lossy().into_owned()),
                        message: "File successfully moved (copy+delete fallback) to quarantine.".to_string(),
                    };
                }
             }

             QuarantineResult {
                success: false,
                original_path: file_path,
                quarantine_path: None,
                message: format!("Failed to move file to quarantine: {}", e),
            }
        }
    }
}
