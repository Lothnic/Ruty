//! Theme configuration

use iced::Color;

/// Dark theme colors
pub struct DarkTheme;

impl DarkTheme {
    pub const BACKGROUND: Color = Color::from_rgb(0.1, 0.1, 0.12);
    pub const SURFACE: Color = Color::from_rgb(0.15, 0.15, 0.18);
    pub const PRIMARY: Color = Color::from_rgb(0.4, 0.6, 1.0);
    pub const TEXT: Color = Color::from_rgb(0.9, 0.9, 0.9);
    pub const TEXT_MUTED: Color = Color::from_rgb(0.6, 0.6, 0.6);
    pub const SELECTION: Color = Color::from_rgb(0.2, 0.2, 0.25);
}
