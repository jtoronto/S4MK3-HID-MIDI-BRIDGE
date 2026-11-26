use super::*;
use s4mk3_hid_midi_bridge::screen_motor_diag::{Capture, Case, Role};

#[test]
fn trace_preserves_logical_write_and_records_actual_padded_native_result() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("s4-native-diag-{}.jsonl", std::process::id()));
    let capture = Capture::start(&path, Case::Normal)?;
    let report = [0x31; 11];
    {
        let _guard = capture.enter(Role::Output);
        let written = write_with(&report, true, |bytes| {
            assert_eq!(bytes, report);
            Ok(95)
        })?;
        assert_eq!(written, 11);
    }
    capture.finish(true)?;
    let rows = std::fs::read_to_string(&path)?
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()?;
    let begin = rows
        .iter()
        .find(|row| row["kind"] == "native_write_begin")
        .context("begin")?;
    let end = rows
        .iter()
        .find(|row| row["kind"] == "native_write_end")
        .context("end")?;
    assert_eq!(end["values"][0], begin["seq"]);
    assert_eq!(end["values"][1], 1);
    assert_eq!(end["values"][2], 95);
    assert_eq!(begin["payload"], serde_json::json!(report.to_vec()));
    std::fs::remove_file(path)?;
    Ok(())
}
