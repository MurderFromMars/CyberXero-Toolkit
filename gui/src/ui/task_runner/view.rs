//! Widget facade for the runner window. The pipeline never touches GTK
//! objects directly — it goes through this struct so the state machine and
//! the UI can evolve independently.

use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Builder, Button, Image, Label, Revealer, ScrolledWindow, Separator, TextBuffer,
    TextTag, TextView, ToggleButton, Window,
};

use crate::ui::utils::extract_widget;

use super::Command;

/// Visual state of a single step in the sidebar list.
#[derive(Clone, Copy, Debug)]
pub(super) enum StepState {
    Pending,
    Running,
    Success,
    Failed,
    Cancelled,
}

/// Named text-tag slot for the output sidebar.
#[derive(Clone, Copy, Debug)]
pub(super) enum Tag {
    Header,
    Stdout,
    Stderr,
    Error,
}

impl Tag {
    fn id(self) -> &'static str {
        match self {
            Tag::Header => "header",
            Tag::Stdout => "stdout",
            Tag::Stderr => "stderr",
            Tag::Error => "error",
        }
    }
}

/// Each step is a horizontal row holding description + either a spinner
/// or a terminal status icon.
struct StepRow {
    container: GtkBox,
    spinner: Image,
    result: Image,
}

impl StepRow {
    fn new(description: &str) -> Self {
        let container = GtkBox::new(gtk4::Orientation::Horizontal, 12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_margin_start(12);
        container.set_margin_end(12);

        let label = Label::new(Some(description));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        label.set_wrap(true);

        let spinner = Image::new();
        spinner.set_icon_name(Some("circle-noth-symbolic"));
        spinner.set_pixel_size(24);
        spinner.set_visible(false);
        spinner.add_css_class("spinning");

        let result = Image::new();
        result.set_pixel_size(24);
        result.set_visible(false);

        container.append(&label);
        container.append(&spinner);
        container.append(&result);

        Self {
            container,
            spinner,
            result,
        }
    }

    fn apply(&self, state: StepState) {
        let (spinner_on, icon) = match state {
            StepState::Pending => (false, None),
            StepState::Running => (true, None),
            StepState::Success => (false, Some("circle-check")),
            StepState::Failed => (false, Some("circle-xmark")),
            StepState::Cancelled => (false, Some("circle-stop")),
        };
        self.spinner.set_visible(spinner_on);
        match icon {
            Some(name) => {
                self.result.set_icon_name(Some(name));
                self.result.set_visible(true);
            }
            None => {
                self.result.set_visible(false);
            }
        }
    }
}

pub(super) struct RunnerView {
    window: Window,
    title: Label,
    cancel_btn: Button,
    close_btn: Button,
    scrolled: ScrolledWindow,
    rows: Vec<StepRow>,
    output_view: TextView,
    output_buf: TextBuffer,
    sidebar_toggle: ToggleButton,
    sidebar_revealer: Revealer,
}

impl RunnerView {
    /// Wire up the window from the bundled .ui resource and populate one row
    /// per step. The returned [`Rc`] is what the pipeline clones to attach
    /// signal handlers.
    pub(super) fn from_builder(builder: &Builder, steps: &[Command]) -> Rc<Self> {
        let window: Window = extract_widget(builder, "task_window");
        let title: Label = extract_widget(builder, "task_title");
        let list: GtkBox = extract_widget(builder, "task_list_container");
        let scrolled: ScrolledWindow = extract_widget(builder, "task_scrolled_window");
        let cancel_btn: Button = extract_widget(builder, "cancel_button");
        let close_btn: Button = extract_widget(builder, "close_button");
        let sidebar_toggle: ToggleButton = extract_widget(builder, "sidebar_toggle_button");
        let sidebar_revealer: Revealer = extract_widget(builder, "sidebar_revealer");
        let output_view: TextView = extract_widget(builder, "output_text_view");
        let output_buf = output_view.buffer();

        let mut rows = Vec::with_capacity(steps.len());
        let last = steps.len().saturating_sub(1);
        for (i, step) in steps.iter().enumerate() {
            let row = StepRow::new(&step.description);
            row.apply(StepState::Pending);
            list.append(&row.container);
            if i != last {
                list.append(&Separator::new(gtk4::Orientation::Horizontal));
            }
            rows.push(row);
        }

        output_buf.set_text("Command output will appear here as each step runs.\n\n");

        let this = Rc::new(Self {
            window,
            title,
            cancel_btn,
            close_btn,
            scrolled,
            rows,
            output_view,
            output_buf,
            sidebar_toggle,
            sidebar_revealer,
        });

        this.install_tags();
        this.bind_sidebar();
        this.collapse_sidebar();

        this
    }

