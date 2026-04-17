//! CyberXero Authentication Daemon
//!
//! Runs as root and executes commands on behalf of authenticated clients.

use clap::Parser;
use simple_logger::SimpleLogger;
use cyberxero_auth::run_daemon;

/// CyberXero Authentication Daemon
///
/// Runs as root and executes commands on behalf of authenticated clients.
/// The daemon creates a Unix socket for communication and maintains an
/// authenticated session to avoid repeated password prompts.
#[derive(Parser, Debug)]
#[command(name = "cyberxero-authd")]
#[command(about = "CyberXero Authentication Daemon", long_about = None)]
struct Args {
    /// User ID of the original user (required when running via pkexec)
    ///
    /// This UID is used to determine where to create the Unix socket.
    /// If not provided, the daemon will use the current user's UID.
    #[arg(short, long)]
    uid: Option<u32>,

    /// Parent process ID to monitor
    ///
    /// The daemon will shut down if this process is no longer running.
    #[arg(short = 'p', long)]
    parent_pid: Option<u32>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let log_level = if args.debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    SimpleLogger::new().with_level(log_level).init().unwrap();

    if let Err(e) = run_daemon(args.uid, args.parent_pid).await {
        eprintln!("Daemon error: {}", e);
        std::process::exit(1);
    }
}
