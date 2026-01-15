//! Python backend sidecar management

use std::process::{Child, Command, Stdio};
use std::path::PathBuf;

/// Manages the Python backend process
pub struct Sidecar {
    process: Option<Child>,
    binary_path: PathBuf,
}

impl Sidecar {
    pub fn new() -> Self {
        // Look for ruty-backend in common locations
        let binary_path = Self::find_backend_binary();
        Self {
            process: None,
            binary_path,
        }
    }

    fn find_backend_binary() -> PathBuf {
        // Check these locations in order:
        // 1. Same directory as main binary
        // 2. /usr/bin/ruty-backend (installed)
        // 3. ./dist/ruty-backend (development)
        
        let candidates = [
            PathBuf::from("/usr/bin/ruty-backend"),
            PathBuf::from("./dist/ruty-backend"),
            PathBuf::from("./ruty-backend"),
        ];

        for path in candidates {
            if path.exists() {
                return path;
            }
        }

        // Fallback - assume it's in PATH
        PathBuf::from("ruty-backend")
    }

    /// Start the Python backend
    pub fn start(&mut self) -> Result<(), String> {
        if self.process.is_some() {
            return Ok(()); // Already running
        }

        let child = Command::new(&self.binary_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start backend: {}", e))?;

        self.process = Some(child);
        tracing::info!("Started Python backend from {:?}", self.binary_path);
        Ok(())
    }

    /// Stop the Python backend
    pub fn stop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            tracing::info!("Stopped Python backend");
        }
    }

    /// Check if backend is running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut process) = self.process {
            match process.try_wait() {
                Ok(Some(_)) => {
                    // Process exited
                    self.process = None;
                    false
                }
                Ok(None) => true, // Still running
                Err(_) => false,
            }
        } else {
            false
        }
    }
}

impl Drop for Sidecar {
    fn drop(&mut self) {
        self.stop();
    }
}
