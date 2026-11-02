use super::*;
use crate::{bridge_runtime::replay, full};

#[test]
fn production_tt_handoff_keeps_assistance_through_stop_reverse_and_recovery() -> anyhow::Result<()>
{
    // Given real feedback and raw HID snapshots after the fixture's startup/ownership checks.
    let fixture = replay::fixture()?;
    let records = RefCell::new(Vec::new());
    // When the production runtime executes braking, stopped/reversed input and acceleration.
    replay::execute(full::jog::DEFAULT, fixture, |record| {
        records.borrow_mut().push(record);
    })?;
    let records = records.into_inner();
    let mut entries = 0;
    let mut releases = 0;
    let mut active = false;
    let mut reverse_seen = false;
    let mut held_reports = 0;
    let mut upper_reports = 0;
    for record in &records {
        let at = record["now_us"].as_u64().context("record time")?;
        if !(3_000_000..16_500_000).contains(&at) {
            continue;
        }
        if record["event"] == "midi" {
            let message: [u8; 3] = serde_json::from_value(record["message"].clone())?;
            if message == [0x92, 7, 127] {
                entries += 1;
                active = true;
            }
            if message == [0x82, 7, 0] {
                releases += 1;
                active = false;
            }
            if active {
                assert!(!(message[0] == 0xb2 && message[1] == 4));
                reverse_seen |= message[0] == 0xb2 && message[1] == 5 && message[2] < 64;
            }
        }
        if record["event"] == "motor" && (3_300_000..4_400_000).contains(&at) {
            let report: [u8; 11] = serde_json::from_value(record["report"].clone())?;
            assert_eq!(u16::from_le_bytes([report[4], report[5]]), 5280);
            held_reports += 1;
        }
        if record["event"] == "motor" && (6_100_000..6_500_000).contains(&at) {
            let report: [u8; 11] = serde_json::from_value(record["report"].clone())?;
            assert_eq!(u16::from_le_bytes([report[4], report[5]]), 0);
            upper_reports += 1;
        }
    }
    // Then actual MIDI ownership and motor output agree, and Stop finishes neutral.
    assert_eq!((entries, releases), (1, 1));
    assert!(reverse_seen && held_reports > 0 && upper_reports > 0);
    assert_eq!(
        records.last().context("final record")?["report"],
        serde_json::json!(neutral_report())
    );
    Ok(())
}

