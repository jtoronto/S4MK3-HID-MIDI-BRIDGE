use super::*;
use crate::full::{Control, Deck, feedback, jog};
use anyhow::Context as _;
use std::time::Duration;

struct Fixture {
    translator: Translator,
    feedback: feedback::FeedbackState,
    index: u32,
    counter: u32,
    fraction: i32,
    side: usize,
}

impl Fixture {
    fn new() -> Self {
        let mut translator = Translator::default();
        translator.motor_enabled[0] = true;
        translator.vinyl[0] = false;
        let mut feedback = feedback::FeedbackState::default();
        for deck in [0, 2] {
            feedback.loaded[deck] = Some(true);
            feedback.playing[deck] = Some(true);
            feedback.internal[deck] = Some(true);
        }
        Self {
            translator,
            feedback,
            index: 0,
            counter: 0,
            fraction: 0,
            side: 0,
        }
    }

    fn drive(&mut self, rpm_tenths: i32, samples: u32) -> anyhow::Result<Vec<[u8; 3]>> {
        let mut output = Vec::new();
        for _ in 0..samples {
            self.index += 1;
            self.fraction += rpm_tenths * 2880;
            let travel = self.fraction / 300_000;
            self.fraction %= 300_000;
            self.counter = self.counter.wrapping_add_signed(travel);
            let mut report = [0_u8; 60];
            report[0] = 3;
            for offset in [8, 36] {
                report[offset..offset + 4].copy_from_slice(&(self.index * 200_000).to_le_bytes());
                report[offset + 4..offset + 8].copy_from_slice(&self.counter.to_le_bytes());
            }
            self.translator.report(
                &[Control::Jog(
                    ["left_jog_wheel", "right_jog_wheel"][self.side],
                    f64::from(travel) / 1000.0,
                )],
                &report,
                Duration::from_micros(u64::from(self.index) * 2000),
                &self.feedback,
                &mut output,
            )?;
        }
        Ok(output)
    }
}

