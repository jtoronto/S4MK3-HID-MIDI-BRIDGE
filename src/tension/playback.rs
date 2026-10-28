//! Playback owns only an untouched, fresh wheel; it never brakes a platter.

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
}

pub(super) struct Wheel {
    settings: Settings,
    controller: Controller,
    counts: i64,
    allowed: bool,
}

impl Wheel {
    pub(super) fn new(settings: Settings) -> Self {
        Self {
            settings,
            controller: Controller::default(),
            counts: 0,
            allowed: false,
        }
    }

    pub(super) fn reset(&mut self) {
        self.controller = Controller::default();
        self.counts = 0;
        self.allowed = false;
    }

    pub(super) fn observe(
        &mut self,
        motion: &Motion,
        side: usize,
        allowed: bool,
    ) -> anyhow::Result<()> {
        if !allowed {
            self.reset();
            return Ok(());
        }
        if motion.delta[side] < 0 {
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
        self.controller.observe(
            motion.timers[side],
            motion.host_us,
            self.counts,
            COUNTS_PER_REVOLUTION,
            self.settings,
        )
    }

    pub(super) fn force(&self) -> i32 {
        // The controller clamps to a u16 cap and only produces clockwise force.
        i32::from(self.controller.state.force)
    }
}
