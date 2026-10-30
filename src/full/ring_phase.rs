//! Continuous normal-mode playback phase with Scratch deltas; TT stays physical.

use super::super::{
    feedback::FeedbackState,
    leds::{LedConfig, LocalState},
};
use anyhow::Context as _;
use std::time::Duration;

const SCALE: i128 = 1_000_000;
const TURN: i128 = 2880 * SCALE;

#[derive(Default)]
pub(super) struct Rings {
    phases: [i128; 4],
    previous: Option<Snapshot>,
}

struct Snapshot {
    now: Duration,
    playing: [bool; 4],
    scratch: [Option<usize>; 2],
    counters: [Option<u32>; 2],
}

impl Rings {
    pub(super) fn positions(
        &mut self,
        config: &LedConfig,
        feedback: &FeedbackState,
        local: &LocalState,
        now: Duration,
    ) -> anyhow::Result<[u16; 2]> {
        if let Some(previous) = &self.previous {
            let elapsed = now.saturating_sub(previous.now).as_nanos();
            let advance = i128::try_from(elapsed * 2880 / u128::from(config.chase_period_ms))
                .context("ring playback phase interval")?;
            for (deck, phase) in self.phases.iter_mut().enumerate() {
                if previous.playing[deck] && !previous.scratch.contains(&Some(deck)) {
                    *phase = (*phase + advance).rem_euclid(TURN);
                }
            }
            for side in 0..2 {
                if let (Some(deck), Some(counter), Some(old)) = (
                    local.scratch_decks[side],
                    local.ring_counters[side],
                    previous.counters[side],
                ) {
                    let delta = i32::from_le_bytes(counter.wrapping_sub(old).to_le_bytes());
                    self.phases[deck] =
                        (self.phases[deck] + i128::from(delta) * SCALE).rem_euclid(TURN);
                }
            }
        }
        self.previous = Some(Snapshot {
            now,
            playing: feedback.playing.map(|value| value == Some(true)),
            scratch: local.scratch_decks,
            counters: local.ring_counters,
        });
        let mut positions = local.ring_positions;
        for (side, position) in positions.iter_mut().enumerate() {
            if !local.motor_enabled[side] {
                *position = u16::try_from(self.phases[local.decks[side]] / SCALE)?;
            }
        }
        Ok(positions)
    }
}
