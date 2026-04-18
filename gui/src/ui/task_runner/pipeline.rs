//! State machine that drives a [`CommandSequence`] to completion.
//!
//! A [`Pipeline`] owns the runner's view and the step list for the duration
//! of a run. All call sites into GTK are issued from the main thread via a
//! single `timeout_add_local` pump; subprocess I/O is handled on worker
//! threads and forwarded over `mpsc` channels.

use std::cell::Cell;
use std::process::{Child, Command as SysCommand, Stdio};
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use cyberxero_auth::utils::read_buffer_with_line_processing;
use gtk4::glib;
use gtk4::prelude::*;
use log::{error, info, warn};

use super::view::{RunnerView, StepState, Tag};
use super::{Command, Mode, ACTION_RUNNING};

const MSG_CANCEL_PENDING: &str = "Waiting for current step to finish…";
const MSG_CANCELLED: &str = "Operation cancelled by user";
const MSG_SUCCESS: &str = "All steps completed successfully";

pub(super) struct Pipeline {
    view: Rc<RunnerView>,
    steps: Rc<Vec<Command>>,
    cursor: Cell<usize>,
    cancelled: Cell<bool>,
}

impl Pipeline {
    pub(super) fn new(view: Rc<RunnerView>, steps: Vec<Command>) -> Rc<Self> {
        Rc::new(Self {
            view,
            steps: Rc::new(steps),
            cursor: Cell::new(0),
            cancelled: Cell::new(false),
        })
    }

    /// Attach handlers to the view's cancel/close/window signals and kick off
    /// the first step.
    pub(super) fn start(self: Rc<Self>) {
        let me = self.clone();
        self.view.on_cancel(move || {
            me.cancelled.set(true);
            me.view.disable_cancel();
            me.view.set_title(MSG_CANCEL_PENDING);
        });

        let view_for_close = self.view.clone();
        self.view.on_close(move || view_for_close.window().close());

        let me = self.clone();
        self.view.on_window_close(move || {
            ACTION_RUNNING.store(false, Ordering::SeqCst);
            me.cancelled.set(true);
        });

        self.advance();
    }

    /// Dispatch the next step, or terminate if the sequence is done or the
    /// user has asked to cancel.
    fn advance(self: &Rc<Self>) {
        let cursor = self.cursor.get();

        if self.cancelled.get() {
            if cursor < self.steps.len() {
                self.view.set_step_state(cursor, StepState::Cancelled);
            }
            self.conclude(false, MSG_CANCELLED);
            return;
        }

        if cursor >= self.steps.len() {
            self.conclude(true, MSG_SUCCESS);
            return;
        }

        let step = &self.steps[cursor];
        self.view.set_step_state(cursor, StepState::Running);
        self.view.set_title(&step.description);
        self.view.emit_step_banner(&step.description);

        let (program, args) = match resolve(step) {
            Ok(pair) => pair,
            Err(e) => {
                let text = format!("Failed to prepare command: {}\n", e);
                self.view.append(&text, Tag::Error);
                self.view.set_step_state(cursor, StepState::Failed);
                self.conclude(false, &format!("Failed to prepare command: {}", e));
                return;
            }
        };

        info!("running: {} {:?}", program, args);

        let mut sys = SysCommand::new(&program);
        sys.args(&args).stdout(Stdio::piped()).stderr(Stdio::piped());
        install_path_shim(&mut sys);

        let child = match sys.spawn() {
            Ok(c) => c,
            Err(e) => {
                let text = format!("Failed to start operation: {}\n", e);
                self.view.append(&text, Tag::Error);
                self.view.set_step_state(cursor, StepState::Failed);
                self.conclude(false, &format!("Failed to start operation: {}", e));
                return;
            }
        };

        self.pump(child);
    }

