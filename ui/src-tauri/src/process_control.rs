use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Serialize, Deserialize)]
pub struct ProcessControlResult {
    pub success: bool,
    pub pid: u32,
    pub message: String,
}

fn incidents_path() -> PathBuf {
    std::env::current_dir()
        .ok()
        .and_then(|cwd| {
            cwd.ancestors()
                .find(|candidate| candidate.join("config.yaml").exists())
                .map(|root| root.join("logs").join("incidents.json"))
        })
        .unwrap_or_else(|| PathBuf::from("../logs/incidents.json"))
}

fn audit_process_action(pid: u32, success: bool, message: &str) {
    let path = incidents_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let event = serde_json::json!({
        "timestamp": chrono::Local::now().to_rfc3339(),
        "module": "ProcessControl",
        "severity": if success { "WARNING" } else { "ERROR" },
        "message": message,
        "metadata": { "pid": pid, "success": success }
    });
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", event);
    }
}

fn protected_pid_reason(pid: u32) -> Option<String> {
    if pid == 0 || pid == 1 {
        return Some("Cannot terminate critical system process (PID 0 or 1).".to_string());
    }
    if pid == std::process::id() {
        return Some("Cannot terminate the guardian UI process.".to_string());
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .arg("-o")
            .arg("comm=")
            .output()
        {
            let name = String::from_utf8_lossy(&output.stdout);
            let protected = [
                "kernel_task",
                "launchd",
                "WindowServer",
                "loginwindow",
                "syslogd",
            ];
            if protected.iter().any(|item| name.contains(item)) {
                return Some(format!(
                    "Cannot terminate protected process: {}",
                    name.trim()
                ));
            }
        }
    }
    None
}

#[tauri::command]
pub fn terminate_process(pid: u32) -> ProcessControlResult {
    if let Some(reason) = protected_pid_reason(pid) {
        audit_process_action(
            pid,
            false,
            &format!("Process termination denied: {}", reason),
        );
        return ProcessControlResult {
            success: false,
            pid,
            message: format!("Action denied: {}", reason),
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
            Ok(out) if out.status.success() => {
                let message = "Process terminated successfully (SIGTERM).".to_string();
                audit_process_action(pid, true, &message);
                ProcessControlResult {
                    success: true,
                    pid,
                    message,
                }
            }
            Ok(_out) => {
                // Try SIGKILL (-9) if SIGTERM doesn't work (permission denied, etc.)
                // Note: Requires root if the process is owned by another user.
                let force_output = Command::new("kill").arg("-9").arg(pid.to_string()).output();

                match force_output {
                    Ok(f_out) if f_out.status.success() => {
                        let message = "Process force-terminated (SIGKILL).".to_string();
                        audit_process_action(pid, true, &message);
                        ProcessControlResult {
                            success: true,
                            pid,
                            message,
                        }
                    }
                    Ok(f_out) => {
                        let message = format!(
                            "Failed to force-terminate. Exit code: {:?}",
                            f_out.status.code()
                        );
                        audit_process_action(pid, false, &message);
                        ProcessControlResult {
                            success: false,
                            pid,
                            message,
                        }
                    }
                    Err(e) => {
                        let message = format!("Failed to execute kill -9: {}", e);
                        audit_process_action(pid, false, &message);
                        ProcessControlResult {
                            success: false,
                            pid,
                            message,
                        }
                    }
                }
            }
            Err(e) => {
                let message = format!("Failed to execute kill command: {}", e);
                audit_process_action(pid, false, &message);
                ProcessControlResult {
                    success: false,
                    pid,
                    message,
                }
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        ProcessControlResult {
            success: false,
            pid,
            message:
                "Process termination currently only supported on macOS target inside this module."
                    .to_string(),
        }
    }
}
