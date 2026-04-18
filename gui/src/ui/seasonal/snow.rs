//! Christmas snow overlay.
//!
//! Soft, parallax-lit snowflakes drift down with a slow lateral sway. A
//! low-level wind value shifts over time to give the field a breathing
//! motion, and a dim glow along the bottom edge hints at a snowbank.

use std::f64::consts::TAU;
use std::rc::Rc;

use gtk4::cairo;
use gtk4::glib;
use gtk4::{ApplicationWindow, DrawingArea};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::config::seasonal_debug;
use crate::ui::seasonal::common::{mount_effect, rescale_point, MouseContext, ParticleField};
use crate::ui::seasonal::SeasonalEffect;

/// Number of snowflakes drawn per frame.
const FLAKE_COUNT: usize = 80;
/// Maximum magnitude of the drifting wind signal.
const WIND_STRENGTH: f64 = 0.5;
/// Odds (per frame) of retargeting the wind — lower = longer-held gusts.
const WIND_RETARGET_CHANCE: f64 = 0.02;
/// Bottom-edge glow band height.
const GLOW_HEIGHT: f64 = 100.0;

/// December snowfall.
pub struct SnowEffect;

impl SeasonalEffect for SnowEffect {
    fn is_active(&self) -> bool {
        if let Some(enabled) = seasonal_debug::check_effect_env(seasonal_debug::ENABLE_SNOW) {
            return enabled;
        }
        glib::DateTime::now_utc().map(|dt| dt.month() == 12).unwrap_or(false)
    }

    fn name(&self) -> &'static str {
        "Snow (Christmas)"
    }

    fn apply(
        &self,
        window: &ApplicationWindow,
        mouse_context: Option<&MouseContext>,
    ) -> Option<Rc<DrawingArea>> {
        mount_effect(window, mouse_context, SnowField::new)
    }
}

// ---------------------------------------------------------------------------
// SnowField — ParticleField for the drifting snow
// ---------------------------------------------------------------------------

struct SnowField {
    flakes: Vec<Snowflake>,
    rng: StdRng,
    wind: f64,
    wind_target: f64,
    width: f64,
    height: f64,
}

impl SnowField {
    fn new(width: f64, height: f64) -> Self {
        let seed = glib::DateTime::now_utc()
            .map(|dt| dt.to_unix())
            .unwrap_or(0) as u64;
        let mut rng = StdRng::seed_from_u64(seed);
        let flakes = (0..FLAKE_COUNT)
            .map(|_| Snowflake::spawn(width, height, &mut rng))
            .collect();
        Self {
            flakes,
            rng,
            wind: 0.0,
            wind_target: 0.0,
            width,
            height,
        }
    }
}

impl ParticleField for SnowField {
    fn tick(&mut self, width: f64, height: f64, dt: f64, _mouse: (f64, f64)) {
        self.width = width;
        self.height = height;

        // Occasionally pick a new target wind, then ease the current wind
        // toward it so gusts ramp up smoothly rather than snapping.
        if self.rng.random::<f64>() < WIND_RETARGET_CHANCE {
            self.wind_target = (self.rng.random::<f64>() - 0.5) * WIND_STRENGTH;
        }
        self.wind += (self.wind_target - self.wind) * dt;

        for flake in &mut self.flakes {
            flake.tick(width, height, dt, self.wind, &mut self.rng);
        }
    }

    fn paint(&self, cr: &cairo::Context, width: f64, height: f64) {
        // Paint back-to-front by parallax depth so large near flakes sit in front.
        let mut ordered: Vec<&Snowflake> = self.flakes.iter().collect();
        ordered.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(std::cmp::Ordering::Equal));
        for flake in ordered {
            flake.paint(cr);
        }
        paint_snowbank(cr, width, height);
    }

    fn handle_resize(&mut self, new_width: f64, new_height: f64) {
        let prev = (self.width, self.height);
        for flake in &mut self.flakes {
            rescale_point((&mut flake.x, &mut flake.y), prev, (new_width, new_height));
        }
        self.width = new_width;
        self.height = new_height;
    }
}

fn paint_snowbank(cr: &cairo::Context, width: f64, height: f64) {
    let _ = cr.save();
    let glow = cairo::LinearGradient::new(0.0, height - GLOW_HEIGHT, 0.0, height);
    glow.add_color_stop_rgba(0.0, 1.0, 1.0, 1.0, 0.00);
    glow.add_color_stop_rgba(1.0, 1.0, 1.0, 1.0, 0.15);
    let _ = cr.set_source(&glow);
    cr.rectangle(0.0, height - GLOW_HEIGHT, width, GLOW_HEIGHT);
    let _ = cr.fill();
    let _ = cr.restore();
}

// ---------------------------------------------------------------------------
// Individual flake
// ---------------------------------------------------------------------------

struct Snowflake {
    x: f64,
    y: f64,
    /// Parallax depth in [0.5, 1.5] — used to scale size, speed, and opacity.
    z: f64,
    fall_speed: f64,
    sway_phase: f64,
    sway_speed: f64,
    size: f64,
}

impl Snowflake {
    fn spawn(width: f64, height: f64, rng: &mut StdRng) -> Self {
        let z = rng.random_range(0.5..1.5);
        Self {
            x: rng.random_range(0.0..width),
            y: rng.random_range(0.0..height),
            z,
            fall_speed: rng.random_range(30.0..70.0) * z,
            sway_phase: rng.random_range(0.0..TAU),
            sway_speed: rng.random_range(0.5..2.0),
            size: rng.random_range(2.0..5.0) * z,
        }
    }

    fn tick(&mut self, width: f64, height: f64, dt: f64, wind: f64, rng: &mut StdRng) {
        self.y += self.fall_speed * dt;
        self.sway_phase += self.sway_speed * dt;
        let sway = self.sway_phase.sin() * 20.0 * self.z;
        self.x += (sway + wind * 50.0) * dt;

        // Respawn when the flake drifts past any edge.
        if self.y > height + 10.0 {
            self.y = rng.random_range(-10.0..0.0);
            self.x = rng.random_range(0.0..width);
        }
        if self.x < -20.0 {
            self.x = rng.random_range(width..width + 20.0);
            self.y = rng.random_range(0.0..height);
        } else if self.x > width + 20.0 {
            self.x = rng.random_range(-20.0..0.0);
            self.y = rng.random_range(0.0..height);
        }
    }

    fn paint(&self, cr: &cairo::Context) {
        let _ = cr.save();
        let radial = cairo::RadialGradient::new(self.x, self.y, 0.0, self.x, self.y, self.size);
        // Nearer flakes (higher z) are slightly more opaque.
        let opacity = (0.3 + (self.z - 0.5) * 0.5).min(0.8);
        radial.add_color_stop_rgba(0.0, 1.0, 1.0, 1.0, opacity);
        radial.add_color_stop_rgba(1.0, 1.0, 1.0, 1.0, 0.0);
        let _ = cr.set_source(&radial);
        cr.arc(self.x, self.y, self.size, 0.0, TAU);
        let _ = cr.fill();
        let _ = cr.restore();
    }
}
