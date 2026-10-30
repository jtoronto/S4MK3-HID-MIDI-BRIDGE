//! Decode the full 32-bit counters without assuming a hardware timer frequency.

use anyhow::ensure;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RawFrame {
    pub host_us: u64,
    pub buttons: Vec<u8>,
    pub jog: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub analog: Option<Vec<u8>>,
}

pub const FRESHNESS_US: u64 = 50_000;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Baseline,
    Moving,
    Rest,
    Duplicate,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Motion {
    pub host_us: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deck1_fader: Option<u16>,
    pub touch: [bool; 2],
    pub counters: [u32; 2],
    pub timers: [u32; 2],
    pub delta: [i32; 2],
    pub elapsed_us: [u64; 2],
    pub status: [Status; 2],
}

#[derive(Clone, Copy)]
struct Observation {
    host_us: u64,
    timer: u32,
    counter: u32,
}

#[derive(Default)]
pub struct Decoder {
    previous: [Option<Observation>; 2],
    last_host_us: Option<u64>,
}

impl Decoder {
    /// Decode a recorded diagnostic frame.
    ///
    /// # Errors
    /// Rejects malformed framing, nonmonotonic timestamps, and implausible motion.
    pub fn decode(&mut self, frame: &RawFrame) -> anyhow::Result<Motion> {
        self.decode_snapshot(
            &frame.buttons,
            &frame.jog,
            frame.analog.as_deref(),
            frame.host_us,
        )
    }

    /// Decode the shared numbered report fields without allocating.
    ///
    /// # Errors
    /// Rejects malformed framing, nonmonotonic timestamps, and implausible motion.
    pub fn decode_snapshot(
        &mut self,
        buttons: &[u8],
        jog: &[u8],
        analog: Option<&[u8]>,
        host_us: u64,
    ) -> anyhow::Result<Motion> {
        ensure!(
            buttons.len() == 23 && buttons[0] == 1,
            "expected numbered 23-byte buttons report"
        );
        ensure!(
            jog.len() == 60 && jog[0] == 3,
            "expected numbered 60-byte jog report"
        );
        if let Some(last) = self.last_host_us {
            ensure!(host_us > last, "host timestamps must strictly increase");
        }
        let deck1_fader = if let Some(analog) = analog {
            ensure!(
                analog.len() == 79 && analog[0] == 2,
                "expected numbered 79-byte analog report"
            );
            let value = u16::from_le_bytes([analog[3], analog[4]]);
            ensure!(value <= 4095, "deck1 fader exceeds 12-bit range 4095");
            Some(value)
        } else {
            None
        };
        let mut motion = Motion {
            host_us,
            deck1_fader,
            touch: [buttons[17] & 0x10 != 0, buttons[17] & 0x20 != 0],
            counters: [word(jog, 12), word(jog, 40)],
            timers: [word(jog, 8), word(jog, 36)],
            delta: [0; 2],
            elapsed_us: [0; 2],
            status: [Status::Baseline; 2],
        };
        for side in 0..2 {
            let current = Observation {
                host_us,
                timer: motion.timers[side],
                counter: motion.counters[side],
            };
            if let Some(old) = self.previous[side] {
                let elapsed = current.host_us - old.host_us;
                ensure!(
                    elapsed <= FRESHNESS_US,
                    "wheel {side} input gap exceeds 50ms"
                );
                if current.timer == old.timer {
                    ensure!(
                        current.counter == old.counter,
                        "counter changed without sensor time"
                    );
                    motion.status[side] = Status::Duplicate;
                    continue; // A duplicate cannot refresh the freshness deadline.
                }
                // Compare modular distance, not absolute proximity to zero:
                // a valid polling interval can span millions of sensor ticks.
                // A backwards timestamp has a distance in the upper half.
                ensure!(
                    current.timer.wrapping_sub(old.timer) <= i32::MAX.unsigned_abs(),
                    "wheel {side} sensor time reversed: previous={} current={} host_us={host_us} elapsed_us={elapsed}",
                    old.timer,
                    current.timer
                );
                let delta =
                    i32::from_le_bytes(current.counter.wrapping_sub(old.counter).to_le_bytes());
                ensure!(
                    delta.unsigned_abs() <= 512,
                    "wheel {side} count jump exceeds 512"
                );
                motion.delta[side] = delta;
                motion.elapsed_us[side] = elapsed;
                motion.status[side] = if delta == 0 {
                    Status::Rest
                } else {
                    Status::Moving
                };
            }
            self.previous[side] = Some(current);
        }
        self.last_host_us = Some(host_us);
        Ok(motion)
    }
}

fn word(report: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        report[offset],
        report[offset + 1],
        report[offset + 2],
        report[offset + 3],
    ])
}
