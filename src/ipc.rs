//! IPC module for Ruty toggle functionality
//!
//! Uses a Unix socket for communication between CLI and running instance.
//! This allows "ruty toggle" to work on Wayland where global hotkeys don't work.

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Flag to signal the main app that a toggle was requested
pub static TOGGLE_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Flag to signal the main app to close
pub static CLOSE_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Get the IPC socket path
fn socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("ruty.sock")
}

/// Start the IPC server in a background thread
pub fn start_server() {
    let path = socket_path();
    
    // Remove old socket if exists
    let _ = std::fs::remove_file(&path);
    
    std::thread::spawn(move || {
        let listener = match UnixListener::bind(&path) {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind IPC socket: {}", e);
                return;
            }
        };
        
        tracing::info!("IPC server listening at {:?}", path);
        
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buf = [0u8; 32];
                    if let Ok(n) = stream.read(&mut buf) {
                        let cmd = String::from_utf8_lossy(&buf[..n]);
                        let cmd = cmd.trim();
                        
                        match cmd {
                            "toggle" => {
                                tracing::info!("IPC: toggle command received");
                                TOGGLE_REQUESTED.store(true, Ordering::SeqCst);
                                let _ = stream.write_all(b"ok");
                            }
                            "close" => {
                                tracing::info!("IPC: close command received");
                                CLOSE_REQUESTED.store(true, Ordering::SeqCst);
                                let _ = stream.write_all(b"ok");
                            }
                            _ => {
                                tracing::warn!("IPC: unknown command: {}", cmd);
                                let _ = stream.write_all(b"error");
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("IPC connection error: {}", e);
                }
            }
        }
    });
}

/// Try to toggle an existing instance
pub fn try_toggle_existing() -> bool {
    send_command("toggle")
}

/// Try to close an existing instance
pub fn try_close_existing() -> bool {
    send_command("close")
}

/// Send a command to a running instance
fn send_command(cmd: &str) -> bool {
    let path = socket_path();
    
    match UnixStream::connect(&path) {
        Ok(mut stream) => {
            stream.set_write_timeout(Some(Duration::from_secs(1))).ok();
            stream.set_read_timeout(Some(Duration::from_secs(1))).ok();
            
            if stream.write_all(cmd.as_bytes()).is_err() {
                return false;
            }
            
            let mut response = [0u8; 8];
            if let Ok(n) = stream.read(&mut response) {
                let resp = String::from_utf8_lossy(&response[..n]);
                return resp.trim() == "ok";
            }
            false
        }
        Err(_) => false,
    }
}

/// Check if toggle was requested (called from app tick)
pub fn check_toggle_requested() -> bool {
    TOGGLE_REQUESTED.swap(false, Ordering::SeqCst)
}

/// Check if close was requested (called from app tick)
pub fn check_close_requested() -> bool {
    CLOSE_REQUESTED.swap(false, Ordering::SeqCst)
}
