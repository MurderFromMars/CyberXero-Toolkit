//! Halloween effect overlay.
//!
//! Adds an animated Halloween bat effect to the window during October.
//! Features:
//! - Swooping bat physics (Bézier wings, banking turns).
//! - Atmospheric fog at the bottom (Subtle).
//! - Mouse avoidance (bats scatter when the cursor approaches).

use crate::config::seasonal_debug;
use crate::ui::seasonal::common::{
    add_overlay_to_window, setup_resize_handler, MouseContext, ResizableEffectState,
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

const BAT_COUNT: usize = 15;
const BASE_SPEED: f64 = 100.0;
const MOUSE_AVOID_RADIUS: f64 = 250.0;
const MOUSE_AVOID_FORCE: f64 = 800.0;

/// Halloween bat effect.
pub struct HalloweenEffect;

impl SeasonalEffect for HalloweenEffect {
    fn is_active(&self) -> bool {
        // Check environment variable for debugging (overrides date check)
        if let Some(enabled) = seasonal_debug::check_effect_env(seasonal_debug::ENABLE_HALLOWEEN) {
            return enabled;
        }

        // Default: check if it's October
        if let Ok(dt) = glib::DateTime::now_utc() {
            dt.month() == 10 // October
        } else {
            false
        }
    }

    fn name(&self) -> &'static str {
        "Bats (Halloween)"
    }

    fn apply(
        &self,
        window: &ApplicationWindow,
        mouse_context: Option<&MouseContext>,
    ) -> Option<Rc<DrawingArea>> {
        use log::info;

        let drawing_area = Rc::new(DrawingArea::new());
        drawing_area.set_hexpand(true);
        drawing_area.set_vexpand(true);
        drawing_area.set_can_focus(false);
        drawing_area.set_sensitive(false);
        drawing_area.set_halign(gtk4::Align::Fill);
        drawing_area.set_valign(gtk4::Align::Fill);
        drawing_area.set_visible(crate::ui::seasonal::are_effects_enabled());

        let mouse_pos = if let Some(ctx) = mouse_context {
            ctx.position_internal()
        } else {
            Rc::new(RefCell::new((0.0f64, 0.0f64)))
        };

        let state = Rc::new(RefCell::new(None::<BatState>));
        let setup_state = Rc::clone(&state);
        let draw_mouse_pos = mouse_pos.clone();

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
                *state_ref = Some(BatState::new(width as f64, height as f64));
            }

            if let Some(bat_state) = state_ref.as_mut() {
                let now = std::time::Instant::now();
                let (mx, my) = *draw_mouse_pos.borrow();

                bat_state.update(width as f64, height as f64, now, mx, my);

                let _ = cr.save();
                cr.set_operator(cairo::Operator::Clear);
                let _ = cr.paint();
                cr.set_operator(cairo::Operator::Over);
                let _ = cr.restore();

                bat_state.draw_bats(cr);
                bat_state.draw_fog(cr, width as f64, height as f64);
            }
        });

        // Set up resize handler
        setup_resize_handler(&drawing_area, state);

        if add_overlay_to_window(window, &drawing_area) {
            info!("Halloween effect overlay added successfully");
            crate::ui::seasonal::register_effect(drawing_area.clone(), timer_source);
            Some(drawing_area)
        } else {
            info!("Failed to add Halloween effect overlay");
            None
        }
    }
}

#[derive(Clone)]
struct Bat {
    x: f64,
    y: f64,
    scale: f64,
    velocity_x: f64,
    velocity_y: f64,
    flap_phase: f64,
    flap_speed: f64,
    color_offset: f64,
}

impl Bat {
    fn new(width: f64, height: f64, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let scale = rng.random_range(0.5..1.5);
        let direction = rng.random_range(0.0..2.0 * PI);
        let speed = rng.random_range(BASE_SPEED..BASE_SPEED + 50.0) * scale;

        Self {
            x: rng.random_range(0.0..width),
            y: rng.random_range(0.0..height),
            scale,
            velocity_x: direction.cos() * speed,
            velocity_y: direction.sin() * speed,
            flap_phase: rng.random_range(0.0..2.0 * PI),
            flap_speed: rng.random_range(10.0..15.0),
            color_offset: rng.random_range(0.0..0.1),
        }
    }

    fn update(&mut self, width: f64, height: f64, dt: f64, rng: &mut StdRng, mx: f64, my: f64) {
        self.flap_phase += self.flap_speed * dt;

        if rng.random::<f64>() > 0.92 {
            let random_angle = (rng.random::<f64>() - 0.5) * 3.0;
            let angle = self.velocity_y.atan2(self.velocity_x) + (random_angle * dt * 2.0);
            let current_speed = (self.velocity_x.powi(2) + self.velocity_y.powi(2)).sqrt();
            self.velocity_x = angle.cos() * current_speed;
            self.velocity_y = angle.sin() * current_speed;
        }

        let dx = self.x - mx;
        let dy = self.y - my;
        let dist_sq = dx * dx + dy * dy;

        if dist_sq < (MOUSE_AVOID_RADIUS * MOUSE_AVOID_RADIUS) {
            let dist = dist_sq.sqrt();
            let repulsion_strength = (MOUSE_AVOID_RADIUS - dist) / MOUSE_AVOID_RADIUS;
            let norm_x = dx / dist;
            let norm_y = dy / dist;

            self.velocity_x += norm_x * repulsion_strength * MOUSE_AVOID_FORCE * dt;
            self.velocity_y += norm_y * repulsion_strength * MOUSE_AVOID_FORCE * dt;
        }

        let max_speed = BASE_SPEED * 3.0;
        let current_speed = (self.velocity_x.powi(2) + self.velocity_y.powi(2)).sqrt();
        if current_speed > max_speed {
            let scale = max_speed / current_speed;
            self.velocity_x *= scale;
            self.velocity_y *= scale;
        } else if current_speed < BASE_SPEED * 0.5 {
            let scale = (BASE_SPEED * 0.5) / current_speed;
            self.velocity_x *= scale;
            self.velocity_y *= scale;
        }

        self.x += self.velocity_x * dt;
        self.y += self.velocity_y * dt;

        let buffer = 60.0 * self.scale;
        if self.x < -buffer {
            self.x = width + buffer;
        }
        if self.x > width + buffer {
            self.x = -buffer;
        }
        if self.y < -buffer {
            self.y = height + buffer;
        }
        if self.y > height + buffer {
            self.y = -buffer;
        }
    }

