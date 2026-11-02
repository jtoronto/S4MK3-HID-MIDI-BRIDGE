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
