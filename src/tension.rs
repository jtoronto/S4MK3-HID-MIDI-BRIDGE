//! Offline shared wheel-tension selection and state, with no device I/O.

use std::path::Path;

use anyhow::{Context as _, ensure};
use serde::{Deserialize, Deserializer, de::Error as _};

use crate::jog::{
    Decision, FaderState,
    decode::{Decoder, FRESHNESS_US, Motion},
    model::report_for,
    smooth_wheels::SmoothWheels,
};

mod backspin;
mod playback;

pub use crate::jog::neutral_report;
pub use playback::{COUNTS_PER_REVOLUTION, PlaybackState};

const MAX_ASSIST_FORCE: u16 = 7000;

#[derive(Clone, Copy, Debug, Default)]
pub struct Config {
    tension: i32,
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct SavedConfig {
    tension: i32,
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let saved = SavedConfig::deserialize(deserializer)?;
        if !(-100..=100).contains(&saved.tension) {
            return Err(D::Error::custom("tension must be between -100 and 100"));
        }
        Ok(Self {
            tension: saved.tension,
        })
    }
}

impl Config {
    /// Read a saved selection, or use neutral when no file is supplied.
    ///
    /// # Errors
    /// Rejects unreadable JSON, unknown fields, non-integer tension, and values
    /// outside -100..=100.
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        let data = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
        serde_json::from_slice(&data).with_context(|| format!("parse {}", path.display()))
    }

    #[must_use]
    pub const fn value(&self) -> i32 {
        self.tension
    }
}

pub struct Engine {
    decoder: Decoder,
    wheels: SmoothWheels,
    selection: FaderState,
    last_input_us: u64,
    last_torque: [i32; 2],
    motion: Option<Motion>,
    backspin: [backspin::Assist; 2],
    playback: Option<[playback::Wheel; 2]>,
    playback_state: PlaybackState,
    sensor_seen_us: [u64; 2],
}

#[cfg(test)]
#[path = "tension_tests.rs"]
mod tests;