#[test]
fn tt_floor_backspin_detects_boosts_and_survives_release_without_motor_assist() -> anyhow::Result<()>
{
    for side in 0..2 {
        for (enabled, threshold, increase, detected, gain) in [
            (false, 150_000.0, jog::DEFAULT.scratch_speed, false, 1),
            (true, 1_000_000.0, jog::DEFAULT.scratch_speed, false, 1),
            (true, 150_000.0, 0.0, true, 1),
            (true, 150_000.0, jog::DEFAULT.scratch_speed, true, 2),
        ] {
            let mut fixture = Fixture::new();
            fixture.side = side;
            fixture.translator.motor_enabled = [false; 2];
            fixture.translator.motor_enabled[side] = true;
            fixture.translator.vinyl[side] = false;
            fixture.translator.jog_config.backspin_enabled = enabled;
            fixture.translator.jog_config.backspin_acceleration = threshold;
            fixture.translator.jog_config.backspin_speed_increase = increase;
            fixture.feedback.loaded[side] = Some(true);
            fixture.feedback.playing[side] = Some(true);
            fixture.feedback.internal[side] = Some(true);
            let channel = u8::try_from(side)?;
            let touch = ["left_wheel_touch", "right_wheel_touch"][side];
            fixture.drive(330, 150)?;
            fixture
                .translator
                .translate(&[Control::Button(touch, true)], None, &mut Vec::new())?;
            fixture.drive(0, 50)?;
            fixture.drive(-1200, 30)?;
            assert_eq!(
                fixture.translator.backspin_deck(side),
                detected.then_some(channel)
            );
            assert_eq!(fixture.translator.backspin_assist(), [false; 2]);

            // The detector, screen state and boost follow the synthetic TT
            // Scratch owner both before and after physical top release.
            for release in [false, true] {
                if release {
                    let mut output = Vec::new();
                    fixture.translator.translate(
                        &[Control::Button(touch, false)],
                        None,
                        &mut output,
                    )?;
                    assert!(output.is_empty());
                }
                let before = fixture.counter;
                let output = fixture.drive(-1200, 10)?;
                assert_eq!(
                    fixture.translator.backspin_deck(side),
                    detected.then_some(channel)
                );
                let s4mk3_hid_midi_bridge::screens::page::Page::Info(info) =
                    crate::full::screens::pages(&fixture.translator, &fixture.feedback)[side]
                else {
                    anyhow::bail!("expected info page")
                };
                assert_eq!(info.backspin, detected.then_some(channel));
                assert!(!output.is_empty());
                assert!(
                    output
                        .iter()
                        .all(|message| message[0] == 0xb0 | channel && message[1] == 5)
                );
                let sent: i32 = output
                    .iter()
                    .map(|message| i32::from(message[2]) - 64)
                    .sum();
                let travel = i32::from_le_bytes(fixture.counter.wrapping_sub(before).to_le_bytes());
                assert_eq!(sent, travel * gain);
                assert_eq!(fixture.translator.backspin_assist(), [false; 2]);
            }

            let mut output = Vec::new();
            fixture
                .translator
                .translate(&[Control::Button(touch, true)], None, &mut output)?;
            assert_eq!(fixture.translator.backspin_deck(side), None);
            assert!(output.is_empty()); // Catching the top must not chatter Scratch Note 7.
            let before = fixture.counter;
            let caught = fixture.drive(-1200, 20)?;
            let sent: i32 = caught
                .iter()
                .map(|message| i32::from(message[2]) - 64)
                .sum();
            assert_eq!(
                sent,
                i32::from_le_bytes(fixture.counter.wrapping_sub(before).to_le_bytes())
            );
            assert!(fixture.drive(0, 20)?.is_empty());
            assert_eq!(fixture.translator.backspin_deck(side), None);
            fixture
                .translator
                .translate(&[Control::Button(touch, false)], None, &mut output)?;
            let recovered = fixture.drive(330, 150)?;
            assert_eq!(
                recovered
                    .iter()
                    .filter(|message| **message == [0x80 | channel, 7, 0])
                    .count(),
                1
            );
        }
    }
    Ok(())
}

#[test]
fn tt_backspin_fraction_does_not_change_same_frame_normal_mode_motion() -> anyhow::Result<()> {
    let mut fixture = Fixture::new();
    fixture.translator.jog_config.backspin_speed_increase = jog::DEFAULT.scratch_speed / 2.0;
    fixture.drive(330, 150)?;
    fixture.drive(0, 50)?;
    fixture.drive(-1200, 30)?;
    assert_eq!(fixture.translator.backspin_deck(0), Some(0));

    let mut output = Vec::new();
    fixture.translator.translate(
        &[
            Control::Button("left_turntable_mode", true),
            Control::Jog("left_jog_wheel", 0.001),
        ],
        None,
        &mut output,
    )?;
    assert_eq!(output, [[0xb0, 4, 65]]);
    Ok(())
}

