pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
}

impl SystemInfo {
    pub fn new() -> Self {
        use sysinfo::{System, SystemExt};
        let mut sys = System::new_all();
        sys.refresh_all();
        
        Self {
            hostname: sys.host_name().unwrap_or_else(|| "Unknown".to_string()),
            os_name: sys.name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: sys.os_version().unwrap_or_else(|| "Unknown".to_string()),
        }
    }
}
