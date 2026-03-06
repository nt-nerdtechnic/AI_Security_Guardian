use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct QuarantineManager {
    quarantine_dir: PathBuf,
}

impl QuarantineManager {
    pub fn new() -> Self {
        // Construct the quarantine directory path
        let quarantine_dir = dirs::desktop_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("Aegis_Guardian")
            .join("quarantine");

        // Ensure the directory exists
        if !quarantine_dir.exists() {
            let _ = fs::create_dir_all(&quarantine_dir);
        }

        Self { quarantine_dir }
    }

    /// Safely moves a file to the quarantine directory without deleting it.
    /// This adheres to the STRICT OVERSIGHT mandate (NO AUTO-DELETE, NO 'rm').
    pub fn move_to_quarantine(&self, file_path: &str) -> io::Result<String> {
        let source_path = Path::new(file_path);
        if !source_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "File does not exist",
            ));
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
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_move_to_quarantine() {
        let manager = QuarantineManager::new();
        let path = "/tmp/test_isolation_file.txt";
        {
            let mut file = File::create(path).unwrap();
            writeln!(file, "Malicious content").unwrap();
        }

        let result = manager.move_to_quarantine(path);
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
