//! Bounded, unidirectional speed probe, not a reconstruction of Traktor's loop.

use std::ops::{Deref, DerefMut};

use anyhow::ensure;
use clap::Args;
use s4mk3_hid_midi_bridge::jog::platter as regulator;
use serde::Serialize;

use super::platter::{CommonOptions, Config, Experiment, Polarity};

pub use regulator::{SENSOR_TICKS_PER_US, WINDOW_US};
pub const DEFAULT_BASE_FORCE: u16 = 1650;
pub const DEFAULT_FORCE_PER_RPM: u16 = 110;
// Measured over the complete attended capture: 99.9986 sensor ticks/us.
// This calibration is rotation-only; the shared decoder remains frequency-free.
pub const DEFAULT_KP: u16 = 450;
pub const DEFAULT_KI: u16 = 150;
pub const DEFAULT_FORCE_RISE_PER_SECOND: u32 = 50_000;
pub const DEFAULT_STARTUP_MS: u16 = 200;
pub const DEFAULT_STARTUP_FORCE_RISE_PER_SECOND: u32 = 0;

#[derive(Debug, Args)]
pub struct Options {
    #[command(flatten)]
    common: CommonOptions,
    /// Verified raw motor polarity for the intended direction.
    #[arg(long, value_enum)]
    polarity: Polarity,
    /// Verified raw counter sign for that same physical direction.
    #[arg(long, value_enum)]
    count_direction: Polarity,
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=1_000_000_000))]
    counts_per_revolution: u32,
    /// Positive speed in the explicitly selected direction, not Djay playback.
    #[arg(long, default_value_t = 33, value_parser = clap::value_parser!(u16).range(1..=45))]
    target_rpm: u16,
    /// Abort above this RPM; defaults to 2 * target + 5, and zero disables this guard.
    #[arg(long, value_parser = clap::value_parser!(u16).range(0..=250))]
    overspeed_limit_rpm: Option<u16>,
    #[arg(long, default_value_t = 8000, value_parser = clap::value_parser!(u16).range(1..=25000))]
    max_force: u16,
    /// Maximum powered window after MOVE; defaults to 2 seconds, up to 15.
    #[arg(long, default_value_t = 2000, value_parser = clap::value_parser!(u64).range(1..=15_000))]
    drive_ms: u64,
    /// Feed-forward force at zero RPM.
    #[arg(long, default_value_t = DEFAULT_BASE_FORCE, value_parser = clap::value_parser!(u16).range(0..=25_000))]
    base_force: u16,
    /// Feed-forward force added per target RPM.
    #[arg(long, default_value_t = DEFAULT_FORCE_PER_RPM, value_parser = clap::value_parser!(u16).range(0..=25_000))]
    force_per_rpm: u16,
    /// Proportional correction in force units per RPM of speed error.
    #[arg(long, default_value_t = DEFAULT_KP, value_parser = clap::value_parser!(u16).range(0..=25_000))]
    kp: u16,
    /// Integral correction in force units per RPM-second of speed error.
    #[arg(long, default_value_t = DEFAULT_KI, value_parser = clap::value_parser!(u16).range(0..=25_000))]
    ki: u16,
    /// Maximum force increase per second after startup.
    #[arg(long, default_value_t = DEFAULT_FORCE_RISE_PER_SECOND, value_parser = clap::value_parser!(u32).range(0..=1_000_000))]
    force_rise_per_second: u32,
    /// Startup force behavior duration after MOVE arms.
    #[arg(long, default_value_t = DEFAULT_STARTUP_MS, value_parser = clap::value_parser!(u16).range(0..=2_000))]
    startup_ms: u16,
    /// Maximum startup force increase per second; zero means an immediate step.
    #[arg(long, default_value_t = DEFAULT_STARTUP_FORCE_RISE_PER_SECOND, value_parser = clap::value_parser!(u32).range(0..=1_000_000))]
    startup_force_rise_per_second: u32,
    #[arg(long)]
    enable_motor: bool,
    /// Initialize only the selected wheel, as in the fixed-force diagnostic.
    #[arg(long)]
    initialize_ring: bool,
}

#[cfg(test)]
#[path = "platter_rotation_tests.rs"]
mod tests;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Settings {
    pub polarity: Polarity,
    pub count_direction: Polarity,
    pub target_rpm: u16,
    pub overspeed_limit_rpm: Option<u16>,
    pub max_force: u16,
    pub drive_ms: u64,
    pub base_force: u16,
    pub force_per_rpm: u16,
    pub kp: u16,
    pub ki: u16,
    pub force_rise_per_second: u32,
    pub startup_ms: u16,
    pub startup_force_rise_per_second: u32,
    pub initialize_ring: bool,
}

impl Settings {
    pub const fn effective_overspeed_limit_rpm(&self) -> Option<u16> {
        match self.overspeed_limit_rpm {
            None => Some(self.target_rpm.saturating_mul(2).saturating_add(5)),
            Some(0) => None,
            Some(limit) => Some(limit),
        }
    }

    const fn regulator_settings(self) -> regulator::Settings {
        regulator::Settings {
            target_rpm: self.target_rpm,
            overspeed_limit_rpm: self.overspeed_limit_rpm,
            max_force: self.max_force,
            base_force: self.base_force,
            force_per_rpm: self.force_per_rpm,
            kp: self.kp,
            ki: self.ki,
            startup_ms: self.startup_ms,
            force_rise_per_second: self.force_rise_per_second,
            startup_force_rise_per_second: self.startup_force_rise_per_second,
            count_direction: self.count_direction.sign(),
        }
    }
}

impl Options {
    pub fn resolve(self) -> anyhow::Result<Config> {
        ensure!(self.enable_motor, "platter-rotate requires --enable-motor");
        Ok(Config {
            common: self.common,
            counts_per_revolution: Some(self.counts_per_revolution),
            experiment: Experiment::Rotate(Settings {
                polarity: self.polarity,
                count_direction: self.count_direction,
                target_rpm: self.target_rpm,
                overspeed_limit_rpm: self.overspeed_limit_rpm,
                max_force: self.max_force,
                drive_ms: self.drive_ms,
                base_force: self.base_force,
                force_per_rpm: self.force_per_rpm,
                kp: self.kp,
                ki: self.ki,
                force_rise_per_second: self.force_rise_per_second,
                startup_ms: self.startup_ms,
                startup_force_rise_per_second: self.startup_force_rise_per_second,
                initialize_ring: self.initialize_ring,
            }),
        })
    }
}

pub use regulator::State;

#[derive(Default)]
pub struct Controller(regulator::Controller);

impl Deref for Controller {
    type Target = regulator::Controller;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Controller {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Controller {
    /// Forward a sample to the shared platter regulator.
    pub fn observe(
        &mut self,
        timer: u32,
        now_us: u64,
        counts: i64,
        counts_per_revolution: u32,
        settings: Settings,
    ) -> anyhow::Result<()> {
        self.0.observe(
            timer,
            now_us,
            counts,
            counts_per_revolution,
            settings.regulator_settings(),
        )
    }

    #[cfg(test)]
    fn update(
        &mut self,
        measured: f64,
        elapsed: f64,
        dt: f64,
        settings: Settings,
    ) -> anyhow::Result<()> {
        self.0
            .update(measured, elapsed, dt, settings.regulator_settings())
    }
}
