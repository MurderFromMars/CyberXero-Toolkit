//! Two-stage dialog for downloading an Arch Linux ISO: first resolve which
//! image to pull and where to save it, then run the transfer with live
//! progress, pause/resume, and cancel.
//!
//! Each stage is its own `Rc`-owned struct so the glib signal handlers and
//! worker-thread callbacks can share state without tangled cloning ladders.

use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Button, Entry, Image, Label, ProgressBar, Window};
use log::{error, info};

use crate::core::download::{
    humanize_bytes, humanize_eta, humanize_rate, latest_arch_iso, stream_to_file, Progress,
    TransferFlags,
};
use crate::ui::utils::extract_widget;

/// Open the ISO setup dialog. When the user confirms, the transfer dialog
/// is spawned with the chosen destination.
pub fn show_download_dialog(parent: &Window) {
    info!("opening ISO setup dialog");
    SetupStage::spawn(parent);
}

// ---------------------------------------------------------------------------
// Stage 1 — pick an ISO + destination
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct IsoRef {
    filename: String,
    url: String,
}

struct SetupStage {
    window: adw::Window,
    version_label: Label,
    path_entry: Entry,
    browse_btn: Button,
    start_btn: Button,
    cancel_btn: Button,
    spinner: Image,
    iso: Mutex<Option<IsoRef>>,
    dest: Mutex<Option<String>>,
}

impl SetupStage {
    fn spawn(parent: &Window) {
        let builder =
            gtk4::Builder::from_resource(crate::config::resources::dialogs::DOWNLOAD_SETUP);

        let stage = Rc::new(Self {
            window: extract_widget(&builder, "download_setup_window"),
            version_label: extract_widget(&builder, "version_label"),
            path_entry: extract_widget(&builder, "download_path_entry"),
            browse_btn: extract_widget(&builder, "browse_button"),
            start_btn: extract_widget(&builder, "start_download_button"),
            cancel_btn: extract_widget(&builder, "cancel_button"),
            spinner: extract_widget(&builder, "fetching_spinner"),
            iso: Mutex::new(None),
            dest: Mutex::new(None),
        });

        stage.window.set_transient_for(Some(parent));

        stage.kick_off_iso_lookup();
        stage.wire_buttons(parent);
        stage.window.present();
    }

