//! Track physical encoder phase for the jog ring indicator.

use anyhow::Result;
use s4mk3_hid_midi_bridge::tension::COUNTS_PER_REVOLUTION;

#[derive(Default)]
pub(super) struct RingPosition {
    previous_counter: Option<u32>,
    phase_counts: u32,
}

impl RingPosition {
    pub(super) const fn counter(&self) -> Option<u32> {
        self.previous_counter
    }

    /// Encoder phase is bounded below 2880 and fits the native ring's u16 field.
    pub(super) const fn ring_position(&self) -> u16 {
        let [low, high, ..] = self.phase_counts.to_le_bytes();
        u16::from_le_bytes([low, high])
    }

    pub(super) fn observe(&mut self, counter: u32) -> Result<()> {
        if let Some(previous) = self.previous_counter.replace(counter) {
            let delta = i32::from_le_bytes(counter.wrapping_sub(previous).to_le_bytes());
            self.phase_counts = u32::try_from(
                (i64::from(self.phase_counts) + i64::from(delta))
                    .rem_euclid(i64::from(COUNTS_PER_REVOLUTION)),
            )?;
        } else {
            self.phase_counts = counter % COUNTS_PER_REVOLUTION;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_phase_preserves_rotation_across_sensor_counter_wrap() -> Result<()> {
        // Given a raw counter approaching its own u32 rollover.
        let mut position = RingPosition::default();
        let before = u32::MAX - 10;
        position.observe(before)?;
        let phase_before = position.phase_counts;

        // When the counter wraps while the wheel advances 21 counts.
        position.observe(10)?;

        // Then modular count arithmetic advances the ring phase by 21.
        assert_eq!(
            position.phase_counts,
            (phase_before + 21) % COUNTS_PER_REVOLUTION
        );
        Ok(())
    }
}
