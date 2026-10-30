//! Standalone relative-motion experiments. LED units are not encoder counts.
//!
//! Fixed drive deliberately starts from rest: it measures open-loop response,
//! not speed regulation or playback. Raw motor sign is not a CW/CCW promise.

use std::path::PathBuf;

use anyhow::{Context as _, ensure};
use clap::{Args, ValueEnum};
use num_traits::ToPrimitive as _;
use serde::Serialize;

use super::capture::RawFrame;
use super::decode::{Decoder, FRESHNESS_US, Motion, Status};
use super::platter_rotation;

#[derive(Clone, Copy, Debug, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Wheel {
    Left,
    Right,
}

impl Wheel {
    pub const fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }

    pub const fn wire(self) -> u8 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }

    fn move_pressed(self, frame: &RawFrame) -> bool {
        // Verified payload byte 6/mask 04 (left), 15/mask 20 (right).
        // Decoder validates numbered 23-byte framing before this is called.
        match self {
            Self::Left => frame.buttons[7] & 0x04 != 0,
            Self::Right => frame.buttons[16] & 0x20 != 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Polarity {
    Positive,
    Negative,
}

impl Polarity {
    pub(super) const fn sign(self) -> i32 {
        match self {
            Self::Positive => 1,
            Self::Negative => -1,
        }
    }
}

#[derive(Debug, Args, Serialize)]
pub struct CommonOptions {
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_enum)]
    pub side: Wheel,
    /// Overall time from setup start, including waiting for arming.
    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..=120))]
    pub seconds: u64,
}

#[derive(Debug, Args)]
pub struct MeasureOptions {
    #[command(flatten)]
    common: CommonOptions,
    /// Measured raw counts/revolution, never inferred from LED units.
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=1_000_000_000))]
    counts_per_revolution: Option<u32>,
}

#[derive(Debug, Args)]
pub struct DriveOptions {
    #[command(flatten)]
    common: CommonOptions,
    /// Explicit raw report polarity; physical direction is not assumed.
    #[arg(long, value_enum)]
    polarity: Polarity,
    /// Uncalibrated force units; deliberately below the proven 7000 assist cap.
    #[arg(long, default_value_t = 500, value_parser = clap::value_parser!(u16).range(1..=2000))]
    force: u16,
    /// Maximum duration of each MOVE-armed pulse, even if motion continues.
    #[arg(long, default_value_t = 500, value_parser = clap::value_parser!(u64).range(1..=2000))]
    drive_ms: u64,
    #[arg(long)]
    enable_motor: bool,
    /// Send the existing 0x30 initialization to the selected wheel only.
    #[arg(long)]
    initialize_ring: bool,
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=1_000_000_000))]
    counts_per_revolution: Option<u32>,
}

#[derive(Debug, Args)]
pub struct RingOptions {
    #[command(flatten)]
    common: CommonOptions,
    /// Required measured raw counts/revolution; 2880 is only the LED period.
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=1_000_000_000))]
    counts_per_revolution: u32,
    /// Reverse the raw-count to LED direction, independently for this wheel.
    #[arg(long)]
    reverse_ring: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum Experiment {
    Measure,
    Drive {
        polarity: Polarity,
        force: u16,
        drive_ms: u64,
        initialize_ring: bool,
    },
    Rotate(platter_rotation::Settings),
    Ring {
        reverse_ring: bool,
    },
}

#[derive(Debug, Serialize)]
pub struct Config {
    pub common: CommonOptions,
    pub experiment: Experiment,
    pub counts_per_revolution: Option<u32>,
}

impl From<MeasureOptions> for Config {
    fn from(options: MeasureOptions) -> Self {
        Self {
            common: options.common,
            experiment: Experiment::Measure,
            counts_per_revolution: options.counts_per_revolution,
        }
    }
}

impl DriveOptions {
    pub fn resolve(self) -> anyhow::Result<Config> {
        ensure!(self.enable_motor, "platter-drive requires --enable-motor");
        Ok(Config {
            common: self.common,
            experiment: Experiment::Drive {
                polarity: self.polarity,
                force: self.force,
                drive_ms: self.drive_ms,
                initialize_ring: self.initialize_ring,
            },
            counts_per_revolution: self.counts_per_revolution,
        })
    }
}

