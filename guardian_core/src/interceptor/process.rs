use sysinfo::{System, SystemExt, ProcessExt, Pid, Signal};

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
            return process.kill_with_signal(Signal::Kill).unwrap_or(false);
        }
        false
    }

    /// Kill process by exact name
    pub fn kill_by_name(&mut self, name: &str) -> usize {
        self.sys.refresh_all();
        let mut count = 0;
        for (pid, process) in self.sys.processes() {
            if process.name() == name {
                if process.kill_with_signal(Signal::Kill).unwrap_or(false) {
                    println!("⚔️ [Interceptor] Terminated process {} (PID: {})", name, pid);
                    count += 1;
                }
            }
        }
        count
    }
}
