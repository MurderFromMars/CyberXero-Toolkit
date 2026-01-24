//! SCX Scheduler page handlers.
//!
//! Manages sched-ext BPF CPU schedulers via scxctl.

use crate::ui::dialogs::warning::show_warning_confirmation;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::{
    extract_widget, get_combo_row_value, is_service_enabled, path_exists, run_command,
};
use adw::prelude::*;
use gtk4::glib;
use gtk4::{ApplicationWindow, Box as GtkBox, Builder, Button, Image, Label};
use log::{info, warn};
use std::cell::RefCell;
use std::rc::Rc;

const SCHED_EXT_PATH: &str = "/sys/kernel/sched_ext";

/// Shared state for the scheduler page
#[derive(Default)]
struct State {
    schedulers: Vec<String>,
    kernel_supported: bool,
    is_active: bool,
    selected_scheduler: Option<String>,
}

pub fn setup_handlers(builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    let state = Rc::new(RefCell::new(State::default()));

    init_kernel_support(builder, &state);
    setup_buttons(builder, window, &state);
    setup_persistence(builder, window, &state);

    // Initial scan
    let b = builder.clone();
    let s = Rc::clone(&state);
    glib::idle_add_local_once(move || refresh_state(&b, &s, None));

    // Status monitor
    let b = builder.clone();
    let s = Rc::clone(&state);
    glib::timeout_add_seconds_local(3, move || {
        update_status(&b, &s);
        glib::ControlFlow::Continue
    });
}

fn init_kernel_support(builder: &Builder, state: &Rc<RefCell<State>>) {
    let version = run_command("uname", &["-r"]).unwrap_or_else(|| "Unknown".to_string());
    let supported = path_exists(SCHED_EXT_PATH);

    state.borrow_mut().kernel_supported = supported;

    let icon = extract_widget::<Image>(builder, "kernel_status_icon");
    let label = extract_widget::<Label>(builder, "kernel_version_label");

    if supported {
        icon.set_icon_name(Some("circle-check"));
        icon.add_css_class("success");
        label.set_text(&version);
        label.remove_css_class("warning");
    } else {
        icon.set_icon_name(Some("circle-xmark"));
        icon.add_css_class("error");
        label.set_text(&format!("{} (no sched-ext)", version));
        label.add_css_class("warning");
    }

    // Hidden label for compatibility
    extract_widget::<Label>(builder, "kernel_support_label").set_text(if supported {
        "Supported"
    } else {
        "Not supported"
    });
}

fn setup_buttons(builder: &Builder, window: &ApplicationWindow, state: &Rc<RefCell<State>>) {
    // Scheduler Selection Row
    let b = builder.clone();
    let w = window.clone();
    let s = Rc::clone(state);
    extract_widget::<adw::ActionRow>(builder, "scheduler_selection_row").connect_activated(
        move |_| {
            let schedulers = s.borrow().schedulers.clone();
            let current = s.borrow().selected_scheduler.clone();
            let s = s.clone();
            let b = b.clone();

            show_scheduler_selector(&w, schedulers, current, move |selected| {
                s.borrow_mut().selected_scheduler = Some(selected.clone());
                extract_widget::<Label>(&b, "selected_scheduler_label")
                    .set_label(&humanize_name(&selected));
            });
        },
    );

    // Refresh button
    let b = builder.clone();
    let s = Rc::clone(state);
    extract_widget::<Button>(builder, "btn_refresh_schedulers").connect_clicked(move |btn| {
        refresh_state(&b, &s, Some(btn));
    });

    // Switch button
    let b = builder.clone();
    let w = window.clone();
    let s = Rc::clone(state);
    extract_widget::<Button>(builder, "btn_switch_scheduler").connect_clicked(move |_| {
        let scheduler = s.borrow().selected_scheduler.clone();
        let mode = get_combo_row_value(&extract_widget::<adw::ComboRow>(&b, "mode_combo"))
            .unwrap_or_else(|| "auto".to_string());

        let Some(sched_name) = scheduler else {
            warn!("No valid scheduler selected");
            return;
        };

        let sched = format!("scx_{}", sched_name);
        let cmd = if s.borrow().is_active {
            "switch"
        } else {
            "start"
        };

        info!("{cmd}ing scheduler {sched_name} with mode {mode}");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("scxctl")
                    .args(&[cmd, "--sched", &sched_name, "--mode", &mode])
                    .description(&format!(
                        "{}ing {} ({} mode)...",
                        if cmd == "switch" { "Switch" } else { "Start" },
                        sched,
                        mode
                    ))
                    .build(),
            )
            .build();

        task_runner::run(
            w.upcast_ref(),
            commands,
            if cmd == "switch" {
                "Switch Scheduler"
            } else {
                "Start Scheduler"
            },
        );
    });

    // Stop button
    let w = window.clone();
    extract_widget::<Button>(builder, "btn_stop_scheduler").connect_clicked(move |_| {
        let wc = w.clone();
        show_warning_confirmation(
            w.upcast_ref(),
            "Stop Scheduler",
            "Stop the current scheduler and fall back to EEVDF?",
            move || {
                task_runner::run(
                    wc.upcast_ref(),
                    CommandSequence::new()
                        .then(
                            Command::builder()
                                .normal()
                                .program("scxctl")
                                .args(&["stop"])
                                .description("Stopping scheduler...")
                                .build(),
                        )
                        .build(),
                    "Stop Scheduler",
                );
            },
        );
    });
}

