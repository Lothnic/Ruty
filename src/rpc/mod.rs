//! RPC module for Ruty daemon IPC
//!
//! Implements Gauntlet-style gRPC communication between CLI and daemon.

pub mod server;
pub mod client;

// Include generated protobuf code
pub mod proto {
    tonic::include_proto!("ruty");
}

/// Default port for Ruty daemon
pub const DAEMON_PORT: u16 = 42321;

/// Default address for Ruty daemon
pub fn daemon_addr() -> String {
    format!("http://127.0.0.1:{}", DAEMON_PORT)
}
