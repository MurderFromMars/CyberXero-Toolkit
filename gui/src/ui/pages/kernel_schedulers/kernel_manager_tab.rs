//! Kernel Manager tab: install and remove Arch kernels + headers.
//!
//! Packages are enumerated by scraping `pacman -Sl` for repository listings
//! and `pacman -Q` for installed ones. A "kernel" is only counted when its
//! `*-headers` sibling is present in the same listing, so out-of-tree
//! module packages (zfs-linux, nvidia-linux-lts, etc.) don't pollute the
//! available/installed views.

use std::collections::HashSet;
use std::process::{Command as SysCommand, Stdio};
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, ApplicationWindow, Box as GtkBox, Builder, Button, Image, Label, ListBox, Orientation,
};
use log::{info, warn};

use crate::ui::dialogs::warning::show_warning_confirmation;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;

const POLL: Duration = Duration::from_millis(100);
const POST_ACTION_RESCAN: Duration = Duration::from_secs(2);

pub fn setup_handlers(
    builder: &Builder,
    _main_builder: &Builder,
    window: &ApplicationWindow,
) {
    let tab = KernelTab::new(builder.clone(), window.clone());
    tab.bind_refresh_button();
    tab.rescan(None);
}

struct KernelTab {
    builder: Builder,
    window: ApplicationWindow,
}

impl KernelTab {
    fn new(builder: Builder, window: ApplicationWindow) -> Rc<Self> {
        Rc::new(Self { builder, window })
    }

    fn bind_refresh_button(self: &Rc<Self>) {
        let me = self.clone();
        let btn = extract_widget::<Button>(&self.builder, "btn_refresh_kernels");
        btn.connect_clicked(move |button| me.rescan(Some(button.clone())));
    }

    /// Kick off a background scan, freezing the list controls until the
    /// worker thread posts its result back through the channel.
    fn rescan(self: &Rc<Self>, button: Option<Button>) {
        info!("scanning kernels");
        let lock = ContentLock::engage(&self.builder, button);
        let (tx, rx) = mpsc::channel::<KernelScan>();

        thread::spawn(move || {
            let _ = tx.send(KernelScan::collect());
        });

        let me = self.clone();
        let lock_slot = std::cell::RefCell::new(Some(lock));
        glib::timeout_add_local(POLL, move || match rx.try_recv() {
            Ok(scan) => {
                me.render(&scan);
                if let Some(lock) = lock_slot.borrow_mut().take() {
                    lock.release(&me.builder);
                }
                glib::ControlFlow::Break
            }
            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(mpsc::TryRecvError::Disconnected) => {
                warn!("kernel scan worker dropped without a result");
                if let Some(lock) = lock_slot.borrow_mut().take() {
                    lock.release(&me.builder);
                }
                glib::ControlFlow::Break
            }
        });
    }

    fn render(self: &Rc<Self>, scan: &KernelScan) {
        self.render_installed(&scan.installed);
        self.render_available(&scan.available, &scan.installed);
        self.render_counts(&scan.available, &scan.installed);
    }

    fn render_installed(self: &Rc<Self>, installed: &[String]) {
        let list = extract_widget::<ListBox>(&self.builder, "installed_kernels_list");
        clear_children(&list);
        if installed.is_empty() {
            list.append(&placeholder("No kernels installed"));
            return;
        }
        for kernel in installed {
            let me = self.clone();
            let name = kernel.clone();
            list.append(&build_row(
                kernel,
                RowAction::Remove(Box::new(move || me.confirm_remove(&name))),
            ));
        }
    }

    fn render_available(self: &Rc<Self>, available: &[String], installed: &[String]) {
        let list = extract_widget::<ListBox>(&self.builder, "available_kernels_list");
        clear_children(&list);
        let mut rendered = 0usize;
        for kernel in available {
            if installed.iter().any(|i| i == kernel) {
                continue;
            }
            let me = self.clone();
            let name = kernel.clone();
            list.append(&build_row(
                kernel,
                RowAction::Install(Box::new(move || me.confirm_install(&name))),
            ));
            rendered += 1;
        }
        if rendered == 0 {
            list.append(&placeholder("All available kernels are installed"));
        }
    }

