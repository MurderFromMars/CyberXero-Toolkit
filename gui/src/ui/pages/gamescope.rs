//! Gamescope command-builder page.
//!
//! Every form widget is described by a [`FlagBinding`] — an entry, switch,
//! or combo row paired with the gamescope flag it emits. The form listens
//! for changes on every bound widget, re-renders the command, and copies
//! it to the clipboard on demand.

use std::rc::Rc;

use adw::prelude::*;
use adw::{ComboRow, EntryRow};
use gtk4::{ApplicationWindow, Builder, Button, StringObject, Switch};
use log::info;

use crate::ui::utils::extract_widget;

/// Fixed prefix and suffix wrapping the generated command.
const CMD_HEAD: &str = "gamescope";
const CMD_TAIL: &[&str] = &["--", "%command%"];

pub fn setup_handlers(
    page_builder: &Builder,
    _main_builder: &Builder,
    _window: &ApplicationWindow,
) {
    let form = Rc::new(GamescopeForm::load(page_builder));
    form.wire_change_watchers();
    form.bind_copy_button(page_builder);
    form.refresh();
}

struct GamescopeForm {
    bindings: Vec<FlagBinding>,
    extras: EntryRow,
    output: EntryRow,
}

impl GamescopeForm {
    /// Declarative list of every flag this form can emit. The order here is
    /// the order the flags appear in the generated command.
    fn load(b: &Builder) -> Self {
        let bindings = vec![
            // Output (Visual)
            FlagBinding::entry(b, "entry_output_width", "-W"),
            FlagBinding::entry(b, "entry_output_height", "-H"),
            FlagBinding::entry(b, "entry_max_scale", "-m"),
            // Nested (Game)
            FlagBinding::entry(b, "entry_nested_width", "-w"),
            FlagBinding::entry(b, "entry_nested_height", "-h"),
            FlagBinding::entry(b, "entry_nested_refresh", "-r"),
            // Scaler / Filter — "auto"/"linear" are the defaults we suppress
            FlagBinding::combo(b, "combo_scaler", "-S", "auto"),
            FlagBinding::combo(b, "combo_filter", "-F", "linear"),
            FlagBinding::entry(b, "entry_fsr_sharpness", "--fsr-sharpness"),
            // General gameplay switches
            FlagBinding::switch(b, "check_fullscreen", "-f"),
            FlagBinding::switch(b, "check_grab", "-g"),
            FlagBinding::switch(b, "check_force_grab_cursor", "--force-grab-cursor"),
            FlagBinding::switch(b, "check_adaptive_sync", "--adaptive-sync"),
            FlagBinding::switch(b, "check_immediate_flips", "--immediate-flips"),
            FlagBinding::switch(b, "check_expose_wayland", "--expose-wayland"),
            FlagBinding::switch(
                b,
                "check_force_windows_fullscreen",
                "--force-windows-fullscreen",
            ),
            // Backend / HDR / Cursor / FPS cap
            FlagBinding::combo(b, "combo_backend", "--backend", "auto"),
            FlagBinding::switch(b, "check_hdr_enabled", "--hdr-enabled"),
            FlagBinding::entry(b, "entry_cursor_path", "--cursor"),
            FlagBinding::entry(b, "entry_framerate_limit", "--framerate-limit"),
            // Debug + performance
            FlagBinding::switch(b, "check_debug_layers", "--debug-layers"),
            FlagBinding::switch(b, "check_mangoapp", "--mangoapp"),
            FlagBinding::switch(b, "check_realtime", "--rt"),
        ];
        let extras = extract_widget::<EntryRow>(b, "entry_extra_flags");
        let output = extract_widget::<EntryRow>(b, "text_command_output");
        Self {
            bindings,
            extras,
            output,
        }
    }

    fn render_command(&self) -> String {
        let mut parts = Vec::with_capacity(self.bindings.len() + 4);
        parts.push(CMD_HEAD.to_owned());
        for binding in &self.bindings {
            if let Some(piece) = binding.serialize() {
                parts.push(piece);
            }
        }
        let extras = self.extras.text();
        if !extras.is_empty() {
            parts.push(extras.to_string());
        }
        for tail in CMD_TAIL {
            parts.push((*tail).to_owned());
        }
        parts.join(" ")
    }

    fn refresh(&self) {
        self.output.set_text(&self.render_command());
    }

    fn wire_change_watchers(self: &Rc<Self>) {
        for binding in &self.bindings {
            let me = self.clone();
            binding.on_change(move || me.refresh());
        }
        let me = self.clone();
        self.extras
            .connect_notify_local(Some("text"), move |_, _| me.refresh());
    }

    fn bind_copy_button(self: &Rc<Self>, b: &Builder) {
        let btn = extract_widget::<Button>(b, "btn_copy_command");
        let output = self.output.clone();
        btn.connect_clicked(move |_| {
            let Some(display) = gtk4::gdk::Display::default() else {
                return;
            };
            display.clipboard().set(&output.text().to_string());
            info!("gamescope command copied to clipboard");
        });
    }
}

// ---------------------------------------------------------------------------
// Flag bindings
// ---------------------------------------------------------------------------

/// One widget-to-flag binding. Variants carry the extra bits needed to
/// serialize the widget's current value and subscribe to its change signal.
enum FlagBinding {
    Entry {
        flag: &'static str,
        widget: EntryRow,
    },
    Switch {
        flag: &'static str,
        widget: Switch,
    },
    Combo {
        flag: &'static str,
        widget: ComboRow,
        suppress: &'static str,
    },
}

impl FlagBinding {
    fn entry(b: &Builder, id: &str, flag: &'static str) -> Self {
        Self::Entry {
            flag,
            widget: extract_widget(b, id),
        }
    }

    fn switch(b: &Builder, id: &str, flag: &'static str) -> Self {
        Self::Switch {
            flag,
            widget: extract_widget(b, id),
        }
    }

    fn combo(b: &Builder, id: &str, flag: &'static str, suppress: &'static str) -> Self {
        Self::Combo {
            flag,
            widget: extract_widget(b, id),
            suppress,
        }
    }

    /// Returns the flag fragment for the current widget state, or `None`
    /// when the widget contributes nothing (empty entry, off switch, combo
    /// at its suppress value).
    fn serialize(&self) -> Option<String> {
        match self {
            FlagBinding::Entry { flag, widget } => {
                let text = widget.text();
                if text.is_empty() {
                    None
                } else {
                    Some(format!("{flag} {text}"))
                }
            }
            FlagBinding::Switch { flag, widget } => {
                widget.is_active().then(|| (*flag).to_owned())
            }
            FlagBinding::Combo {
                flag,
                widget,
                suppress,
            } => {
                let value = combo_value(widget)?;
                if value == *suppress {
                    None
                } else {
                    Some(format!("{flag} {value}"))
                }
            }
        }
    }

    fn on_change<F>(&self, on_change: F)
    where
        F: Fn() + 'static,
    {
        match self {
            FlagBinding::Entry { widget, .. } => {
                widget.connect_notify_local(Some("text"), move |_, _| on_change());
            }
            FlagBinding::Switch { widget, .. } => {
                widget.connect_active_notify(move |_| on_change());
            }
            FlagBinding::Combo { widget, .. } => {
                widget.connect_selected_notify(move |_| on_change());
            }
        }
    }
}

fn combo_value(combo: &ComboRow) -> Option<String> {
    combo
        .selected_item()
        .and_then(|item| item.downcast_ref::<StringObject>().map(|s| s.string().to_string()))
}
