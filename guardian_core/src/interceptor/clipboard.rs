use arboard::Clipboard;
use regex::Regex;
use std::sync::{Arc, Mutex};

pub struct ClipboardMonitor {
    clipboard: Arc<Mutex<Clipboard>>,
    sensitive_regex: Regex,
}

impl ClipboardMonitor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let clipboard = Clipboard::new()?;
        let sensitive_regex = Regex::new(r"(?i)(password|key|secret|token|api_key|private_key)[=: ]+[a-zA-Z0-9\-_]{8,}")?;
        
        Ok(Self {
            clipboard: Arc::new(Mutex::new(clipboard)),
            sensitive_regex,
        })
    }

    pub fn check_clipboard(&self) -> Option<String> {
        let mut cb = self.clipboard.lock().ok()?;
        if let Ok(text) = cb.get_text() {
            if self.sensitive_regex.is_match(&text) {
                // Return a redacted version or just a confirmation
                return Some("[REDACTED SENSITIVE DATA]".to_string());
            }
        }
        None
    }
}
