//! Relative jog correction after removing nominal powered rotation.

use std::{collections::VecDeque, time::Duration};

use anyhow::Context as _;
use num_traits::ToPrimitive as _;
use s4mk3_hid_midi_bridge::{jog::decode::FRESHNESS_US, tension::COUNTS_PER_REVOLUTION};

pub(super) struct Bend {
    deck: Option<u8>,
    previous: Option<(u32, u32, Duration)>,
    samples: VecDeque<(u64, u32)>,
    clock: u64,
    qualified: bool,
    active: bool,
    residual: f64,
    rpm: Option<f64>,
}

impl Default for Bend {
    fn default() -> Self {
        Self {
            deck: None,
            previous: None,
            samples: VecDeque::with_capacity(128),
            clock: 0,
            qualified: false,
            active: false,
            residual: 0.0,
            rpm: None,
        }
    }
}

impl Bend {
    pub(super) fn reset(&mut self) {
        self.deck = None;
        self.previous = None;
        self.samples.clear();
        self.clock = 0;
        self.qualified = false;
        self.active = false;
        self.residual = 0.0;
        self.rpm = None;
    }

    pub(super) fn rpm(&self, now: Duration) -> Option<f64> {
        let (_, _, seen) = self.previous?;
        (self.qualified && now.saturating_sub(seen) <= Duration::from_micros(FRESHNESS_US))
            .then_some(self.rpm)
            .flatten()
    }

