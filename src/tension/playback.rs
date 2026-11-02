//! Playback regulation and bounded forward assistance share one fresh motor owner.

use std::collections::VecDeque;

use anyhow::{Context as _, ensure};
use num_traits::ToPrimitive as _;
use serde::Serialize;

use crate::jog::{
    decode::{Motion, Status},
    platter::{Controller, Settings},
};

/// Measured relative encoder scale, independent of the jog-ring LED period.
pub const COUNTS_PER_REVOLUTION: u32 = 2880;

const ASSIST_CONFIRMATION_US: u64 = 100_000;

/// Authorization from real cached Djay feedback and local gesture ownership.
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct PlaybackState {
    pub enabled: [bool; 2],
    pub owned: [bool; 2],
    /// Keep the motor assist active through touch and owned scratch gestures.
    pub continuous: [bool; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Mode {
    Disabled,
    Qualifying,
    Regulating,
    Assisting,
}

#[derive(Serialize)]
pub(super) struct Diagnostic {
    pub mode: Mode,
    motor_rpm: Option<f64>,
    assistance_rpm: Option<f64>,
    window_counts: Option<i64>,
    window_ticks: Option<u64>,
    regulator: Option<crate::jog::platter::State>,
}

pub(super) struct Wheel {
    settings: Settings,
    controller: Controller,
    counts: i64,
    allowed: bool,
    continuous_assist: bool,
    samples: VecDeque<(u64, i64)>,
    last_timer: Option<u32>,
    sensor_ticks: u64,
    measured_rpm: Option<f64>,
    assistance_rpm: Option<f64>,
    window: Option<(i64, u64)>,
    qualified: bool,
}

impl Wheel {
    pub(super) fn new(settings: Settings) -> Self {
        Self {
            settings,
            controller: Controller::default(),
            counts: 0,
            allowed: false,
            continuous_assist: false,
            samples: VecDeque::with_capacity(64),
            last_timer: None,
            sensor_ticks: 0,
            measured_rpm: None,
            assistance_rpm: None,
            window: None,
            qualified: false,
        }
    }

    pub(super) fn reset(&mut self) {
        self.controller = Controller::default();
        self.counts = 0;
        self.allowed = false;
        self.continuous_assist = false;
        self.samples.clear();
        self.last_timer = None;
        self.sensor_ticks = 0;
        self.measured_rpm = None;
        self.assistance_rpm = None;
        self.window = None;
        self.qualified = false;
    }

    pub(super) fn observe(
        &mut self,
        motion: &Motion,
        side: usize,
        allowed: bool,
        continuous: bool,
        owned: bool,
    ) -> anyhow::Result<()> {
        if !allowed {
            self.reset();
            return Ok(());
        }
        if !continuous && motion.delta[side] < 0 {
            // A released scratch may still coast backwards without qualifying
            // for boosted backspin. Never drive against it or feed it into the
            // unidirectional regulator; require a new forward/rest window.
            self.reset();
            return Ok(());
        }
        if !self.allowed {
            self.reset();
            self.allowed = true;
        }
        match motion.status[side] {
            Status::Duplicate => return Ok(()),
            Status::Baseline => self.counts = 0,
            Status::Moving | Status::Rest => {
                self.counts += i64::from(motion.delta[side]);
            }
        }
        if continuous {
            self.measure(motion.timers[side])?;
            if let Some(rpm) = self.measured_rpm {
                if rpm >= f64::from(self.settings.target_rpm) - 0.75 {
                    self.qualified = true;
                }
                // A brief sensor-speed dip is not proof of deliberate rim braking.
                // Confirm new underspeed assistance with the longer encoder window;
                // contact/reverse and recovery of an existing assist stay immediate.
                let assisting = motion.touch[side]
                    || rpm < 0.0
                    || if self.continuous_assist {
                        rpm < f64::from(self.settings.target_rpm) - 0.75
                    } else {
                        (self.qualified || owned)
                            && rpm < f64::from(self.settings.target_rpm) - 2.0
                            && self
                                .assistance_rpm
                                .is_some_and(|rpm| rpm < f64::from(self.settings.target_rpm) - 2.0)
                    };
                if assisting != self.continuous_assist {
                    self.controller = Controller::default();
                    self.continuous_assist = assisting;
                }
                if assisting {
                    return Ok(()); // Never integrate an error against the user's hand.
                }
            }
        }
        self.controller.observe(
            motion.timers[side],
            motion.host_us,
            self.counts,
            COUNTS_PER_REVOLUTION,
            self.settings,
        )
    }

    fn measure(&mut self, timer: u32) -> anyhow::Result<()> {
        if let Some(previous) = self.last_timer.replace(timer) {
            self.sensor_ticks += u64::from(timer.wrapping_sub(previous));
        }
        self.samples.push_back((self.sensor_ticks, self.counts));
        let clock = self.sensor_ticks;
        let counter = self.counts;
        let short_window =
            crate::jog::platter::WINDOW_US * crate::jog::platter::SENSOR_TICKS_PER_US;
        let long_window = ASSIST_CONFIRMATION_US * crate::jog::platter::SENSOR_TICKS_PER_US;
        while self
            .samples
            .get(1)
            .is_some_and(|(ticks, _)| clock - ticks >= long_window)
        {
            self.samples.pop_front();
        }
        let speed = |&(first, counts): &(u64, i64)| -> anyhow::Result<f64> {
            Ok((counter - counts)
                .to_f64()
                .context("continuous motor counts")?
                * 6_000_000_000.0
                / (clock - first)
                    .to_f64()
                    .context("continuous motor interval")?
                / f64::from(COUNTS_PER_REVOLUTION))
        };
        let short = self
            .samples
            .iter()
            .rev()
            .find(|(ticks, _)| clock - ticks >= short_window);
        self.measured_rpm = short.map(speed).transpose()?;
        self.window = short.map(|&(ticks, counts)| (counter - counts, clock - ticks));
        self.assistance_rpm = self
            .samples
            .front()
            .filter(|(ticks, _)| clock - ticks >= long_window)
            .map(speed)
            .transpose()?;
        if let Some(rpm) = self.measured_rpm {
            if let Some(limit) = self.settings.effective_overspeed_limit_rpm() {
                ensure!(rpm <= f64::from(limit), "continuous motor overspeed");
            }
        }
        Ok(())
    }

    pub(super) fn diagnostic(&self, continuous: bool) -> Diagnostic {
        let mode = if !self.allowed {
            Mode::Disabled
        } else if self.measured_rpm.is_none() && continuous {
            Mode::Qualifying
        } else if self.continuous_assist {
            Mode::Assisting
        } else if self.controller.state.measured_rpm.is_none() {
            Mode::Qualifying
        } else {
            Mode::Regulating
        };
        let (motor_rpm, window_counts, window_ticks) = if continuous {
            (
                self.measured_rpm,
                self.window.map(|(counts, _)| counts),
                self.window.map(|(_, ticks)| ticks),
            )
        } else {
            (
                self.controller.state.measured_rpm,
                self.controller.state.window_counts,
                self.controller.state.window_ticks,
            )
        };
        Diagnostic {
            mode,
            motor_rpm,
            assistance_rpm: self.assistance_rpm,
            window_counts,
            window_ticks,
            regulator: (mode == Mode::Regulating).then_some(self.controller.state),
        }
    }

    pub(super) fn force(&self, continuous: bool) -> anyhow::Result<i32> {
        if continuous && self.measured_rpm.is_none() {
            return Ok(0);
        }
        if continuous && (self.continuous_assist || self.controller.state.measured_rpm.is_none()) {
            let target = f64::from(self.settings.target_rpm);
            let rpm = self.measured_rpm.context("continuous assistance speed")?;
            let fraction = ((target - rpm) / target.min(2.0)).clamp(0.0, 1.0);
            let full_force = (u32::from(self.settings.base_force)
                + u32::from(self.settings.force_per_rpm) * u32::from(self.settings.target_rpm))
            .min(u32::from(self.settings.max_force));
            return (f64::from(full_force) * fraction)
                .round()
                .to_i32()
                .context("continuous assistance force");
        }
        // The controller clamps to a u16 cap and only produces clockwise force.
        Ok(i32::from(self.controller.state.force))
    }
}