impl Engine {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self::with_strength(config, 100)
    }

    /// Select optional backspin assistance independently of normal tension.
    ///
    /// # Errors
    /// Rejects percentages above the existing maximum assistance.
    pub fn with_backspin_strength(config: Config, strength: u8) -> anyhow::Result<Self> {
        ensure!(
            strength <= 100,
            "backspin strength must be between 0 and 100"
        );
        Ok(Self::with_strength(config, strength))
    }

    fn with_strength(config: Config, strength: u8) -> Self {
        let value = config.value();
        let selection = match value.cmp(&0) {
            std::cmp::Ordering::Less => {
                // Config bounds this scaled value to 7000.
                let bytes =
                    (value.unsigned_abs() * u32::from(MAX_ASSIST_FORCE) / 100).to_le_bytes();
                let cap = u16::from_le_bytes([bytes[0], bytes[1]]);
                FaderState::bipolar(0, true, cap, 0)
            }
            std::cmp::Ordering::Greater => {
                // Config bounds this scaled value to 30000.
                let bytes = (value.unsigned_abs() * 30000 / 100).to_le_bytes();
                let cap = u16::from_le_bytes([bytes[0], bytes[1]]);
                FaderState::bipolar(4095, true, 0, cap)
            }
            std::cmp::Ordering::Equal => FaderState::bipolar(2048, true, 0, 0),
        };
        Self {
            decoder: Decoder::default(),
            wheels: SmoothWheels::new([true, true]),
            selection,
            last_input_us: 0,
            last_torque: [0; 2],
            motion: None,
            backspin: [
                backspin::Assist::new(0, strength),
                backspin::Assist::new(1, strength),
            ],
            playback: None,
            playback_state: PlaybackState::default(),
            sensor_seen_us: [0; 2],
        }
    }

    /// Enable the shared speed regulator without changing diagnostic defaults.
    pub fn configure_playback(&mut self, settings: crate::jog::platter::Settings) {
        self.playback = Some(std::array::from_fn(|_| playback::Wheel::new(settings)));
    }

    /// Update cached real playback authorization; cancellation is an immediate zero.
    #[must_use]
    pub fn set_playback(&mut self, state: PlaybackState) -> Option<[u8; 11]> {
        let mut changed = false;
        if let Some(wheels) = &mut self.playback {
            for (side, wheel) in wheels.iter_mut().enumerate() {
                if state.enabled[side] != self.playback_state.enabled[side]
                    || state.owned[side] != self.playback_state.owned[side]
                {
                    wheel.reset();
                    if self.last_torque[side] != 0 {
                        self.last_torque[side] = 0;
                        changed = true;
                    }
                }
            }
        }
        self.playback_state = state;
        changed.then(|| report_for(self.last_torque))
    }

    /// Discard all movement and smoothing history, preserving the selection.
    /// The caller must neutralize hardware first. This does not refresh the
    /// input deadline; a new snapshot must establish a baseline before ticking.
    pub fn clear_motion(&mut self) {
        self.decoder = Decoder::default();
        self.wheels = SmoothWheels::new([true, true]);
        self.last_torque = [0; 2];
        self.motion = None;
        if let Some(wheels) = &mut self.playback {
            for wheel in wheels {
                wheel.reset();
            }
        }
        for assist in &mut self.backspin {
            assist.clear();
        }
    }

    /// Supply current full-profile gesture qualification without changing touch.
    /// Returns an immediate partial neutral when an override is cancelled.
    #[must_use]
    pub fn set_backspin(&mut self, qualified: [bool; 2]) -> Option<[u8; 11]> {
        let motion = self.motion?;
        let mut changed = false;
        for (side, qualified) in qualified.into_iter().enumerate() {
            if self.backspin[side].qualify(qualified, &motion) && self.last_torque[side] != 0 {
                self.last_torque[side] = 0;
                changed = true;
            }
        }
        changed.then(|| report_for(self.last_torque))
    }

    /// Observe one numbered buttons/jog snapshot and return only an immediate
    /// zero-gate report. The caller owns all report writes and cleanup.
    ///
    /// # Errors
    /// Rejects stale, future, malformed, or discontinuous input.
    pub fn observe(
        &mut self,
        buttons: &[u8],
        jog: &[u8],
        host_us: u64,
        now_us: u64,
    ) -> anyhow::Result<Option<[u8; 11]>> {
        ensure!(
            now_us >= host_us && now_us - host_us <= FRESHNESS_US,
            "queued input is older than 50ms or from the future"
        );
        let sensor_gap = self.playback.is_some()
            && jog.len() == 60
            && jog[0] == 3
            && self.motion.is_some_and(|previous| {
                [(0, 8), (1, 36)].into_iter().any(|(side, offset)| {
                    let timer = u32::from_le_bytes([
                        jog[offset],
                        jog[offset + 1],
                        jog[offset + 2],
                        jog[offset + 3],
                    ]);
                    u64::from(timer.wrapping_sub(previous.timers[side]))
                        > FRESHNESS_US * crate::jog::platter::SENSOR_TICKS_PER_US
                })
            });
        if sensor_gap {
            self.clear_motion();
        }
        let motion = self.decoder.decode_snapshot(buttons, jog, None, host_us)?;
        self.motion = Some(motion);
        self.last_input_us = host_us;
        for (side, status) in motion.status.iter().enumerate() {
            if !matches!(status, crate::jog::decode::Status::Duplicate) {
                self.sensor_seen_us[side] = host_us;
            }
        }
        let selection = self.selection;
        let mut tension_motion = motion;
        if self.playback.is_some() {
            for side in 0..2 {
                if self.playback_state.enabled[side]
                    && !self.playback_state.owned[side]
                    && !motion.touch[side]
                {
                    // Powered motion is not a newly qualified manual gesture.
                    tension_motion.status[side] = crate::jog::decode::Status::Baseline;
                    tension_motion.delta[side] = 0;
                }
            }
        }
        let mut decision = self.wheels.observe(
            &tension_motion,
            Decision {
                torque: [0; 2],
                report: neutral_report(),
                fader: Some(selection),
            },
        );
        for (side, assist) in self.backspin.iter_mut().enumerate() {
            if let Some(torque) = assist.observe(&motion, now_us) {
                decision.torque[side] = torque;
            }
        }
        if let Some(wheels) = &mut self.playback {
            for (side, wheel) in wheels.iter_mut().enumerate() {
                let allowed = self.playback_state.enabled[side]
                    && !self.playback_state.owned[side]
                    && !motion.touch[side];
                wheel.observe(&motion, side, allowed)?;
                if allowed {
                    decision.torque[side] = wheel.force();
                } else if motion.touch[side]
                    && self.playback_state.enabled[side]
                    && !self.playback_state.owned[side]
                {
                    // Raw physical touch must suspend playback before MIDI or LEDs.
                    decision.torque[side] = 0;
                }
            }
        }
        let immediate_zero =
            (0..2).any(|side| decision.torque[side] == 0 && self.last_torque[side] != 0);
        if sensor_gap {
            Ok(Some(neutral_report()))
        } else if immediate_zero {
            for (side, torque) in self.last_torque.iter_mut().enumerate() {
                if decision.torque[side] == 0 {
                    *torque = 0;
                }
            }
            Ok(Some(report_for(self.last_torque)))
        } else {
            Ok(None)
        }
    }

    /// Produce the next best-effort paced output from qualified fresh movement.
    ///
    /// # Errors
    /// Rejects ticks beyond the 50 ms input freshness deadline.
    pub fn tick(&mut self, now_us: u64) -> anyhow::Result<[u8; 11]> {
        ensure!(
            now_us <= self.input_deadline_us(),
            "no fresh input within 50ms"
        );
        let mut torque = self
            .wheels
            .at(now_us)
            .map_or([0; 2], |decision| decision.torque);
        for (side, assist) in self.backspin.iter_mut().enumerate() {
            if let Some(override_torque) = assist.at(now_us, self.motion.as_ref()) {
                torque[side] = override_torque;
            }
        }
        if let Some(wheels) = &self.playback {
            for (side, wheel) in wheels.iter().enumerate() {
                if self.playback_state.enabled[side]
                    && !self.playback_state.owned[side]
                    && self.motion.is_none_or(|motion| !motion.touch[side])
                {
                    torque[side] = wheel.force();
                }
            }
        }
        self.last_torque = torque;
        Ok(report_for(torque))
    }

    #[must_use]
    pub const fn input_deadline_us(&self) -> u64 {
        let last = if self.playback.is_some() {
            if self.sensor_seen_us[0] < self.sensor_seen_us[1] {
                self.sensor_seen_us[0]
            } else {
                self.sensor_seen_us[1]
            }
        } else {
            self.last_input_us
        };
        last.saturating_add(FRESHNESS_US)
    }
}