    fn render_counts(&self, available: &[String], installed: &[String]) {
        extract_widget::<Label>(&self.builder, "installed_count_label")
            .set_text(&format!("{} installed", installed.len()));
        let remaining = available.iter().filter(|k| !installed.contains(k)).count();
        extract_widget::<Label>(&self.builder, "available_count_label")
            .set_text(&format!("{} available", remaining));
    }

    fn confirm_install(self: &Rc<Self>, kernel: &str) {
        let kernel = kernel.to_owned();
        let headers = format!("{kernel}-headers");
        let me = self.clone();
        show_warning_confirmation(
            self.window.upcast_ref(),
            "Confirm Installation",
            &format!(
                "Install <b>{kernel}</b> and <b>{headers}</b>?\n\n\
                 This will download and install the kernel and its headers."
            ),
            move || {
                info!("installing {kernel} + {headers}");
                me.run_action(
                    "Install Kernel",
                    &["-S", "--noconfirm", "--needed", &kernel, &headers],
                    &format!("Installing {kernel} and {headers}..."),
                );
            },
        );
    }

    fn confirm_remove(self: &Rc<Self>, kernel: &str) {
        let kernel = kernel.to_owned();
        let headers = format!("{kernel}-headers");
        let me = self.clone();
        show_warning_confirmation(
            self.window.upcast_ref(),
            "Confirm Removal",
            &format!(
                "Remove <b>{kernel}</b> and <b>{headers}</b>?\n\n\
                 <span foreground=\"red\" weight=\"bold\">Warning:</span> \
                 This will uninstall the kernel and its headers.\n\
                 Make sure you have at least one other kernel installed."
            ),
            move || {
                info!("removing {kernel} + {headers}");
                me.run_action(
                    "Remove Kernel",
                    &["-R", "--noconfirm", &kernel, &headers],
                    &format!("Removing {kernel} and {headers}..."),
                );
            },
        );
    }

    fn run_action(self: &Rc<Self>, title: &str, args: &[&str], description: &str) {
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(args)
                    .description(description)
                    .build(),
            )
            .build();
        task_runner::run(self.window.upcast_ref(), commands, title);