    fn kick_off_iso_lookup(self: &Rc<Self>) {
        let (tx, rx) = mpsc::channel::<Result<IsoRef, String>>();

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(Err(e.to_string()));
                    return;
                }
            };
            let result = rt
                .block_on(async { latest_arch_iso().await })
                .map(|(filename, url)| IsoRef { filename, url })
                .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });

        let me = self.clone();
        glib::timeout_add_local(Duration::from_millis(50), move || match rx.try_recv() {
            Ok(Ok(iso)) => {
                me.on_iso_resolved(iso);
                glib::ControlFlow::Break
            }
            Ok(Err(e)) => {
                me.on_iso_failed(&e);
                glib::ControlFlow::Break
            }
            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(mpsc::TryRecvError::Disconnected) => {
                me.on_iso_failed("worker thread hung up");
                glib::ControlFlow::Break
            }
        });
    }

    fn on_iso_resolved(&self, iso: IsoRef) {
        info!("ISO resolved: {}", iso.filename);

        // Parse `archlinux-YYYY.MM.DD-x86_64.iso` → `Version: YYYY.MM.DD`.
        let version_text = iso
            .filename
            .strip_prefix("archlinux-")
            .and_then(|rest| rest.split('-').next())
            .map(|date| format!("Version: {date}"))
            .unwrap_or_else(|| String::from("Latest Version"));
        self.version_label.set_text(&version_text);

        self.spinner.set_visible(false);

        let default_dest = format!(
            "{}/Downloads/{}",
            crate::config::env::get().home,
            iso.filename,
        );
        self.path_entry.set_text(&default_dest);
        *self.dest.lock().unwrap() = Some(default_dest);
        *self.iso.lock().unwrap() = Some(iso);

        self.browse_btn.set_sensitive(true);
        self.start_btn.set_sensitive(true);
    }

    fn on_iso_failed(&self, reason: &str) {
        error!("ISO lookup failed: {reason}");
        self.spinner.remove_css_class("spinning");
        self.spinner.set_icon_name(Some("circle-xmark"));
        self.version_label.set_text("Failed to fetch version");
        self.version_label.remove_css_class("accent");
        self.version_label.add_css_class("error");
    }

    fn wire_buttons(self: &Rc<Self>, parent: &Window) {
        let win = self.window.clone();
        self.cancel_btn.connect_clicked(move |_| win.close());

        let me = self.clone();
        self.browse_btn.connect_clicked(move |_| me.open_file_picker());

        let me = self.clone();
        let parent_owned = parent.clone();
        self.start_btn.connect_clicked(move |_| {
            let iso = me.iso.lock().unwrap().clone();
            let dest = me.dest.lock().unwrap().clone();
            if let (Some(iso), Some(dest)) = (iso, dest) {
                info!("starting transfer: {} -> {}", iso.filename, dest);
                me.window.close();
                TransferStage::spawn(&parent_owned, iso, dest);
            }
        });
    }

    fn open_file_picker(self: &Rc<Self>) {
        let iso_snapshot = self.iso.lock().unwrap().clone();
        let Some(iso) = iso_snapshot else {
            return;
        };

        let dialog = gtk4::FileDialog::new();
        dialog.set_initial_name(Some(&iso.filename));

        let path_entry = self.path_entry.clone();
        let start_btn = self.start_btn.clone();
        let me = self.clone();
        let parent_window = self.window.clone();

        glib::spawn_future_local(async move {
            if let Ok(file) = dialog.save_future(Some(&parent_window)).await {
                if let Some(path) = file.path() {
                    let path_str = path.to_string_lossy().into_owned();
                    path_entry.set_text(&path_str);
                    *me.dest.lock().unwrap() = Some(path_str);
                    start_btn.set_sensitive(true);
                }
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Stage 2 — run the transfer with progress UI
// ---------------------------------------------------------------------------

enum TransferEvent {
    Progress(Progress),
    Done,
    Failed(String),
}

struct TransferStage {
    window: adw::Window,
    progress_bar: ProgressBar,
    speed_label: Label,
    downloaded_label: Label,
    eta_label: Label,
    pause_btn: Button,
    cancel_btn: Button,
    flags: TransferFlags,
}

impl TransferStage {
    fn spawn(parent: &Window, iso: IsoRef, dest: String) {
        let builder = gtk4::Builder::from_resource(crate::config::resources::dialogs::DOWNLOAD);

        let filename_label: Label = extract_widget(&builder, "filename_label");
        filename_label.set_text(&iso.filename);

        let stage = Rc::new(Self {
            window: extract_widget(&builder, "download_window"),
            progress_bar: extract_widget(&builder, "progress_bar"),
            speed_label: extract_widget(&builder, "speed_label"),
            downloaded_label: extract_widget(&builder, "downloaded_label"),
            eta_label: extract_widget(&builder, "time_remaining_label"),
            pause_btn: extract_widget(&builder, "pause_button"),
            cancel_btn: extract_widget(&builder, "cancel_button"),
            flags: TransferFlags::new(),
        });
        stage.window.set_transient_for(Some(parent));

        stage.wire_controls();

        let (tx, rx) = mpsc::channel::<TransferEvent>();
        stage.install_event_pump(parent.clone(), rx);
        stage.launch_worker(iso.url, dest, tx);

        stage.window.present();
    }

    fn wire_controls(self: &Rc<Self>) {
        // Pause toggles the flag and flips the button label.
        let flags = self.flags.clone();
        let btn = self.pause_btn.clone();
        self.pause_btn.connect_clicked(move |_| {
            let was_paused = flags.is_paused();
            flags.set_paused(!was_paused);
            btn.set_label(if was_paused { "Pause" } else { "Resume" });
        });

        // Cancel flips the flag, then closes the window.
        let flags = self.flags.clone();
        let win = self.window.clone();
        self.cancel_btn.connect_clicked(move |_| {
            flags.request_cancel();
            win.close();
        });

        // If the user closes the window via the titlebar, still mark the
        // transfer cancelled so the worker cleans up the partial file.
        let flags = self.flags.clone();
        self.window.connect_close_request(move |_| {
            flags.request_cancel();
            glib::Propagation::Proceed
        });
    }

    fn install_event_pump(
        self: &Rc<Self>,
        parent: Window,
        rx: mpsc::Receiver<TransferEvent>,
    ) {
        let me = self.clone();
        glib::timeout_add_local(Duration::from_millis(50), move || {
            while let Ok(evt) = rx.try_recv() {
                match evt {
                    TransferEvent::Progress(p) => me.render_progress(&p),
                    TransferEvent::Done => {
                        me.render_done();
                        return glib::ControlFlow::Break;
                    }
                    TransferEvent::Failed(e) => {
                        if !e.contains("cancelled") {
                            alert(&parent, "Download Failed", &e);
                        }
                        me.window.close();
                        return glib::ControlFlow::Break;
                    }
                }
            }
            glib::ControlFlow::Continue
        });
    }

    fn launch_worker(&self, url: String, dest: String, tx: mpsc::Sender<TransferEvent>) {
        let flags = self.flags.clone();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(TransferEvent::Failed(e.to_string()));
                    return;
                }
            };
            let progress_tx = tx.clone();
            let outcome = rt.block_on(async {
                stream_to_file(
                    url,
                    dest,
                    move |p| {
                        let _ = progress_tx.send(TransferEvent::Progress(p));
                    },
                    flags,
                )
                .await
            });
            let _ = match outcome {
                Ok(()) => tx.send(TransferEvent::Done),
                Err(e) => tx.send(TransferEvent::Failed(e.to_string())),
            };
        });
    }

    fn render_progress(&self, p: &Progress) {
        let fraction = if p.bytes_total > 0 {
            p.bytes_received as f64 / p.bytes_total as f64
        } else {
            0.0
        };
        self.progress_bar.set_fraction(fraction);
        self.progress_bar
            .set_text(Some(&format!("{:.1}%", fraction * 100.0)));

        self.speed_label.set_text(&humanize_rate(p.bytes_per_second));

        self.downloaded_label.set_text(&format!(
            "{} / {}",
            humanize_bytes(p.bytes_received),
            humanize_bytes(p.bytes_total),
        ));

        if p.bytes_total > 0 && p.bytes_received >= p.bytes_total {
            self.eta_label.set_text("Completed");
            self.eta_label.add_css_class("success");
        } else {
            let eta_seconds = if p.bytes_per_second > 0.0 {
                let remaining = p.bytes_total.saturating_sub(p.bytes_received);
                (remaining as f64 / p.bytes_per_second) as u64
            } else {
                0
            };
            self.eta_label.set_text(&humanize_eta(eta_seconds));
            self.eta_label.remove_css_class("success");
        }
    }

    fn render_done(&self) {
        info!("transfer complete");
        self.progress_bar.set_fraction(1.0);
        self.progress_bar.set_text(Some("100%"));
        self.speed_label.set_text("-");
        self.speed_label.remove_css_class("success");
        self.eta_label.set_text("Completed");
        self.eta_label.add_css_class("success");
        self.pause_btn.set_sensitive(false);
        self.cancel_btn.set_label("Close");
        self.cancel_btn.add_css_class("suggested-action");
    }
}

fn alert(parent: &Window, title: &str, message: &str) {
    use adw::prelude::*;
    let dialog = adw::AlertDialog::new(Some(title), Some(message));
    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));
    dialog.present(Some(parent));
}
