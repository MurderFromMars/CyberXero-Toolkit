//! SCX scheduler tab.
//!
//! Detects sched-ext kernel support, enumerates available BPF schedulers
//! via `scxctl list`, and wires the buttons that start / switch / stop the
//! active scheduler and toggle the `scx.service` persistence unit.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use adw::prelude::*;
use gtk4::glib;
use gtk4::{ApplicationWindow, Box as GtkBox, Builder, Button, Image, Label};
use log::{info, warn};

use crate::ui::dialogs::warning::show_warning_confirmation;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::{
    extract_widget, get_combo_row_value, is_service_enabled, path_exists, run_command,
};

const SCHED_EXT_PATH: &str = "/sys/kernel/sched_ext";
const SERVICE_NAME: &str = "scx.service";
const STAGING_PATH: &str = "/tmp/scx.service";
const SERVICE_INSTALL_PATH: &str = "/etc/systemd/system/scx.service";
const SYSINIT_TARGET_DIR: &str = "/etc/systemd/system/sysinit.target.wants";

const POLL: Duration = Duration::from_millis(100);
const STATUS_REFRESH: Duration = Duration::from_secs(3);

/// Scheduler groupings shown in the selector dialog. Each tuple is
/// `(group title, scheduler ids)`. Ids that aren't present on the system
/// are silently dropped.
const GROUPS: &[(&str, &[&str])] = &[
    ("Gaming", &["scx_rusty", "scx_lavd", "scx_bpfland"]),
    ("Desktop", &["scx_cosmos", "scx_flash"]),
    ("Servers", &["scx_layered", "scx_flatcg", "scx_tickless"]),
    ("Low Latency", &["scx_nest"]),
    ("Testing", &["scx_simple", "scx_chaos", "scx_userland"]),
];

/// Preference order used when the user hasn't picked a scheduler yet.
const DEFAULT_PICK_ORDER: &[&str] = &["scx_rusty", "scx_lavd"];

pub fn setup_handlers(
    builder: &Builder,
    _main_builder: &Builder,
    window: &ApplicationWindow,
) {
    let tab = SchedTab::new(builder.clone(), window.clone());
    tab.install_kernel_info();
    tab.bind_buttons();
    tab.bind_persistence();

    // Initial scan on the next idle tick so the tab paints before we run
    // any subprocess calls.
    let initial = tab.clone();
    glib::idle_add_local_once(move || initial.rescan(None));

    // Status poller — cheap, read-only, and cheap enough to run every few
    // seconds while the tab is visible.
    let ticking = tab.clone();
    glib::timeout_add_local(STATUS_REFRESH, move || {
        ticking.poll_status();
        glib::ControlFlow::Continue
    });
}

#[derive(Default)]
struct SchedState {
    schedulers: Vec<String>,
    kernel_supported: bool,
    active: bool,
    picked: Option<String>,
}

struct SchedTab {
    builder: Builder,
    window: ApplicationWindow,
    state: RefCell<SchedState>,
}

impl SchedTab {
    fn new(builder: Builder, window: ApplicationWindow) -> Rc<Self> {
        Rc::new(Self {
            builder,
            window,
            state: RefCell::new(SchedState::default()),
        })
    }

    // -- kernel-support banner ----------------------------------------------

    fn install_kernel_info(self: &Rc<Self>) {
        let version = run_command("uname", &["-r"]).unwrap_or_else(|| "Unknown".to_owned());
        let supported = path_exists(SCHED_EXT_PATH);
        self.state.borrow_mut().kernel_supported = supported;

        let icon = extract_widget::<Image>(&self.builder, "kernel_status_icon");
        let version_label = extract_widget::<Label>(&self.builder, "kernel_version_label");
        let legend = extract_widget::<Label>(&self.builder, "kernel_support_label");

        if supported {
            icon.set_icon_name(Some("circle-check"));
            icon.add_css_class("success");
            version_label.set_text(&version);
            version_label.remove_css_class("warning");
            legend.set_text("Supported");
        } else {
            icon.set_icon_name(Some("circle-xmark"));
            icon.add_css_class("error");
            version_label.set_text(&format!("{version} (no sched-ext)"));
            version_label.add_css_class("warning");
            legend.set_text("Not supported");
        }
    }

