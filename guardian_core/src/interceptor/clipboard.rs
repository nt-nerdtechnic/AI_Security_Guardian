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
        // Enhanced regex to catch:
        // 1. Common sensitive keywords with values (mnemonic/seed added)
        // 2. SSH private key headers
        // 3. Credit card numbers
        // 4. OpenAI / AWS / Stripe keys
        // 5. Potential email addresses
        let sensitive_regex = Regex::new(r"(?i)(password|key|secret|token|api_key|private_key|auth_token|access_key|mnemonic|seed_phrase)[=: ]+[a-zA-Z0-9\-_\s]{8,}|-----BEGIN\ (RSA|OPENSSH|DSA|EC)\ PRIVATE\ KEY-----|session_key[=: ]+[a-zA-Z0-9\-_]{16,}|\b(?:\d[ -]*?){13,16}\b|\bsk-[a-zA-Z0-9]{20,}\b|\bAKIA[a-zA-Z0-9]{16}\b|\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")?;
        
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitive_regex() {
        let monitor = ClipboardMonitor::new().unwrap();
        
        assert!(monitor.sensitive_regex.is_match("my password=secret123"));
        assert!(monitor.sensitive_regex.is_match("API_KEY: abcdef123456"));
        assert!(monitor.sensitive_regex.is_match("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(monitor.sensitive_regex.is_match("user@example.com"));
        assert!(monitor.sensitive_regex.is_match("session_key = unique-token-456"));
        assert!(monitor.sensitive_regex.is_match("auth_token = high-security-token-789"));
        assert!(monitor.sensitive_regex.is_match("mnemonic: word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11 word12"));
        
        assert!(!monitor.sensitive_regex.is_match("just some normal text"));
        assert!(!monitor.sensitive_regex.is_match("short: 123"));
    }
}