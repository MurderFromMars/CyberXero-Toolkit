//! Xero Authentication Client
//!
//! Command-line client for testing the authentication daemon.

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use tokio::net::UnixStream;
use xero_auth::protocol::{ClientMessage, DaemonMessage};
use xero_auth::protocol_io::{read_message, write_message};
use xero_auth::shared::{get_socket_path, is_daemon_running};

#[derive(Parser, Debug)]
#[command(name = "xero-auth")]
#[command(about = "Xero Authentication Client", long_about = None)]
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

struct Client {
    stream: UnixStream,
}

impl Client {
    async fn new() -> Result<Self> {
        let socket_path = get_socket_path(None)?;

        use tokio::time::{timeout, Duration};
        let stream = timeout(Duration::from_secs(5), UnixStream::connect(&socket_path))
            .await
            .context("Connection timeout")?
            .context("Failed to connect to daemon")?;

        Ok(Self { stream })
    }

    async fn execute<F, G>(
        &mut self,
        program: &str,
        args: &[String],
        env: Vec<String>,
        working_dir: Option<&str>,
        on_output: F,
        on_error: G,
    ) -> Result<i32>
    where
        F: Fn(&str),
        G: Fn(&str),
    {
        let (mut reader, mut writer) = self.stream.split();

        // Write request message
        let message = ClientMessage::Execute {
            program: program.to_string(),
            args: args.to_vec(),
            env,
            working_dir: working_dir.map(|s| s.to_string()),
        };
        write_message(&mut writer, &message).await?;

        let mut exit_code = None;

        loop {
            let response = match read_message::<_, DaemonMessage>(&mut reader).await? {
                Some(msg) => msg,
                None => break, // EOF
            };

            match response {
                DaemonMessage::Output(text) => {
                    on_output(&text);
                }
                DaemonMessage::Error(text) => {
                    on_error(&text);
                }
                DaemonMessage::Completed { exit_code: code } => {
                    exit_code = Some(code);
                    break;
                }
                DaemonMessage::ErrorMessage(msg) => {
                    anyhow::bail!("Daemon error: {}", msg);
                }
                _ => {}
            }
        }

        Ok(exit_code.unwrap_or(-1))
    }
}

#[tokio::main]
async fn main() {
    if !is_daemon_running() {
        eprintln!("Error: xero-auth daemon is not running");
        std::process::exit(1);
    }

    let args = Args::parse();

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
