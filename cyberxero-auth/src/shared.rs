//! Shared utilities for client and daemon.

use anyhow::Result;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Get the socket path for the daemon.
///
/// # Arguments
///
/// * `effective_uid` - Optional user ID to use for the socket path. If None, uses the current user's UID.
///   This is used when the daemon runs as root but needs to create the socket in the
///   original user's runtime directory.
pub fn get_socket_path(effective_uid: Option<u32>) -> Result<PathBuf> {
    let uid = unsafe { libc::getuid() };
    let target_uid = effective_uid.unwrap_or(uid);

    let runtime_dir = if target_uid != 0 {
        format!("/run/user/{}", target_uid)
    } else {
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| format!("/run/user/{}", uid))
    };

    Ok(PathBuf::from(runtime_dir).join("cyberxero-authd.sock"))
}

/// Check if a process with the given PID is still running.
pub fn is_process_running(pid: u32) -> bool {
    unsafe {
        // Send signal 0 to check if process exists (doesn't actually send a signal)
        libc::kill(pid as libc::pid_t, 0) == 0
    }
}

/// Check if the daemon is running by checking if the socket exists.
pub fn is_daemon_running() -> bool {
    get_socket_path(None)
        .map(|path| path.exists())
        .unwrap_or(false)
}

/// Wait for the daemon socket to become available.
///
/// Polls the socket path at regular intervals until it appears or the timeout is reached.
/// This is used when starting the daemon to wait for it to be ready to accept connections.
///
/// # Arguments
///
/// * `timeout` - Maximum time to wait for the socket to appear
/// * `poll_interval` - How often to check for the socket (default: 50ms)
///
/// # Returns
///
/// * `Ok(())` if the socket appeared within the timeout
/// * `Err` if the timeout was reached or an error occurred
pub fn wait_for_socket(timeout: Duration, poll_interval: Duration) -> Result<()> {
    let socket_path = get_socket_path(None)?;
    let start = Instant::now();

    loop {
        if socket_path.exists() {
            return Ok(());
        }

        if start.elapsed() >= timeout {
            anyhow::bail!(
                "Socket did not appear within {:?} at {:?}",
                timeout,
                socket_path
            );
        }

        std::thread::sleep(poll_interval);
    }
}