    fn draw(&self, cr: &cairo::Context) {
        let _ = cr.save();

        cr.translate(self.x, self.y);
        cr.scale(self.scale, self.scale);

        let flight_angle = self.velocity_y.atan2(self.velocity_x);
        let visual_rotation = if self.velocity_x < 0.0 {
            (flight_angle - PI) * 0.5
        } else {
            flight_angle * 0.5
        };
        cr.rotate(visual_rotation);

        let flap = self.flap_phase.sin();
        cr.set_source_rgba(
            0.05 + self.color_offset,
            0.05,
            0.05 + self.color_offset,
            0.85,
        );

        let _ = cr.save();
        cr.scale(1.0, 1.5);
        cr.arc(0.0, 0.0, 3.0, 0.0, 2.0 * PI);
        let _ = cr.fill();
        let _ = cr.restore();

        cr.move_to(-2.0, -3.0);
        cr.line_to(-3.0, -8.0);
        cr.line_to(0.0, -4.0);
        cr.line_to(3.0, -8.0);
        cr.line_to(2.0, -3.0);
        let _ = cr.fill();

        for dir in [-1.0, 1.0] {
            let _ = cr.save();
            cr.scale(dir, 1.0);
            cr.move_to(1.0, 0.0);
            let wing_span = 25.0;
            let tip_y = flap * 10.0 - 5.0;

            cr.curve_to(
                10.0,
                -5.0 + (flap * 5.0),
                20.0,
                -10.0 + (flap * 8.0),
                wing_span,
                tip_y,
            );
            cr.curve_to(
                wing_span - 5.0,
                tip_y + 5.0,
                wing_span - 8.0,
                tip_y + 10.0,
                15.0,
                5.0 + (flap * 2.0),
            );
            cr.curve_to(10.0, 10.0 + (flap * 2.0), 5.0, 5.0, 1.0, 2.0);

            cr.close_path();
            let _ = cr.fill();
            let _ = cr.restore();
        }

        let eye_offset = if self.velocity_x > 0.0 { 1.0 } else { -1.0 };
        cr.set_source_rgba(1.0, 0.8, 0.0, 0.8);

        for side in [-1.0, 1.0] {
            let _ = cr.save();
            cr.translate(side * 1.5 + (eye_offset * 0.5), -2.0);
            cr.arc(0.0, 0.0, 0.8, 0.0, 2.0 * PI);
            let _ = cr.fill();
            let _ = cr.restore();
        }

        let _ = cr.restore();
    }
}

struct BatState {
    bats: Vec<Bat>,
    rng: StdRng,
    last_frame_time: std::time::Instant,
    current_width: f64,
    current_height: f64,
}

impl BatState {
    fn new(width: f64, height: f64) -> Self {
        let seed = glib::DateTime::now_utc()
            .map(|dt| dt.to_unix())
            .unwrap_or(0) as u64;

        let bats = (0..BAT_COUNT)
            .map(|i| Bat::new(width, height, seed.wrapping_add(i as u64 * 100)))
            .collect();

        Self {
            bats,
            rng: StdRng::seed_from_u64(seed),
            last_frame_time: std::time::Instant::now(),
            current_width: width,
            current_height: height,
        }
    }

    fn update(&mut self, width: f64, height: f64, now: std::time::Instant, mx: f64, my: f64) {
        // Sync dimensions
        self.current_width = width;
        self.current_height = height;

        let dt_duration = now.duration_since(self.last_frame_time);
        let mut dt = dt_duration.as_secs_f64();
        if dt > 0.1 {
            dt = 0.1;
        }
        self.last_frame_time = now;

        for bat in &mut self.bats {
            bat.update(width, height, dt, &mut self.rng, mx, my);
        }
    }

    fn draw_bats(&self, cr: &cairo::Context) {
        let mut sorted_bats: Vec<&Bat> = self.bats.iter().collect();
        sorted_bats.sort_by(|a, b| {
            a.scale
                .partial_cmp(&b.scale)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for bat in sorted_bats {
            bat.draw(cr);
        }
    }

    fn draw_fog(&self, cr: &cairo::Context, width: f64, height: f64) {
        let _ = cr.save();

        let pattern = cairo::LinearGradient::new(0.0, height - 250.0, 0.0, height);
        pattern.add_color_stop_rgba(0.0, 0.1, 0.05, 0.1, 0.0);
        pattern.add_color_stop_rgba(1.0, 0.2, 0.15, 0.25, 0.3);

        let _ = cr.set_source(&pattern);
        cr.rectangle(0.0, height - 250.0, width, 250.0);
        let _ = cr.fill();

        let _ = cr.restore();
    }
}

impl ResizableEffectState for BatState {
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

        // Apply proportional scaling to all bats
        for bat in &mut self.bats {
            bat.x *= scale_x;
            bat.y *= scale_y;
        }

        // Update stored dimensions
        self.current_width = new_width;
        self.current_height = new_height;
    }
}
