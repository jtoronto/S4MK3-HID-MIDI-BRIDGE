//! Remove nominal powered rotation, preserving only measured rim deviations.

use num_traits::ToPrimitive as _;
use s4mk3_hid_midi_bridge::{
    jog::{
        decode::FRESHNESS_US,
        platter::{SENSOR_TICKS_PER_US, WINDOW_US},
    },
    tension::COUNTS_PER_REVOLUTION,
};

#[derive(Clone, Copy, Default)]
enum Mode {
    #[default]
    Manual,
    Powered,
    Coast,
}

#[derive(Default)]
pub(super) struct Nudge {
    mode: Mode,
    previous: Option<(u32, u32, u64)>,
    ticks: u64,
    counts: i64,
    travel: u64,
    ready: bool,
    residual: f64,
}

impl Nudge {
    /// None leaves ordinary manual translation intact; Some replaces raw motion.
    pub(super) fn observe(
        &mut self,
        timer: u32,
        counter: u32,
        host_us: u64,
        powered: bool,
        owned: bool,
        rpm: u16,
    ) -> anyhow::Result<Option<i32>> {
        let transition = match self.mode {
            Mode::Manual => powered && !owned,
            Mode::Powered => !powered || owned,
            Mode::Coast => powered || owned,
        };
        if transition {
            self.mode = if owned {
                Mode::Manual
            } else if powered {
                Mode::Powered
            } else {
                Mode::Coast
            };
            self.previous = None;
            self.clear_window();
            self.ready = false;
            self.residual = 0.0;
        }
        let previous = self.previous.replace((timer, counter, host_us));
        if matches!(self.mode, Mode::Manual) {
            return Ok(None);
        }
        let Some((old_timer, old_counter, old_host)) = previous else {
            return Ok(Some(0));
        };
        let ticks = u64::from(timer.wrapping_sub(old_timer));
        let delta = i32::from_le_bytes(counter.wrapping_sub(old_counter).to_le_bytes());
        if host_us.saturating_sub(old_host) >= FRESHNESS_US
            || ticks > FRESHNESS_US * SENSOR_TICKS_PER_US
            || delta.unsigned_abs() > 512
            || (ticks == 0 && delta != 0)
        {
            self.clear_window();
            self.ready = false;
            self.residual = 0.0;
            return Ok(Some(0));
        }
        self.ticks += ticks;
        self.counts += i64::from(delta);
        self.travel += u64::from(delta.unsigned_abs());
        if self.ticks < WINDOW_US * SENSOR_TICKS_PER_US {
            return Ok(Some(0));
        }
        // Two counts cover captured sensor chatter plus integer quantization.
        // Compare over sensor time, not host scheduling latency.
        let nominal = f64::from(rpm)
            * f64::from(COUNTS_PER_REVOLUTION)
            * self.ticks.to_f64().context("nudge sensor interval")?
            / (60_000_000.0 * SENSOR_TICKS_PER_US.to_f64().context("sensor scale")?);
        let deviation = self.counts.to_f64().context("nudge relative counts")? - nominal;
        let result = match self.mode {
            Mode::Powered => {
                if self.ready {
                    let outside_noise = deviation.signum() * (deviation.abs() - 2.0).max(0.0);
                    let accumulated = outside_noise + self.residual;
                    let steps = accumulated.trunc().to_i32().context("nudge MIDI range")?;
                    self.residual = accumulated - f64::from(steps);
                    steps
                } else {
                    // Spinup is not a nudge. Establish nominal movement first.
                    self.ready = deviation.abs() <= 2.0;
                    0
                }
            }
            Mode::Coast => {
                if self.travel <= 2 {
                    self.mode = Mode::Manual;
                }
                0
            }
            Mode::Manual => 0,
        };
        self.clear_window();
        Ok(Some(result))
    }

    const fn clear_window(&mut self) {
        self.ticks = 0;
        self.counts = 0;
        self.travel = 0;
    }
}

use anyhow::Context as _;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sensor_relative_nudges_keep_fractions_across_counter_and_timer_wrap() -> anyhow::Result<()> {
        // Given exact 33 RPM motion, then five windows with three extra counts.
        let mut nudge = Nudge::default();
        let mut deviations = Vec::new();
        for index in 0_u32..=60 {
            let nominal = index * 33 * 2880 * 2 / 60_000;
            let extra = index.saturating_sub(10) / 10 * 3;
            // When host jitter varies but both sensor clocks wrap naturally.
            let result = nudge.observe(
                (u32::MAX - 300_000).wrapping_add(index * 200_000),
                (u32::MAX - 100).wrapping_add(nominal + extra),
                u64::from(index) * 2000 + u64::from(index % 3) * 317,
                true,
                false,
                33,
            )?;
            if index > 10 && index.is_multiple_of(10) {
                deviations.push(result.context("powered override")?);
            }
        }
        // Then nominal fractions are subtracted rather than becoming jog counts,
        // and fractional nudge residue survives between measurement windows.
        assert_eq!(deviations, [1, 1, 0, 2, 1]);
        Ok(())
    }

    #[test]
    fn nominal_rotation_and_two_count_chatter_never_emit_nudges() -> anyhow::Result<()> {
        // Given integer quantization and alternating one-count absolute noise.
        let mut nudge = Nudge::default();
        // When only nominal rotation is observed over many sensor windows.
        for index in 0_u32..=500 {
            let counter = index * 33 * 2880 * 2 / 60_000 + index % 2;
            let result = nudge.observe(
                index * 200_000,
                counter,
                u64::from(index) * 2000,
                true,
                false,
                33,
            )?;
            // Then neither noise nor rounding accumulates into phantom MIDI.
            assert_eq!(result, Some(0));
        }
        Ok(())
    }
}
