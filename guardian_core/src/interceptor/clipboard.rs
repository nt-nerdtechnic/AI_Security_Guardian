use arboard::Clipboard;
use regex::Regex;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    behavior_firewall: BehaviorFirewall,
}

#[derive(Debug, Serialize, Deserialize)]
struct BehaviorFirewall {
    regex_rules: std::collections::HashMap<String, String>,
}

pub struct ClipboardMonitor {
    clipboard: Arc<Mutex<Clipboard>>,
    sensitive_regexes: Vec<(String, Regex)>,
}

impl ClipboardMonitor {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let clipboard = Clipboard::new()?;
        
        let mut sensitive_regexes = Vec::new();
        
        // Load from config.yaml
        if let Ok(config_str) = fs::read_to_string("../config.yaml") {
            if let Ok(config) = serde_yaml::from_str::<Config>(&config_str) {
                for (name, pattern) in config.behavior_firewall.regex_rules {
                    if let Ok(re) = Regex::new(&pattern) {
                        sensitive_regexes.push((name, re));
                    }
                }
            }
        }

        // Fallback or hardcoded critical rules
        if sensitive_regexes.is_empty() {
             let re = Regex::new(r"(?i)(password|key|secret|token|api_key|private_key|auth_token|access_key|mnemonic|seed_phrase|credentials|passwd|passphrase)[=: ]+[a-zA-Z0-9\-_\s]{8,}")?;
             sensitive_regexes.push(("fallback".to_string(), re));
        }
        
        Ok(Self {
            clipboard: Arc::new(Mutex::new(clipboard)),
            sensitive_regexes,
        })
    }

    pub fn check_clipboard(&self) -> Option<String> {
        let mut cb = self.clipboard.lock().ok()?;
        if let Ok(text) = cb.get_text() {
            for (name, re) in &self.sensitive_regexes {
                 if re.is_match(&text) {
                     return Some(format!("[DETECTED: {}]", name));
                 }
            }
        }
        None
    }
}
