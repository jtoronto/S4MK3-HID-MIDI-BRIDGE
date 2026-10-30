use super::*;
use crate::full::{Control, Deck, Translator};

struct Rig {
    translator: Translator,
    feedback: FeedbackState,
    now_us: u32,
    touch: bool,
    phase: ring_phase::Rings,
}

impl Rig {
    fn new(tt: bool) -> Self {
        let translator = Translator {
            motor_enabled: [tt; 2],
            jog_config: crate::full::jog::Config {
                backspin_enabled: false,
                ..crate::full::jog::DEFAULT
            },
            ..Translator::default()
        };
        let mut feedback = FeedbackState::default();
        feedback.loaded = [Some(true); 4];
        feedback.playing = [Some(true); 4];
        feedback.internal = [Some(true); 4];
        Self {
            translator,
            feedback,
            now_us: 0,
            touch: true,
            phase: ring_phase::Rings::default(),
        }
    }

    fn sample(&mut self, counters: [u32; 2]) -> anyhow::Result<[[u8; 41]; 2]> {
        self.now_us += 2000;
        let mut report = [0; 60];
        report[0] = 3;
        for (side, offset) in [(0, 8), (1, 36)] {
            report[offset..offset + 4].copy_from_slice(&(self.now_us * 100).to_le_bytes());
            report[offset + 4..offset + 8].copy_from_slice(&counters[side].to_le_bytes());
        }
        self.translator.report(
            &[
                Control::Button("left_wheel_touch", self.touch),
                Control::Button("right_wheel_touch", self.touch),
            ],
            &report,
            Duration::from_micros(u64::from(self.now_us)),
            &self.feedback,
            &mut Vec::new(),
        )?;
        let mut local = self.translator.local_led_state();
        local.ring_positions = self.phase.positions(
            &LedConfig::default(),
            &self.feedback,
            &local,
            Duration::from_micros(u64::from(self.now_us)),
        )?;
        Ok([0_u8, 1].map(|side| {
            render_ring(
                &LedConfig::default(),
                &self.feedback,
                &local,
                side,
                Duration::from_micros(u64::from(self.now_us)),
            )
        }))
    }
}

fn position(report: [u8; 41]) -> u16 {
    u16::from_le_bytes([report[3], report[4]])
}

#[test]
fn normal_playback_advances_with_stationary_untouched_wheels() -> anyhow::Result<()> {
    // Given normal playback with physically stationary, untouched wheels.
    let mut rig = Rig::new(false);
    rig.touch = false;
    let before = rig.sample([900, 1800])?.map(position);
    // When playback advances without physical movement.
    let after = rig.sample([900, 1800])?.map(position);
    // Then the normal ring advances rather than following only the stationary wheel.
    assert_ne!(before, after);
    Ok(())
}

#[test]
fn physical_ring_follows_scratch_and_holds_through_pause_and_deck_changes() -> anyhow::Result<()> {
    // Given raw physical counters and touched wheels in TT mode.
    let mut rig = Rig::new(true);
    assert_eq!(rig.sample([900, 1800])?.map(position), [900, 1800]);
    // When the sides move in opposite directions, hold, pause and change decks.
    assert_eq!(rig.sample([960, 1768])?.map(position), [960, 1768]);
    assert_eq!(rig.sample([960, 1768])?.map(position), [960, 1768]);
    rig.feedback.playing = [Some(false); 4];
    assert_eq!(rig.sample([960, 1768])?.map(position), [960, 1768]);
    rig.translator.decks = [Deck::C, Deck::D];
    assert_eq!(rig.sample([960, 1768])?.map(position), [960, 1768]);
    rig.feedback.playing = [Some(true); 4];
    // Then neither playback nor ownership changes reset or advance physical phase.
    assert_eq!(rig.sample([960, 1768])?.map(position), [960, 1768]);
    Ok(())
}

#[test]
fn normal_pause_scratch_and_release_preserve_displayed_phase() -> anyhow::Result<()> {
    // Given advancing normal playback and a stationary untouched wheel.
    let mut rig = Rig::new(false);
    rig.touch = false;
    rig.sample([900, 1800])?;
    rig.now_us += 500_000;
    let playing = rig.sample([900, 1800])?.map(position);
    assert_ne!(playing, [0; 2]);
    // When playback pauses, a stopped-track scratch moves, then playback resumes.
    rig.feedback.playing = [Some(false); 4];
    let paused = rig.sample([900, 1800])?.map(position);
    rig.now_us += 500_000;
    assert_eq!(rig.sample([900, 1800])?.map(position), paused);
    rig.touch = true;
    let scratched = rig.sample([910, 1790])?.map(position);
    assert_eq!(
        scratched,
        [(paused[0] + 10) % 2880, (paused[1] + 2870) % 2880]
    );
    rig.now_us += 500_000;
    assert_eq!(rig.sample([910, 1790])?.map(position), scratched);
    rig.touch = false;
    assert_eq!(rig.sample([910, 1790])?.map(position), scratched);
    rig.feedback.playing = [Some(true); 4];
    assert_eq!(rig.sample([910, 1790])?.map(position), scratched);
    rig.now_us += 250_000;
    // Then playback continues from the scratch-adjusted position, never from noon.
    assert_ne!(rig.sample([910, 1790])?.map(position), scratched);
    Ok(())
}

#[test]
fn normal_scratch_holds_phase_while_playback_flag_remains_on() -> anyhow::Result<()> {
    // Given normal playback entering top-touch Scratch.
    let mut rig = Rig::new(false);
    rig.touch = false;
    rig.sample([100, 200])?;
    rig.now_us += 200_000;
    rig.sample([100, 200])?;
    rig.touch = true;
    let held = rig.sample([100, 200])?.map(position);
    // When the wheel is held still while Djay's playing flag stays true.
    rig.now_us += 500_000;
    // Then the playback clock cannot advance the Scratch-owned ring.
    assert_eq!(rig.sample([100, 200])?.map(position), held);
    Ok(())
}

#[test]
fn normal_phase_routes_scratch_to_its_original_deck() -> anyhow::Result<()> {
    // Given held Scratch on A/B before selecting the other logical decks.
    let mut rig = Rig::new(false);
    rig.sample([100, 200])?;
    rig.translator.decks = [Deck::C, Deck::D];
    // When the original owners move while C/D remain selected and playing.
    assert_eq!(rig.sample([120, 230])?.map(position), [2, 2]);
    rig.translator.decks = [Deck::A, Deck::B];
    // Then A/B retain the Scratch delta, not the newly selected decks' clock phase.
    assert_eq!(rig.sample([120, 230])?.map(position), [20, 30]);
    Ok(())
}

#[test]
fn physical_ring_preserves_signed_phase_across_raw_counter_rollover() -> anyhow::Result<()> {
    // Given both raw counters close to their u32 rollover.
    let mut rig = Rig::new(true);
    let before = rig.sample([u32::MAX - 10; 2])?.map(position);
    // When the counters advance 21 counts across zero and then reverse.
    let forward = rig.sample([10; 2])?.map(position);
    let reverse = rig.sample([u32::MAX - 10; 2])?.map(position);
    // Then native ring phase advances by exactly 21 and returns to its old position.
    assert_eq!(forward, before.map(|phase| (phase + 21) % 2880));
    assert_eq!(reverse, before);
    Ok(())
}
