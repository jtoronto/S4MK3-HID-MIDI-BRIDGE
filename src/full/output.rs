//! Paced native HID lighting, without motor or screen reports.

use std::time::Duration;

use anyhow::{Context as _, ensure};
use hidapi::HidDevice;
use tracing::{debug, info, warn};

use super::feedback::FeedbackState;
use super::leds::{LedConfig, LocalState, render_buttons};

const BUTTON_INTERVAL: Duration = Duration::from_millis(10);
const RING_INTERVAL: Duration = Duration::from_millis(20);
const METER_INTERVAL: Duration = Duration::from_millis(50);

#[path = "ring_phase.rs"]
mod ring_phase;

#[cfg(test)]
#[path = "ring_tests.rs"]
mod ring_tests;

pub struct LedOutput<'a> {
    hid: &'a HidDevice,
    buttons: Option<[u8; 95]>,
    rings: [Option<[u8; 41]>; 2],
    meters: Option<[u8; 79]>,
    button_written: Option<Duration>,
    ring_written: Option<Duration>,
    meter_written: Option<Duration>,
    ring_phase: ring_phase::Rings,
}

impl<'a> LedOutput<'a> {
    pub fn new(hid: &'a HidDevice) -> anyhow::Result<Self> {
        // Traktor capture: 28 bytes including ID; separate motor report is excluded.
        let mut init = [0_u8; 28];
        init[0] = 0x30;
        init[2] = 1;
        init[3] = 3;
        // Mixxx's third sendOutputReport argument is FIFO ordering, not report type.
        for side in 0_u8..2 {
            init[1] = side;
            let written = hid.write(&init).context("initialize S4 ring output")?;
            ensure!(
                written == init.len(),
                "short S4 ring initialization output report"
            );
        }
        info!("LED_FEEDBACK_ACTIVE: software state starts unknown; deck/routing lights are local");
        info!("END_WARNING_UNAVAILABLE: no verified Djay remaining-time/end-warning output");
        Ok(Self {
            hid,
            buttons: None,
            rings: [None; 2],
            meters: None,
            button_written: None,
            ring_written: None,
            meter_written: None,
            ring_phase: ring_phase::Rings::default(),
        })
    }

    pub fn tick(
        &mut self,
        config: &LedConfig,
        feedback: &FeedbackState,
        local: &LocalState,
        now: Duration,
    ) -> anyhow::Result<usize> {
        let mut writes = 0;
        // Update phase on every service tick, independently of HID write pacing.
        let mut presented = *local;
        presented.ring_positions = self.ring_phase.positions(config, feedback, local, now)?;
        if due(self.button_written, now, BUTTON_INTERVAL) {
            let report = render_button_frame(config, feedback, local, now);
            if self.buttons != Some(report) {
                self.write(&report)?;
                self.buttons = Some(report);
                self.button_written = Some(now);
                writes += 1;
            }
        }
        if due(self.ring_written, now, RING_INTERVAL) {
            for side in 0_u8..2 {
                let report = render_ring(config, feedback, &presented, side, now);
                let index = usize::from(side);
                if self.rings[index] != Some(report) {
                    self.write(&report)?;
                    debug!(side, mode = report[2], color = report[5], "RING_LED_WRITE");
                    self.rings[index] = Some(report);
                    self.ring_written = Some(now);
                    writes += 1;
                }
            }
        }
        // Until Djay supplies a level, do not write the meter report at all.
        if (self.meters.is_some() || feedback.meters.iter().any(Option::is_some))
            && due(self.meter_written, now, METER_INTERVAL)
        {
            let report = render_meters(config, feedback, now);
            if self.meters != Some(report) {
                self.write(&report)?;
                self.meters = Some(report);
                self.meter_written = Some(now);
                writes += 1;
            }
        }
        Ok(writes)
    }

    fn write(&self, report: &[u8]) -> anyhow::Result<()> {
        let written = self.hid.write(report).context("write S4 LED output")?;
        ensure!(written == report.len(), "short S4 LED output write");
        debug!(id = report[0], written, "LED_WRITE");
        Ok(())
    }

    fn clear(&self) -> anyhow::Result<()> {
        let mut buttons = [0; 95];
        buttons[0] = 0x80;
        self.write(&buttons)?;
        for side in 0_u8..2 {
            let mut report = [0; 41];
            report[0] = 0x32;
            report[1] = side;
            self.write(&report)?;
        }
        if self.meters.is_some() {
            let mut meters = [0; 79];
            meters[0] = 0x81;
            self.write(&meters)?;
        }
        Ok(())
    }
}

impl Drop for LedOutput<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.clear() {
            warn!(%error, "could not clear S4 lights during shutdown");
        }
    }
}