    /// Spawn worker threads to drain stdout/stderr and reap the child, then
    /// install a GLib tick that forwards the channels to the text buffer and
    /// hands control back to [`advance`] when the process exits.
    fn pump(self: &Rc<Self>, mut child: Child) {
        let (tx_out, rx_out) = mpsc::channel::<String>();
        let (tx_err, rx_err) = mpsc::channel::<String>();
        let exit: Arc<Mutex<Option<Option<i32>>>> = Arc::new(Mutex::new(None));

        let stdout_thread = child.stdout.take().map(|stream| {
            let tx = tx_out;
            thread::spawn(move || {
                read_buffer_with_line_processing(
                    stream,
                    move |line| tx.send(line).is_ok(),
                    |e| warn!("stdout reader: {}", e),
                );
            })
        });

        let stderr_thread = child.stderr.take().map(|stream| {
            let tx = tx_err;
            thread::spawn(move || {
                read_buffer_with_line_processing(
                    stream,
                    move |line| tx.send(line).is_ok(),
                    |e| warn!("stderr reader: {}", e),
                );
            })
        });

        let slot = exit.clone();
        thread::spawn(move || {
            if let Some(h) = stdout_thread {
                if let Err(e) = h.join() {
                    warn!("joining stdout thread: {:?}", e);
                }
            }
            if let Some(h) = stderr_thread {
                if let Err(e) = h.join() {
                    warn!("joining stderr thread: {:?}", e);
                }
            }

            let code = match child.wait() {
                Ok(status) => {
                    if status.success() {
                        Some(0)
                    } else {
                        status.code()
                    }
                }
                Err(e) => {
                    error!("wait() failed: {}", e);
                    None
                }
            };
            *slot.lock().unwrap() = Some(code);
        });

        let me = self.clone();
        glib::timeout_add_local(Duration::from_millis(40), move || {
            drain(&rx_out, |line| me.view.append_stream(&line, Tag::Stdout));
            drain(&rx_err, |line| me.view.append_stream(&line, Tag::Stderr));

            let done = exit.lock().unwrap().take();
            if let Some(code) = done {
                // Drain any remaining residual lines before finalizing.
                drain(&rx_out, |line| me.view.append_stream(&line, Tag::Stdout));
                drain(&rx_err, |line| me.view.append_stream(&line, Tag::Stderr));
                me.finish_step(code);
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    fn finish_step(self: &Rc<Self>, code: Option<i32>) {
        let cursor = self.cursor.get();

        if self.cancelled.get() {
            self.view.set_step_state(cursor, StepState::Cancelled);
            self.conclude(false, MSG_CANCELLED);
            return;
        }

        let success = matches!(code, Some(0));
        let exit_line = match code {
            Some(c) => format!("\n[exit code: {}]\n", c),
            None => String::from("\n[exit code: unknown]\n"),
        };
        self.view.append(
            &exit_line,
            if success { Tag::Stdout } else { Tag::Stderr },
        );

        if success {
            self.view.set_step_state(cursor, StepState::Success);
            self.cursor.set(cursor + 1);
            self.advance();
        } else {
            self.view.set_step_state(cursor, StepState::Failed);
            let suffix = code
                .map(|c| format!(" (exit code: {})", c))
                .unwrap_or_default();
            let msg = format!(
                "Operation failed at step {} of {}{}",
                cursor + 1,
                self.steps.len(),
                suffix
            );
            self.conclude(false, &msg);
        }
    }

    fn conclude(self: &Rc<Self>, success: bool, message: &str) {
        stop_daemon();
        let tag = if success { Tag::Stdout } else { Tag::Error };
        self.view.append(&format!("\n{}\n", message), tag);
        ACTION_RUNNING.store(false, Ordering::SeqCst);
        self.view.finalize(success, message);
    }
}

fn drain<F: FnMut(String)>(rx: &mpsc::Receiver<String>, mut visit: F) {
    while let Ok(line) = rx.try_recv() {
        visit(line);
    }
}

/// Push the bundled scripts directory to the front of `PATH` so the sudo
/// shim can intercept sudo invocations issued from helper scripts.
fn install_path_shim(cmd: &mut SysCommand) {
    let scripts_dir = crate::config::paths::scripts();
    if !scripts_dir.exists() {
        return;
    }
    if let Ok(path) = std::env::var("PATH") {
        cmd.env("PATH", format!("{}:{}", scripts_dir.display(), path));
    }
}

/// Translate a logical [`Command`] into the concrete `(program, args)` pair
/// that gets spawned. Elevated and AUR commands are funnelled through the
/// auth daemon so users authenticate once per sequence.
fn resolve(cmd: &Command) -> Result<(String, Vec<String>), String> {
    use crate::core::daemon::get_cyberxero_auth_path;

    let scripts_dir = crate::config::paths::scripts();
    let path_override = if scripts_dir.exists() {
        std::env::var("PATH")
            .ok()
            .map(|p| format!("PATH={}:{}", scripts_dir.display(), p))
    } else {
        None
    };

    let auth_path = || get_cyberxero_auth_path().to_string_lossy().into_owned();

    match cmd.mode {
        Mode::Plain => Ok((cmd.program.clone(), cmd.args.clone())),
        Mode::Elevated => {
            let mut args = Vec::with_capacity(cmd.args.len() + 3);
            if let Some(env) = path_override {
                args.push(String::from("--env"));
                args.push(env);
            }
            args.push(cmd.program.clone());
            args.extend(cmd.args.iter().cloned());
            Ok((auth_path(), args))
        }
        Mode::Aur => {
            let helper = crate::core::aur_helper()
                .ok_or_else(|| String::from("AUR helper not available (paru or yay required)"))?;
            let mut args = Vec::with_capacity(cmd.args.len() + 2);
            args.push(String::from("--sudo"));
            args.push(auth_path());
            args.extend(cmd.args.iter().cloned());
            Ok((helper.to_owned(), args))
        }
    }
}

/// Shut the auth daemon down on a throw-away Tokio runtime. Failures here
/// are logged but not surfaced to the user since the sequence itself has
/// already finished one way or another.
fn stop_daemon() {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            error!("tokio runtime: {}", e);
            return;
        }
    };
    if let Err(e) = rt.block_on(crate::core::daemon::stop_daemon()) {
        error!("daemon shutdown: {}", e);
    }
}
