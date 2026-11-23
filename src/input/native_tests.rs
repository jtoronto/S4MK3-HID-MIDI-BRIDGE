use super::*;
use s4mk3_hid_midi_bridge::screen_motor_diag::{Capture, Case, Role};

#[test]
fn capture_keeps_native_report_length_payload_and_operation_identity() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("s4-native-input-{}.jsonl", std::process::id()));
    let capture = Capture::start(&path, Case::Normal)?;
    {
        let _guard = capture.enter(Role::Input);
        let mut report = [0; 79];
        let length = native_read(1, &mut report, |report| {
            report.fill(0x5a);
            report[0] = 1;
            Ok(79)
        })?;
        assert_eq!(length, 79);
        assert_eq!(normalized_length(&report, true)?, 23);
    }
    capture.finish(true)?;
    let rows = std::fs::read_to_string(&path)?
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()?;
    let begin = rows
        .iter()
        .find(|row| row["kind"] == "input_begin")
        .context("begin")?;
    let end = rows
        .iter()
        .find(|row| row["kind"] == "input_end")
        .context("end")?;
    assert_eq!(end["values"][0], begin["seq"]);
    assert_eq!(end["values"][2], 1);
    assert_eq!(end["values"][3], 79);
    assert_eq!(end["payload"][0], 1);
    assert_eq!(end["payload"][78], 0x5a);
    std::fs::remove_file(path)?;
    Ok(())
}
