//! Halloween bat overlay.
//!
//! A flock of bats swoops across the window with Bézier-curve wings,
//! subtle banking turns, and panic behaviour when the pointer gets too
//! close. A dim fog gradient hugs the bottom edge of the screen to sell
//! the mood.

use std::f64::consts::{PI, TAU};
use std::rc::Rc;

use gtk4::cairo;
use gtk4::glib;
use gtk4::{ApplicationWindow, DrawingArea};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::config::seasonal_debug;
use crate::ui::seasonal::common::{mount_effect, rescale_point, MouseContext, ParticleField};
use crate::ui::seasonal::SeasonalEffect;

/// Number of bats in the flock.
const BAT_COUNT: usize = 15;
/// Nominal cruising speed in pixels / second.
const BASE_SPEED: f64 = 100.0;
/// Pointer panic radius — bats inside this distance are pushed away.
const AVOID_RADIUS: f64 = 250.0;
/// Strength of the repulsion impulse applied each second.
const AVOID_FORCE: f64 = 800.0;
/// Height of the fog band along the bottom of the overlay.
const FOG_HEIGHT: f64 = 250.0;

/// October bat flock.
pub struct HalloweenEffect;

impl SeasonalEffect for HalloweenEffect {
    fn is_active(&self) -> bool {
        if let Some(enabled) = seasonal_debug::check_effect_env(seasonal_debug::ENABLE_HALLOWEEN) {
            return enabled;
        }
        glib::DateTime::now_utc().map(|dt| dt.month() == 10).unwrap_or(false)
    }

    fn name(&self) -> &'static str {
        "Bats (Halloween)"
    }

    fn apply(
        &self,
        window: &ApplicationWindow,
        mouse_context: Option<&MouseContext>,
    ) -> Option<Rc<DrawingArea>> {
        mount_effect(window, mouse_context, BatField::new)
    }
}

// ---------------------------------------------------------------------------
// BatField — ParticleField driving the flock
// ---------------------------------------------------------------------------

struct BatField {
    bats: Vec<Bat>,
    rng: StdRng,
    width: f64,
    height: f64,
}

impl BatField {
    fn new(width: f64, height: f64) -> Self {
        let seed = glib::DateTime::now_utc()
            .map(|dt| dt.to_unix())
            .unwrap_or(0) as u64;
        let bats = (0..BAT_COUNT)
            .map(|i| Bat::new(width, height, seed.wrapping_add((i as u64) * 100)))
            .collect();
        Self {
            bats,
            rng: StdRng::seed_from_u64(seed),
            width,
            height,
        }
    }
}

impl ParticleField for BatField {
    fn tick(&mut self, width: f64, height: f64, dt: f64, mouse: (f64, f64)) {
        self.width = width;
        self.height = height;
        for bat in &mut self.bats {
            bat.tick(width, height, dt, mouse, &mut self.rng);
        }
    }

    fn paint(&self, cr: &cairo::Context, width: f64, height: f64) {
        // Back-to-front by scale so smaller bats render behind larger ones.
        let mut ordered: Vec<&Bat> = self.bats.iter().collect();
        ordered.sort_by(|a, b| {
            a.scale.partial_cmp(&b.scale).unwrap_or(std::cmp::Ordering::Equal)
        });
        for bat in ordered {
            bat.paint(cr);
        }
        paint_fog(cr, width, height);
    }

    fn handle_resize(&mut self, new_width: f64, new_height: f64) {
        let prev = (self.width, self.height);
        for bat in &mut self.bats {
            rescale_point((&mut bat.x, &mut bat.y), prev, (new_width, new_height));
        }
        self.width = new_width;
        self.height = new_height;
    }
}

fn paint_fog(cr: &cairo::Context, width: f64, height: f64) {
    let _ = cr.save();
    let fog = cairo::LinearGradient::new(0.0, height - FOG_HEIGHT, 0.0, height);
    fog.add_color_stop_rgba(0.0, 0.10, 0.05, 0.10, 0.00);
    fog.add_color_stop_rgba(1.0, 0.20, 0.15, 0.25, 0.30);
    let _ = cr.set_source(&fog);
    cr.rectangle(0.0, height - FOG_HEIGHT, width, FOG_HEIGHT);
    let _ = cr.fill();
    let _ = cr.restore();
}

// ---------------------------------------------------------------------------
// A single bat
// ---------------------------------------------------------------------------

struct Bat {
    x: f64,
    y: f64,
    scale: f64,
    vx: f64,
    vy: f64,
    flap_phase: f64,
    flap_speed: f64,
    tint_shift: f64,
}

impl Bat {
    fn new(width: f64, height: f64, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let scale = rng.random_range(0.5..1.5);
        let heading = rng.random_range(0.0..TAU);
        let speed = rng.random_range(BASE_SPEED..BASE_SPEED + 50.0) * scale;
        Self {
            x: rng.random_range(0.0..width),
            y: rng.random_range(0.0..height),
            scale,
            vx: heading.cos() * speed,
            vy: heading.sin() * speed,
            flap_phase: rng.random_range(0.0..TAU),
            flap_speed: rng.random_range(10.0..15.0),
            tint_shift: rng.random_range(0.0..0.1),
        }
    }