fn due(last: Option<Duration>, now: Duration, interval: Duration) -> bool {
    last.is_none_or(|last| now.saturating_sub(last) >= interval)
}

fn render_button_frame(
    config: &LedConfig,
    feedback: &FeedbackState,
    local: &LocalState,
    now: Duration,
) -> [u8; 95] {
    let mut report = render_buttons(config, feedback, local);
    for side in 0..2 {
        for (knob, mismatch) in super::leds::fx_pickup(feedback, local, side)
            .into_iter()
            .enumerate()
        {
            if mismatch == Some(true) {
                let button = if knob == 3 { 0 } else { knob + 1 };
                report[[63, 74][side] + button] = if (now.as_millis() / 300).is_multiple_of(2) {
                    config.deck_colors[local.fx[side]].base() + config.palette_active_intensity
                } else {
                    0
                };
            }
        }
    }
    if local.eq_selecting {
        report[55] = if (now.as_millis() / 300).is_multiple_of(2) {
            super::leds::Color::Cyan.base() + config.palette_active_intensity
        } else {
            0
        };
    }
    // A 1.2-second palette pulse, including a fully dark trough, remains
    // distinguishable even when inactive-display preferences are dark.
    let intensity = match (now.as_millis() / 200) % 6 {
        0 => 3,
        1 | 5 => 2,
        2 | 4 => 1,
        _ => 0,
    };
    for side in 0..2 {
        let deck = local.decks[side];
        if local.move_selecting[deck] {
            let slot = deck / 2;
            report[[[13, 14], [36, 37]][side][slot]] = if intensity == 0 {
                0
            } else {
                config.deck_colors[deck].base() + config.palette_active_intensity * intensity / 3
            };
        }
    }
    report
}

fn render_ring(
    config: &LedConfig,
    feedback: &FeedbackState,
    local: &LocalState,
    side: u8,
    _now: Duration,
) -> [u8; 41] {
    let deck = local.decks[usize::from(side)];
    let looping = config.loop_enabled && feedback.notes[deck][6].is_some_and(|value| value != 0);
    let color = if looping {
        config.loop_color
    } else {
        config.deck_colors[deck]
    };
    let mut report = [0; 41];
    report[0] = 0x32;
    report[1] = side; // Physical side, not logical A/B/C/D.
    // Firmware flashes mode 3 until loop exit; mode 2 is a stationary spot.
    report[2] = if looping { 3 } else { 2 };
    if !looping {
        report[3..5].copy_from_slice(&local.ring_positions[usize::from(side)].to_le_bytes());
    }
    report[5] = color.base().saturating_add(config.palette_active_intensity);
    report
}

fn render_meters(config: &LedConfig, feedback: &FeedbackState, now: Duration) -> [u8; 79] {
    let mut report = [0; 79];
    report[0] = 0x81;
    for deck in 0..4 {
        let fresh = feedback.meter_updated[deck].is_some_and(|updated| {
            now.saturating_sub(updated) <= Duration::from_millis(config.meter_stale_ms)
        });
        if !fresh {
            continue;
        }
        let Some(value) = feedback.meters[deck] else {
            continue;
        };
        let scaled = (f32::from(value) / 127.0).powf(config.meter_gamma) * 14.0;
        for segment in 0_u8..14 {
            let intensity = if scaled >= f32::from(segment.saturating_add(1)) {
                config.meter_brightness
            } else if scaled > f32::from(segment) {
                config.meter_brightness.min(125)
            } else {
                0
            };
            report[1 + deck * 15 + usize::from(segment)] = intensity;
        }
        // Clip output is unknown: no verified Djay clip binding, so do not invent it.
    }
    // Master meters are hardware-driven; the tail must be checked physically.
    report
}

#[cfg(test)]
mod tests {
    use super::super::leds::Color;
    use super::*;

    #[test]
    fn fx_mismatch_blinks_only_owned_buttons_and_restores_normal_feedback() {
        // Given both strips targeting C, mismatched parameter1 and dry/wet.
        let config = LedConfig::default();
        let mut feedback = FeedbackState::default();
        feedback.fx_values[2] = [Some(100); 6];
        feedback.notes[2][32..36].fill(Some(0));
        let mut local = LocalState {
            fx: [2; 2],
            fx_positions: [[Some(100); 4]; 2],
            ..LocalState::default()
        };
        local.fx_positions[0][0] = Some(0);
        local.fx_positions[1][3] = Some(0);
        let steady = render_buttons(&config, &feedback, &local);
        // When exact bright and dark phases render without a wall clock.
        for (millis, bright) in [(0, true), (299, true), (300, false), (600, true)] {
            let mut expected = steady;
            let value = if bright {
                config.deck_colors[2].base() + config.palette_active_intensity
            } else {
                0
            };
            expected[64] = value;
            expected[74] = value;
            assert_eq!(
                render_button_frame(&config, &feedback, &local, Duration::from_millis(millis)),
                expected
            );
        }
        // Then picking up the values restores the real disabled-effect lamps.
        local.fx_positions = [[Some(100); 4]; 2];
        assert_eq!(
            render_button_frame(&config, &feedback, &local, Duration::from_millis(300)),
            steady
        );
    }

