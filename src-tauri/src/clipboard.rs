//! Clipboard Manager Module
//!
//! polls system clipboard and maintains a history of copied text.
//! Supports Wayland (wl-clipboard) and X11 (xclip).

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const HISTORY_LIMIT: usize = 50;
const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipboardItem {
    pub content: String,
    pub timestamp: u64,
}

pub struct ClipboardManager {
    history: Arc<Mutex<VecDeque<ClipboardItem>>>,
    last_content: Arc<Mutex<String>>,
    running: Arc<Mutex<bool>>,
}

impl ClipboardManager {
    pub fn new() -> Self {
        Self {
            history: Arc::new(Mutex::new(VecDeque::with_capacity(HISTORY_LIMIT))),
            last_content: Arc::new(Mutex::new(String::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the background polling thread
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return;
        }
        *running = true;

        let history = self.history.clone();
        let last_content = self.last_content.clone();
        let running_clone = self.running.clone();

        thread::spawn(move || {
            loop {
                if !*running_clone.lock().unwrap() {
                    break;
                }

                if let Some(content) = Self::get_system_clipboard() {
                    let mut last = last_content.lock().unwrap();
                    if *last != content && !content.trim().is_empty() {
                        *last = content.clone();
                        
                        let mut hist = history.lock().unwrap();
                        
                        // Remove if exists (to move to top)
                        if let Some(pos) = hist.iter().position(|x| x.content == content) {
                            hist.remove(pos);
                        }
                        
                        // Add to front
                        hist.push_front(ClipboardItem {
                            content,
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        });

                        // Trim history
                        if hist.len() > HISTORY_LIMIT {
                            hist.pop_back();
                        }
                    }
                }

                thread::sleep(POLL_INTERVAL);
            }
        });
    }

    /// Stop the polling thread
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }

    /// Get current history
    pub fn get_history(&self) -> Vec<ClipboardItem> {
        let hist = self.history.lock().unwrap();
        hist.iter().cloned().collect()
    }

    /// Read system clipboard
    fn get_system_clipboard() -> Option<String> {
        // Try wl-paste first (Wayland)
        if let Ok(output) = Command::new("wl-paste")
            .arg("--no-newline") // Don't add newline
            .output() 
        {
            if output.status.success() {
                // Ensure valid UTF-8
                if let Ok(text) = String::from_utf8(output.stdout) {
                    return Some(text);
                }
            }
        }

        // Try xclip (X11)
        if let Ok(output) = Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output() 
        {
            if output.status.success() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    return Some(text);
                }
            }
        }
        
        None
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}