impl From<RingOptions> for Config {
    fn from(options: RingOptions) -> Self {
        Self {
            common: options.common,
            experiment: Experiment::Ring {
                reverse_ring: options.reverse_ring,
            },
            counts_per_revolution: Some(options.counts_per_revolution),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Gate {
    NeedRelease,
    Ready,
    Pressed,
    Driving { until_us: u64 },
}

impl Gate {
    const fn observe(self, pressed: bool, touch: bool, now_us: u64, drive_ms: u64) -> Self {
        if touch {
            return Self::NeedRelease;
        }
        match self {
            Self::NeedRelease if !pressed => Self::Ready,
            Self::Ready if pressed => Self::Pressed,
            Self::Pressed if !pressed => Self::Driving {
                until_us: now_us + drive_ms * 1000,
            },
            Self::Driving { until_us } if now_us >= until_us => Self::NeedRelease,
            Self::NeedRelease | Self::Ready | Self::Pressed | Self::Driving { .. } => self,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Sample {
    pub motion: Motion,
    /// Signed raw counter integration relative to the first accepted snapshot.
    pub relative_counts: i64,
    pub travel_counts: u64,
    /// None on baseline/duplicate, not a fabricated zero-speed measurement.
    pub counts_per_second: Option<f64>,
    pub rpm: Option<f64>,
    pub requested_force: [i32; 2],
    pub gate: Gate,
    pub ring_held: bool,
    pub ring_position: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<platter_rotation::State>,
}

pub struct Model {
    pub config: Config,
    decoder: Decoder,
    pub gate: Gate,
    pub relative_counts: i64,
    pub travel_counts: u64,
    ring_counts: i64,
    pub ring_held: bool,
    last_touch: bool,
    last_sensor_us: u64,
    rotation: platter_rotation::Controller,
}

impl Model {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            decoder: Decoder::default(),
            gate: Gate::NeedRelease,
            relative_counts: 0,
            travel_counts: 0,
            ring_counts: 0,
            ring_held: false,
            last_touch: false,
            last_sensor_us: 0,
            rotation: platter_rotation::Controller::default(),
        }
    }

    pub const fn sensor_deadline_us(&self) -> u64 {
        self.last_sensor_us + FRESHNESS_US
    }

    pub const fn deadline_us(&self) -> u64 {
        self.config.common.seconds * 1_000_000
    }

    pub const fn pulse_deadline_us(&self) -> u64 {
        match self.gate {
            Gate::Driving { until_us } => until_us,
            Gate::NeedRelease | Gate::Ready | Gate::Pressed => u64::MAX,
        }
    }

    pub fn check_time(&mut self, now_us: u64) -> anyhow::Result<Option<&'static str>> {
        if now_us >= self.deadline_us() {
            return Ok(Some("duration"));
        }
        ensure!(
            now_us < self.sensor_deadline_us(),
            "selected sensor freshness watchdog"
        );
        if let Gate::Driving { until_us } = self.gate
            && now_us >= until_us
        {
            self.gate = Gate::NeedRelease;
            self.rotation = platter_rotation::Controller::default();
        }
        Ok(None)
    }

    pub fn observe(&mut self, frame: &RawFrame, now_us: u64) -> anyhow::Result<Sample> {
        ensure!(
            self.check_time(now_us)?.is_none(),
            "overall duration elapsed"
        );
        ensure!(
            now_us >= frame.host_us && now_us - frame.host_us < FRESHNESS_US,
            "queued snapshot stale or from future"
        );
        let motion = self.decoder.decode(frame)?;
        let side = self.config.common.side.index();
        let fresh = !matches!(motion.status[side], Status::Duplicate);
        if fresh {
            self.last_sensor_us = frame.host_us;
        }
        let delta = motion.delta[side];
        self.relative_counts = self
            .relative_counts
            .checked_add(i64::from(delta))
            .context("count overflow")?;
        self.travel_counts = self
            .travel_counts
            .checked_add(u64::from(delta.unsigned_abs()))
            .context("travel overflow")?;
        // Hold both boundary samples: on release do not integrate movement that
        // happened while touch was held. Resume on the next untouched sample.
        self.ring_held = motion.touch[side];
        if !self.ring_held && !self.last_touch {
            self.ring_counts = self
                .ring_counts
                .checked_add(i64::from(delta))
                .context("ring count overflow")?;
        }
        self.last_touch = self.ring_held;
        let drive_ms = match self.config.experiment {
            Experiment::Drive { drive_ms, .. } => Some(drive_ms),
            Experiment::Rotate(settings) => Some(settings.drive_ms),
            Experiment::Measure | Experiment::Ring { .. } => None,
        };
        let was_driving = matches!(self.gate, Gate::Driving { .. });
        if let Some(drive_ms) = drive_ms {
            if motion.touch[side] {
                self.gate = Gate::NeedRelease;
            } else if fresh {
                self.gate = self.gate.observe(
                    self.config.common.side.move_pressed(frame),
                    false,
                    now_us,
                    drive_ms,
                );
            }
        }
        self.observe_rotation(motion.timers[side], now_us, fresh, was_driving)?;
        let counts_per_second = match motion.status[side] {
            Status::Moving | Status::Rest => Some(
                f64::from(delta) * 1_000_000.0
                    / motion.elapsed_us[side]
                        .to_f64()
                        .context("elapsed conversion")?,
            ),
            Status::Baseline | Status::Duplicate => None,
        };
        let rpm = counts_per_second
            .zip(self.config.counts_per_revolution)
            .map(|(speed, counts)| speed * 60.0 / f64::from(counts));
        Ok(Sample {
            motion,
            relative_counts: self.relative_counts,
            travel_counts: self.travel_counts,
            counts_per_second,
            rpm,
            requested_force: self.force(),
            gate: self.gate,
            ring_held: self.ring_held,
            ring_position: self.ring_position()?,
            rotation: matches!(self.config.experiment, Experiment::Rotate(_))
                .then_some(self.rotation.state),
        })
    }

    fn observe_rotation(
        &mut self,
        timer: u32,
        now_us: u64,
        fresh: bool,
        was_driving: bool,
    ) -> anyhow::Result<()> {
        if let Experiment::Rotate(settings) = self.config.experiment {
            let driving = matches!(self.gate, Gate::Driving { .. });
            if !driving || !was_driving {
                self.rotation = platter_rotation::Controller::default();
            }
            if driving && fresh {
                self.rotation.observe(
                    timer,
                    now_us,
                    self.relative_counts,
                    self.config
                        .counts_per_revolution
                        .context("rotation requires measured counts per revolution")?,
                    settings,
                )?;
            }
        }
        Ok(())
    }

    pub fn force(&self) -> [i32; 2] {
        let mut output = [0; 2];
        if matches!(self.gate, Gate::Driving { .. }) {
            output[self.config.common.side.index()] = match self.config.experiment {
                Experiment::Drive {
                    polarity, force, ..
                } => polarity.sign() * i32::from(force),
                Experiment::Rotate(settings) => {
                    settings.polarity.sign() * i32::from(self.rotation.state.force)
                }
                Experiment::Measure | Experiment::Ring { .. } => 0,
            };
        }
        output
    }

    pub fn ring_position(&self) -> anyhow::Result<u16> {
        let Some(counts) = self.config.counts_per_revolution else {
            return Ok(0);
        };
        let direction = match self.config.experiment {
            Experiment::Ring { reverse_ring: true } => -1,
            Experiment::Ring {
                reverse_ring: false,
            }
            | Experiment::Measure
            | Experiment::Drive { .. }
            | Experiment::Rotate(_) => 1,
        };
        // Reduce before scaling: negative integration wraps Euclideanly, and
        // neither i64 negation nor multiplication can overflow at the boundary.
        let phase = (self.ring_counts.rem_euclid(i64::from(counts)) * direction)
            .rem_euclid(i64::from(counts));
        Ok(u16::try_from(phase * 2880 / i64::from(counts))?)
    }
}

pub fn ring_initialization(side: Wheel) -> [u8; 28] {
    // Existing src/full/output.rs LedOutput::new layout, selected side only.
    let mut report = [0; 28];
    report[..4].copy_from_slice(&[0x30, side.wire(), 1, 3]);
    report
}

pub fn ring_report(side: Wheel, position: u16) -> [u8; 41] {
    // Existing src/full/output.rs render_ring mode-2 spot layout.
    let mut report = [0; 41];
    report[..3].copy_from_slice(&[0x32, side.wire(), 2]);
    report[3..5].copy_from_slice(&position.to_le_bytes());
    report[5] = 71; // src/full/leds.rs: white base 68 + intensity 3.
    report
}

#[cfg(test)]
#[path = "platter_tests.rs"]
mod tests;
