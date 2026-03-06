use std::fs::File;
use std::io::{self, Read};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct FileIntegrityMonitor {
    baseline_hashes: HashMap<String, String>,
    quarantine_dir: PathBuf,
}

impl FileIntegrityMonitor {
    pub fn new() -> Self {
        let quarantine_dir = dirs::desktop_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("AI_Security_Guardian")
            .join("quarantine");
        
        if !quarantine_dir.exists() {
            let _ = fs::create_dir_all(&quarantine_dir);
        }

        Self {
            baseline_hashes: HashMap::new(),
            quarantine_dir,
        }
    }

    pub fn add_to_baseline(&mut self, path: &str) -> io::Result<()> {
        if !Path::new(path).exists() {
             return Err(io::Error::new(io::ErrorKind::NotFound, format!("File not found: {}", path)));
        }
        let hash = self.calculate_hash(path)?;
        self.baseline_hashes.insert(path.to_string(), hash);
        Ok(())
    }

    pub fn scan_directory_recursive(&mut self, dir_path: &str) -> io::Result<Vec<String>> {
        let mut new_files = Vec::new();
        self.walk_dir(Path::new(dir_path), &mut new_files)?;
        Ok(new_files)
    }

    fn walk_dir(&self, dir: &Path, new_files: &mut Vec<String>) -> io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    // Skip hidden directories like .git or target
                    if let Some(name) = path.file_name() {
                        let name_str = name.to_string_lossy();
                        if name_str.starts_with('.') || name_str == "target" || name_str == "venv" {
                            continue;
                        }
                    }
                    self.walk_dir(&path, new_files)?;
                } else if path.is_file() {
                    let path_str = path.to_string_lossy().to_string();
                    if !self.baseline_hashes.contains_key(&path_str) {
                        new_files.push(path_str);
                    }
                }
            }
        }
        Ok(())
    }

    fn calculate_hash(&self, path: &str) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 1024];

        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 { break; }
            hasher.update(&buffer[..count]);
        }

        Ok(hex::encode(hasher.finalize()))
    }

    pub fn check_integrity(&self) -> Vec<String> {
        let mut alerts = Vec::new();

        for (path, baseline) in &self.baseline_hashes {
            match self.calculate_hash(path) {
                Ok(current_hash) => {
                    if current_hash != *baseline {
                        alerts.push(format!("File modified: {} (Expected: {}, Got: {})", path, baseline, current_hash));
                    }
                }
                Err(e) => {
                    alerts.push(format!("Failed to access file: {} ({})", path, e));
                }
            }
        }

        alerts
    }

    /// Safely moves a file to the quarantine directory without deleting it.
    /// This adheres to the STRICT OVERSIGHT mandate (NO AUTO-DELETE, NO 'rm').
    pub fn move_to_quarantine(&self, file_path: &str) -> io::Result<String> {
        let source_path = Path::new(file_path);
        if !source_path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "File does not exist"));
        }

        let file_name = source_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
            
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Append timestamp to avoid overwriting existing quarantined files
        let safe_name = format!("{}_{}", timestamp, file_name);
        let dest_path = self.quarantine_dir.join(&safe_name);

        fs::rename(source_path, &dest_path)?;

        Ok(dest_path.to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::fs::File;

    #[test]
    fn test_file_integrity() {
        let mut monitor = FileIntegrityMonitor::new();
        let path = "/tmp/test_integrity_file.txt";
        {
            let mut file = File::create(path).unwrap();
            writeln!(file, "Hello, world!").unwrap();
        }
        monitor.add_to_baseline(path).unwrap();

        // Initial check should be fine
        assert!(monitor.check_integrity().is_empty());

        // Modify file
        {
            let mut file = File::create(path).unwrap();
            writeln!(file, "Goodbye, world!").unwrap();
        }
        let alerts = monitor.check_integrity();
        assert!(!alerts.is_empty());
        assert!(alerts[0].contains("File modified"));

        // Clean up
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_move_to_quarantine() {
        let monitor = FileIntegrityMonitor::new();
        let path = "/tmp/test_quarantine_file.txt";
        {
            let mut file = File::create(path).unwrap();
            writeln!(file, "Malicious content").unwrap();
        }

        let result = monitor.move_to_quarantine(path);
        assert!(result.is_ok());

        // File should no longer exist at the original path
        assert!(!Path::new(path).exists());

        // File should exist in the quarantine dir
        let quarantined_path = result.unwrap();
        assert!(Path::new(&quarantined_path).exists());

        // Clean up the quarantined file
        let _ = std::fs::remove_file(&quarantined_path);
    }
}
