//! User Interface handling functionality.
//!
//! This module contains all UI-related components organized by functionality:
//! - `app`: Application setup and initialization
//! - `pages`: Page-specific button handlers
//! - `tabs`: Tab navigation and management
//! - `terminal`: Terminal command execution with PTY support
//! - `selection_dialog`: Reusable multi-choice selection dialogs

pub mod app;
pub mod pages;
pub mod selection_dialog;
pub mod tabs;
pub mod terminal;

// Re-export commonly used items
pub use app::setup_application_ui;