    #[test]
    fn eq_selection_flashes_cyan_until_selection_or_cancel() {
        // Given a pending selection, even with other inactive lamps set dark.
        let config = LedConfig {
            inactive_display: super::super::leds::InactiveDisplay::Dark,
            ..LedConfig::default()
        };
        let feedback = FeedbackState::default();
        let mut local = LocalState {
            eq_selecting: true,
            ..LocalState::default()
        };
        // When rendering exact points in the 600 ms flash cycle.
        for millis in [0, 300, 600, 120_300] {
            let report =
                render_button_frame(&config, &feedback, &local, Duration::from_millis(millis));
            // Then cyan alternates with dark without wall-clock sleeps.
            assert_eq!(
                report[55],
                if (millis / 300) % 2 == 0 {
                    Color::Cyan.base() + config.palette_active_intensity
                } else {
                    0
                }
            );
        }
        local.eq_selecting = false;
        local.held[10] = true;
        assert_eq!(
            render_button_frame(&config, &feedback, &local, Duration::ZERO)[55],
            Color::Cyan.base() + config.palette_inactive_intensity
        );
    }

    #[test]
    fn jump_selection_pulses_only_the_visible_selected_deck_without_expiring() {
        // Given A and D selecting jump size, with D visible on the right.
        let config = LedConfig::default();
        let feedback = FeedbackState::default();
        let mut local = LocalState {
            decks: [0, 3],
            move_selecting: [true, false, false, true],
            ..LocalState::default()
        };
        let steady = render_buttons(&config, &feedback, &local);
        // When the pulse reaches its trough at 600 ms, and much later.
        for now in [Duration::from_millis(600), Duration::from_millis(120_600)] {
            let report = render_button_frame(&config, &feedback, &local, now);
            let mut expected = steady;
            expected[13] = 0;
            expected[37] = 0;
            assert_eq!(report, expected);
        }
        assert_eq!(
            render_button_frame(&config, &feedback, &local, Duration::ZERO),
            steady
        );
        local.decks = [2, 1];
        assert_eq!(
            render_button_frame(&config, &feedback, &local, Duration::from_millis(600)),
            render_buttons(&config, &feedback, &local)
        );
        local.move_selecting.fill(false);
        local.decks = [0, 3];
        assert_eq!(
            render_button_frame(&config, &feedback, &local, Duration::from_millis(600)),
            steady
        );
    }

    #[test]
    fn ring_uses_physical_side_and_selected_deck_color_without_motors() {
        // Given C selected on the left while FX targets A.
        let mut local = LocalState::default();
        local.decks[0] = 2;
        // When the ring is rendered.
        let report = render_ring(
            &LedConfig::default(),
            &FeedbackState::default(),
            &local,
            0,
            Duration::ZERO,
        );
        // Then only the left LED output report contains C's color.
        assert_eq!(
            report[..9],
            [0x32, 0, 2, 0, 0, Color::Yellow.base() + 2, 0, 0, 0]
        );
        assert!(report[9..].iter().all(|value| *value == 0));
    }

    #[test]
    fn loop_flash_stays_green_until_loop_exit() {
        // Given B looping on the right.
        let config = LedConfig::default();
        let mut feedback = FeedbackState::default();
        feedback.notes[1][6] = Some(127);
        let local = LocalState::default();
        // When the loop state is rendered.
        let on = render_ring(&config, &feedback, &local, 1, Duration::from_millis(500));
        // Then firmware owns flashing, with no clock-dependent packet changes.
        assert_eq!(on[5], Color::Green.base() + 2);
        assert_eq!(on[2], 3);
        feedback.notes[1][6] = Some(0);
        let exited = render_ring(&config, &feedback, &local, 1, Duration::from_millis(1000));
        assert_eq!(exited[2], 2);
        assert_eq!(exited[5], Color::Blue.base() + 2);
    }

    #[test]
    fn disabled_loop_indication_restores_deck_color() {
        // Given known looping but a disabled indication.
        let config = LedConfig {
            loop_enabled: false,
            ..LedConfig::default()
        };
        let mut feedback = FeedbackState::default();
        feedback.notes[0][6] = Some(127);
        // When the green phase would otherwise occur.
        let report = render_ring(
            &config,
            &feedback,
            &LocalState::default(),
            0,
            Duration::ZERO,
        );
        // Then the configured deck color remains.
        assert_eq!(report[5], Color::Red.base() + 2);
        assert_eq!(report[2], 2);
    }

