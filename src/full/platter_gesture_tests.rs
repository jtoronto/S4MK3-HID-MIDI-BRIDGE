use super::*;
use crate::full::{Control, Deck, feedback};
use anyhow::Context as _;
use std::time::Duration;

struct Fixture {
    translator: Translator,
    feedback: feedback::FeedbackState,
    index: u32,
    counter: u32,
    fraction: i32,
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
                &[Control::Jog("left_jog_wheel", f64::from(travel) / 1000.0)],
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
fn upper_speed_limit_uses_the_same_scratch_handoff_with_hysteresis() -> anyhow::Result<()> {
    // Given qualified TT at 33 RPM and the symmetric 50%/150% diagnostic bounds.
    let mut fixture = Fixture::new();
    fixture.drive(330, 150)?;
    // When forward movement crosses the ceiling then slows through its exit band.
    let fast = fixture.drive(600, 150)?;
    let near_ceiling = fixture.drive(480, 150)?;
    let recovered = fixture.drive(450, 150)?;
    // Then upper Scratch suppresses bend, holds through hysteresis and releases once.
    let entry = fast
        .iter()
        .position(|message| *message == [0x90, 7, 127])
        .context("upper speed Scratch entry")?;
    assert!(
        fast[entry + 1..]
            .iter()
            .any(|message| message[0] == 0xb0 && message[1] == 5)
    );
    assert!(
        !fast[entry + 1..]
            .iter()
            .any(|message| message[0] == 0xb0 && message[1] == 4)
    );
    assert!(!near_ceiling.contains(&[0x80, 7, 0]));
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
fn deck_switch_and_stale_input_release_the_original_speed_owner() -> anyhow::Result<()> {
    for stale in [false, true] {
        // Given deck A speed-owned Scratch with C also authorized.
        let mut fixture = Fixture::new();
        fixture.drive(330, 150)?;
        fixture.drive(120, 150)?;
        let mut output = Vec::new();
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
