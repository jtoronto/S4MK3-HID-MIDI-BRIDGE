//! Relative jog correction after removing nominal powered rotation.

use std::{collections::VecDeque, time::Duration};

use anyhow::Context as _;
use num_traits::ToPrimitive as _;
use s4mk3_hid_midi_bridge::{jog::decode::FRESHNESS_US, tension::COUNTS_PER_REVOLUTION};

use super::jog;

pub(super) struct Bend {
    deck: Option<u8>,
    previous: Option<(u32, u32, Duration)>,
    samples: VecDeque<(u64, u32)>,
    clock: u64,
    qualified: bool,
    residual: f64,
    rpm: Option<f64>,
    delta: Option<i32>,
    abrupt_stop_ticks: Option<u64>,
}

impl Default for Bend {
    fn default() -> Self {
        Self {
            deck: None,
            previous: None,
            samples: VecDeque::with_capacity(128),
            clock: 0,
            qualified: false,
            residual: 0.0,
            rpm: None,
            delta: None,
            abrupt_stop_ticks: None,
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
        self.residual = 0.0;
        self.rpm = None;
        self.delta = None;
        self.abrupt_stop_ticks = None;
    }

    pub(super) fn fresh(&self, now: Duration) -> bool {
        self.previous.is_some_and(|(_, _, seen)| {
            now.saturating_sub(seen) <= Duration::from_micros(FRESHNESS_US)
        })
    }

    pub(super) fn measured_rpm(&self, now: Duration) -> Option<f64> {
        self.fresh(now).then_some(self.rpm).flatten()
    }

    pub(super) const fn movement(&self) -> Option<i32> {
        self.delta
    }

    pub(super) fn abruptly_stopped(&self) -> bool {
        self.qualified && self.abrupt_stop_ticks.is_some_and(|ticks| ticks >= 400_000)
    }

    pub(super) fn rpm(&self, now: Duration) -> Option<f64> {
        self.qualified.then(|| self.measured_rpm(now)).flatten()
    }

    pub(super) fn observe(
        &mut self,
        deck: u8,
        counter: u32,
        timer: u32,
        now: Duration,
        preferences: &jog::Config,
    ) -> anyhow::Result<Option<(i32, f64)>> {
        self.delta = None;
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
        self.delta = Some(counts);
        let target = f64::from(preferences.platter_target_rpm);
        // Confirm 4ms of fresh stationary sensor time after normal-speed
        // rotation. Duplicates cannot advance this, and gradual low-speed
        // count quantization must not become an abrupt-stop detector.
        self.abrupt_stop_ticks = if counts != 0 {
            None
        } else if let Some(stopped) = self.abrupt_stop_ticks {
            Some(stopped + u64::from(ticks))
        } else {
            self.rpm
                .filter(|rpm| (*rpm - target).abs() <= 2.25 + 1e-9)
                .map(|_| u64::from(ticks))
        };
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
        let error = rpm - target;
        if !self.qualified {
            self.qualified = error.abs() <= 2.25 + 1e-9;
            return Ok(None);
        }
        if error.abs() <= 2.25 + 1e-9 {
            self.residual = 0.0;
            return Ok(None);
        }
        // Smooth onset outside the true quiet band, reaching the original law
        // one RPM later without amplifying saved high-gain preferences.
        let onset = (error.abs() - 2.25).clamp(0.0, 1.0);
        let weight = onset * onset * 2.0_f64.mul_add(-onset, 3.0);
        if self.residual * error < 0.0 {
            self.residual = 0.0;
        }
        // Integrate only this fresh report's time, not the whole rolling window.
        // This preserves relative travel without batching 100 ms into one burst.
        let (gain, exponent) = if error < 0.0 {
            (
                preferences
                    .tt_brake_gain
                    .unwrap_or(2.0 * preferences.tt_pitch_bend_gain),
                preferences.tt_brake_curve_exponent,
            )
        } else {
            (
                preferences
                    .tt_nudge_gain
                    .unwrap_or(preferences.tt_pitch_bend_gain),
                preferences.tt_nudge_curve_exponent,
            )
        };
        // Shape only the configured target-speed span; retain linear continuation
        // beyond it instead of raising extreme high-speed inputs to a power.
        let shape = (error.abs() / target).min(1.0).powf(exponent - 1.0);
        self.residual +=
            error * weight * gain * shape * counts_per_rpm * (f64::from(ticks) / 100_000_000.0);
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
    fn quiet_rotation_clears_bend_after_braking_and_nudging_at_high_gain() -> anyhow::Result<()> {
        for gain in [1.0, 5.0, 10.0] {
            for gesture in [270_u32, 400] {
                let mut bend = Bend::default();
                let mut counter = 0_u32;
                let mut fraction = 0_u32;
                let mut deliberate = 0;
                for index in 0_u32..1000 {
                    let rpm = match index {
                        0..150 => 330,
                        150..300 => gesture,
                        300..600 => 350,
                        _ => {
                            if (index / 50) % 2 == 0 {
                                320
                            } else {
                                350
                            }
                        }
                    };
                    fraction += rpm * 2880 * 2000;
                    counter += fraction / 600_000_000;
                    fraction %= 600_000_000;
                    let output = bend.observe(
                        0,
                        counter,
                        index * 200_000,
                        Duration::from_micros(u64::from(index) * 2000),
                        &jog::Config {
                            tt_pitch_bend_gain: gain,
                            ..jog::DEFAULT
                        },
                    )?;
                    if (200..300).contains(&index) {
                        deliberate += output.map_or(0, |(steps, _)| steps.abs());
                    }
                    if index >= 351 {
                        assert!(
                            output.is_none(),
                            "gain {gain}, gesture {gesture}, sample {index}: {output:?}"
                        );
                    }
                }
                assert!(deliberate > 0, "the fixture must first activate a bend");
            }
        }
        Ok(())
    }

    #[test]
    fn soft_knee_preserves_deep_braking_and_separates_directional_response() -> anyhow::Result<()> {
        for (gesture, expected) in [(225_u32, -75), (275, 37), (200, -960), (300, 480)] {
            let mut bend = Bend::default();
            let mut counter = 0_u32;
            let mut fraction = 0_u32;
            let mut total = 0;
            for index in 0_u32..1400 {
                let rpm = if index < 150 { 250 } else { gesture };
                fraction += rpm * 2880 * 2000;
                counter += fraction / 600_000_000;
                fraction %= 600_000_000;
                let output = bend.observe(
                    0,
                    counter,
                    index * 200_000,
                    Duration::from_micros(u64::from(index) * 2000),
                    &jog::Config {
                        platter_target_rpm: 25,
                        ..jog::DEFAULT
                    },
                )?;
                if index >= 400 {
                    total += output.map_or(0, |(steps, _)| steps);
                }
            }
            assert!(
                (total - expected).abs() <= 1,
                "{gesture}: {total}, expected {expected}"
            );
        }
        Ok(())
    }

    #[test]
    fn quiet_band_startup_qualifies_following_braking() -> anyhow::Result<()> {
        for baseline in [320_u32, 340] {
            let mut bend = Bend::default();
            let mut counter = 0_u32;
            let mut fraction = 0_u32;
            let mut braking = Vec::new();
            for index in 0_u32..300 {
                let rpm = if index < 150 { baseline } else { 270 };
                fraction += rpm * 2880 * 2000;
                counter += fraction / 600_000_000;
                fraction %= 600_000_000;
                let correction = bend.observe(
                    0,
                    counter,
                    index * 200_000,
                    Duration::from_micros(u64::from(index) * 2000),
                    &jog::DEFAULT,
                )?;
                if index < 150 {
                    assert!(correction.is_none(), "quiet baseline must not bend");
                } else {
                    braking.extend(correction);
                }
            }
            assert!(!braking.is_empty(), "{baseline}: braking never qualified");
            assert!(braking.iter().all(|(steps, _)| *steps < 0));
        }
        Ok(())
    }

    #[test]
    fn braking_gain_doubles_negative_travel_without_boosting_forward_nudge() -> anyhow::Result<()> {
        // Given identical +/-4 RPM gestures beyond the smooth onset.
        for (gesture_rpm, expected_steps) in [(290_u32, -384_i32), (370, 192)] {
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
                    &jog::DEFAULT,
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
                &jog::DEFAULT,
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
    fn startup_and_normal_wobble_are_quiet_after_gesture_release() -> anyhow::Result<()> {
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
                    &jog::DEFAULT,
                )? {
                    output.push((phase, sample, steps));
                }
                index += 1;
            }
        }
        // Then startup and baseline stay silent; returning to the quiet region
        // clears the brake even when the wheel settles one RPM below target.
        assert!(
            output
                .iter()
                .all(|(phase, _, steps)| *phase >= 3 && *steps < 0)
        );
        assert!(output.iter().any(|(phase, _, _)| *phase == 3));
        assert!(
            !output
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
                    &jog::DEFAULT,
                )?
                .is_none()
            );
        }
        // When a duplicate cannot refresh freshness, then a new deck takes over.
        assert!(
            bend.observe(0, 160, 8_000_000, Duration::from_millis(120), &jog::DEFAULT)?
                .is_none()
        );
        assert!(
            bend.observe(0, 180, 9_000_000, Duration::from_millis(140), &jog::DEFAULT)?
                .is_none()
        );
        assert!(
            bend.observe(
                2,
                200,
                10_000_000,
                Duration::from_millis(150),
                &jog::DEFAULT
            )?
            .is_none()
        );
        // Then neither the stale window nor the old deck's counts produce a bend.
        assert!(
            bend.observe(
                2,
                220,
                11_000_000,
                Duration::from_millis(160),
                &jog::DEFAULT
            )?
            .is_none()
        );
        Ok(())
    }
}
