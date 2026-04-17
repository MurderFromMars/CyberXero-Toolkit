//! CyberXero Authentication Client
//!
//! Command-line client for testing the authentication daemon.

use clap::Parser;
use cyberxero_auth::shared::is_daemon_running;
use cyberxero_auth::Client;

#[derive(Parser, Debug)]
#[command(name = "cyberxero-auth")]
#[command(about = "CyberXero Authentication Client", long_about = None)]
struct Args {
    /// Environment variables to set (KEY=VALUE)
    #[arg(short, long)]
    env: Vec<String>,

    /// The program to execute
    program: String,

    /// Arguments for the program
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[tokio::main]
async fn main() {
    if !is_daemon_running() {
        eprintln!("Error: cyberxero-auth daemon is not running");
        std::process::exit(1);
    }

    let mut args = Args::parse();

    // Capture current environment variables
    let mut env_vars = Vec::new();
    for (key, value) in std::env::vars() {
        env_vars.push(format!("{}={}", key, value));
    }

    // Prepend inherited environment variables so explicit --env overrides them
    env_vars.extend(args.env);
    args.env = env_vars;

    let mut client = match Client::new().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to daemon: {}", e);
            std::process::exit(1);
        }
    };

    let exit_code = match client
        .execute(
            &args.program,
            &args.args,
            args.env,
            None,
            |line| print!("{}", line),
            |line| eprint!("{}", line),
        )
        .await
    {
        Ok(code) => code,
        Err(e) => {
            eprintln!("Failed to execute command: {}", e);
            std::process::exit(1);
        }
    };

    std::process::exit(exit_code);
}
