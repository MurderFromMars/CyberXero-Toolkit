//! Christmas snow effect overlay.
//!
//! Adds a high-quality animated snow effect with parallax and soft-glow flakes.

use crate::config::seasonal_debug;
use crate::ui::seasonal::common::{
    add_overlay_to_window, setup_resize_handler, ResizableEffectState,
};
use crate::ui::seasonal::SeasonalEffect;
use gtk4::cairo;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, DrawingArea};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

const SNOW_COUNT: usize = 80;
const WIND_STRENGTH: f64 = 0.5;

/// Christmas snow effect.
pub struct SnowEffect;

impl SeasonalEffect for SnowEffect {
    fn is_active(&self) -> bool {
        // Check environment variable for debugging (overrides date check)
        if let Some(enabled) = seasonal_debug::check_effect_env(seasonal_debug::ENABLE_SNOW) {
            return enabled;
        }

        // Default: check if it's December
        if let Ok(dt) = glib::DateTime::now_utc() {
            dt.month() == 12
        } else {
            false
        }
    }

    fn name(&self) -> &'static str {
        "Snow (Christmas)"
    }

    fn apply(
        &self,
        window: &ApplicationWindow,
        _mouse_context: Option<&crate::ui::seasonal::common::MouseContext>,
    ) -> Option<Rc<DrawingArea>> {
        let drawing_area = Rc::new(DrawingArea::new());
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);
        drawing_area.set_can_focus(false);
        drawing_area.set_sensitive(false);
        drawing_area.set_halign(gtk4::Align::Fill);
        drawing_area.set_valign(gtk4::Align::Fill);
        drawing_area.set_visible(crate::ui::seasonal::are_effects_enabled());

        let state = Rc::new(RefCell::new(None::<SnowState>));
        let setup_state = Rc::clone(&state);

        // Capture the SourceId so the global toggle can stop/restart this timer.
        let timer_source: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
        let timer_source_fill = Rc::clone(&timer_source);
        let drawing_area_clone = drawing_area.clone();
        let source_id = glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
            drawing_area_clone.queue_draw();
            glib::ControlFlow::Continue
        });
        *timer_source_fill.borrow_mut() = Some(source_id);

        drawing_area.set_draw_func(move |_da, cr, width, height| {
            let mut state_ref = setup_state.borrow_mut();

            if state_ref.is_none() {
                *state_ref = Some(SnowState::new(width as f64, height as f64));
            }

            if let Some(snow_state) = state_ref.as_mut() {
                let now = std::time::Instant::now();
                snow_state.update(width as f64, height as f64, now);

                let _ = cr.save();
                cr.set_operator(cairo::Operator::Clear);
                let _ = cr.paint();
                cr.set_operator(cairo::Operator::Over);
                let _ = cr.restore();

                snow_state.draw(cr, width as f64, height as f64);
            }
        });

        // Set up resize handler
        setup_resize_handler(&drawing_area, state);

        if add_overlay_to_window(window, &drawing_area) {
            crate::ui::seasonal::register_effect(drawing_area.clone(), timer_source);
            Some(drawing_area)
        } else {
            None
        }
    }
}

#[derive(Clone)]
struct Snowflake {
    x: f64,
    y: f64,
    z: f64,
    speed_y: f64,
    sway_offset: f64,
    sway_speed: f64,
    size: f64,
}

impl Snowflake {
    fn new(width: f64, height: f64, rng: &mut StdRng) -> Self {
        let z = rng.random_range(0.5..1.5);
        Self {
            x: rng.random_range(0.0..width),
            y: rng.random_range(0.0..height),
            z,
            speed_y: rng.random_range(30.0..70.0) * z,
            sway_offset: rng.random_range(0.0..2.0 * PI),
            sway_speed: rng.random_range(0.5..2.0),
            size: rng.random_range(2.0..5.0) * z,
        }
    }

