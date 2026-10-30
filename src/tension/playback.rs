//! Playback regulation and bounded forward assistance share one fresh motor owner.

use std::collections::VecDeque;

use anyhow::{Context as _, ensure};
use num_traits::ToPrimitive as _;

use crate::jog::{
    decode::{Motion, Status},
    platter::{Controller, Settings},
};

/// Measured relative encoder scale, independent of the jog-ring LED period.
pub const COUNTS_PER_REVOLUTION: u32 = 2880;

/// Authorization from real cached Djay feedback and local gesture ownership.
#[derive(Clone, Copy, Debug, Default)]
pub struct PlaybackState {
    pub enabled: [bool; 2],
    pub owned: [bool; 2],
    /// Keep the motor assist active through touch and owned scratch gestures.
    pub continuous: [bool; 2],
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
            samples: VecDeque::with_capacity(32),
            last_timer: None,
            sensor_ticks: 0,
            measured_rpm: None,
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
                // MIDI Scratch ownership does not imply physical contact, especially
                // at the upper handoff. Only contact bypasses speed regulation.
                let assisting = motion.touch[side]
                    || rpm < 0.0
                    || ((self.qualified || self.continuous_assist || owned)
                        && if self.continuous_assist {
                            rpm < f64::from(self.settings.target_rpm) - 0.75
                        } else {
                            rpm < f64::from(self.settings.target_rpm) - 2.0
                        });
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
        let window = crate::jog::platter::WINDOW_US * crate::jog::platter::SENSOR_TICKS_PER_US;
        while self
            .samples
            .get(1)
            .is_some_and(|(ticks, _)| self.sensor_ticks - ticks >= window)
        {
            self.samples.pop_front();
        }
        let &(first, counts) = self
            .samples
            .front()
            .context("continuous motor speed baseline")?;
        let elapsed = self.sensor_ticks - first;
        if elapsed >= window {
            let rpm = (self.counts - counts)
                .to_f64()
                .context("continuous motor counts")?
                * 6_000_000_000.0
                / elapsed.to_f64().context("continuous motor interval")?
                / f64::from(COUNTS_PER_REVOLUTION);
            if let Some(limit) = self.settings.effective_overspeed_limit_rpm() {
                ensure!(rpm <= f64::from(limit), "continuous motor overspeed");
            }
            self.measured_rpm = Some(rpm);
        }
        Ok(())
    }

    pub(super) fn force(&self, continuous: bool) -> i32 {
        if continuous && self.measured_rpm.is_none() {
            return 0;
        }
        if continuous && (self.continuous_assist || self.controller.state.measured_rpm.is_none()) {
            return i32::from(
                self.settings
                    .base_force
                    .saturating_add(
                        self.settings
                            .force_per_rpm
                            .saturating_mul(self.settings.target_rpm),
                    )
                    .min(self.settings.max_force),
            );
        }
        // The controller clamps to a u16 cap and only produces clockwise force.
        i32::from(self.controller.state.force)
    }
}