    pub(super) fn observe(
        &mut self,
        deck: u8,
        counter: u32,
        timer: u32,
        now: Duration,
        target_rpm: u16,
    ) -> anyhow::Result<Option<(i32, f64)>> {
        if self.deck != Some(deck) {
            self.reset();
            self.deck = Some(deck);
        }
        let previous = self.previous.replace((counter, timer, now));
        let Some((old_counter, old_timer, old_now)) = previous else {
            self.samples.push_back((self.clock, counter));
            return Ok(None);
        };
        if counter == old_counter && timer == old_timer {
            self.previous = previous;
            return Ok(None);
        }
        let ticks = timer.wrapping_sub(old_timer);
        let counts = i32::from_le_bytes(counter.wrapping_sub(old_counter).to_le_bytes());
        if now <= old_now
            || now - old_now > Duration::from_micros(FRESHNESS_US)
            || ticks == 0
            || u64::from(ticks) > FRESHNESS_US * 100
            || counts.unsigned_abs() > 512
        {
            self.reset();
            self.deck = Some(deck);
            self.previous = Some((counter, timer, now));
            self.samples.push_back((0, counter));
            return Ok(None);
        }
        self.clock += u64::from(ticks);
        self.samples.push_back((self.clock, counter));
        while self
            .samples
            .get(1)
            .is_some_and(|(time, _)| self.clock - time >= 10_000_000)
        {
            self.samples.pop_front();
        }
        let &(first_time, first_counter) = self.samples.front().context("jog speed baseline")?;
        let elapsed = self.clock - first_time;
        // The S4 sensor clock advances 100 ticks per microsecond.
        if elapsed < 10_000_000 {
            return Ok(None);
        }
        let seconds = Duration::from_nanos(elapsed * 10).as_secs_f64();
        let counts_per_rpm = f64::from(COUNTS_PER_REVOLUTION) / 60.0;
        let window_counts = i32::from_le_bytes(counter.wrapping_sub(first_counter).to_le_bytes());
        let rpm = f64::from(window_counts) / (seconds * counts_per_rpm);
        self.rpm = Some(rpm);
        let error = rpm - f64::from(target_rpm);
        if !self.qualified {
            // Startup and post-touch acceleration must first reach nominal speed.
            self.qualified = error.abs() <= 0.75 + 1e-9;
            return Ok(None);
        }
        if error.abs() <= 0.75 + 1e-9 {
            self.active = false;
        } else if error.abs() > 2.0 + 1e-9 {
            self.active = true;
        }
        if !self.active {
            self.residual = 0.0;
            return Ok(None);
        }
        // Integrate only this fresh report's time, not the whole rolling window.
        // This preserves relative travel without batching 100 ms into one burst.
        let gain = if error < 0.0 { 2.0 } else { 1.0 };
        self.residual += error * gain * counts_per_rpm * (f64::from(ticks) / 100_000_000.0);
        let steps = self
            .residual
            .trunc()
            .to_i32()
            .context("motorized jog deviation outside MIDI translation range")?;
        self.residual -= f64::from(steps);
        Ok((steps != 0).then_some((steps, rpm)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn braking_gain_doubles_negative_travel_without_boosting_forward_nudge() -> anyhow::Result<()> {
        // Given identical +/-3 RPM gestures after nominal-speed qualification.
        for (gesture_rpm, expected_steps) in [(300_u32, -288_i32), (360, 144)] {
            let mut bend = Bend::default();
            let mut counter = 0_u32;
            let mut fraction = 0_u32;
            let mut total = 0_i32;
            // When fresh reports cover startup, the gesture and its release.
            for index in 0_u32..1000 {
                let rpm = if (250..850).contains(&index) {
                    gesture_rpm
                } else {
                    330
                };
                fraction += rpm * 2880 * 2000;
                counter += fraction / 600_000_000;
                fraction %= 600_000_000;
                let movement = bend.observe(
                    0,
                    counter,
                    index * 200_000,
                    Duration::from_micros(u64::from(index) * 2000),
                    33,
                )?;
                if !(250..950).contains(&index) {
                    assert!(movement.is_none());
                }
                if (350..850).contains(&index) {
                    total += movement.map_or(0, |(steps, _)| steps);
                }
            }
            // Then one settled second carries 2x braking or unchanged 1x nudging,
            // allowing at most one count of fractional quantization.
            assert!(
                (total - expected_steps).abs() <= 1,
                "{gesture_rpm}: {total}"
            );
        }
        Ok(())
    }

    #[test]
    fn speed_correction_survives_counter_and_sensor_clock_wrap() -> anyhow::Result<()> {
        // Given nominal rotation followed by a 40 RPM wheel near both boundaries.
        let mut bend = Bend::default();
        let mut output = Vec::new();
        // When fresh 10 ms samples cross both boundaries.
        for index in 0_u32..=60 {
            let travel = if index <= 20 {
                index * 1584 / 100
            } else {
                316 + (index - 20) * 192 / 10
            };
            let counter = (u32::MAX - 400).wrapping_add(travel);
            let timer = (u32::MAX - 25_000_000).wrapping_add(index * 1_000_000);
            output.extend(bend.observe(
                0,
                counter,
                timer,
                Duration::from_millis(u64::from(index) * 10),
                33,
            )?);
        }
        // Then only positive deviations, not wrap-sized jumps, are emitted.
        assert!(output.len() >= 30);
        assert!(
            output
                .iter()
                .all(|(steps, rpm)| (1..=4).contains(steps) && *rpm > 35.0)
        );
        Ok(())
    }

    #[test]
    fn startup_and_normal_wobble_are_quiet_with_hysteretic_gesture_release() -> anyhow::Result<()> {
        // Given a wheel ramping, settling, wobbling, braking and returning to target.
        let mut bend = Bend::default();
        let mut counter = 0_u32;
        let mut fraction = 0_u32;
        let mut index = 0_u32;
        let mut output = Vec::new();
        // When fresh 2 ms reports follow each 300 ms phase.
        for (phase, rpm_tenths) in [200_u32, 330, 317, 290, 320, 330].into_iter().enumerate() {
            for sample in 0..150 {
                fraction += rpm_tenths * 2880 * 2000;
                counter += fraction / 600_000_000;
                fraction %= 600_000_000;
                if let Some((steps, _)) = bend.observe(
                    0,
                    counter,
                    index * 200_000,
                    Duration::from_micros(u64::from(index) * 2000),
                    33,
                )? {
                    output.push((phase, sample, steps));
                }
                index += 1;
            }
        }
        // Then startup and baseline stay silent; a brake persists through the
        // hysteresis band and stops once fresh motion returns inside 0.75 RPM.
        assert!(
            output
                .iter()
                .all(|(phase, _, steps)| *phase >= 3 && *steps < 0)
        );
        assert!(output.iter().any(|(phase, _, _)| *phase == 3));
        assert!(
            output
                .iter()
                .any(|(phase, sample, _)| *phase == 4 && *sample > 100)
        );
        assert!(
            !output
                .iter()
                .any(|(phase, sample, _)| *phase == 5 && *sample > 100)
        );
        Ok(())
    }

    #[test]
    fn stale_duplicate_and_deck_change_require_a_new_speed_window() -> anyhow::Result<()> {
        // Given an incomplete speed window.
        let mut bend = Bend::default();
        for index in 0_u32..=8 {
            assert!(
                bend.observe(
                    0,
                    index * 20,
                    index * 1_000_000,
                    Duration::from_millis(u64::from(index) * 10),
                    33,
                )?
                .is_none()
            );
        }
        // When a duplicate cannot refresh freshness, then a new deck takes over.
        assert!(
            bend.observe(0, 160, 8_000_000, Duration::from_millis(120), 33)?
                .is_none()
        );
        assert!(
            bend.observe(0, 180, 9_000_000, Duration::from_millis(140), 33)?
                .is_none()
        );
        assert!(
            bend.observe(2, 200, 10_000_000, Duration::from_millis(150), 33)?
                .is_none()
        );
        // Then neither the stale window nor the old deck's counts produce a bend.
        assert!(
            bend.observe(2, 220, 11_000_000, Duration::from_millis(160), 33)?
                .is_none()
        );
        Ok(())
    }
}
