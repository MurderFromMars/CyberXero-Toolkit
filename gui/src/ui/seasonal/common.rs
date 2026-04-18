//! Shared scaffolding for seasonal overlay effects.
//!
//! Seasonal effects all follow the same lifecycle: create a transparent
//! `DrawingArea`, install a ~60 fps redraw timer, let a particle
//! simulation advance each tick, and overlay the area on top of the main
//! window content. The machinery is packaged up here so each concrete
//! effect only has to supply its own [`ParticleField`] implementation.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use adw::prelude::*;
use gtk4::cairo;
use gtk4::glib;
use gtk4::{Align, ApplicationWindow, DrawingArea, EventControllerMotion, Widget};
use log::info;

/// Target framerate for seasonal animations (roughly 60 fps).
const TICK: Duration = Duration::from_millis(16);

/// Upper bound on per-frame simulation time. Longer gaps (background tab,
/// paused event loop) still only advance `MAX_STEP` seconds so the
/// simulation doesn't teleport after a pause.
const MAX_STEP: f64 = 0.1;

// ---------------------------------------------------------------------------
// Mouse tracking
// ---------------------------------------------------------------------------

/// Shared cursor position in window coordinates. Effects that react to the
/// pointer borrow this; effects that don't ignore it.
pub struct MouseContext {
    pos: Rc<RefCell<(f64, f64)>>,
}

impl MouseContext {
    pub(super) fn position(&self) -> Rc<RefCell<(f64, f64)>> {
        self.pos.clone()
    }
}

/// Attach an `EventControllerMotion` to `window` that keeps the returned
/// [`MouseContext`] up to date with the current cursor position.
pub fn setup_mouse_tracking(window: &ApplicationWindow) -> MouseContext {
    let pos = Rc::new(RefCell::new((0.0_f64, 0.0_f64)));
    let motion = EventControllerMotion::new();
    let sink = pos.clone();
    motion.connect_motion(move |_, x, y| {
        *sink.borrow_mut() = (x, y);
    });
    window.add_controller(motion);
    MouseContext { pos }
}

// ---------------------------------------------------------------------------
// ParticleField trait + mount_effect harness
// ---------------------------------------------------------------------------

/// A time-stepped particle simulation that renders onto a Cairo surface.
///
/// The harness constructs one of these on the first draw (once the real
/// surface size is known) and calls [`tick`](Self::tick),
/// [`paint`](Self::paint), [`handle_resize`](Self::handle_resize) over the
/// lifetime of the effect.
pub trait ParticleField {
    /// Advance the simulation by `dt` seconds. `mouse` is the current
    /// cursor position in window coordinates; effects that don't care
    /// about the pointer can ignore it.
    fn tick(&mut self, width: f64, height: f64, dt: f64, mouse: (f64, f64));

    /// Paint the current state onto `cr`.
    fn paint(&self, cr: &cairo::Context, width: f64, height: f64);

    /// Called when the containing drawing area changes size. Typical
    /// implementations rescale particle positions proportionally so the
    /// simulation still fills the visible area after a resize.
    fn handle_resize(&mut self, new_width: f64, new_height: f64);
}

/// Build a particle field, wire it up to `window`, and register it with
/// the seasonal toggle. Returns the drawing area on success, or `None` if
/// the overlay could not be attached (e.g. non-Adw window).
pub fn mount_effect<F, C>(
    window: &ApplicationWindow,
    mouse_context: Option<&MouseContext>,
    ctor: C,
) -> Option<Rc<DrawingArea>>
where
    F: ParticleField + 'static,
    C: Fn(f64, f64) -> F + 'static,
{
    let area = new_transparent_area();
    let mouse = mouse_context
        .map(MouseContext::position)
        .unwrap_or_else(|| Rc::new(RefCell::new((0.0, 0.0))));

    // State is filled lazily from `ctor` on the first draw, when we know
    // the real drawing area dimensions.
    let state: Rc<RefCell<Option<SimState<F>>>> = Rc::new(RefCell::new(None));

    install_draw_func(&area, state.clone(), Rc::new(ctor), mouse);
    install_resize_handler(&area, state);

    let timer_slot: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    let redraw_target = area.clone();
    let source_id = glib::timeout_add_local(TICK, move || {
        redraw_target.queue_draw();
        glib::ControlFlow::Continue
    });
    *timer_slot.borrow_mut() = Some(source_id);

    if !attach_overlay(window, &area) {
        info!("could not attach seasonal overlay");
        if let Some(id) = timer_slot.borrow_mut().take() {
            id.remove();
        }
        return None;
    }

    super::register_effect(area.clone(), timer_slot);
    Some(area)
}