#[test]
fn tt_shift_requires_top_contact_for_seek_but_normal_shift_does_not() -> anyhow::Result<()> {
    for (side, deck) in [(0, Deck::A), (0, Deck::C), (1, Deck::B), (1, Deck::D)] {
        for motor_enabled in [false, true] {
            for vinyl in [false, true] {
                let mut fixture = Fixture::new();
                fixture.side = side;
                fixture.translator.decks[side] = deck;
                fixture.translator.motor_enabled = [false; 2];
                fixture.translator.motor_enabled[side] = motor_enabled;
                fixture.translator.vinyl[side] = vinyl;
                let channel = deck.channel();
                let index = usize::from(channel);
                fixture.feedback.loaded[index] = Some(true);
                fixture.feedback.playing[index] = Some(true);
                fixture.feedback.internal[index] = Some(true);
                fixture.drive(330, 150)?;
                let shift = ["left_shift", "right_shift"][side];
                let touch = ["left_wheel_touch", "right_wheel_touch"][side];
                let mut output = Vec::new();
                fixture
                    .translator
                    .translate(&[Control::Button(shift, true)], None, &mut output)?;
                assert!(output.is_empty());

                // Holding Shift must not seek from untouched TT rotation.
                let untouched = fixture.drive(330, 100)?;
                if motor_enabled {
                    assert!(
                        untouched.is_empty(),
                        "untouched TT rotation emitted {untouched:?}"
                    );
                } else {
                    assert!(!untouched.is_empty());
                    assert!(untouched.iter().all(|message| message[0] == 0xb0 | channel
                        && message[1] == 6
                        && message[2] > 64));
                }

                // Explicit Shift+top contact still seeks in both directions.
                fixture
                    .translator
                    .translate(&[Control::Button(touch, true)], None, &mut output)?;
                for (speed, direction) in [
                    (330, std::cmp::Ordering::Greater),
                    (-330, std::cmp::Ordering::Less),
                ] {
                    let seeking = fixture.drive(speed, 100)?;
                    assert!(!seeking.is_empty());
                    assert!(seeking.iter().all(|message| message[0] == 0xb0 | channel
                        && message[1] == 6
                        && message[2].cmp(&64) == direction));
                }
                fixture.translator.translate(
                    &[Control::Button(touch, false)],
                    None,
                    &mut output,
                )?;
                if motor_enabled {
                    assert!(fixture.drive(330, 100)?.is_empty());
                }
            }
        }
    }
    Ok(())
}

#[test]
fn tt_gain_scales_only_bend_without_changing_measured_rpm() -> anyhow::Result<()> {
    for (gesture_rpm, direction) in [(300, -1.0), (360, 1.0)] {
        let mut totals = Vec::new();
        for gain in [0.5, 1.0, 2.0, 10.0] {
            let mut fixture = Fixture::new();
            fixture.translator.jog_config.tt_pitch_bend_gain = gain;
            fixture.drive(330, 150)?;
            let output = fixture.drive(gesture_rpm, 600)?;
            assert!(
                output
                    .iter()
                    .all(|message| message[0] == 0xb0 && message[1] == 4)
            );
            let total: i32 = output
                .iter()
                .map(|message| i32::from(message[2]) - 64)
                .sum();
            totals.push((gain, total));
            let expected_rpm = f64::from(gesture_rpm) / 10.0;
            assert!(
                fixture.translator.platter_bend[0]
                    .measured_rpm(fixture.translator.jog_now)
                    .is_some_and(|rpm| (rpm - expected_rpm).abs() < 0.3)
            );
        }
        let baseline = f64::from(totals[1].1);
        assert!(baseline * direction > 0.0);
        for (gain, total) in totals {
            assert!(
                baseline.mul_add(-gain, f64::from(total)).abs() <= gain + 1.0,
                "gain {gain}: {total} did not scale baseline {baseline}"
            );
        }
    }
    Ok(())
}

#[test]
fn independent_curves_shape_bend_in_both_directions_and_continue_linearly() -> anyhow::Result<()> {
    // Explicit gains override legacy 5x independently; a complete 100ms window
    // at 20, 30 or 75 RPM has exact integer travel, avoiding estimator rounding.
    for side in 0..2 {
        for (rpm_tenths, expected) in [(200, -144), (300, 805), (750, 3600)] {
            let mut fixture = Fixture::new();
            fixture.side = side;
            fixture.translator.motor_enabled = [false; 2];
            fixture.translator.motor_enabled[side] = true;
            fixture.translator.jog_config = jog::Config {
                platter_target_rpm: 25,
                tt_pitch_bend_gain: 5.0,
                tt_brake_gain: Some(3.0),
                tt_nudge_gain: Some(1.5),
                tt_brake_curve_exponent: 2.0,
                tt_nudge_curve_exponent: 0.5,
                ..jog::DEFAULT
            };
            fixture.feedback.loaded[side] = Some(true);
            fixture.feedback.playing[side] = Some(true);
            fixture.feedback.internal[side] = Some(true);
            fixture.drive(250, 150)?;
            fixture.drive(rpm_tenths, 100)?;
            // Exactly one settled second through real decoding/routing:
            // +/-5 RPM, gains3/1.5, shapes2/.5, 48 counts/s/RPM.
            // At +50 RPM the shape factor stays1 rather than extrapolating it.
            let output = fixture.drive(rpm_tenths, 500)?;
            let channel = u8::try_from(side)?;
            assert!(
                output
                    .iter()
                    .all(|message| message[0] == 0xb0 | channel && message[1] == 4)
            );
            let total: i32 = output
                .iter()
                .map(|message| i32::from(message[2]) - 64)
                .sum();
            assert!(
                (total - expected).abs() <= 1,
                "side {side}, speed {rpm_tenths}: {total}, expected {expected}"
            );
        }
    }
    Ok(())
}