    fn install_tags(&self) {
        // (id, foreground, optional bold-weight)
        const SPECS: &[(&str, &str, Option<i32>)] = &[
            ("header", "rgb(100, 149, 237)", Some(700)),
            ("stdout", "rgb(46, 204, 113)", None),
            ("stderr", "rgb(255, 140, 0)", None),
            ("error", "rgb(231, 76, 60)", Some(700)),
        ];

        let table = self.output_buf.tag_table();
        for &(id, color, weight) in SPECS {
            let tag = TextTag::new(Some(id));
            tag.set_property("foreground", color);
            if let Some(w) = weight {
                tag.set_property("weight", w);
            }
            table.add(&tag);
        }
    }

    fn bind_sidebar(&self) {
        self.sidebar_toggle
            .bind_property("active", &self.sidebar_revealer, "reveal-child")
            .sync_create()
            .bidirectional()
            .build();

        let toggle = self.sidebar_toggle.clone();
        let revealer = self.sidebar_revealer.clone();
        self.sidebar_revealer.connect_reveal_child_notify(move |r| {
            let showing = r.reveals_child();
            toggle.set_tooltip_text(Some(if showing {
                "Hide command output"
            } else {
                "Show command output"
            }));
            // Prevent the collapsed pane from stealing pointer events.
            revealer.set_can_target(showing);
        });
    }

    fn collapse_sidebar(&self) {
        self.sidebar_toggle.set_active(false);
        self.sidebar_revealer.set_reveal_child(false);
    }

    // ----- accessors the pipeline uses ------------------------------------

    pub(super) fn window(&self) -> &Window {
        &self.window
    }

    pub(super) fn set_title(&self, text: &str) {
        self.title.set_text(text);
    }

    pub(super) fn disable_cancel(&self) {
        self.cancel_btn.set_sensitive(false);
    }

    pub(super) fn on_cancel<F: Fn() + 'static>(&self, handler: F) {
        self.cancel_btn.connect_clicked(move |_| handler());
    }

    pub(super) fn on_close<F: Fn() + 'static>(&self, handler: F) {
        self.close_btn.connect_clicked(move |_| handler());
    }

    pub(super) fn on_window_close<F: Fn() + 'static>(&self, handler: F) {
        self.window.connect_close_request(move |_| {
            handler();
            gtk4::glib::Propagation::Proceed
        });
    }

    pub(super) fn set_step_state(&self, index: usize, state: StepState) {
        if let Some(row) = self.rows.get(index) {
            row.apply(state);
            self.focus_step(index);
        }
    }

    /// Keep the active step in view without jumping the scroll when the user
    /// has manually scrolled to a still-visible location.
    fn focus_step(&self, index: usize) {
        let total = self.rows.len();
        if total == 0 || index >= total {
            return;
        }
        let adj = self.scrolled.vadjustment();
        let span = adj.upper();
        if span <= 0.0 {
            return;
        }
        let row_height = span / total as f64;
        let row_top = index as f64 * row_height;
        let row_bottom = row_top + row_height;
        let view_top = adj.value();
        let view_bottom = view_top + adj.page_size();

        if row_bottom > view_bottom {
            let max = (adj.upper() - adj.page_size()).max(0.0);
            adj.set_value((row_bottom - adj.page_size()).clamp(0.0, max));
        } else if row_top < view_top {
            adj.set_value(row_top.max(0.0));
        }
    }

    /// Append raw text with a style tag and scroll to the bottom.
    pub(super) fn append(&self, text: &str, tag: Tag) {
        let start_offset = self.output_buf.end_iter().offset();
        let mut end = self.output_buf.end_iter();
        self.output_buf.insert(&mut end, text);

        let start = self.output_buf.iter_at_offset(start_offset);
        let fresh_end = self.output_buf.end_iter();
        if let Some(t) = self.output_buf.tag_table().lookup(tag.id()) {
            self.output_buf.apply_tag(&t, &start, &fresh_end);
        }
        self.pin_output_to_bottom();
    }

    /// Strip ANSI escapes from captured subprocess output before appending.
    pub(super) fn append_stream(&self, text: &str, tag: Tag) {
        let cleaned = strip_ansi_escapes::strip_str(text);
        self.append(&cleaned, tag);
    }

    pub(super) fn emit_step_banner(&self, description: &str) {
        self.append(&format!("\n=== {} ===\n", description), Tag::Header);
    }

    fn pin_output_to_bottom(&self) {
        let mut end = self.output_buf.end_iter();
        let _ = self
            .output_view
            .scroll_to_iter(&mut end, 0.0, false, 0.0, 0.0);
    }

    /// Flip the window into its terminal state: hide Cancel, show Close, and
    /// style the title according to success/failure.
    pub(super) fn finalize(&self, success: bool, message: &str) {
        self.set_title(message);

        let (add, remove) = if success {
            ("success", "error")
        } else {
            ("error", "success")
        };
        self.title.add_css_class(add);
        self.title.remove_css_class(remove);

        if success {
            self.close_btn.add_css_class("suggested-action");
        } else {
            self.close_btn.remove_css_class("suggested-action");
        }

        self.cancel_btn.set_visible(false);
        self.close_btn.set_visible(true);
        self.close_btn.set_sensitive(true);
    }
}
