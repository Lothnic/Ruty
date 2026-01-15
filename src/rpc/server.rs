//! gRPC server for Ruty daemon
//!
//! Handles IPC requests from CLI to control window visibility.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tonic::{Request, Response, Status};

use super::proto::ruty_service_server::{RutyService, RutyServiceServer};
use super::proto::{Empty, WindowState};
use super::DAEMON_PORT;

/// Shared state for window visibility
#[derive(Debug)]
pub struct WindowController {
    pub visible: AtomicBool,
    pub toggle_requested: AtomicBool,
    pub quit_requested: AtomicBool,
}

impl WindowController {
    pub fn new() -> Self {
        Self {
            visible: AtomicBool::new(true),
            toggle_requested: AtomicBool::new(false),
            quit_requested: AtomicBool::new(false),
        }
    }
}

impl Default for WindowController {
    fn default() -> Self {
        Self::new()
    }
}

/// gRPC service implementation
pub struct RutyServiceImpl {
    controller: Arc<WindowController>,
}

impl RutyServiceImpl {
    pub fn new(controller: Arc<WindowController>) -> Self {
        Self { controller }
    }
}

#[tonic::async_trait]
impl RutyService for RutyServiceImpl {
    async fn ping(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        tracing::debug!("RPC: ping received");
        Ok(Response::new(Empty {}))
    }

    async fn show_window(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        tracing::info!("RPC: show_window");
        self.controller.visible.store(true, Ordering::SeqCst);
        self.controller.toggle_requested.store(true, Ordering::SeqCst);
        Ok(Response::new(Empty {}))
    }

    async fn hide_window(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        tracing::info!("RPC: hide_window");
        self.controller.visible.store(false, Ordering::SeqCst);
        self.controller.toggle_requested.store(true, Ordering::SeqCst);
        Ok(Response::new(Empty {}))
    }

    async fn toggle_window(&self, _request: Request<Empty>) -> Result<Response<WindowState>, Status> {
        let current = self.controller.visible.load(Ordering::SeqCst);
        let new_state = !current;
        tracing::info!("RPC: toggle_window {} -> {}", current, new_state);
        self.controller.visible.store(new_state, Ordering::SeqCst);
        self.controller.toggle_requested.store(true, Ordering::SeqCst);
        Ok(Response::new(WindowState { visible: new_state }))
    }

    async fn get_window_state(&self, _request: Request<Empty>) -> Result<Response<WindowState>, Status> {
        let visible = self.controller.visible.load(Ordering::SeqCst);
        Ok(Response::new(WindowState { visible }))
    }

    async fn quit(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        tracing::info!("RPC: quit");
        self.controller.quit_requested.store(true, Ordering::SeqCst);
        Ok(Response::new(Empty {}))
    }
}

/// Start the gRPC server in a background task
pub async fn start_server(controller: Arc<WindowController>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("127.0.0.1:{}", DAEMON_PORT).parse()?;
    let service = RutyServiceImpl::new(controller);

    tracing::info!("Starting gRPC server on {}", addr);

    tonic::transport::Server::builder()
        .add_service(RutyServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
