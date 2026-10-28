//! Bounded, unidirectional speed regulator for a platter.

use std::collections::VecDeque;

use anyhow::{Context as _, ensure};
use num_traits::ToPrimitive as _;
use serde::Serialize;

/// Duration of the sensor-count measurement window in microseconds.
pub const WINDOW_US: u64 = 20_000;
/// Calibrated sensor timer ticks per microsecond.
pub const SENSOR_TICKS_PER_US: u64 = 100;

/// Tunable parameters for the platter speed regulator.
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    /// Positive target speed in revolutions per minute.
    pub target_rpm: u16,
    /// Optional maximum permitted measured speed.
    pub overspeed_limit_rpm: Option<u16>,
    /// Maximum output force.
    pub max_force: u16,
    /// Feed-forward force at zero RPM.
    pub base_force: u16,
    /// Feed-forward force added per target RPM.
    pub force_per_rpm: u16,
    /// Proportional correction in force units per RPM of speed error.
    pub kp: u16,
    /// Integral correction in force units per RPM-second of speed error.
    pub ki: u16,
    /// Duration of startup behavior in milliseconds.
    pub startup_ms: u16,
    /// Maximum force increase per second after startup.
    pub force_rise_per_second: u32,
    /// Maximum startup force increase per second; zero selects a step.
    pub startup_force_rise_per_second: u32,
    /// Sign applied to raw counter movement for positive speed.
    pub count_direction: i32,
}

impl Settings {
    /// Return the configured overspeed limit, applying the diagnostic default.
    #[must_use]
    pub const fn effective_overspeed_limit_rpm(self) -> Option<u16> {
        match self.overspeed_limit_rpm {
            None => Some(self.target_rpm.saturating_mul(2).saturating_add(5)),
            Some(0) => None,
            Some(limit) => Some(limit),
        }
    }
}

/// Current commanded and measured regulator values.
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct State {
    /// Configured target speed.
    pub commanded_rpm: f64,
    /// Measured speed once a complete sensor window is available.
    pub measured_rpm: Option<f64>,
    /// Difference between target and measured speed.
    pub error_rpm: Option<f64>,
    /// Feed-forward component of the force command.
    pub feed_forward_force: f64,
    /// Integral component of the force command.
    pub integral_force: f64,
    /// Force before output clamping.
    pub unclamped_force: f64,
    /// Clamped force command.
    pub force: u16,
    /// Whether the force command is at the configured cap.
    pub at_force_cap: bool,
}

/// Stateful speed-window measurement and feed-forward PI force controller.
#[derive(Default)]
pub struct Controller {
    history: VecDeque<(u64, i64)>,
    last_timer: Option<u32>,
    sensor_ticks: u64,
    started_us: Option<u64>,
    updated_us: u64,
    /// Latest regulator output.
    pub state: State,
}

