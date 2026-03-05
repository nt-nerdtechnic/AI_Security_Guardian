pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
}

impl SystemInfo {
    pub fn new() -> Self {
        use sysinfo::System;
        let mut sys = System::new_all();
        sys.refresh_all();
        
        Self {
            hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
        }
    }
}
