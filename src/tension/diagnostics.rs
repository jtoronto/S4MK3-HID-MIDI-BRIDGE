//! Debug-only sampled observations; never change motor selection or force.

use serde::Serialize;

use super::{Engine, playback};
use crate::jog::decode::{Motion, Status};

const INTERVAL_US: u64 = 20_000;

#[derive(Serialize)]
struct Sample {
    host_us: u64,
    side: usize,
    counter: u32,
    sensor_timer: u32,
    delta: i32,
    sensor_status: Status,
    touch: bool,
    authorization: playback::PlaybackState,
    requested_force: i32,
    #[serde(flatten)]
    playback: playback::Diagnostic,
}

impl Engine {
    pub(super) fn log_regulator(&mut self, motion: &Motion, torque: [i32; 2]) {
        use crate::screen_motor_diag::{self as diag, Kind};
        if diag::enabled() {
            for side in 0..2 {
                let status = match motion.status[side] {
                    Status::Baseline => 0,
                    Status::Moving => 1,
                    Status::Rest => 2,
                    Status::Duplicate => 3,
                };
                diag::record(
                    Kind::Sensor,
                    [
                        motion.host_us,
                        u64::try_from(side).unwrap_or(0),
                        u64::from(motion.counters[side]),
                        u64::from(motion.timers[side]),
                        u64::from(u32::from_le_bytes(motion.delta[side].to_le_bytes())),
                        status,
                        self.sensor_seen_us[side],
                        self.input_deadline_us(),
                    ],
                    &[],
                );
                let mode = self.playback.as_ref().map_or(0, |wheels| {
                    match wheels[side]
                        .diagnostic(self.playback_state.continuous[side])
                        .mode
                    {
                        playback::Mode::Disabled => 1,
                        playback::Mode::Qualifying => 2,
                        playback::Mode::Regulating => 3,
                        playback::Mode::Assisting => 4,
                    }
                });
                diag::record(
                    Kind::MotorState,
                    [
                        motion.host_us,
                        u64::try_from(side).unwrap_or(0),
                        u64::from(motion.touch[side]),
                        u64::from(self.playback_state.enabled[side]),
                        u64::from(self.playback_state.owned[side]),
                        u64::from(self.playback_state.continuous[side]),
                        mode,
                        u64::from(u32::from_le_bytes(torque[side].to_le_bytes())),
                    ],
                    &[],
                );
            }
        }
        if !tracing::enabled!(tracing::Level::DEBUG) {
            return;
        }
        let Some(wheels) = &self.playback else {
            return;
        };
        for (side, wheel) in wheels.iter().enumerate() {
            let playback = wheel.diagnostic(self.playback_state.continuous[side]);
            if self.last_regulator_diagnostic[side].is_some_and(|(at, mode)| {
                mode == playback.mode && motion.host_us.saturating_sub(at) < INTERVAL_US
            }) {
                continue;
            }
            self.last_regulator_diagnostic[side] = Some((motion.host_us, playback.mode));
            let sample = Sample {
                host_us: motion.host_us,
                side,
                counter: motion.counters[side],
                sensor_timer: motion.timers[side],
                delta: motion.delta[side],
                sensor_status: motion.status[side],
                touch: motion.touch[side],
                authorization: self.playback_state,
                requested_force: torque[side],
                playback,
            };
            match serde_json::to_string(&sample) {
                Ok(data) => tracing::debug!(%data, "MOTOR_REGULATOR"),
                Err(error) => tracing::warn!(%error, "MOTOR_REGULATOR_SERIALIZATION_FAILED"),
            }
        }
    }
}