impl Controller {
    /// Observe a counter sample and update the speed regulator when its window is ready.
    ///
    /// # Errors
    /// Returns an error for stale sensor timing, invalid speed, or failed numeric conversion.
    pub fn observe(
        &mut self,
        timer: u32,
        now_us: u64,
        counts: i64,
        counts_per_revolution: u32,
        settings: Settings,
    ) -> anyhow::Result<()> {
        let started_us = *self.started_us.get_or_insert_with(|| {
            self.updated_us = now_us;
            now_us
        });
        if let Some(previous) = self.last_timer.replace(timer) {
            let elapsed_ticks = u64::from(timer.wrapping_sub(previous));
            ensure!(
                elapsed_ticks <= super::decode::FRESHNESS_US * SENSOR_TICKS_PER_US,
                "rotation sensor time gap exceeds 50ms"
            );
            self.sensor_ticks += elapsed_ticks;
        }
        self.history.push_back((self.sensor_ticks, counts));
        // Retain one boundary sample, so the window is never shortened below 20ms.
        let window_ticks = WINDOW_US * SENSOR_TICKS_PER_US;
        while self.history.len() > 2
            && self.history[1].0 <= self.sensor_ticks.saturating_sub(window_ticks)
        {
            self.history.pop_front();
        }
        let (first_ticks, first_counts) = self.history[0];
        let span_ticks = self.sensor_ticks - first_ticks;
        if span_ticks < window_ticks {
            self.updated_us = now_us;
            return Ok(());
        }
        let measured = (counts - first_counts)
            .to_f64()
            .context("rotation count conversion")?
            * f64::from(settings.count_direction)
            * 60_000_000.0
            * SENSOR_TICKS_PER_US
                .to_f64()
                .context("sensor scale conversion")?
            / span_ticks.to_f64().context("rotation span conversion")?
            / f64::from(counts_per_revolution);
        ensure!(measured >= -1.0, "rotation counter direction reversed");
        if let Some(limit) = settings.effective_overspeed_limit_rpm() {
            ensure!(
                measured <= f64::from(limit),
                "rotation overspeed: measured {measured:.3} RPM exceeds {limit} RPM"
            );
        }
        let elapsed = (now_us - started_us)
            .to_f64()
            .context("rotation elapsed conversion")?
            / 1_000_000.0;
        let dt = (now_us - self.updated_us)
            .to_f64()
            .context("rotation interval conversion")?
            / 1_000_000.0;
        self.updated_us = now_us;
        self.update(measured, elapsed, dt, settings)
    }

    /// Apply the regulator law to a measured speed and elapsed intervals.
    ///
    /// # Errors
    /// Returns an error if the calculated force cannot be represented as `u16`.
    pub fn update(
        &mut self,
        measured: f64,
        elapsed: f64,
        dt: f64,
        settings: Settings,
    ) -> anyhow::Result<()> {
        let commanded = f64::from(settings.target_rpm);
        let error = commanded - measured;
        let feed_forward_force = (f64::from(settings.force_per_rpm) * commanded).floor()
            + f64::from(settings.base_force);
        let cap = f64::from(settings.max_force);
        let proportional = f64::from(settings.kp) * error;
        let candidate = (f64::from(settings.ki) * error)
            .mul_add(dt, self.state.integral_force)
            .clamp(-cap, cap);
        let candidate_force = feed_forward_force + proportional + candidate;
        // Do not wind up further into saturation; opposite error may unwind it.
        let integral_force =
            if (candidate_force > cap && error > 0.0) || (candidate_force < 0.0 && error < 0.0) {
                self.state.integral_force
            } else {
                candidate
            };
        let unclamped_force = feed_forward_force + proportional + integral_force;
        let desired = unclamped_force.clamp(0.0, cap);
        let startup_seconds = f64::from(settings.startup_ms) / 1000.0;
        let startup_rise = f64::from(settings.startup_force_rise_per_second);
        // A zero startup rate selects a step only inside the configured startup
        // interval. A delayed boundary-crossing update gets no borrowed step.
        let force = if elapsed <= startup_seconds && startup_rise == 0.0 {
            desired
        } else if startup_rise == 0.0 {
            let rise =
                f64::from(settings.force_rise_per_second) * dt.min(elapsed - startup_seconds);
            desired.min(f64::from(self.state.force) + rise)
        } else {
            let startup_dt = (startup_seconds - (elapsed - dt)).clamp(0.0, dt);
            let steady_dt = dt - startup_dt;
            let rise = startup_rise.mul_add(
                startup_dt,
                f64::from(settings.force_rise_per_second) * steady_dt,
            );
            desired.min(f64::from(self.state.force) + rise)
        }
        .round()
        .to_u16()
        .context("rotation force conversion")?;
        self.state = State {
            commanded_rpm: commanded,
            measured_rpm: Some(measured),
            error_rpm: Some(error),
            feed_forward_force,
            integral_force,
            unclamped_force,
            force,
            at_force_cap: force == settings.max_force,
        };
        Ok(())
    }
}
