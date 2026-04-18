//! Sequential command runner with a live progress dialog.
//!
//! Call sites construct a [`CommandSequence`] of [`Command`]s using the
//! builder API and hand it to [`run`], which opens a modal window, spawns
//! each command in turn, and streams output into a collapsible sidebar.
//!
//! ```no_run
//! use crate::ui::task_runner::{self, Command, CommandSequence};
//!
//! let seq = CommandSequence::new()
//!     .then(
//!         Command::builder()
//!             .privileged()
//!             .program("systemctl")
//!             .args(&["enable", "--now", "foo.service"])
//!             .description("Enabling foo")
//!             .build(),
//!     )
//!     .build();
//!
//! task_runner::run(&parent, seq, "Setup");
//! ```

mod pipeline;
mod view;

use std::sync::atomic::{AtomicBool, Ordering};

use gtk4::prelude::*;
use gtk4::Window;
use log::{error, info, warn};

use self::pipeline::Pipeline;
use self::view::{RunnerView, Tag};

// ---------------------------------------------------------------------------
// Public command API
// ---------------------------------------------------------------------------

/// A command scheduled by the runner.
#[derive(Clone, Debug)]
pub struct Command {
    pub(super) mode: Mode,
    pub(super) program: String,
    pub(super) args: Vec<String>,
    pub(super) description: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Plain,
    Elevated,
    Aur,
}

impl Command {
    /// Entry point for the fluent builder.
    pub fn builder() -> CommandInit {
        CommandInit
    }
}

/// Empty builder root; pick an execution flavour to continue.
#[derive(Debug)]
pub struct CommandInit;

impl CommandInit {
    /// Plain child process — no privilege escalation, no helper wrapping.
    pub fn normal(self) -> CommandDraft {
        CommandDraft::fresh(Mode::Plain)
    }

    /// Runs through the CyberXero auth daemon with root.
    pub fn privileged(self) -> CommandDraft {
        CommandDraft::fresh(Mode::Elevated)
    }

    /// Runs under the configured AUR helper (paru/yay) with `--sudo` set to
    /// the daemon binary so users aren't prompted multiple times.
    pub fn aur(self) -> CommandDraft {
        CommandDraft::fresh(Mode::Aur)
    }
}

/// Mutable draft assembled by chained setters.
#[derive(Debug)]
pub struct CommandDraft {
    mode: Mode,
    program: Option<String>,
    args: Vec<String>,
    description: Option<String>,
}

impl CommandDraft {
    fn fresh(mode: Mode) -> Self {
        Self {
            mode,
            program: None,
            args: Vec::new(),
            description: None,
        }
    }

    /// Program to run. Ignored for AUR commands — the helper is picked
    /// automatically.
    pub fn program(mut self, program: &str) -> Self {
        self.program = Some(program.to_owned());
        self
    }

    /// Replace the argument list wholesale.
    pub fn args(mut self, args: &[&str]) -> Self {
        self.args = args.iter().map(|s| (*s).to_owned()).collect();
        self
    }

    /// Short line shown next to the step in the progress list.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_owned());
        self
    }

    /// Finish the draft. Panics if required fields are missing.
    pub fn build(self) -> Command {
        let program = match self.mode {
            Mode::Aur => String::from("aur"),
            _ => self
                .program
                .expect("program is required for normal and privileged commands"),
        };
        let description = self.description.expect("description is required");
        Command {
            mode: self.mode,
            program,
            args: self.args,
            description,
        }
    }
}

/// Ordered collection of commands ready to hand to [`run`].
#[derive(Debug, Default)]
pub struct CommandSequence {
    pub(super) steps: Vec<Command>,
}

impl CommandSequence {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a step. Chainable.
    pub fn then(mut self, cmd: Command) -> Self {
        self.steps.push(cmd);
        self
    }

    /// Identity terminator kept for call-site readability.
    pub fn build(self) -> Self {
        self
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Run loop entry point
// ---------------------------------------------------------------------------

static ACTION_RUNNING: AtomicBool = AtomicBool::new(false);

/// Returns true while a sequence is in flight.
pub fn is_running() -> bool {
    ACTION_RUNNING.load(Ordering::SeqCst)
}

/// Open the runner dialog and drive the sequence to completion.
///
/// A second call while another sequence is already running is ignored and
/// logged — the caller should gate on [`is_running`] if that matters.
pub fn run(parent: &Window, commands: CommandSequence, title: &str) {
    if commands.is_empty() {
        error!("run() called with an empty sequence");
        return;
    }
    if is_running() {
        warn!("run() called while another sequence is active — ignoring");
        return;
    }

    ACTION_RUNNING.store(true, Ordering::SeqCst);

    let builder = gtk4::Builder::from_resource(crate::config::resources::dialogs::TASK_LIST);
    let view = RunnerView::from_builder(&builder, &commands.steps);
    view.window().set_transient_for(Some(parent));
    view.window().set_title(Some(title));

    let wants_daemon = commands
        .steps
        .iter()
        .any(|c| matches!(c.mode, Mode::Elevated | Mode::Aur));

    if wants_daemon {
        if let Err(e) = crate::core::daemon::start_daemon() {
            error!("daemon start failed: {}", e);
            let msg = format!("Failed to start authentication daemon: {}\n", e);
            view.append(&msg, Tag::Error);
            view.window().present();
            view.finalize(
                false,
                &format!("Failed to start authentication daemon: {}", e),
            );
            return;
        }
        info!("auth daemon ready");
    }

    view.window().present();
    Pipeline::new(view, commands.steps).start();
}
