//! Core functionality and business logic.
//!
//! This module contains:
//! - `context`: Application state and context
//! - `helpers`: Utility functions for package checking
//! - `system_check`: System dependency validation

pub mod context;
pub mod helpers;
pub mod system_check;

// Re-export commonly used items
pub use context::{AppContext, UiComponents};
pub use helpers::*;
pub use system_check::check_system_requirements;