    fn update(&mut self, width: f64, height: f64, dt: f64, wind: f64, rng: &mut StdRng) {
        self.y += self.speed_y * dt;
        self.sway_offset += self.sway_speed * dt;
        let horizontal_move = (self.sway_offset.sin() * 20.0 * self.z) + (wind * 50.0);
        self.x += horizontal_move * dt;

        // When wrapping around, respawn at random position in current window dimensions
        if self.y > height + 10.0 {
            self.y = rng.random_range(-10.0..0.0);
            self.x = rng.random_range(0.0..width);
        }
        if self.x < -20.0 {
            self.x = rng.random_range(width..width + 20.0);
            self.y = rng.random_range(0.0..height);
        }
        if self.x > width + 20.0 {
            self.x = rng.random_range(-20.0..0.0);
            self.y = rng.random_range(0.0..height);
        }
    }

    fn draw(&self, cr: &cairo::Context) {
        let _ = cr.save();

        let radial = cairo::RadialGradient::new(self.x, self.y, 0.0, self.x, self.y, self.size);
        let opacity = (0.3 + (self.z - 0.5) * 0.5).min(0.8);

        radial.add_color_stop_rgba(0.0, 1.0, 1.0, 1.0, opacity);
        radial.add_color_stop_rgba(1.0, 1.0, 1.0, 1.0, 0.0);

        let _ = cr.set_source(&radial);
        cr.arc(self.x, self.y, self.size, 0.0, 2.0 * PI);
        let _ = cr.fill();

        let _ = cr.restore();
    }
}

struct SnowState {
    snowflakes: Vec<Snowflake>,
    rng: StdRng,
    last_time: std::time::Instant,
    wind: f64,
    wind_target: f64,
    current_width: f64,
    current_height: f64,
}

impl SnowState {
    fn new(width: f64, height: f64) -> Self {
        let seed = glib::DateTime::now_utc()
            .map(|dt| dt.to_unix())
            .unwrap_or(0) as u64;
        let mut rng = StdRng::seed_from_u64(seed);
        let snowflakes = (0..SNOW_COUNT)
            .map(|_| Snowflake::new(width, height, &mut rng))
            .collect();

        Self {
            snowflakes,
            rng,
            last_time: std::time::Instant::now(),
            wind: 0.0,
            wind_target: 0.0,
            current_width: width,
            current_height: height,
        }
    }

    fn update(&mut self, width: f64, height: f64, now: std::time::Instant) {
        // Update stored dimensions during normal loop just in case,
        // though handle_resize does the heavy lifting.
        self.current_width = width;
        self.current_height = height;

        let dt = now.duration_since(self.last_time).as_secs_f64().min(0.1);
        self.last_time = now;

        if self.rng.random::<f64>() > 0.98 {
            self.wind_target = (self.rng.random::<f64>() - 0.5) * WIND_STRENGTH;
        }
        self.wind += (self.wind_target - self.wind) * dt;

        for flake in &mut self.snowflakes {
            flake.update(width, height, dt, self.wind, &mut self.rng);
        }
    }

    fn draw(&self, cr: &cairo::Context, width: f64, height: f64) {
        let mut sorted: Vec<&Snowflake> = self.snowflakes.iter().collect();
        sorted.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());

        for flake in sorted {
            flake.draw(cr);
        }

        let _ = cr.save();
        let glow = cairo::LinearGradient::new(0.0, height - 100.0, 0.0, height);
        glow.add_color_stop_rgba(0.0, 1.0, 1.0, 1.0, 0.0);
        glow.add_color_stop_rgba(1.0, 1.0, 1.0, 1.0, 0.15);
        let _ = cr.set_source(&glow);
        cr.rectangle(0.0, height - 100.0, width, 100.0);
        let _ = cr.fill();
        let _ = cr.restore();
    }
}

impl ResizableEffectState for SnowState {
    fn handle_resize(&mut self, new_width: f64, new_height: f64) {
        // Avoid division by zero
        if self.current_width <= 0.0 || self.current_height <= 0.0 {
            self.current_width = new_width;
            self.current_height = new_height;
            return;
        }

        // Calculate scale ratios
        let scale_x = new_width / self.current_width;
        let scale_y = new_height / self.current_height;

        // Apply proportional scaling to all flakes
        for flake in &mut self.snowflakes {
            flake.x *= scale_x;
            flake.y *= scale_y;
        }

        // Update stored dimensions
        self.current_width = new_width;
        self.current_height = new_height;
    }
}