#[test]
fn newly_loaded_tt_track_can_touch_scratch_before_playback_feedback_without_motor_force()
-> anyhow::Result<()> {
    for (loaded, internal, thru, admitted) in [
        (true, Some(true), false, true),
        (false, Some(true), false, false),
        (true, Some(false), false, false),
        (true, None, false, false),
        (true, Some(true), true, false),
    ] {
        // Real loaded/Internal feedback arrives without any playing message.
        let mut steps: Vec<_> = replay::fixture()?.into_iter().take(80).collect();
        steps.push(replay::Step {
            frame: None,
            feedback: vec![],
            now_us: 200_001,
            label: "manual_scratch_after_release",
        });
        for (index, step) in steps.iter_mut().enumerate() {
            step.feedback.clear();
            if let Some(frame) = &mut step.frame {
                frame.buffers[0][17] &= !0x30;
                if (10..70).contains(&index) {
                    frame.buffers[0][17] |= 0x30;
                }
            }
        }
        for deck in 0_u8..2 {
            steps[0]
                .feedback
                .extend([0x96, deck, if loaded { 127 } else { 0 }]);
            steps[0]
                .feedback
                .extend([0xb0 | deck, 21, if thru { 127 } else { 0 }]);
            if let Some(internal) = internal {
                steps[0]
                    .feedback
                    .extend([0xb0 | deck, 22, if internal { 127 } else { 0 }]);
            }
        }
        let records = RefCell::new(Vec::new());
        replay::execute(full::jog::DEFAULT, steps, |record| {
            records.borrow_mut().push(record);
        })?;
        let records = records.into_inner();
        let midi: Vec<[u8; 3]> = records
            .iter()
            .filter(|record| record["event"] == "midi")
            .map(|record| serde_json::from_value(record["message"].clone()))
            .collect::<Result<_, _>>()?;
        for channel in 0_u8..2 {
            assert_eq!(
                midi.iter()
                    .filter(|message| **message == [0x90 | channel, 7, 127])
                    .count(),
                usize::from(admitted),
                "first-load Scratch entry, loaded={loaded}, internal={internal:?}, thru={thru}"
            );
            assert_eq!(
                midi.iter()
                    .filter(|message| **message == [0x80 | channel, 7, 0])
                    .count(),
                usize::from(admitted)
            );
            assert_eq!(
                midi.iter().any(|message| message[0] == 0xb0 | channel
                    && message[1] == 5
                    && message[2] > 64),
                admitted
            );
        }
        let motors: Vec<_> = records
            .iter()
            .filter(|record| record["event"] == "motor")
            .collect();
        assert!(!motors.is_empty());
        for motor in motors {
            let report: [u8; 11] = serde_json::from_value(motor["report"].clone())?;
            assert_eq!(&report[4..6], &[0, 0]);
            assert_eq!(&report[9..11], &[0, 0]);
        }
        let ready = records
            .iter()
            .find(|record| record["event"] == "platter_status" && record["now_us"] == 200_001)
            .context("first-load diagnostic state after touch release")?;
        for wheel in ready["status"]["wheels"]
            .as_array()
            .context("wheel states")?
        {
            assert_eq!(
                wheel["mode"],
                if admitted {
                    "manual_scratch"
                } else {
                    "blocked"
                }
            );
        }
    }
    Ok(())
}

#[test]
fn production_tt_tail_rejects_noise_and_bounds_touched_motor_force() -> anyhow::Result<()> {
    let records = RefCell::new(Vec::new());
    replay::execute(
        full::jog::Config {
            tt_pitch_bend_gain: 10.0,
            ..full::jog::DEFAULT
        },
        replay::fixture()?,
        |record| records.borrow_mut().push(record),
    )?;
    let records = records.into_inner();
    for (start, end) in [(17_700_000, 19_000_000), (19_700_000, 21_000_000)] {
        let mut frames = 0;
        for record in &records {
            let at = record["now_us"].as_u64().context("record time")?;
            if !(start..end).contains(&at) {
                continue;
            }
            if record["event"] == "midi" {
                let message: [u8; 3] = serde_json::from_value(record["message"].clone())?;
                assert!(!((0xb0..=0xb3).contains(&message[0]) && message[1] == 4));
            }
            if record["event"] == "platter_status" {
                for wheel in record["status"]["wheels"].as_array().context("wheels")? {
                    assert_eq!(wheel["mode"], "bend");
                    assert!(
                        wheel["bend_counts_per_second"]
                            .as_f64()
                            .context("bend rate")?
                            .abs()
                            < f64::EPSILON
                    );
                }
                frames += 1;
            }
        }
        assert!(frames > 0);
    }
    for (start, end, expected) in [
        (21_100_000, 21_500_000, 5280),
        (22_100_000, 23_000_000, 0),
        (23_600_000, 24_500_000, 5280),
    ] {
        let mut writes = 0;
        for record in &records {
            let at = record["now_us"].as_u64().context("record time")?;
            if record["event"] == "motor" && (start..end).contains(&at) {
                let report: [u8; 11] = serde_json::from_value(record["report"].clone())?;
                for side in 0..2 {
                    assert_eq!(
                        u16::from_le_bytes([report[4 + side * 5], report[5 + side * 5]]),
                        expected,
                        "at {at}"
                    );
                }
                writes += 1;
            }
        }
        assert!(writes > 0);
    }
    Ok(())
}