#[test]
fn tt_gain_leaves_normal_jog_and_paused_scratch_unchanged() -> anyhow::Result<()> {
    for paused in [false, true] {
        let mut outputs = Vec::new();
        for gain in [1.0, 10.0] {
            let mut fixture = Fixture::new();
            fixture.translator.jog_config.tt_pitch_bend_gain = gain;
            if gain > 1.0 {
                fixture.translator.jog_config.tt_brake_gain = Some(20.0);
                fixture.translator.jog_config.tt_nudge_gain = Some(0.1);
                fixture.translator.jog_config.tt_brake_curve_exponent = 4.0;
                fixture.translator.jog_config.tt_nudge_curve_exponent = 0.25;
            }
            fixture.translator.motor_enabled[0] = paused;
            fixture.feedback.playing[0] = Some(!paused);
            outputs.push(fixture.drive(-120, 150)?);
        }
        assert!(!outputs[0].is_empty());
        assert_eq!(outputs[0], outputs[1]);
    }
    Ok(())
}

#[test]
fn paused_tt_moves_the_track_without_vinyl_or_motor_authorization() -> anyhow::Result<()> {
    for side in 0..2 {
        let mut fixture = Fixture::new();
        fixture.side = side;
        fixture.translator.motor_enabled = [false; 2];
        fixture.translator.motor_enabled[side] = true;
        fixture.translator.vinyl[side] = false;
        fixture.feedback.loaded[side] = Some(true);
        fixture.feedback.internal[side] = Some(true);
        fixture.feedback.playing[side] = Some(false);

        let output = fixture.drive(-120, 150)?;
        let channel = u8::try_from(side)?;
        assert!(output.contains(&[0x90 | channel, 7, 127]));
        assert!(
            output.iter().any(|message| {
                message[0] == 0xb0 | channel && message[1] == 5 && message[2] < 64
            })
        );
        assert!(!output.iter().any(|message| message[1] == 4));
        assert!(!fixture.translator.vinyl[side]);
        assert!(!fixture.translator.playback_state().enabled[side]);
    }
    Ok(())
}

#[test]
fn paused_touch_scratch_keeps_its_deck_until_release() -> anyhow::Result<()> {
    let mut fixture = Fixture::new();
    fixture.feedback.playing[0] = Some(false);
    fixture.feedback.playing[2] = Some(false);
    fixture.drive(120, 20)?;
    let mut output = Vec::new();
    fixture.translator.translate(
        &[Control::Button("left_wheel_touch", true)],
        None,
        &mut output,
    )?;
    fixture.translator.translate(
        &[Control::Button("left_deck_switch_c", true)],
        None,
        &mut output,
    )?;

    let held = fixture.drive(-120, 20)?;
    assert!(
        held.iter()
            .any(|message| message[0] == 0xb0 && message[1] == 5)
    );
    assert!(
        !held
            .iter()
            .any(|message| message[0] == 0xb2 && message[1] == 5)
    );
    assert!(!held.contains(&[0x80, 7, 0]));
    output.clear();
    fixture.translator.translate(
        &[Control::Button("left_wheel_touch", false)],
        None,
        &mut output,
    )?;
    assert_eq!(output, [[0x80, 7, 0]]);

    let next = fixture.drive(0, 4)?;
    assert!(next.contains(&[0x92, 7, 127]));
    output.clear();
    fixture.translator.release_all(&mut output);
    assert_eq!(output, [[0x82, 7, 0]]);
    Ok(())
}