        // Poll the runner until it finishes, then rescan once so the rows
        // reflect the new state without the user hitting Refresh.
        let me = self.clone();
        glib::timeout_add_local(POST_ACTION_RESCAN, move || {
            if task_runner::is_running() {
                glib::ControlFlow::Continue
            } else {
                me.rescan(None);
                glib::ControlFlow::Break
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Pacman scan
// ---------------------------------------------------------------------------

struct KernelScan {
    available: Vec<String>,
    installed: Vec<String>,
}

impl KernelScan {
    fn collect() -> Self {
        let available = match scan_pacman(&["-Sl"], |line| {
            if line.contains("testing/") {
                None
            } else {
                line.split_whitespace().nth(1).map(str::to_owned)
            }
        }) {
            Ok(pkgs) => pair_kernels_with_headers(&pkgs),
            Err(e) => {
                warn!("pacman -Sl failed: {e}");
                Vec::new()
            }
        };

        let installed = match scan_pacman(&["-Q"], |line| {
            line.split_whitespace().next().map(str::to_owned)
        }) {
            Ok(pkgs) => pair_kernels_with_headers(&pkgs),
            Err(e) => {
                warn!("pacman -Q failed: {e}");
                Vec::new()
            }
        };

        info!(
            "kernel scan: {} available, {} installed",
            available.len(),
            installed.len()
        );
        Self {
            available,
            installed,
        }
    }
}

/// Invoke pacman with the given args and project each stdout line through
/// `extract`. `extract` returns `None` for lines the caller wants to skip.
fn scan_pacman<F>(args: &[&str], extract: F) -> anyhow::Result<Vec<String>>
where
    F: Fn(&str) -> Option<String>,
{
    let output = SysCommand::new("pacman")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !output.status.success() {
        anyhow::bail!("pacman {} exited unsuccessfully", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(extract)
        .collect())
}

/// Return the names of every `linux*` package whose `*-headers` twin is
/// also present in `packages`. `linux-api-headers` is explicitly excluded
/// because it is not a kernel.
fn pair_kernels_with_headers(packages: &[String]) -> Vec<String> {
    let names: HashSet<&str> = packages
        .iter()
        .map(String::as_str)
        .filter(|p| p.starts_with("linux"))
        .collect();

    let mut kernels: Vec<String> = names
        .iter()
        .filter_map(|pkg| {
            let stem = pkg.strip_suffix("-headers")?;
            if *pkg == "linux-api-headers" || !names.contains(stem) {
                return None;
            }
            Some(stem.to_owned())
        })
        .collect();
    kernels.sort();
    kernels.dedup();
    kernels
}

// ---------------------------------------------------------------------------
// Row construction
// ---------------------------------------------------------------------------

enum RowAction {
    Install(Box<dyn Fn() + 'static>),
    Remove(Box<dyn Fn() + 'static>),
}

fn build_row(name: &str, action: RowAction) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 8);
    row.set_margin_start(12);
    row.set_margin_end(12);
    row.set_margin_top(8);
    row.set_margin_bottom(8);

    let label = Label::new(Some(name));
    label.set_xalign(0.0);
    label.set_hexpand(true);
    row.append(&label);

    let button = Button::new();
    button.set_valign(Align::Center);
    button.add_css_class("flat");

    match action {
        RowAction::Install(callback) => {
            button.set_icon_name("download-symbolic");
            button.add_css_class("suggested-action");
            button.connect_clicked(move |_| callback());
        }
        RowAction::Remove(callback) => {
            button.set_icon_name("trash-symbolic");
            button.add_css_class("destructive-action");
            button.connect_clicked(move |_| callback());
        }
    }
    row.append(&button);
    row
}

fn placeholder(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class("dim-label");
    label.set_margin_start(12);
    label.set_margin_end(12);
    label.set_margin_top(8);
    label.set_margin_bottom(8);
    label
}

fn clear_children(list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

// ---------------------------------------------------------------------------
// UI freeze shared between initial and user-triggered scans
// ---------------------------------------------------------------------------

struct ContentLock {
    refresh_btn: Option<Button>,
}

impl ContentLock {
    fn engage(builder: &Builder, button: Option<Button>) -> Self {
        extract_widget::<GtkBox>(builder, "content_box").set_sensitive(false);
        if let Some(b) = &button {
            b.set_sensitive(false);
            toggle_spin(b, true);
        }
        Self {
            refresh_btn: button,
        }
    }

    fn release(self, builder: &Builder) {
        extract_widget::<GtkBox>(builder, "content_box").set_sensitive(true);
        if let Some(b) = self.refresh_btn {
            b.set_sensitive(true);
            toggle_spin(&b, false);
        }
    }
}

/// The refresh button's icon can be wrapped in a GtkBox (layout tweak from
/// the .ui file), so we look one level deep for the Image to toggle.
fn toggle_spin(button: &Button, spinning: bool) {
    let Some(child) = button.child() else { return };
    let apply = |img: &Image| {
        if spinning {
            img.add_css_class("spinning");
        } else {
            img.remove_css_class("spinning");
        }
    };
    if let Some(img) = child.downcast_ref::<Image>() {
        apply(img);
    } else if let Some(row) = child.downcast_ref::<GtkBox>() {
        if let Some(img) = row.first_child().and_downcast::<Image>() {
            apply(&img);
        }
    }
}
