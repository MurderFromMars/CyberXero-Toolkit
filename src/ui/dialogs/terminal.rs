//! Interactive terminal dialog for running shell commands.

use gtk4::prelude::*;
use vte4::prelude::*;
use vte4::Terminal;
use gtk4::{Button, Window};
use gtk4::gdk::RGBA;
use std::str::FromStr;
use std::rc::Rc;
use std::cell::RefCell;
use log::{info, error};

fn update_terminal_style(terminal: &Terminal) {
    let style_manager = adw::StyleManager::default();
    let is_dark = style_manager.is_dark();

    let bg_color = if is_dark {
        RGBA::from_str("#1e1e1e").unwrap()
    } else {
        RGBA::from_str("#ffffff").unwrap()
    };

    let fg_color = if is_dark {
        RGBA::from_str("#ffffff").unwrap()
    } else {
        RGBA::from_str("#000000").unwrap()
    };

    // Gnome Console / Adwaita Palette
    let palette_strs = [
        "#241f31", "#c01c28", "#2ec27e", "#f5c211", "#1e78e4", "#9841bb", "#00c0a0", "#9a9996", // Normal
        "#5e5c64", "#ed333b", "#57e389", "#f8e45c", "#3584e4", "#9141ac", "#26a269", "#ffffff"  // Bright
    ];

    let palette: Vec<RGBA> = palette_strs
        .iter()
        .map(|s| RGBA::from_str(s).unwrap())
        .collect();

    let palette_refs: Vec<&RGBA> = palette.iter().collect();
    terminal.set_colors(Some(&fg_color), Some(&bg_color), &palette_refs);
}

/// Shows an interactive terminal window for the given command.
pub fn show_terminal_dialog(parent: &Window, title: &str, command: &str, args: &[&str]) {
    // Load the UI
    let builder = gtk4::Builder::from_resource(
        "/xyz/xerolinux/xero-toolkit/ui/dialogs/terminal_dialog.ui",
    );

    let window: adw::Window = builder
        .object("terminal_window")
        .expect("Failed to get terminal_window");
    let terminal: Terminal = builder
        .object("terminal")
        .expect("Failed to get terminal");
    let close_button: Button = builder
        .object("close_button")
        .expect("Failed to get close_button");

    window.set_transient_for(Some(parent));
    window.set_title(Some(title));
    
    // Set a nice monospace font
    let font_desc = gtk4::pango::FontDescription::from_string("Monospace 11");
    terminal.set_font(Some(&font_desc));

    // Setup theming
    update_terminal_style(&terminal);

    let terminal_weak = terminal.downgrade();
    let style_manager = adw::StyleManager::default();
    let signal_id = style_manager.connect_dark_notify(move |_| {
        if let Some(term) = terminal_weak.upgrade() {
            update_terminal_style(&term);
        }
    });

    // Clean up signal handler when window closes
    let signal_id_wrapper = Rc::new(RefCell::new(Some(signal_id)));
    let window_widget: &gtk4::Widget = window.as_ref();
    window_widget.connect_unmap(move |_| {
        if let Some(id) = signal_id_wrapper.borrow_mut().take() {
            adw::StyleManager::default().disconnect(id);
        }
    });

    // Setup close button
    let window_clone = window.clone();
    close_button.connect_clicked(move |_| {
        window_clone.close();
    });

    // Spawn the command
    let mut argv = vec![command.to_string()];
    argv.extend(args.iter().map(|s| s.to_string()));
    let argv_refs: Vec<&str> = argv.iter().map(|s| s.as_str()).collect();

    info!("Terminal: Spawning {:?} in interactive window", argv_refs);

    let close_button_clone = close_button.clone();
    terminal.spawn_async(
        vte4::PtyFlags::DEFAULT,
        None,
        &argv_refs,
        &[],
        gtk4::glib::SpawnFlags::SEARCH_PATH,
        || {}, // child setup
        -1,
        None::<&gtk4::gio::Cancellable>,
        move |result| {
            if let Err(e) = result {
                error!("Failed to spawn terminal command: {}", e);
            }
        }
    );

    // Enable close button when child exits
    terminal.connect_child_exited(move |_, _| {
        close_button_clone.set_sensitive(true);
    });

    window.present();
}