#[test]
fn paused_scratch_releases_on_resume_mode_changes_and_freshness_loss() -> anyhow::Result<()> {
    for cancellation in ["resume", "tt_off", "shift", "grid", "stale", "unknown"] {
        let mut fixture = Fixture::new();
        fixture.feedback.playing[0] = Some(false);
        fixture.drive(120, 20)?;
        let mut output = Vec::new();
        match cancellation {
            "resume" => fixture.feedback.playing[0] = Some(true),
            "tt_off" => fixture.translator.motor_enabled[0] = false,
            "shift" => fixture.translator.shifted[0] = true,
            "grid" => fixture.translator.grid[0] = true,
            "stale" => fixture.translator.jog_now += Duration::from_millis(51),
            "unknown" => fixture.feedback.playing[0] = None,
            _ => unreachable!(),
        }
        if cancellation != "stale" {
            fixture
                .translator
                .sync_front_panel(&fixture.feedback, &mut output)?;
        }
        fixture.translator.motorized_bend(
            None,
            fixture.translator.playback_state(),
            &mut output,
        )?;
        assert_eq!(output, [[0x80, 7, 0]], "{cancellation}");
        assert!(fixture.translator.platter_scratch[0].is_none());
    }
    Ok(())
}

#[test]
fn paused_scratch_drops_implausible_counter_jumps() -> anyhow::Result<()> {
    let mut fixture = Fixture::new();
    fixture.feedback.playing[0] = Some(false);
    fixture.drive(120, 20)?;
    fixture.counter = fixture.counter.wrapping_add(100_000);

    let output = fixture.drive(0, 1)?;
    assert!(!output.iter().any(|message| message[1] == 5));
    Ok(())
}

#[test]
fn speed_handoff_stops_bend_and_routes_raw_scratch_without_touch() -> anyhow::Result<()> {
    // Given authorized, qualified TT playback with Vinyl off.
    let mut fixture = Fixture::new();
    assert!(fixture.drive(330, 150)?.is_empty());
    // When the wheel crosses the diagnostic half-speed floor.
    let output = fixture.drive(120, 150)?;
    // Then Scratch mode precedes raw Scratch movement and replaces Pitch Bend.
    let entry = output
        .iter()
        .position(|message| *message == [0x90, 7, 127])
        .context("speed Scratch entry")?;
    assert!(
        output[entry + 1..]
            .iter()
            .any(|message| message[0] == 0xb0 && message[1] == 5)
    );
    assert!(
        !output[entry + 1..]
            .iter()
            .any(|message| message[0] == 0xb0 && message[1] == 4)
    );
    assert_eq!(fixture.translator.platter_scratch[0], Some(Deck::A));
    assert!(fixture.translator.playback_state().owned[0]);
    Ok(())
}

