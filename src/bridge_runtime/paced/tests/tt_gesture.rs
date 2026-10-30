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
        if at < 3_000_000 {
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
    assert_eq!((entries, releases), (2, 2));
    assert!(reverse_seen && held_reports > 0 && upper_reports > 0);
    assert_eq!(
        records.last().context("final record")?["report"],
        serde_json::json!(neutral_report())
    );
    Ok(())
}