    // -- button wiring ------------------------------------------------------

    fn bind_buttons(self: &Rc<Self>) {
        self.bind_selection_row();
        self.bind_refresh_button();
        self.bind_switch_button();
        self.bind_stop_button();
    }

    fn bind_selection_row(self: &Rc<Self>) {
        let me = self.clone();
        let row = extract_widget::<adw::ActionRow>(&self.builder, "scheduler_selection_row");
        row.connect_activated(move |_| me.open_selector());
    }

    fn bind_refresh_button(self: &Rc<Self>) {
        let me = self.clone();
        let btn = extract_widget::<Button>(&self.builder, "btn_refresh_schedulers");
        btn.connect_clicked(move |button| me.rescan(Some(button.clone())));
    }

    fn bind_switch_button(self: &Rc<Self>) {
        let me = self.clone();
        let btn = extract_widget::<Button>(&self.builder, "btn_switch_scheduler");
        btn.connect_clicked(move |_| me.switch_or_start());
    }

    fn bind_stop_button(self: &Rc<Self>) {
        let me = self.clone();
        let btn = extract_widget::<Button>(&self.builder, "btn_stop_scheduler");
        btn.connect_clicked(move |_| me.confirm_stop());
    }

    // -- persistence switch -------------------------------------------------

    fn bind_persistence(self: &Rc<Self>) {
        let switch = extract_widget::<adw::SwitchRow>(&self.builder, "persist_switch");
        switch.set_active(is_service_enabled(SERVICE_NAME));
        let me = self.clone();
        switch.connect_active_notify(move |sw| {
            if sw.is_active() {
                if !me.enable_persistence() {
                    sw.set_active(false);
                }
            } else {
                me.disable_persistence();
            }
        });
    }

    fn enable_persistence(self: &Rc<Self>) -> bool {
        let Some(sched_name) = self.state.borrow().picked.clone() else {
            warn!("persistence requested with no scheduler selected");
            return false;
        };
        let mode = self.current_mode();
        let sched_unit = format!("scx_{}", strip_prefix(&sched_name));

        let template_path = crate::config::paths::systemd().join("scx.service.in");
        let Ok(template) = std::fs::read_to_string(&template_path) else {
            warn!("could not read {:?}", template_path);
            return false;
        };
        let rendered = template
            .replace("@SCHEDULER@", &sched_unit)
            .replace("@SCHEDULER_NAME@", &sched_name)
            .replace("@MODE@", &mode);

        if std::fs::write(STAGING_PATH, &rendered).is_err() {
            warn!("could not stage {STAGING_PATH}");
            return false;
        }

        let sysinit_link = format!("{SYSINIT_TARGET_DIR}/scx.service");
        let seq = CommandSequence::new()
            .then(priv_cmd(
                "cp",
                &[STAGING_PATH, SERVICE_INSTALL_PATH],
                "Installing service...",
            ))
            .then(priv_cmd(
                "systemctl",
                &["daemon-reload"],
                "Reloading systemd...",
            ))
            .then(priv_cmd(
                "systemctl",
                &["enable", "--now", "scx.service"],
                "Enabling and starting service...",
            ))
            .then(priv_cmd(
                "mkdir",
                &["-p", SYSINIT_TARGET_DIR],
                "Preparing sysinit target...",
            ))
            .then(priv_cmd(
                "ln",
                &["-sf", SERVICE_INSTALL_PATH, &sysinit_link],
                "Linking to sysinit...",
            ))
            .build();
        task_runner::run(self.window.upcast_ref(), seq, "Enable Persistence");
        true
    }

    fn disable_persistence(self: &Rc<Self>) {
        let seq = CommandSequence::new()
            .then(priv_cmd(
                "systemctl",
                &["stop", "scx.service"],
                "Stopping service...",
            ))
            .then(priv_cmd(
                "systemctl",
                &["disable", "scx.service"],
                "Disabling service...",
            ))
            .build();
        task_runner::run(self.window.upcast_ref(), seq, "Disable Persistence");
    }

    // -- scheduler picker ---------------------------------------------------