#[test]
fn abrupt_stop_enters_scratch_after_fresh_confirmation_not_bend_averaging() -> anyhow::Result<()> {
    for side in 0..2 {
        for floor in [1, 50, 95] {
            for touched in [false, true] {
                let mut fixture = Fixture::new();
                fixture.side = side;
                fixture.translator.motor_enabled = [false; 2];
                fixture.translator.motor_enabled[side] = true;
                fixture.translator.vinyl[side] = false;
                fixture.translator.jog_config.tt_pitch_bend_floor_percent = floor;
                fixture.feedback.loaded[side] = Some(true);
                fixture.feedback.playing[side] = Some(true);
                fixture.feedback.internal[side] = Some(true);
                // Stopped startup must still qualify at normal speed first.
                assert!(fixture.drive(0, 100)?.is_empty());
                fixture.drive(330, 150)?;
                let mut output = Vec::new();
                if touched {
                    fixture.translator.translate(
                        &[Control::Button(
                            ["left_wheel_touch", "right_wheel_touch"][side],
                            true,
                        )],
                        None,
                        &mut output,
                    )?;
                }

                // A single 2ms zero-travel sample, even replayed, is not a stop.
                assert!(fixture.drive(0, 1)?.is_empty());
                let mut duplicate = [0_u8; 60];
                duplicate[0] = 3;
                for offset in [8, 36] {
                    duplicate[offset..offset + 4]
                        .copy_from_slice(&(fixture.index * 200_000).to_le_bytes());
                    duplicate[offset + 4..offset + 8]
                        .copy_from_slice(&fixture.counter.to_le_bytes());
                }
                fixture.translator.report(
                    &[],
                    &duplicate,
                    Duration::from_micros(u64::from(fixture.index) * 2000 + 1000),
                    &fixture.feedback,
                    &mut output,
                )?;
                assert!(output.is_empty());

                // The next fresh sample completes 4ms at rest, long before
                // the 100ms average reaches even the 95% floor.
                let channel = u8::try_from(side)?;
                assert_eq!(fixture.drive(0, 1)?, [[0x90 | channel, 7, 127]]);
                assert!(fixture.drive(0, 10)?.is_empty());
            }
        }
    }
    Ok(())
}

#[test]
fn gradual_low_speed_zero_counts_do_not_bypass_the_saved_scratch_floor() -> anyhow::Result<()> {
    let mut fixture = Fixture::new();
    fixture.translator.jog_config.tt_pitch_bend_floor_percent = 1;
    fixture.drive(330, 150)?;
    let mut output = Vec::new();
    fixture.translator.translate(
        &[Control::Button("left_wheel_touch", true)],
        None,
        &mut output,
    )?;

    // Slow deceleration leaves multiple zero-count reports between encoder
    // increments at 1 RPM, still above the saved 0.33 RPM entry floor.
    for speed in [300, 270, 240, 210, 180, 150, 120, 90, 60, 30, 20, 10] {
        output.extend(fixture.drive(speed, 100)?);
    }
    assert!(
        output
            .iter()
            .all(|message| message[0] == 0xb0 && message[1] == 4)
    );
    assert!(output.iter().any(|message| message[2] < 64));
    Ok(())
}

#[test]
fn floor_scratch_stays_through_touched_fast_strokes_until_release() -> anyhow::Result<()> {
    for side in 0..2 {
        for floor in [1, 50, 95] {
            for touch_before_entry in [false, true] {
                let mut fixture = Fixture::new();
                fixture.side = side;
                fixture.translator.motor_enabled = [false; 2];
                fixture.translator.motor_enabled[side] = true;
                fixture.translator.vinyl[side] = false;
                fixture.translator.jog_config.tt_pitch_bend_floor_percent = floor;
                fixture.feedback.loaded[side] = Some(true);
                fixture.feedback.playing[side] = Some(true);
                fixture.feedback.internal[side] = Some(true);
                fixture.drive(330, 150)?;
                let touch = ["left_wheel_touch", "right_wheel_touch"][side];
                let channel = u8::try_from(side)?;
                let mut output = Vec::new();
                if touch_before_entry {
                    fixture.translator.translate(
                        &[Control::Button(touch, true)],
                        None,
                        &mut output,
                    )?;
                    assert!(output.is_empty());
                    assert!(fixture.drive(330, 100)?.is_empty());
                }
                let entry = fixture.drive(0, 150)?;
                assert_eq!(
                    entry
                        .iter()
                        .filter(|message| **message == [0x90 | channel, 7, 127])
                        .count(),
                    1
                );
                if !touch_before_entry {
                    fixture.translator.translate(
                        &[Control::Button(touch, true)],
                        None,
                        &mut output,
                    )?;
                }

                // Full measurement windows cross the floor in both directions
                // while the same top contact remains held.
                for speed in [600, -600, 900] {
                    output.extend(fixture.drive(speed, 100)?);
                }
                assert!(
                    output
                        .iter()
                        .all(|message| message[0] == 0xb0 | channel && message[1] == 5),
                    "side {side}, floor {floor}: touched Scratch changed routes"
                );
                for direction in [std::cmp::Ordering::Less, std::cmp::Ordering::Greater] {
                    assert!(
                        output
                            .iter()
                            .any(|message| message[2].cmp(&64) == direction)
                    );
                }

                output.clear();
                fixture.translator.translate(
                    &[Control::Button(touch, false)],
                    None,
                    &mut output,
                )?;
                output.extend(fixture.drive(900, 100)?);
                let release = output
                    .iter()
                    .position(|message| *message == [0x80 | channel, 7, 0])
                    .context("released top contact must allow recovered-speed exit")?;
                assert_eq!(output.iter().filter(|message| message[1] == 7).count(), 1);
                assert!(
                    output[release + 1..]
                        .iter()
                        .all(|message| { message[0] == 0xb0 | channel && message[1] == 4 })
                );
                assert!(output[release + 1..].iter().any(|message| message[2] > 64));
            }
        }
    }
    Ok(())
}

