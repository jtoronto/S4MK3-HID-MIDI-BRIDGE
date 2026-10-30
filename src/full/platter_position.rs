//! Convert the wheel counter into absolute Djay platter-position MIDI.

use anyhow::Result;
use s4mk3_hid_midi_bridge::tension::COUNTS_PER_REVOLUTION;

#[derive(Default)]
pub(super) struct Position {
    active_deck: Option<u8>,
    previous_counter: Option<u32>,
    phase_counts: u32,
    last_value: Option<u8>,
}

impl Position {
    pub(super) const fn counter(&self) -> Option<u32> {
        self.previous_counter
    }

    /// Encoder phase is bounded below 2880 and fits the native ring's u16 field.
    pub(super) const fn ring_position(&self) -> u16 {
        let [low, high, ..] = self.phase_counts.to_le_bytes();
        u16::from_le_bytes([low, high])
    }

    pub(super) fn is_active_for(&self, deck: u8) -> bool {
        self.active_deck == Some(deck)
    }

    pub(super) fn activate(&mut self, counter: u32, deck: u8) -> Result<Option<u8>> {
        self.active_deck = Some(deck);
        self.previous_counter = Some(counter);
        self.phase_counts = counter % COUNTS_PER_REVOLUTION;
        self.last_value = None;
        self.changed_value()
    }

    pub(super) fn observe(&mut self, counter: u32) -> Result<Option<u8>> {
        if let Some(previous) = self.previous_counter.replace(counter) {
            let delta = i32::from_le_bytes(counter.wrapping_sub(previous).to_le_bytes());
            self.phase_counts = u32::try_from(
                (i64::from(self.phase_counts) + i64::from(delta))
                    .rem_euclid(i64::from(COUNTS_PER_REVOLUTION)),
            )?;
        } else {
            self.phase_counts = counter % COUNTS_PER_REVOLUTION;
        }
        self.changed_value()
    }

    pub(super) const fn rebase(&mut self, counter: u32) {
        self.previous_counter = Some(counter);
    }

    pub(super) const fn suspend(&mut self, counter: u32) {
        self.active_deck = None;
        self.previous_counter = Some(counter);
        self.last_value = None;
    }

    fn changed_value(&mut self) -> Result<Option<u8>> {
        let value =
            u8::try_from(u64::from(self.phase_counts) * 128 / u64::from(COUNTS_PER_REVOLUTION))?;
        if self.last_value == Some(value) {
            return Ok(None);
        }
        self.last_value = Some(value);
        Ok(Some(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_position_quantizes_platter_angle_and_suppresses_unchanged_values() -> Result<()> {
        // Given a wheel counter at its absolute zero position.
        let mut position = Position::default();
        assert_eq!(position.activate(0, 1)?, Some(0));

        // When the wheel moves to adjacent MIDI positions and then within one bin.
        assert_eq!(position.observe(23)?, Some(1));
        assert_eq!(position.observe(24)?, None);

        // Then only position changes become absolute MIDI values.
        assert_eq!(position.observe(46)?, Some(2));
        Ok(())
    }

    #[test]
    fn absolute_position_wraps_clockwise_and_counterclockwise() -> Result<()> {
        // Given the wheel one encoder count before its revolution boundary.
        let mut position = Position::default();
        assert_eq!(position.activate(COUNTS_PER_REVOLUTION - 1, 1)?, Some(127));

        // When it crosses clockwise, then counterclockwise, over the boundary.
        assert_eq!(position.observe(COUNTS_PER_REVOLUTION)?, Some(0));
        assert_eq!(position.observe(COUNTS_PER_REVOLUTION - 1)?, Some(127));
        Ok(())
    }

    #[test]
    fn absolute_position_preserves_phase_across_sensor_counter_wrap() -> Result<()> {
        // Given a raw counter approaching its own u32 rollover.
        let mut position = Position::default();
        let before = u32::MAX - 10;
        position.activate(before, 1)?;
        let phase_before = position.phase_counts;

        // When the counter wraps while the wheel advances 21 counts.
        position.observe(10)?;

        // Then modular count arithmetic advances the platter phase by 21.
        assert_eq!(
            position.phase_counts,
            (phase_before + 21) % COUNTS_PER_REVOLUTION
        );
        Ok(())
    }

    #[test]
    fn touch_rebase_discards_manual_motion_from_motorized_position_stream() -> Result<()> {
        // Given a powered absolute-position stream on deck A.
        let mut position = Position::default();
        position.activate(100, 1)?;
        // When touch-owned motion is translated by the normal jog path.
        position.rebase(1000);
        position.observe(1023)?;

        // Then only post-touch movement advances the absolute motor stream.
        assert_eq!(position.phase_counts, 123);
        Ok(())
    }
}
