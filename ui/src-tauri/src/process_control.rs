use serde::{Serialize, Deserialize};
use std::process::Command;

#[derive(Serialize, Deserialize)]
pub struct ProcessControlResult {
    pub success: bool,
    pub pid: u32,
    pub message: String,
}

#[tauri::command]
pub fn terminate_process(pid: u32) -> ProcessControlResult {
    // Basic validation to prevent killing critical system processes casually.
    // In a real production system, this would be much more robust.
    if pid == 0 || pid == 1 {
         return ProcessControlResult {
            success: false,
            pid,
            message: "Action denied: Cannot terminate critical system process (PID 0 or 1).".to_string(),
        };
    }

    #[cfg(target_os = "macos")]
    {
        // Use 'kill -9' or 'kill -15' (SIGTERM). We use SIGTERM (15) for a safer shutdown first.
        let output = Command::new("kill")
            .arg("-15")
            .arg(pid.to_string())
            .output();

        match output {
            Ok(out) if out.status.success() => ProcessControlResult {
                success: true,
                pid,
                message: "Process terminated successfully (SIGTERM).".to_string(),
            },
            Ok(out) => {
                 // Try SIGKILL (-9) if SIGTERM doesn't work (permission denied, etc.)
                 // Note: Requires root if the process is owned by another user.
                 let force_output = Command::new("kill")
                    .arg("-9")
                    .arg(pid.to_string())
                    .output();
                 
                 match force_output {
                     Ok(f_out) if f_out.status.success() => ProcessControlResult {
                        success: true,
                        pid,
                        message: "Process force-terminated (SIGKILL).".to_string(),
                    },
                    Ok(f_out) => ProcessControlResult {
                        success: false,
                        pid,
                        message: format!("Failed to force-terminate. Exit code: {:?}", f_out.status.code()),
                    },
                     Err(e) => ProcessControlResult {
                        success: false,
                        pid,
                        message: format!("Failed to execute kill -9: {}", e),
                    }
                 }
            },
            Err(e) => ProcessControlResult {
                success: false,
                pid,
                message: format!("Failed to execute kill command: {}", e),
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
         ProcessControlResult {
            success: false,
            pid,
            message: "Process termination currently only supported on macOS target inside this module.".to_string(),
        }
    }
}
