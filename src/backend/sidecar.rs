//! Python backend sidecar management
//!
//! Spawns and manages the Python FastAPI backend process.

use std::process::{Child, Command, Stdio};
use std::path::PathBuf;
use std::time::Duration;

/// Backend server port
pub const BACKEND_PORT: u16 = 3847;

/// Backend server URL
pub fn backend_url() -> String {
    format!("http://127.0.0.1:{}", BACKEND_PORT)
}

/// Manages the Python backend process
pub struct Sidecar {
    process: Option<Child>,
    project_dir: PathBuf,
}

impl Sidecar {
    pub fn new() -> Self {
        // Get project directory (where ruty/ python module lives)
        let project_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        
        Self {
            process: None,
            project_dir,
        }
    }
    
    /// Set project directory explicitly
    pub fn with_project_dir(mut self, dir: PathBuf) -> Self {
        self.project_dir = dir;
        self
    }

    /// Start the Python backend
    pub fn start(&mut self) -> Result<(), String> {
        if self.process.is_some() {
            return Ok(()); // Already running
        }

        // Try different ways to start the backend
        let result = self.try_start_python_module()
            .or_else(|_| self.try_start_binary());
        
        match result {
            Ok(child) => {
                self.process = Some(child);
                tracing::info!("Started Python backend");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    
    /// Try starting via python -m ruty.server
    fn try_start_python_module(&self) -> Result<Child, String> {
        Command::new("python")
            .args(["-m", "ruty.server"])
            .current_dir(&self.project_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start python module: {}", e))
    }
    
    /// Try starting bundled binary (PyInstaller-built)
    fn try_start_binary(&self) -> Result<Child, String> {
        let candidates = [
            self.project_dir.join("ruty-backend"),
            PathBuf::from("/usr/bin/ruty-backend"),
            PathBuf::from("./dist/ruty-backend"),
        ];
        
        for path in candidates {
            if path.exists() {
                return Command::new(&path)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .map_err(|e| format!("Failed to start binary: {}", e));
            }
        }
        
        Err("No backend binary found".to_string())
    }

    /// Stop the Python backend
    pub fn stop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
            tracing::info!("Stopped Python backend");
        }
    }

    /// Check if backend is running (process check)
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut process) = self.process {
            match process.try_wait() {
                Ok(Some(_)) => {
                    self.process = None;
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }
    
    /// Health check - try to connect to backend
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/health", backend_url());
        match reqwest::get(&url).await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
    
    /// Start and wait for backend to be ready
    pub async fn start_and_wait(&mut self, timeout: Duration) -> Result<(), String> {
        self.start()?;
        
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if self.health_check().await {
                tracing::info!("Backend is ready");
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        
        Err("Backend failed to start within timeout".to_string())
    }
}

impl Default for Sidecar {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Sidecar {
    fn drop(&mut self) {
        self.stop();
    }
}