/// Holds the live simulation plus the timestamp of the last tick so the
/// draw function can compute `dt` frame-to-frame.
struct SimState<F: ParticleField> {
    field: F,
    last_tick: Instant,
}

fn install_draw_func<F, C>(
    area: &Rc<DrawingArea>,
    state: Rc<RefCell<Option<SimState<F>>>>,
    ctor: Rc<C>,
    mouse: Rc<RefCell<(f64, f64)>>,
) where
    F: ParticleField + 'static,
    C: Fn(f64, f64) -> F + 'static,
{
    area.set_draw_func(move |_, cr, width, height| {
        let w = width as f64;
        let h = height as f64;

        let mut slot = state.borrow_mut();
        if slot.is_none() {
            *slot = Some(SimState {
                field: ctor(w, h),
                last_tick: Instant::now(),
            });
        }
        let Some(sim) = slot.as_mut() else { return };

        let now = Instant::now();
        let dt = now
            .duration_since(sim.last_tick)
            .as_secs_f64()
            .min(MAX_STEP);
        sim.last_tick = now;

        let mouse = *mouse.borrow();
        sim.field.tick(w, h, dt, mouse);

        clear_frame(cr);
        sim.field.paint(cr, w, h);
    });
}

fn install_resize_handler<F: ParticleField + 'static>(
    area: &Rc<DrawingArea>,
    state: Rc<RefCell<Option<SimState<F>>>>,
) {
    area.connect_resize(move |da, w, h| {
        if w <= 0 || h <= 0 {
            return;
        }
        if let Some(sim) = state.borrow_mut().as_mut() {
            sim.field.handle_resize(w as f64, h as f64);
        }
        da.queue_draw();
    });
}

fn new_transparent_area() -> Rc<DrawingArea> {
    let area = DrawingArea::new();
    area.set_hexpand(true);
    area.set_vexpand(true);
    area.set_can_focus(false);
    area.set_sensitive(false);
    area.set_halign(Align::Fill);
    area.set_valign(Align::Fill);
    area.set_visible(super::are_effects_enabled());
    Rc::new(area)
}

/// Wipe the drawing area to fully transparent. Called before every paint
/// so we don't accumulate translucent trails.
fn clear_frame(cr: &cairo::Context) {
    let _ = cr.save();
    cr.set_operator(cairo::Operator::Clear);
    let _ = cr.paint();
    cr.set_operator(cairo::Operator::Over);
    let _ = cr.restore();
}

/// Wrap `window`'s content in a GtkOverlay (if it isn't already wrapped)
/// and add `area` as an overlay child on top.
fn attach_overlay(window: &ApplicationWindow, area: &DrawingArea) -> bool {
    let Some(adw_win) = window.downcast_ref::<adw::ApplicationWindow>() else {
        info!("window is not an AdwApplicationWindow");
        return false;
    };
    let Some(content) = adw_win.content() else {
        info!("window has no content widget to overlay on");
        return false;
    };
    if content.downcast_ref::<adw::ToolbarView>().is_none() {
        info!("window content is not a ToolbarView — overlay may misbehave");
    }

    if let Some(existing) = content.downcast_ref::<gtk4::Overlay>() {
        existing.add_overlay(area);
        return true;
    }

    // First effect — wrap the existing content in a new Overlay so later
    // effects can stack on top without re-wrapping.
    let overlay = gtk4::Overlay::new();
    adw_win.set_content(Option::<&Widget>::None);
    overlay.set_child(Some(&content));
    overlay.add_overlay(area);
    adw_win.set_content(Some(&overlay));
    true
}

// ---------------------------------------------------------------------------
// Shared resize helper used by effect implementations
// ---------------------------------------------------------------------------

/// Rescale `(x, y)` proportionally when the containing surface resizes
/// from `(prev_w, prev_h)` to `(new_w, new_h)`. `(new_w, new_h)` is
/// returned so the caller can update its stored dimensions in one step.
pub fn rescale_point(
    (x, y): (&mut f64, &mut f64),
    prev: (f64, f64),
    new_size: (f64, f64),
) {
    if prev.0 <= 0.0 || prev.1 <= 0.0 {
        return;
    }
    *x *= new_size.0 / prev.0;
    *y *= new_size.1 / prev.1;
}
