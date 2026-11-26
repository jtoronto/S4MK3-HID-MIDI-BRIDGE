//! Sensor decoding, freshness ownership and input-history resets.
use super::{
    Decision, Decoder, Engine, FRESHNESS_US, SmoothWheels, ensure, neutral_report, report_for,
};

impl Engine {
    /// Discard all movement and smoothing history, preserving the selection.
    /// The caller must neutralize hardware first. This does not refresh the
    /// input deadline; a new snapshot must establish a baseline before ticking.
    pub fn clear_motion(&mut self) {
        crate::screen_motor_diag::record(
            crate::screen_motor_diag::Kind::MotionReset,
            [
                self.last_input_us,
                self.sensor_seen_us[0],
                self.sensor_seen_us[1],
                0,
                0,
                0,
                0,
                0,
            ],
            &[],
        );
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
                if self.playback_state.continuous[side]
                    || (self.playback_state.enabled[side]
                        && !self.playback_state.owned[side]
                        && !motion.touch[side])
                {
                    // TT movement must never prime a competing normal motor owner.
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
            if !self.playback_state.continuous[side]
                && let Some(torque) = assist.observe(&motion, now_us)
            {
                decision.torque[side] = torque;
            }
        }
        if let Some(wheels) = &mut self.playback {
            for (side, wheel) in wheels.iter_mut().enumerate() {
                let continuous = self.playback_state.continuous[side];
                let allowed = self.playback_state.enabled[side]
                    && (continuous || (!self.playback_state.owned[side] && !motion.touch[side]));
                wheel.observe(
                    &motion,
                    side,
                    allowed,
                    continuous,
                    self.playback_state.owned[side],
                )?;
                if allowed {
                    decision.torque[side] = wheel.force(continuous)?;
                } else if motion.touch[side] && self.playback_state.enabled[side] {
                    // Raw physical touch must suspend playback before MIDI or LEDs.
                    decision.torque[side] = 0;
                }
            }
        }
        for (side, torque) in decision.torque.iter_mut().enumerate() {
            if self.playback_state.continuous[side] && !self.playback_state.enabled[side] {
                *torque = 0;
            }
        }
        let immediate_zero =
            (0..2).any(|side| decision.torque[side] == 0 && self.last_torque[side] != 0);
        self.log_regulator(&motion, decision.torque);
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
