//! Command types and definitions for task execution.

/// Type of command to execute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandType {
    /// Normal command (no special handling)
    Normal,
    /// Command that needs privilege escalation (pkexec)
    Privileged,
    /// AUR helper command (paru/yay)
    Aur,
}

/// Status of a task in the UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is pending (not started yet)
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Success,
    /// Task failed with error
    Failed,
    /// Task was cancelled by user
    Cancelled,
}

/// Result of command execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandResult {
    /// Command executed successfully
    Success,
    /// Command failed with optional exit code
    Failure { exit_code: Option<i32> },
}

/// A command step to be executed by the task runner.
#[derive(Clone, Debug)]
pub struct Command {
    pub command_type: CommandType,
    pub program: String,
    pub args: Vec<String>,
    pub description: String,
}

impl Command {
    /// Create a new command with an explicit command type.
    pub fn new(command_type: CommandType, program: &str, args: &[&str], description: &str) -> Self {
        Self {
            command_type,
            program: program.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            description: description.to_string(),
        }
    }

    /// Create a normal command (no special handling).
    pub fn normal(program: &str, args: &[&str], description: &str) -> Self {
        Self::new(CommandType::Normal, program, args, description)
    }

    /// Create a privileged command (runs through pkexec).
    pub fn privileged(program: &str, args: &[&str], description: &str) -> Self {
        Self::new(CommandType::Privileged, program, args, description)
    }

    /// Create an AUR helper command (paru/yay).
    pub fn aur(args: &[&str], description: &str) -> Self {
        Self::new(CommandType::Aur, "aur", args, description)
    }
}

impl CommandResult {
    /// Check if the result indicates success.
    #[allow(dead_code)]
    pub fn is_success(&self) -> bool {
        matches!(self, CommandResult::Success)
    }

    /// Check if the result indicates failure.
    #[allow(dead_code)]
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }

    /// Get the exit code if this is a failure.
    #[allow(dead_code)]
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            CommandResult::Failure { exit_code } => *exit_code,
            _ => None,
        }
    }
}
