//! CyberXero Authentication Daemon
//!
//! Provides a daemon-based privilege escalation system that maintains
//! an authenticated session to avoid repeated password prompts.

pub mod client;
pub mod daemon;
pub mod protocol;
pub mod protocol_io;
pub mod shared;
pub mod utils;

pub use client::Client;
pub use daemon::run_daemon;
pub use shared::{get_socket_path, is_daemon_running, wait_for_socket};
