//! Ruty: AI-powered productivity launcher
//!
//! A Raycast-inspired application launcher with:
//! - Application launcher
//! - File search
//! - Clipboard history
//! - AI chat with memory (via Python backend)

mod app;
mod ui;
mod backend;
mod native;
mod hotkey;

use app::{Ruty, Message};
use iced::{window, Size};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> iced::Result {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Ruty...");

    // Initialize global hotkey (Super+Space)
    if let Err(e) = hotkey::init_hotkeys() {
        tracing::warn!("Could not register global hotkey: {} (app will still work)", e);
    }

    // Use iced::application() builder pattern for 0.13
    iced::application("Ruty", Ruty::update, Ruty::view)
        .subscription(Ruty::subscription)
        .theme(Ruty::theme)
        .window(window::Settings {
            size: Size::new(700.0, 400.0),
            position: window::Position::Centered,
            decorations: false,
            transparent: true,
            level: window::Level::AlwaysOnTop,
            resizable: true,
            ..Default::default()
        })
        .antialiasing(true)
        .run()
}
