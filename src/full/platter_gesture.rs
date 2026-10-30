//! TT-only speed-to-Scratch handoff, independent of capacitive touch.

use super::{Address, Translator, catalog, relative};
use anyhow::ensure;
use s4mk3_hid_midi_bridge::tension::PlaybackState;
use tracing::{debug, info};

const KEYS: [&str; 2] = ["left_motor_scratch", "right_motor_scratch"];

#[cfg(test)]
#[path = "platter_gesture_tests.rs"]
mod tests;

impl Translator {
    pub(super) fn toggle_motor_mode(&mut self, side: usize) {
        self.mode_vinyl[side][usize::from(self.motor_enabled[side])] = Some(self.vinyl[side]);
        self.motor_enabled[side] = !self.motor_enabled[side];
        self.vinyl[side] = self.mode_vinyl[side][usize::from(self.motor_enabled[side])]
            .unwrap_or_else(|| {
                if self.motor_enabled[side] {
                    bool::from(self.jog_config.tt_startup_mode)
                } else {
                    self.jog_config.vinyl_enabled_at_startup
                }
            });
    }

    fn end_motor_scratch(&mut self, side: usize, output: &mut Vec<[u8; 3]>) {
        if let Some(deck) = self.platter_scratch[side].take() {
            self.release(KEYS[side], output);
            info!(
                side,
                deck = deck.channel() + 1,
                active = false,
                "MOTOR_SCRATCH_HANDOFF"
            );
        }
    }

    pub(super) fn motorized_bend(
        &mut self,
        report: Option<&[u8]>,
        state: PlaybackState,
        output: &mut Vec<[u8; 3]>,
    ) -> anyhow::Result<()> {
        for (side, offset) in [(0, 8), (1, 36)] {
            if !state.enabled[side] || self.wheels[side].is_some() {
                self.end_motor_scratch(side, output);
                self.platter_bend[side].reset();
                self.platter_counter[side] = None;
                continue;
            }
            let Some(report) = report.filter(|report| report.first() == Some(&3)) else {
                if self.platter_bend[side].rpm(self.jog_now).is_none() {
                    self.end_motor_scratch(side, output);
                }
                continue;
            };
            ensure!(report.len() == 60, "full jog report must have 60 bytes");
            let timer = u32::from_le_bytes(report[offset..offset + 4].try_into()?);
            let counter = u32::from_le_bytes(report[offset + 4..offset + 8].try_into()?);
            let previous = self.platter_counter[side].replace(counter);
            let deck = self.decks[side];
            let target = f64::from(self.jog_config.platter_target_rpm);
            let correction = self.platter_bend[side].observe(
                deck.channel(),
                counter,
                timer,
                self.jog_now,
                self.jog_config.platter_target_rpm,
            )?;
            let Some(rpm) = self.platter_bend[side].rpm(self.jog_now) else {
                self.end_motor_scratch(side, output);
                continue;
            };
            let floor = target * f64::from(self.jog_config.tt_pitch_bend_floor_percent) / 100.0;
            let ceiling = target.mul_add(2.0, -floor);
            let hysteresis = ((target - floor) * 0.25).min(2.0);
            let bend_range = (floor + hysteresis)..=(ceiling - hysteresis);
            if !self.vinyl[side]
                && self.platter_scratch[side].is_none()
                && (rpm <= floor || rpm >= ceiling)
            {
                let address = Address {
                    channel: deck.channel(),
                    number: catalog::SCRATCHING_MODE_NOTE,
                };
                if !self.held.values().flatten().any(|held| *held == address) {
                    output.push(address.press());
                }
                self.held.insert(KEYS[side], vec![address]);
                self.platter_scratch[side] = Some(deck);
                info!(
                    side,
                    deck = deck.channel() + 1,
                    rpm,
                    floor,
                    ceiling,
                    active = true,
                    "MOTOR_SCRATCH_HANDOFF"
                );
            } else if self.platter_scratch[side].is_some()
                && (self.vinyl[side] || bend_range.contains(&rpm))
            {
                self.end_motor_scratch(side, output);
            }
            if let Some(owner) = self.platter_scratch[side] {
                if let Some(previous) = previous {
                    let steps = i32::from_le_bytes(counter.wrapping_sub(previous).to_le_bytes());
                    relative(
                        Address {
                            channel: owner.channel(),
                            number: 5,
                        },
                        steps,
                        output,
                    )?;
                }
            } else if let Some((steps, rpm)) = correction {
                relative(
                    Address {
                        channel: deck.channel(),
                        number: 4,
                    },
                    steps,
                    output,
                )?;
                debug!(
                    side,
                    deck = deck.channel() + 1,
                    rpm,
                    target_rpm = target,
                    steps,
                    "MOTOR_JOG_BEND"
                );
            }
        }
        Ok(())
    }
}
