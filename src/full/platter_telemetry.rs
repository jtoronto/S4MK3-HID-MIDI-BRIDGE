//! Bounded physical-speed and successfully sent MIDI diagnostics, not audio feedback.

use std::time::Duration;

use serde::Serialize;

use super::{Translator, jog, platter_bend};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Mode {
    Off,
    Waiting,
    Bend,
    Scratch,
    PausedScratch,
    Blocked,
    Stale,
    Seek,
    Grid,
}

#[derive(Debug, Serialize)]
pub(super) struct Wheel {
    deck: u8,
    rpm: Option<f64>,
    bend_counts_per_second: f64,
    mode: Mode,
}

#[derive(Debug, Serialize)]
pub(super) struct Frame {
    host_us: u128,
    wheels: [Wheel; 2],
}

pub(super) struct Telemetry {
    speed: [platter_bend::Bend; 2],
    bend_steps: [i32; 2],
    published: Option<Duration>,
}

impl Default for Telemetry {
    fn default() -> Self {
        Self {
            speed: std::array::from_fn(|_| platter_bend::Bend::default()),
            bend_steps: [0; 2],
            published: None,
        }
    }
}

impl Telemetry {
    pub(super) fn observe(
        &mut self,
        report: &[u8],
        now: Duration,
        target_rpm: u16,
    ) -> anyhow::Result<()> {
        if report.first() != Some(&3) {
            return Ok(());
        }
        anyhow::ensure!(report.len() == 60, "full jog report must have 60 bytes");
        for (side, offset) in [(0, 8), (1, 36)] {
            // Reuse the sensor-clock window independently of bend authorization,
            // so paused, startup and touched wheels still have measured RPM.
            let _correction = self.speed[side].observe(
                u8::try_from(side)?,
                u32::from_le_bytes(report[offset + 4..offset + 8].try_into()?),
                u32::from_le_bytes(report[offset..offset + 4].try_into()?),
                now,
                &jog::Config {
                    platter_target_rpm: target_rpm,
                    ..jog::DEFAULT
                },
            )?;
        }
        Ok(())
    }

    pub(super) fn sent(&mut self, messages: &[[u8; 3]]) {
        for &[status, number, value] in messages {
            if (0xb0..=0xb3).contains(&status) && number == 4 {
                let side = usize::from(status & 1);
                self.bend_steps[side] += i32::from(value) - 64;
            }
        }
    }

    pub(super) fn frame(&mut self, state: &Translator, now: Duration) -> Option<Frame> {
        let elapsed = self.published.map(|previous| now.saturating_sub(previous));
        if elapsed.is_some_and(|elapsed| elapsed < Duration::from_millis(100)) {
            return None;
        }
        let wheels = std::array::from_fn(|side| {
            let owner = state.wheels[side];
            let deck = owner
                .map(|owner| owner.deck)
                .or(state.platter_scratch[side])
                .unwrap_or(state.decks[side]);
            let mode = if !state.motor_enabled[side] {
                Mode::Off
            } else if !self.speed[side].fresh(now) {
                Mode::Stale
            } else if state.grid[side] {
                Mode::Grid
            } else if state.shifted[side] {
                Mode::Seek
            } else if state.paused_tt(side) {
                Mode::PausedScratch
            } else if owner.is_some_and(|owner| owner.action == "jog_scratch")
                || state.platter_scratch[side].is_some()
            {
                Mode::Scratch
            } else if !state.playback_state().enabled[side] {
                Mode::Blocked
            } else if state.platter_bend[side].rpm(now).is_none() {
                Mode::Waiting
            } else {
                Mode::Bend
            };
            Wheel {
                deck: deck.channel() + 1,
                rpm: self.speed[side].measured_rpm(now),
                bend_counts_per_second: elapsed.map_or(0.0, |elapsed| {
                    f64::from(self.bend_steps[side]) / elapsed.as_secs_f64()
                }),
                mode,
            }
        });
        self.bend_steps = [0; 2];
        self.published = Some(now);
        Some(Frame {
            host_us: now.as_micros(),
            wheels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::full::{Address, relative};

    #[test]
    fn telemetry_counts_only_sent_bend_and_publishes_at_ten_hz() -> anyhow::Result<()> {
        let mut telemetry = Telemetry::default();
        let state = Translator::default();
        assert!(telemetry.frame(&state, Duration::ZERO).is_some());
        let mut messages = Vec::new();
        relative(
            Address {
                channel: 2,
                number: 4,
            },
            -128,
            &mut messages,
        )?;
        relative(
            Address {
                channel: 1,
                number: 4,
            },
            32,
            &mut messages,
        )?;
        relative(
            Address {
                channel: 0,
                number: 5,
            },
            -40,
            &mut messages,
        )?;
        messages.push([0x90, 4, 127]);
        telemetry.sent(&messages);
        assert!(telemetry.frame(&state, Duration::from_millis(99)).is_none());

        let frame = telemetry
            .frame(&state, Duration::from_millis(100))
            .ok_or_else(|| anyhow::anyhow!("diagnostic frame missing"))?;
        assert_eq!(frame.host_us, 100_000);
        assert!((frame.wheels[0].bend_counts_per_second + 1280.0).abs() < 1e-9);
        assert!((frame.wheels[1].bend_counts_per_second - 320.0).abs() < 1e-9);
        let next = telemetry
            .frame(&state, Duration::from_millis(200))
            .ok_or_else(|| anyhow::anyhow!("next diagnostic frame missing"))?;
        assert!(next.wheels[0].bend_counts_per_second.abs() < f64::EPSILON);
        Ok(())
    }

    #[test]
    fn physical_rpm_is_visible_while_paused_and_clears_when_stale() -> anyhow::Result<()> {
        let mut telemetry = Telemetry::default();
        let mut state = Translator::default();
        state.motor_enabled[0] = true;
        state.playback_paused[0] = true;
        for index in 0_u32..=10 {
            let mut report = [0_u8; 60];
            report[0] = 3;
            for (offset, counter) in [(8, index * 8), (36, 0)] {
                report[offset..offset + 4].copy_from_slice(&(index * 1_000_000).to_le_bytes());
                report[offset + 4..offset + 8].copy_from_slice(&counter.to_le_bytes());
            }
            telemetry.observe(&report, Duration::from_millis(u64::from(index) * 10), 33)?;
        }
        let frame = telemetry
            .frame(&state, Duration::from_millis(100))
            .ok_or_else(|| anyhow::anyhow!("paused diagnostic frame missing"))?;
        assert_eq!(frame.wheels[0].rpm, Some(80.0 / (0.1 * 48.0)));
        assert_eq!(frame.wheels[1].rpm, Some(0.0));
        assert_eq!(frame.wheels[0].mode, Mode::PausedScratch);
        assert!(!state.playback_state().enabled[0]);

        let expired = telemetry
            .frame(&state, Duration::from_millis(200))
            .ok_or_else(|| anyhow::anyhow!("stale diagnostic frame missing"))?;
        assert_eq!(expired.wheels[0].rpm, None);
        assert_eq!(expired.wheels[0].mode, Mode::Stale);
        Ok(())
    }
}
