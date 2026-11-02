use std::process::Command;

use anyhow::Context as _;

#[test]
fn tension_config_rejects_nonruntime_profiles_before_hid() -> anyhow::Result<()> {
    // Given commands that must never activate runtime hardware tension.
    for args in [
        vec!["--midi", "minimal"],
        vec!["--list"],
        vec!["--generate-mapping", "/dev/null/not-created.plist"],
        vec![],
    ] {
        // When the actual CLI receives the runtime-only configuration option.
        let output = Command::new(env!("CARGO_BIN_EXE_s4mk3-hid-midi-bridge"))
            .args(args)
            .args(["--tension-config", "/dev/null/not-read.json"])
            .output()?;
        // Then it fails argument/profile admission before scanning or opening HID.
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8(output.stderr)?.contains("S4_HID_OPEN"));
    }
    Ok(())
}

#[test]
fn full_runtime_validates_config_before_enumeration_or_hid() -> anyhow::Result<()> {
    // Given an unreadable configuration path with an otherwise valid full profile.
    // When the actual CLI starts.
    let output = Command::new(env!("CARGO_BIN_EXE_s4mk3-hid-midi-bridge"))
        .args([
            "--midi",
            "full",
            "--until-stopped",
            "--tension-config",
            "/dev/null/not-read.json",
        ])
        .output()?;
    // Then configuration loading fails before it can enumerate/open the controller.
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error = String::from_utf8(output.stderr)?;
    assert!(error.contains("/dev/null/not-read.json"));
    assert!(!error.contains("S4_HID_OPEN"));
    Ok(())
}

#[test]
fn offline_replay_emits_synchronized_regulator_and_bend_diagnostics() -> anyhow::Result<()> {
    // Given the real executable's hardware-free playback fixture.
    let output = Command::new(env!("CARGO_BIN_EXE_s4mk3-hid-midi-bridge"))
        .args(["--replay-motor-fixture", "playback"])
        .env("RUST_LOG", "debug")
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // When Debug output is parsed, all motor modes must have real JSON records.
    let stderr = String::from_utf8(output.stderr)?;
    let samples: Vec<serde_json::Value> = stderr
        .lines()
        .filter_map(|line| {
            line.split_once("MOTOR_REGULATOR data=")
                .map(|(_, data)| data)
        })
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    let modes: std::collections::BTreeSet<_> = samples
        .iter()
        .map(|sample| sample["mode"].as_str().context("regulator mode"))
        .collect::<anyhow::Result<_>>()?;
    assert_eq!(
        modes,
        ["assisting", "disabled", "qualifying", "regulating"].into()
    );
    let mut previous: [Option<(u64, &str)>; 2] = [None; 2];
    for sample in &samples {
        let side = usize::try_from(sample["side"].as_u64().context("side")?)?;
        let at = sample["host_us"].as_u64().context("sample time")?;
        let mode = sample["mode"].as_str().context("sample mode")?;
        if let Some((before, old_mode)) = previous[side]
            && old_mode == mode
        {
            assert!(at - before >= 20_000, "unchanged mode exceeded 50 Hz");
        }
        previous[side] = Some((at, mode));
        assert_eq!(
            sample["sensor_timer"].as_u64(),
            Some((at * 100) % (1_u64 << 32))
        );
    }
    // Then the independent bend frames share the same replay clock.
    let rows: Vec<serde_json::Value> = String::from_utf8(output.stdout)?
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    let statuses: Vec<_> = rows
        .iter()
        .filter(|row| row["event"] == "platter_status")
        .collect();
    assert!(!statuses.is_empty(), "missing bend status stream");
    for row in statuses {
        assert_eq!(row["status"]["host_us"], row["now_us"]);
    }
    Ok(())
}