    fn open_selector(self: &Rc<Self>) {
        let schedulers = self.state.borrow().schedulers.clone();
        let current = self.state.borrow().picked.clone();
        let me = self.clone();
        present_selector(
            &self.window,
            &schedulers,
            current.as_deref(),
            move |chosen| {
                me.state.borrow_mut().picked = Some(chosen.clone());
                extract_widget::<Label>(&me.builder, "selected_scheduler_label")
                    .set_label(&humanize(&chosen));
            },
        );
    }

    // -- switch / start / stop ---------------------------------------------

    fn switch_or_start(self: &Rc<Self>) {
        let Some(sched_name) = self.state.borrow().picked.clone() else {
            warn!("start/switch with no scheduler picked");
            return;
        };
        let mode = self.current_mode();
        let active = self.state.borrow().active;

        let (verb, title, label_verb) = if active {
            ("switch", "Switch Scheduler", "Switch")
        } else {
            ("start", "Start Scheduler", "Start")
        };

        info!("{verb} {sched_name} ({mode} mode)");
        let description = format!(
            "{label_verb}ing scx_{} ({} mode)...",
            strip_prefix(&sched_name),
            mode
        );
        let seq = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("scxctl")
                    .args(&[verb, "--sched", &sched_name, "--mode", &mode])
                    .description(&description)
                    .build(),
            )
            .build();
        task_runner::run(self.window.upcast_ref(), seq, title);
    }

    fn confirm_stop(self: &Rc<Self>) {
        let me = self.clone();
        show_warning_confirmation(
            self.window.upcast_ref(),
            "Stop Scheduler",
            "Stop the current scheduler and fall back to EEVDF?",
            move || {
                let seq = CommandSequence::new()
                    .then(
                        Command::builder()
                            .normal()
                            .program("scxctl")
                            .args(&["stop"])
                            .description("Stopping scheduler...")
                            .build(),
                    )
                    .build();
                task_runner::run(me.window.upcast_ref(), seq, "Stop Scheduler");
            },
        );
    }

    // -- passive status polling --------------------------------------------

    fn poll_status(self: &Rc<Self>) {
        let status = ScxStatus::current();
        self.state.borrow_mut().active = status.active;
        render_active(&self.builder, &status);
        extract_widget::<Button>(&self.builder, "btn_stop_scheduler")
            .set_sensitive(status.active);
    }

    // -- full rescan (schedulers + kernel support + status) ----------------

    fn rescan(self: &Rc<Self>, button: Option<Button>) {
        let lock = ControlLock::engage(&self.builder, button);
        let (tx, rx) = mpsc::channel::<SchedScan>();

        thread::spawn(move || {
            let _ = tx.send(SchedScan::collect());
        });

        let me = self.clone();
        let lock_slot = RefCell::new(Some(lock));
        glib::timeout_add_local(POLL, move || match rx.try_recv() {
            Ok(scan) => {
                me.apply_scan(&scan);
                if let Some(lock) = lock_slot.borrow_mut().take() {
                    lock.release(
                        &me.builder,
                        scan.kernel_supported && !scan.schedulers.is_empty(),
                        scan.active,
                    );
                }
                glib::ControlFlow::Break
            }
            Err(mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(mpsc::TryRecvError::Disconnected) => {
                warn!("scheduler scan worker dropped without a result");
                if let Some(lock) = lock_slot.borrow_mut().take() {
                    lock.release(&me.builder, false, false);
                }
                glib::ControlFlow::Break
            }
        });
    }

    fn apply_scan(self: &Rc<Self>, scan: &SchedScan) {
        {
            let mut s = self.state.borrow_mut();
            s.schedulers = scan.schedulers.clone();
            s.kernel_supported = scan.kernel_supported;
            s.active = scan.active;
            if s.picked.is_none() && !scan.schedulers.is_empty() {
                s.picked = Some(default_pick(&scan.schedulers));
            }
        }

        if let Some(pick) = self.state.borrow().picked.clone() {
            extract_widget::<Label>(&self.builder, "selected_scheduler_label")
                .set_label(&humanize(&pick));
        }

        render_active(
            &self.builder,
            &ScxStatus {
                active: scan.active,
                name: scan.name.clone(),
                mode: scan.mode.clone(),
            },
        );

        extract_widget::<adw::SwitchRow>(&self.builder, "persist_switch")
            .set_active(is_service_enabled(SERVICE_NAME));

        info!(
            "scheduler scan: {} schedulers, active={}",
            scan.schedulers.len(),
            scan.active
        );
    }

    fn current_mode(&self) -> String {
        get_combo_row_value(&extract_widget::<adw::ComboRow>(&self.builder, "mode_combo"))
            .unwrap_or_else(|| "auto".to_owned())
    }
}

