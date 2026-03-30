//! Seasonal overlay effects for the application window.
//!
//! This module provides animated overlay effects that appear during specific
//! times of the year (e.g., snow for December, Halloween effects for October).
//!
//! Each registered effect owns its `DrawingArea` **and** the GLib timer that
//! drives redraws.  When effects are disabled the timer is removed — no more
//! wasted CPU ticks while the overlay is hidden.  Re-enabling restarts it.

mod common;
mod halloween;
mod snow;

use crate::ui::seasonal::common::MouseContext;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, DrawingArea};
use log::info;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

pub use halloween::HalloweenEffect;
pub use snow::SnowEffect;

/// Global flag for whether seasonal effects are enabled.
static EFFECTS_ENABLED: AtomicBool = AtomicBool::new(true);

// ---------------------------------------------------------------------------
// Effect registry
// ---------------------------------------------------------------------------

/// One registered effect — its drawing area plus the timer that redraws it.
struct EffectEntry {
    drawing_area: Rc<DrawingArea>,
    /// The active GLib timeout source, if any.  `None` when the timer is stopped.
    timer_source: Rc<RefCell<Option<glib::SourceId>>>,
}

/// Thread-tagged wrapper so the `OnceLock` is happy.
///
/// SAFETY: GTK is single-threaded; all access happens on the main thread.
struct EffectRegistry(RefCell<Vec<EffectEntry>>);
unsafe impl Send for EffectRegistry {}
unsafe impl Sync for EffectRegistry {}

static EFFECT_REGISTRY: std::sync::OnceLock<EffectRegistry> = std::sync::OnceLock::new();

fn get_effect_registry() -> &'static RefCell<Vec<EffectEntry>> {
    &EFFECT_REGISTRY
        .get_or_init(|| EffectRegistry(RefCell::new(Vec::new())))
        .0
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Whether seasonal effects are currently enabled.
pub fn are_effects_enabled() -> bool {
    EFFECTS_ENABLED.load(Ordering::Relaxed)
}

/// Enable or disable all registered seasonal effects.
///
/// - **Disabling** hides every drawing area *and* removes its GLib timer so no
///   redraws fire while the overlay is invisible.
/// - **Enabling** makes the drawing areas visible again *and* restarts the
///   16 ms redraw timer for each one.
pub fn set_effects_enabled(enabled: bool) {
    EFFECTS_ENABLED.store(enabled, Ordering::Relaxed);

    let registry = get_effect_registry();
    for entry in registry.borrow().iter() {
        entry.drawing_area.set_visible(enabled);

        if enabled {
            // Restart the timer only if it isn't already running.
            let mut timer_ref = entry.timer_source.borrow_mut();
            if timer_ref.is_none() {
                let area = entry.drawing_area.clone();
                let source_id =
                    glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                        area.queue_draw();
                        glib::ControlFlow::Continue
                    });
                *timer_ref = Some(source_id);
                info!("Seasonal effect timer restarted");
            }
        } else {
            // Stop the timer to avoid burning CPU on invisible redraws.
            let mut timer_ref = entry.timer_source.borrow_mut();
            if let Some(source_id) = timer_ref.take() {
                source_id.remove();
                info!("Seasonal effect timer stopped");
            }
        }
    }
}

/// `true` if at least one seasonal effect is active right now (calendar check).
pub fn has_active_effect() -> bool {
    let effects: Vec<Box<dyn SeasonalEffect>> =
        vec![Box::new(SnowEffect), Box::new(HalloweenEffect)];
    effects.iter().any(|e| e.is_active())
}

/// Register an effect's drawing area and timer so `set_effects_enabled` can
/// manage them.  Called by each effect's `apply` implementation.
pub fn register_effect(
    drawing_area: Rc<DrawingArea>,
    timer_source: Rc<RefCell<Option<glib::SourceId>>>,
) {
    let registry = get_effect_registry();
    registry
        .borrow_mut()
        .push(EffectEntry { drawing_area, timer_source });
}

// ---------------------------------------------------------------------------
// SeasonalEffect trait
// ---------------------------------------------------------------------------

/// A seasonal effect that can be applied to the application window.
pub trait SeasonalEffect {
    /// `true` when this effect should be active (based on the current date).
    fn is_active(&self) -> bool;

    /// Human-readable name used for logging.
    fn name(&self) -> &'static str;

    /// Overlay the effect on `window`.
    ///
    /// Implementations are expected to call [`register_effect`] with their
    /// drawing area and timer source so the global toggle can manage them.
    ///
    /// Returns the drawing area on success, `None` on failure.
    fn apply(
        &self,
        window: &ApplicationWindow,
        mouse_context: Option<&MouseContext>,
    ) -> Option<Rc<DrawingArea>>;
}

// ---------------------------------------------------------------------------
// Activation
// ---------------------------------------------------------------------------

/// Check for active seasonal effects and apply any that are relevant.
pub fn apply_seasonal_effects(window: &ApplicationWindow) {
    if !are_effects_enabled() {
        info!("Seasonal effects are disabled — skipping");
        return;
    }

    info!("Checking for active seasonal effects...");

    let mouse_context = common::setup_mouse_tracking(window);

    let effects: Vec<Box<dyn SeasonalEffect>> =
        vec![Box::new(SnowEffect), Box::new(HalloweenEffect)];

    for effect in effects {
        if effect.is_active() {
            info!("Active seasonal effect detected: {}", effect.name());
            if effect.apply(window, Some(&mouse_context)).is_some() {
                // Effect registers itself via register_effect().
                info!("Successfully applied {} effect", effect.name());
            } else {
                info!("Failed to apply {} effect", effect.name());
            }
        }
    }
}
