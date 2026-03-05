use sysinfo::{System, Pid, Signal};

pub struct ProcessInterceptor {
    sys: System,
}

impl ProcessInterceptor {
    pub fn new() -> Self {
        Self {
            sys: System::new_all(),
        }
    }

    /// Kill a process by PID
    pub fn terminate(&mut self, pid: u32) -> bool {
        self.sys.refresh_process(Pid::from(pid as usize));
        if let Some(process) = self.sys.process(Pid::from(pid as usize)) {
            println!("⚔️ [Interceptor] Attempting to kill PID {}: {}", pid, process.name());
            return process.kill_with(Signal::Kill).unwrap_or(false);
        }
        false
    }

    pub fn kill_by_name(&mut self, name: &str) -> usize {
        self.sys.refresh_all();
        let mut count = 0;
        for (pid, process) in self.sys.processes() {
            if process.name() == name {
                if process.kill_with(Signal::Kill).unwrap_or(false) {
                    println!("⚔️ [Interceptor] Terminated process {} (PID: {})", name, pid);
                    count += 1;
                }
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_kill_by_name() {
        // Create a uniquely named dummy process by copying sleep (to avoid killing real user sleeps)
        let dummy_name = "guardian_dummy_test_proc";
        let dummy_path = format!("/tmp/{}", dummy_name);
        
        let _ = std::fs::copy("/bin/sleep", &dummy_path);

        // Spawn the process
        let mut child = Command::new(&dummy_path)
            .arg("10")
            .spawn()
            .expect("Failed to spawn dummy process");

        // Give it a tiny bit of time to start and register in sysinfo
        thread::sleep(Duration::from_millis(100));

        let mut interceptor = ProcessInterceptor::new();
        let killed_count = interceptor.kill_by_name(dummy_name);

        assert!(killed_count > 0, "kill_by_name should terminate at least one process");

        // Wait a bit to ensure the process exits, and wait() should fail or return an exit status rather than matching active
        // Actually, we can just check self.sys.processes() again or rely on kill_count
        let exit_status = child.wait().expect("Failed to wait on child");
        assert!(!exit_status.success(), "Process should have been killed, not exited successfully");

        // Cleanup
        let _ = std::fs::remove_file(&dummy_path);
    }
}