    fn tick(&mut self, width: f64, height: f64, dt: f64, mouse: (f64, f64), rng: &mut StdRng) {
        self.flap_phase += self.flap_speed * dt;

        // Occasional heading wobble.
        if rng.random::<f64>() > 0.92 {
            let wobble = (rng.random::<f64>() - 0.5) * 3.0;
            let heading = self.vy.atan2(self.vx) + wobble * dt * 2.0;
            let speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
            self.vx = heading.cos() * speed;
            self.vy = heading.sin() * speed;
        }

        // Pointer repulsion.
        let dx = self.x - mouse.0;
        let dy = self.y - mouse.1;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < AVOID_RADIUS * AVOID_RADIUS && dist_sq > 0.0 {
            let dist = dist_sq.sqrt();
            let strength = (AVOID_RADIUS - dist) / AVOID_RADIUS;
            self.vx += (dx / dist) * strength * AVOID_FORCE * dt;
            self.vy += (dy / dist) * strength * AVOID_FORCE * dt;
        }

        // Clamp speed into [BASE/2, BASE*3].
        let speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
        let max_speed = BASE_SPEED * 3.0;
        let min_speed = BASE_SPEED * 0.5;
        if speed > max_speed {
            let k = max_speed / speed;
            self.vx *= k;
            self.vy *= k;
        } else if speed > 0.0 && speed < min_speed {
            let k = min_speed / speed;
            self.vx *= k;
            self.vy *= k;
        }

        self.x += self.vx * dt;
        self.y += self.vy * dt;

        // Wrap around with a margin so bats slide off-screen before reappearing.
        let margin = 60.0 * self.scale;
        if self.x < -margin {
            self.x = width + margin;
        } else if self.x > width + margin {
            self.x = -margin;
        }
        if self.y < -margin {
            self.y = height + margin;
        } else if self.y > height + margin {
            self.y = -margin;
        }
    }

    fn paint(&self, cr: &cairo::Context) {
        let _ = cr.save();
        cr.translate(self.x, self.y);
        cr.scale(self.scale, self.scale);

        let heading = self.vy.atan2(self.vx);
        // Gentle bank — flip orientation when flying leftward so the ears stay up.
        let rotation = if self.vx < 0.0 { (heading - PI) * 0.5 } else { heading * 0.5 };
        cr.rotate(rotation);

        let flap = self.flap_phase.sin();
        cr.set_source_rgba(0.05 + self.tint_shift, 0.05, 0.05 + self.tint_shift, 0.85);

        self.paint_body(cr);
        self.paint_ears(cr);
        self.paint_wings(cr, flap);
        self.paint_eyes(cr);

        let _ = cr.restore();
    }

    fn paint_body(&self, cr: &cairo::Context) {
        let _ = cr.save();
        cr.scale(1.0, 1.5);
        cr.arc(0.0, 0.0, 3.0, 0.0, TAU);
        let _ = cr.fill();
        let _ = cr.restore();
    }

    fn paint_ears(&self, cr: &cairo::Context) {
        cr.move_to(-2.0, -3.0);
        cr.line_to(-3.0, -8.0);
        cr.line_to(0.0, -4.0);
        cr.line_to(3.0, -8.0);
        cr.line_to(2.0, -3.0);
        let _ = cr.fill();
    }

    fn paint_wings(&self, cr: &cairo::Context, flap: f64) {
        let span = 25.0;
        let tip_y = flap * 10.0 - 5.0;
        for mirror in [-1.0, 1.0] {
            let _ = cr.save();
            cr.scale(mirror, 1.0);
            cr.move_to(1.0, 0.0);
            cr.curve_to(10.0, -5.0 + flap * 5.0, 20.0, -10.0 + flap * 8.0, span, tip_y);
            cr.curve_to(
                span - 5.0,
                tip_y + 5.0,
                span - 8.0,
                tip_y + 10.0,
                15.0,
                5.0 + flap * 2.0,
            );
            cr.curve_to(10.0, 10.0 + flap * 2.0, 5.0, 5.0, 1.0, 2.0);
            cr.close_path();
            let _ = cr.fill();
            let _ = cr.restore();
        }
    }

    fn paint_eyes(&self, cr: &cairo::Context) {
        // Tiny lateral shift so eyes lean into the direction of travel.
        let lean = if self.vx > 0.0 { 0.5 } else { -0.5 };
        cr.set_source_rgba(1.0, 0.8, 0.0, 0.8);
        for side in [-1.5, 1.5] {
            let _ = cr.save();
            cr.translate(side + lean, -2.0);
            cr.arc(0.0, 0.0, 0.8, 0.0, TAU);
            let _ = cr.fill();
            let _ = cr.restore();
        }
    }
}