// ---------------------------------------------------------------------------
// Scan result + status query
// ---------------------------------------------------------------------------

struct SchedScan {
    schedulers: Vec<String>,
    kernel_supported: bool,
    active: bool,
    name: String,
    mode: String,
}

impl SchedScan {
    fn collect() -> Self {
        let schedulers = list_schedulers();
        let status = ScxStatus::current();
        Self {
            schedulers,
            kernel_supported: path_exists(SCHED_EXT_PATH),
            active: status.active,
            name: status.name,
            mode: status.mode,
        }
    }
}

struct ScxStatus {
    active: bool,
    name: String,
    mode: String,
}

impl ScxStatus {
    fn current() -> Self {
        let Some(out) = run_command("scxctl", &["get"]) else {
            return Self::inactive();
        };
        let lower = out.to_lowercase();
        if out.is_empty() || lower.contains("not running") {
            return Self::inactive();
        }
        if !lower.starts_with("running") {
            return Self::inactive();
        }
        let parts: Vec<&str> = out.split_whitespace().collect();
        if parts.len() < 2 {
            return Self::inactive();
        }
        let name = format!("scx_{}", parts[1].to_lowercase());
        let mode = out
            .split(" in ")
            .nth(1)
            .and_then(|s| s.split(" mode").next())
            .map(|s| s.trim().to_owned())
            .unwrap_or_else(|| "N/A".to_owned());
        Self {
            active: true,
            name,
            mode,
        }
    }

    fn inactive() -> Self {
        Self {
            active: false,
            name: String::new(),
            mode: String::new(),
        }
    }
}

fn list_schedulers() -> Vec<String> {
    let Some(out) = run_command("scxctl", &["list"]) else {
        return Vec::new();
    };
    const MARKER: &str = "supported schedulers:";
    let Some(heading) = out.find(MARKER) else {
        return Vec::new();
    };
    let tail = &out[heading + MARKER.len()..];
    let Some(open) = tail.find('[') else {
        return Vec::new();
    };
    let Some(close_rel) = tail[open..].find(']') else {
        return Vec::new();
    };
    let body = &tail[open + 1..open + close_rel];
    body.split(',')
        .map(|s| format!("scx_{}", s.trim().trim_matches('"')))
        .filter(|s| s.len() > 4)
        .collect()
}

fn default_pick(available: &[String]) -> String {
    for preferred in DEFAULT_PICK_ORDER {
        if available.iter().any(|s| s == preferred) {
            return (*preferred).to_owned();
        }
    }
    available[0].clone()
}

// ---------------------------------------------------------------------------
// Rendering + formatting helpers
// ---------------------------------------------------------------------------

fn render_active(builder: &Builder, status: &ScxStatus) {
    let label = extract_widget::<Label>(builder, "active_scheduler_label");
    if status.active {
        label.set_text(&format!("{} ({})", humanize(&status.name), status.mode));
        label.remove_css_class("dim-label");
        label.add_css_class("accent");
    } else {
        label.set_text("EEVDF (Default)");
        label.remove_css_class("accent");
        label.add_css_class("dim-label");
    }
}