    #[test]
    fn stationary_ring_does_not_follow_playback_clock() {
        // Given an unchanged wheel in either normal or TT mode.
        for tt in [false, true] {
            let config = LedConfig::default();
            let mut feedback = FeedbackState::default();
            feedback.playing[0] = Some(true);
            let mut local = LocalState::default();
            local.motor_enabled[0] = tt;
            // When only wall-clock time changes while the deck keeps playing.
            let first = render_ring(&config, &feedback, &local, 0, Duration::from_millis(100));
            let later = render_ring(&config, &feedback, &local, 0, Duration::from_millis(400));
            // Then no ring movement is invented.
            assert_eq!(first, later);
        }
    }

    #[test]
    fn pausing_does_not_reposition_stationary_ring() {
        // Given a stationary physical wheel while software playback is active.
        let config = LedConfig::default();
        let mut feedback = FeedbackState::default();
        feedback.playing[0] = Some(true);
        let local = LocalState::default();
        let now = Duration::from_millis(500);
        let playing = render_ring(&config, &feedback, &local, 0, now);
        // When the software deck pauses without any wheel movement.
        feedback.playing[0] = Some(false);
        let paused = render_ring(&config, &feedback, &local, 0, now);
        // Then the ring keeps its position instead of returning to noon.
        assert_eq!(playing, paused);
    }

    #[test]
    fn ring_uses_physical_phase_independently_of_selected_deck_playback() {
        let config = LedConfig::default();
        let mut feedback = FeedbackState::default();
        let mut local = LocalState::default();
        local.decks[0] = 2;
        local.ring_positions = [960, 1800];
        feedback.playing[2] = Some(true);
        let quarter = render_ring(&config, &feedback, &local, 0, Duration::from_millis(500));
        let half = render_ring(&config, &feedback, &local, 0, Duration::from_millis(1000));
        assert_eq!(u16::from_le_bytes([quarter[3], quarter[4]]), 960);
        assert_eq!(quarter, half);
        assert_eq!(half[5], Color::Yellow.base() + 2);
        feedback.playing[2] = Some(false);
        for now in [Duration::from_millis(500), Duration::from_millis(1000)] {
            let stopped = render_ring(&config, &feedback, &local, 0, now);
            assert_eq!(stopped, half);
        }
        let right = render_ring(&config, &feedback, &local, 1, Duration::from_millis(500));
        assert_eq!(u16::from_le_bytes([right[3], right[4]]), 1800);
    }

    #[test]
    fn meters_render_fixed_channels_and_leave_master_and_clip_bytes_unassigned() {
        // Given a full C meter independent of selected A/B decks.
        let mut feedback = FeedbackState::default();
        feedback.meters[2] = Some(127);
        feedback.meter_updated[2] = Some(Duration::ZERO);
        // When the native report is rendered.
        let report = render_meters(&LedConfig::default(), &feedback, Duration::ZERO);
        // Then only C's 14 level bytes light, not clips or master bytes.
        assert!(report[31..45].iter().all(|byte| *byte == 127));
        assert_eq!(report[45], 0);
        assert!(report[61..].iter().all(|byte| *byte == 0));
        assert!(report[1..31].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn stale_meters_go_dark_without_expiring_latched_button_state() {
        // Given a meter update and a latched Play state.
        let mut feedback = FeedbackState::default();
        feedback.meters[0] = Some(127);
        feedback.meter_updated[0] = Some(Duration::ZERO);
        feedback.notes[0][0] = Some(127);
        let config = LedConfig::default();
        // When the explicit freshness deadline has passed.
        let now = Duration::from_millis(config.meter_stale_ms.saturating_add(1));
        let meter = render_meters(&config, &feedback, now);
        let buttons = render_buttons(&config, &feedback, &LocalState::default());
        // Then stale continuous levels clear, Play remains lit.
        assert!(meter[1..].iter().all(|byte| *byte == 0));
        assert_eq!(buttons[56], 0x47);
    }

    #[test]
    fn output_deadline_coalesces_updates_until_interval_elapsed() {
        // Given a button image just sent.
        let last = Some(Duration::from_millis(100));
        // When the next deadline is checked.
        let before = due(last, Duration::from_millis(109), BUTTON_INTERVAL);
        let at = due(last, Duration::from_millis(110), BUTTON_INTERVAL);
        // Then only the exact deadline admits the next write.
        assert!(!before);
        assert!(at);
        assert!(!due(last, Duration::from_millis(119), RING_INTERVAL));
        assert!(due(last, Duration::from_millis(120), RING_INTERVAL));
    }
}
