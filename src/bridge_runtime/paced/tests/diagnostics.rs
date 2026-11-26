use super::*;
use s4mk3_hid_midi_bridge::screen_motor_diag::{Capture, Case, Role};

fn exercise() -> anyhow::Result<Vec<[u8; 11]>> {
    let writes = RefCell::new(Vec::new());
    let mut engine = Engine::new(config(-77)?);
    let mut output = Output::new(|action| {
        if let Action::Motor(report) = action {
            writes.borrow_mut().push(report);
        }
        Ok(())
    })?;
    for index in 0_u32..24 {
        let at = u64::from(index) * 2000 + 1;
        output.sample(&mut engine, &frame(at, [index * 10; 2], 0x30), at)?;
        output.tick(&mut engine, at)?;
    }
    // Expiry and recovery use the same real production gates in both runs.
    output.tick(&mut engine, 100_001)?;
    output.sample(&mut engine, &frame(102_001, [500; 2], 0), 102_001)?;
    output.finish()?;
    drop(output);
    Ok(writes.into_inner())
}

#[test]
fn capture_preserves_real_force_reports_through_expiry_recovery_and_stop() -> anyhow::Result<()> {
    let path =
        std::env::temp_dir().join(format!("s4-production-diag-{}.jsonl", std::process::id()));
    let baseline = exercise()?;
    let capture = Capture::start(&path, Case::Normal)?;
    let actual = {
        let _guard = capture.enter(Role::Output);
        exercise()?
    };
    capture.finish(true)?;
    let records = std::fs::read_to_string(&path)?;
    let rows: Vec<serde_json::Value> = records
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    assert_eq!(actual, baseline);
    assert_eq!(actual.last(), Some(&neutral_report()));
    assert!(rows.iter().any(|row| row["kind"] == "input_timeout"));
    assert!(rows.iter().any(|row| row["kind"] == "input_recovered"));
    assert_eq!(rows.last().context("footer")?["incomplete"], false);
    std::fs::remove_file(path)?;
    Ok(())
}