fn humanize(name: &str) -> String {
    let core = strip_prefix(name);
    let mut chars = core.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn strip_prefix(name: &str) -> &str {
    name.strip_prefix("scx_").unwrap_or(name)
}

fn priv_cmd(program: &str, args: &[&str], description: &str) -> Command {
    Command::builder()
        .privileged()
        .program(program)
        .args(args)
        .description(description)
        .build()
}

// ---------------------------------------------------------------------------
// UI lock used while a scan is in flight
// ---------------------------------------------------------------------------

struct ControlLock {
    refresh_btn: Option<Button>,
}

impl ControlLock {
    fn engage(builder: &Builder, button: Option<Button>) -> Self {
        let c = Controls::fetch(builder);
        c.set_all_sensitive(false);
        if let Some(b) = &button {
            b.set_sensitive(false);
            toggle_spin(b, true);
        }
        Self {
            refresh_btn: button,
        }
    }

    fn release(self, builder: &Builder, can_switch: bool, can_stop: bool) {
        let c = Controls::fetch(builder);
        c.row.set_sensitive(true);
        c.mode.set_sensitive(true);
        c.persist.set_sensitive(true);
        c.switch.set_sensitive(can_switch);
        c.stop.set_sensitive(can_stop);
        if let Some(b) = self.refresh_btn {
            b.set_sensitive(true);
            toggle_spin(&b, false);
        }
    }
}

struct Controls {
    row: adw::ActionRow,
    mode: adw::ComboRow,
    switch: Button,
    stop: Button,
    persist: adw::SwitchRow,
}

impl Controls {
    fn fetch(builder: &Builder) -> Self {
        Self {
            row: extract_widget(builder, "scheduler_selection_row"),
            mode: extract_widget(builder, "mode_combo"),
            switch: extract_widget(builder, "btn_switch_scheduler"),
            stop: extract_widget(builder, "btn_stop_scheduler"),
            persist: extract_widget(builder, "persist_switch"),
        }
    }

    fn set_all_sensitive(&self, on: bool) {
        self.row.set_sensitive(on);
        self.mode.set_sensitive(on);
        self.switch.set_sensitive(on);
        self.stop.set_sensitive(on);
        self.persist.set_sensitive(on);
    }
}

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

// ---------------------------------------------------------------------------
// Scheduler selector dialog
// ---------------------------------------------------------------------------

fn present_selector<F>(
    parent: &ApplicationWindow,
    available: &[String],
    current: Option<&str>,
    on_select: F,
) where
    F: Fn(String) + 'static,
{
    let builder = Builder::from_resource(crate::config::resources::dialogs::SCHEDULER_SELECTION);
    let window: adw::Window = extract_widget(&builder, "scheduler_selection_window");
    window.set_transient_for(Some(parent));

    let content: GtkBox = extract_widget(&builder, "schedulers_container");
    let window_weak = window.downgrade();
    let on_select = Rc::new(on_select);
    let mut placed: HashSet<String> = HashSet::new();

    for (title, members) in GROUPS {
        let items: Vec<&&str> = members
            .iter()
            .filter(|m| available.iter().any(|a| a == **m))
            .collect();
        if items.is_empty() {
            continue;
        }
        let group = adw::PreferencesGroup::new();
        group.set_title(title);
        for item in items {
            placed.insert((*item).to_owned());
            group.add(&selector_row(item, current, &on_select, &window_weak));
        }
        content.append(&group);
    }

    let mut others: Vec<&String> = available.iter().filter(|s| !placed.contains(*s)).collect();
    others.sort();
    if !others.is_empty() {
        let group = adw::PreferencesGroup::new();
        group.set_title("Other");
        for item in others {
            group.add(&selector_row(item, current, &on_select, &window_weak));
        }
        content.append(&group);
    }

    window.present();
}

fn selector_row<F>(
    item: &str,
    current: Option<&str>,
    on_select: &Rc<F>,
    window_weak: &glib::WeakRef<adw::Window>,
) -> adw::ActionRow
where
    F: Fn(String) + 'static,
{
    let row = adw::ActionRow::new();
    row.set_title(&humanize(item));
    if current == Some(item) {
        row.add_suffix(&gtk4::Image::from_icon_name("circle-check-symbolic"));
    }
    row.set_activatable(true);

    let on_select = on_select.clone();
    let window_weak = window_weak.clone();
    let item = item.to_owned();
    row.connect_activated(move |_| {
        on_select(item.clone());
        if let Some(win) = window_weak.upgrade() {
            win.close();
        }
    });
    row
}
