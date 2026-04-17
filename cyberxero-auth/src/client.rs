//! Client implementation for communicating with the cyberxero-auth daemon.

use crate::protocol::{ClientMessage, DaemonMessage};
use crate::protocol_io::{read_message, write_message};
use crate::shared::get_socket_path;
use anyhow::{Context, Result};
use tokio::net::UnixStream;

/// Client for communicating with the cyberxero-auth daemon.
pub struct Client {
    stream: UnixStream,
}

impl Client {
    /// Connect to the daemon.
    pub async fn new() -> Result<Self> {
        let socket_path = get_socket_path(None)?;

        use tokio::time::{timeout, Duration};
        let stream = timeout(Duration::from_secs(5), UnixStream::connect(&socket_path))
            .await
            .context("Connection timeout")?
            .context("Failed to connect to daemon")?;

        Ok(Self { stream })
    }

    /// Execute a command on the daemon.
    ///
    /// # Arguments
    ///
    /// * `program` - The program to execute.
    /// * `args` - Arguments for the program.
    /// * `env` - Environment variables to set (KEY=VALUE).
    /// * `working_dir` - Optional working directory.
    /// * `on_output` - Callback for stdout output.
    /// * `on_error` - Callback for stderr output.
    ///
    /// # Returns
    ///
    /// The exit code of the command.
    pub async fn execute<F, G>(
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

    /// Send a shutdown request to the daemon.
    pub async fn shutdown(&mut self) -> Result<()> {
        let (mut reader, mut writer) = self.stream.split();

        // Send shutdown message
        let message = ClientMessage::Shutdown;
        write_message(&mut writer, &message).await?;

        // Read shutdown acknowledgment
        match read_message::<_, DaemonMessage>(&mut reader).await? {
            Some(DaemonMessage::ShutdownAck) => Ok(()),
            Some(msg) => anyhow::bail!("Unexpected response to shutdown: {:?}", msg),
            None => anyhow::bail!("Connection closed before shutdown acknowledgment"),
        }
    }
}
