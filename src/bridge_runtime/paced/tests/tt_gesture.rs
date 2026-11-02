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