#[test]
fn floor_scratch_touch_release_below_exit_waits_for_speed_recovery() -> anyhow::Result<()> {
    let mut fixture = Fixture::new();
    fixture.drive(330, 150)?;
    fixture.drive(0, 150)?;
    let mut output = Vec::new();
    for touched in [true, false] {
        fixture.translator.translate(
            &[Control::Button("left_wheel_touch", touched)],
            None,
            &mut output,
        )?;
    }

    output.extend(fixture.drive(0, 100)?);
    assert!(output.is_empty());
    let recovered = fixture.drive(330, 150)?;
    assert_eq!(
        recovered
            .iter()
            .filter(|message| message[1] == 7)
            .copied()
            .collect::<Vec<_>>(),
        [[0x80, 7, 0]]
    );
    Ok(())
}

#[test]
fn held_stop_and_reverse_stay_in_scratch_until_speed_recovers() -> anyhow::Result<()> {
    // Given speed-owned Scratch without capacitive touch.
    let mut fixture = Fixture::new();
    fixture.drive(330, 150)?;
    fixture.drive(120, 150)?;
    // When motion stops, reverses, then resumes below and above the exit threshold.
    let held = fixture.drive(0, 150)?;
    let reversed = fixture.drive(-60, 150)?;
    let below_exit = fixture.drive(180, 150)?;
    let recovered = fixture.drive(330, 150)?;
    // Then stationary Scratch is held, reversal remains signed and recovery releases once.
    assert!(held.is_empty());
    assert!(
        reversed
            .iter()
            .any(|message| message[0] == 0xb0 && message[1] == 5 && message[2] < 64)
    );
    assert!(!below_exit.contains(&[0x80, 7, 0]));
    assert_eq!(
        recovered
            .iter()
            .filter(|message| **message == [0x80, 7, 0])
            .count(),
        1
    );
    assert!(fixture.translator.platter_scratch[0].is_none());
    Ok(())
}

#[test]
fn forward_nudges_never_enter_speed_scratch_at_any_floor() -> anyhow::Result<()> {
    for floor in [1, 50, 95] {
        for speed in [600, 667, 903] {
            let mut fixture = Fixture::new();
            fixture.translator.jog_config.tt_pitch_bend_floor_percent = floor;
            fixture.drive(330, 150)?;
            let fast = fixture.drive(speed, 150)?;
            assert!(
                !fast
                    .iter()
                    .any(|message| message[1] == 7 || message[1] == 5),
                "floor {floor}, speed {speed}: forward nudge became Scratch"
            );
            assert!(
                fast.iter()
                    .any(|message| message[0] == 0xb0 && message[1] == 4 && message[2] > 64)
            );
            assert!(fixture.translator.platter_scratch[0].is_none());
        }
    }
    Ok(())
}