fn setup_persistence(builder: &Builder, window: &ApplicationWindow, state: &Rc<RefCell<State>>) {
    let switch = extract_widget::<adw::SwitchRow>(builder, "persist_switch");
    switch.set_active(is_service_enabled("scx.service"));

    let b = builder.clone();
    let w = window.clone();
    let s = state.clone();
    switch.connect_active_notify(move |sw| {
        if sw.is_active() {
            let scheduler = s.borrow().selected_scheduler.clone();
            let mode = get_combo_row_value(&extract_widget::<adw::ComboRow>(&b, "mode_combo"))
                .unwrap_or_else(|| "auto".to_string());

            let Some(sched_name) = scheduler else {
                warn!("No valid scheduler selected for persistence");
                sw.set_active(false);
                return;
            };

            let sched = format!("scx_{}", sched_name);
            let template_path = crate::config::paths::systemd().join("scx.service.in");

            let Ok(content) = std::fs::read_to_string(&template_path) else {
                warn!("Failed to read service template");
                sw.set_active(false);
                return;
            };

            let service = content
                .replace("@SCHEDULER@", &sched)
                .replace("@SCHEDULER_NAME@", &sched_name)
                .replace("@MODE@", &mode);

            if std::fs::write("/tmp/scx.service", &service).is_err() {
                sw.set_active(false);
                return;
            }

            task_runner::run(
                w.upcast_ref(),
                CommandSequence::new()
                    .then(
                        Command::builder()
                            .privileged()
                            .program("cp")
                            .args(&["/tmp/scx.service", "/etc/systemd/system/scx.service"])
                            .description("Installing service...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .privileged()
                            .program("systemctl")
                            .args(&["daemon-reload"])
                            .description("Reloading systemd...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .privileged()
                            .program("systemctl")
                            .args(&["enable", "--now", "scx.service"])
                            .description("Enabling and starting service...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .privileged()
                            .program("mkdir")
                            .args(&["-p", "/etc/systemd/system/sysinit.target.wants"])
                            .description("Preparing sysinit target...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .privileged()
                            .program("ln")
                            .args(&[
                                "-sf",
                                "/etc/systemd/system/scx.service",
                                "/etc/systemd/system/sysinit.target.wants/scx.service",
                            ])
                            .description("Linking to sysinit...")
                            .build(),
                    )
                    .build(),
                "Enable Persistence",
            );
        } else {
            task_runner::run(
                w.upcast_ref(),
                CommandSequence::new()
                    .then(
                        Command::builder()
                            .privileged()
                            .program("systemctl")
                            .args(&["stop", "scx.service"])
                            .description("Stopping service...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .privileged()
                            .program("systemctl")
                            .args(&["disable", "scx.service"])
                            .description("Disabling service...")
                            .build(),
                    )
                    .build(),
                "Disable Persistence",
            );
        }
    });
}

fn refresh_state(builder: &Builder, state: &Rc<RefCell<State>>, refresh_btn: Option<&Button>) {
    let builder = builder.clone();
    let state = state.clone();
    let btn_opt = refresh_btn.cloned();

    // Disable controls while refreshing
    let row = extract_widget::<adw::ActionRow>(&builder, "scheduler_selection_row");
    let mode_combo = extract_widget::<adw::ComboRow>(&builder, "mode_combo");
    let switch_btn = extract_widget::<Button>(&builder, "btn_switch_scheduler");
    let stop_btn = extract_widget::<Button>(&builder, "btn_stop_scheduler");
    let persist = extract_widget::<adw::SwitchRow>(&builder, "persist_switch");

    row.set_sensitive(false);
    mode_combo.set_sensitive(false);
    switch_btn.set_sensitive(false);
    stop_btn.set_sensitive(false);
    persist.set_sensitive(false);

    if let Some(btn) = refresh_btn {
        btn.set_sensitive(false);
        // Try to find image child to animate
        if let Some(child) = btn.child() {
            if let Some(img) = child.downcast_ref::<Image>() {
                img.add_css_class("spinning");
            } else if let Some(box_child) = child.downcast_ref::<GtkBox>() {
                if let Some(img) = box_child.first_child().and_downcast::<Image>() {
                    img.add_css_class("spinning");
                }
            }
        }
    }

    // Use std::sync::mpsc for thread communication
    let (sender, receiver) =
        std::sync::mpsc::channel::<(Vec<String>, bool, String, String, bool)>();

    // Run blocking operations in a separate thread
    std::thread::spawn(move || {
        let schedulers = get_schedulers();
        let (is_active, name, mode) = get_status();
        let kernel_supported = path_exists(SCHED_EXT_PATH);
        let _ = sender.send((schedulers, is_active, name, mode, kernel_supported));
    });

    // Poll for results in main thread
    glib::timeout_add_local(
        std::time::Duration::from_millis(100),
        move || match receiver.try_recv() {
            Ok((schedulers, is_active, name, mode, kernel_supported)) => {
                {
                    let mut s = state.borrow_mut();
                    s.schedulers = schedulers.clone();
                    s.kernel_supported = kernel_supported;
                    s.is_active = is_active;
                }

                // Select default scheduler if none selected
                {
                    let mut s = state.borrow_mut();
                    if s.selected_scheduler.is_none() && !schedulers.is_empty() {
                        // Prefer scx_rusty or scx_lavd if available, otherwise first
                        if schedulers.iter().any(|s| s == "scx_rusty") {
                            s.selected_scheduler = Some("scx_rusty".to_string());
                        } else if schedulers.iter().any(|s| s == "scx_lavd") {
                            s.selected_scheduler = Some("scx_lavd".to_string());
                        } else {
                            s.selected_scheduler = Some(schedulers[0].clone());
                        }
                    }
                }

                // Update selected label
                if let Some(selected) = &state.borrow().selected_scheduler {
                    extract_widget::<Label>(&builder, "selected_scheduler_label")
                        .set_label(&humanize_name(selected));
                }

                // Update status display
                update_status_labels(&builder, is_active, &name, &mode);

                // Update buttons and re-enable controls
                row.set_sensitive(true);
                mode_combo.set_sensitive(true);
                persist.set_sensitive(true);

                let can_switch = kernel_supported && !schedulers.is_empty();
                switch_btn.set_sensitive(can_switch);
                stop_btn.set_sensitive(is_active);

                // Update persistence state
                persist.set_active(is_service_enabled("scx.service"));

                // Restore refresh button
                if let Some(btn) = &btn_opt {
                    btn.set_sensitive(true);
                    if let Some(child) = btn.child() {
                        if let Some(img) = child.downcast_ref::<Image>() {
                            img.remove_css_class("spinning");
                        } else if let Some(box_child) = child.downcast_ref::<GtkBox>() {
                            if let Some(img) = box_child.first_child().and_downcast::<Image>() {
                                img.remove_css_class("spinning");
                            }
                        }
                    }
                }

                info!(
                    "Found {} schedulers, active={}",
                    schedulers.len(),
                    is_active
                );

                glib::ControlFlow::Break
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                warn!("Scheduler scan thread disconnected");
                // Re-enable controls on failure
                row.set_sensitive(true);
                mode_combo.set_sensitive(true);
                switch_btn.set_sensitive(true);
                stop_btn.set_sensitive(true);
                persist.set_sensitive(true);

                if let Some(btn) = &btn_opt {
                    btn.set_sensitive(true);
                    if let Some(child) = btn.child() {
                        if let Some(img) = child.downcast_ref::<Image>() {
                            img.remove_css_class("spinning");
                        } else if let Some(box_child) = child.downcast_ref::<GtkBox>() {
                            if let Some(img) = box_child.first_child().and_downcast::<Image>() {
                                img.remove_css_class("spinning");
                            }
                        }
                    }
                }
                glib::ControlFlow::Break
            }
        },
    );
}

fn update_status(builder: &Builder, state: &Rc<RefCell<State>>) {
    let (is_active, name, mode) = get_status();
    state.borrow_mut().is_active = is_active;

    update_status_labels(builder, is_active, &name, &mode);
    extract_widget::<Button>(builder, "btn_stop_scheduler").set_sensitive(is_active);
}

fn update_status_labels(builder: &Builder, is_active: bool, name: &str, mode: &str) {
    let active_label = extract_widget::<Label>(builder, "active_scheduler_label");

    if is_active {
        active_label.set_text(&format!("{} ({})", humanize_name(name), mode));
        active_label.remove_css_class("dim-label");
        active_label.add_css_class("accent");
    } else {
        active_label.set_text("EEVDF (Default)");
        active_label.remove_css_class("accent");
        active_label.add_css_class("dim-label");
    }
}

fn get_schedulers() -> Vec<String> {
    run_command("scxctl", &["list"])
        .and_then(|out| {
            out.find("supported schedulers:")
                .and_then(|i| out[i + 21..].find('[').map(|j| i + 21 + j))
                .and_then(|start| {
                    out[start..]
                        .find(']')
                        .map(|end| &out[start + 1..start + end])
                })
                .map(|list| {
                    list.split(',')
                        .map(|s| format!("scx_{}", s.trim().trim_matches('"')))
                        .filter(|s| s.len() > 4)
                        .collect()
                })
        })
        .unwrap_or_default()
}

fn get_status() -> (bool, String, String) {
    run_command("scxctl", &["get"])
        .map(|out| {
            let lower = out.to_lowercase();
            if lower.contains("not running") || out.is_empty() {
                return (false, String::new(), String::new());
            }
            if lower.starts_with("running") {
                let parts: Vec<&str> = out.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = format!("scx_{}", parts[1].to_lowercase());
                    let mode = out
                        .split(" in ")
                        .nth(1)
                        .and_then(|s| s.split(" mode").next())
                        .map(|s| s.trim().to_string())
                        .unwrap_or_else(|| "N/A".to_string());
                    return (true, name, mode);
                }
            }
            (false, String::new(), String::new())
        })
        .unwrap_or((false, String::new(), String::new()))
}

fn show_scheduler_selector(
    parent: &ApplicationWindow,
    schedulers: Vec<String>,
    current_selected: Option<String>,
    on_select: impl Fn(String) + 'static,
) {
    // create window
    let window = adw::Window::builder()
        .title("Select Scheduler")
        .modal(true)
        .transient_for(parent)
        .default_width(400)
        .default_height(600)
        .build();

    let page = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    page.add_top_bar(&header);

    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // clamp for better width on wide screens
    let clamp = adw::Clamp::builder()
        .maximum_size(600)
        .child(&content)
        .build();

    // adjust scrolling hierarchy
    let scroll = gtk4::ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vexpand(true)
        .child(&clamp)
        .build();
    page.set_content(Some(&scroll));

    // Categories
    let categories = vec![
        ("Gaming", vec!["scx_rusty", "scx_lavd", "scx_bpfland"]),
        ("Desktop", vec!["scx_cosmos", "scx_flash"]),
        ("Servers", vec!["scx_layered", "scx_flatcg", "scx_tickless"]),
        ("Low Latency", vec!["scx_nest"]),
        ("Testing", vec!["scx_simple", "scx_chaos", "scx_userland"]),
    ];

    // Set for tracking used schedulers
    let mut added = std::collections::HashSet::new();

    let window_weak = window.downgrade();
    let on_select = Rc::new(on_select);

    for (cat_name, items) in categories {
        let group = adw::PreferencesGroup::new();
        group.set_title(cat_name);

        let mut has_items = false;

        for item in items {
            if schedulers.iter().any(|s| s == item) {
                has_items = true;
                added.insert(item.to_string());

                let row = adw::ActionRow::new();
                row.set_title(&humanize_name(item));

                if let Some(ref current) = current_selected {
                    if current == item {
                        row.add_suffix(&gtk4::Image::from_icon_name("object-select-symbolic"));
                    }
                }

                row.set_activatable(true);

                let on_select_clone = on_select.clone();
                let item_string = item.to_string();
                let win_weak = window_weak.clone();

                row.connect_activated(move |_| {
                    on_select_clone(item_string.clone());
                    if let Some(win) = win_weak.upgrade() {
                        win.close();
                    }
                });

                group.add(&row);
            }
        }

        if has_items {
            content.append(&group);
        }
    }

    // Others category
    let mut others = Vec::new();
    for sched in &schedulers {
        if !added.contains(sched) {
            others.push(sched);
        }
    }
    others.sort();

    if !others.is_empty() {
        let group = adw::PreferencesGroup::new();
        group.set_title("Other");
        for item in others {
            let row = adw::ActionRow::new();
            row.set_title(&humanize_name(item));

            if let Some(ref current) = current_selected {
                if current == item {
                    row.add_suffix(&gtk4::Image::from_icon_name("object-select-symbolic"));
                }
            }

            row.set_activatable(true);

            let on_select_clone = on_select.clone();
            let item_string = item.to_string();
            let win_weak = window_weak.clone();

            row.connect_activated(move |_| {
                on_select_clone(item_string.clone());
                if let Some(win) = win_weak.upgrade() {
                    win.close();
                }
            });

            group.add(&row);
        }
        content.append(&group);
    }

    window.set_content(Some(&page));
    window.present();
}

fn humanize_name(name: &str) -> String {
    let name = name.strip_prefix("scx_").unwrap_or(name);
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