#[test]
fn lower_scratch_releases_once_even_above_the_former_ceiling() -> anyhow::Result<()> {
    for floor in [1, 50, 95] {
        let mut fixture = Fixture::new();
        fixture.translator.jog_config.tt_pitch_bend_floor_percent = floor;
        fixture.drive(330, 150)?;
        assert!(fixture.drive(0, 150)?.contains(&[0x90, 7, 127]));
        // One valid (<512-count) sample crosses the old entire exit interval.
        fixture.counter = fixture.counter.wrapping_add(400);
        let recovered = fixture.drive(903, 1)?;
        assert_eq!(
            recovered
                .iter()
                .filter(|message| **message == [0x80, 7, 0])
                .count(),
            1
        );
        assert!(!recovered.contains(&[0x90, 7, 127]));
        assert!(
            recovered
                .iter()
                .any(|message| message[0] == 0xb0 && message[1] == 4 && message[2] > 64)
        );
        assert!(fixture.translator.platter_scratch[0].is_none());
    }
    Ok(())
}

#[test]
fn deck_switch_and_stale_input_release_the_original_speed_owner() -> anyhow::Result<()> {
    for (stale, touched, backspin) in [
        (false, false, false),
        (true, false, false),
        (false, true, false),
        (true, true, false),
        (false, false, true),
        (true, false, true),
        (false, true, true),
        (true, true, true),
    ] {
        // Given deck A speed-owned Scratch with C also authorized.
        let mut fixture = Fixture::new();
        fixture.drive(330, 150)?;
        fixture.drive(120, 150)?;
        let mut output = Vec::new();
        if touched {
            fixture.translator.translate(
                &[Control::Button("left_wheel_touch", true)],
                None,
                &mut output,
            )?;
        }
        if backspin {
            fixture.drive(-1200, 30)?;
            assert_eq!(fixture.translator.backspin_deck(0), Some(0));
        }
        // When selection changes or the last real sensor sample becomes stale.
        if stale {
            fixture.translator.jog_now += Duration::from_millis(51);
        } else {
            fixture
                .translator
                .selectors(&[Control::Button("left_deck_switch_c", true)], &mut output);
            assert!(!fixture.translator.playback_state().enabled[0]);
        }
        fixture.translator.motorized_bend(
            None,
            fixture.translator.playback_state(),
            &mut output,
        )?;
        // Then the original deck receives release, never the newly selected deck.
        assert!(output.contains(&[0x80, 7, 0]));
        assert!(!output.contains(&[0x82, 7, 0]));
        assert!(fixture.translator.platter_scratch[0].is_none());
        assert_eq!(fixture.translator.backspin_deck(0), None);
        fixture
            .translator
            .selectors(&[Control::Button("left_turntable_mode", true)], &mut output);
        let mut normal = Vec::new();
        fixture
            .translator
            .jog("left_jog_wheel", 0.001, &mut normal)?;
        assert_eq!(normal, [[if stale { 0xb0 } else { 0xb2 }, 4, 65]]);
    }
    Ok(())
}

#[test]
fn tt_and_normal_modes_remember_independent_vinyl_choices() {
    // Given normal mode's default Vinyl on.
    let mut state = Translator::default();
    let mut output = Vec::new();
    assert!(state.vinyl[0]);
    // When TT is selected, its Vinyl choice is changed, and modes are revisited.
    for (name, vinyl) in [
        ("left_turntable_mode", false),
        ("left_jog_mode", true),
        ("left_turntable_mode", true),
        ("left_jog_mode", false),
        ("left_turntable_mode", true),
        ("left_turntable_mode", false),
    ] {
        state.selectors(&[Control::Button(name, true)], &mut output);
        state.release(name, &mut output);
        // Then each mode restores only its own choice, without replacing the other.
        assert_eq!(state.vinyl[0], vinyl);
    }
}
