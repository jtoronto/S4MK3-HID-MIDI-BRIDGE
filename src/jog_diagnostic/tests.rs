use super::*;

#[test]
fn invalid_live_trial_fails_before_device_or_file_access() -> anyhow::Result<()> {
    // Given a live command without authorization and an unwritable path.
    let args = Args::try_parse_from([
        "diagnostic",
        "live",
        "--out",
        "/invalid/never-created.jsonl",
        "--mode",
        "assist",
        "--seconds",
        "5",
    ])?;
    // When the command runs.
    let error = run(args).err().context("missing motor opt-in must fail")?;
    // Then the validation gate, not HID or file creation, is the failure.
    assert!(error.to_string().contains("--enable-motor"));
    Ok(())
}

#[test]
fn cli_rejects_unbounded_and_invalid_control_values() {
    // Given out-of-range diagnostic options.
    for (flag, value) in [
        ("--seconds", "0"),
        ("--seconds", "31"),
        ("--gain", "0"),
        ("--gain", "101"),
        ("--max-force", "0"),
        ("--max-force", "60001"),
    ] {
        // When the standalone argument parser receives them.
        let result = Args::try_parse_from([
            "diagnostic",
            "live",
            "--out",
            "unused.jsonl",
            "--mode",
            "assist",
            "--enable-motor",
            flag,
            value,
        ]);
        // Then no live command can be constructed.
        assert!(result.is_err());
    }
}

#[test]
fn live_ceiling_is_stricter_than_offline_replay() -> anyhow::Result<()> {
    // Given a force supported for offline analysis but not a live trial.
    let args = Args::try_parse_from([
        "diagnostic",
        "live",
        "--out",
        "/invalid/never-created.jsonl",
        "--mode",
        "assist",
        "--enable-motor",
        "--max-force",
        "501",
    ])?;
    // When validation runs.
    let error = run(args).err().context("live cap must fail")?;
    // Then it fails before touching the unwritable path or HID.
    assert!(error.to_string().contains("<=500"));
    Ok(())
}

#[test]
fn raw_json_rejects_unrecognized_fields() {
    // Given a typo in a raw-frame schema.
    let raw = r#"{"host_us":0,"buttons":[],"jog":[],"ignored":true}"#;
    // When evidence is parsed, then the error is visible.
    assert!(serde_json::from_str::<capture::RawFrame>(raw).is_err());
}

#[test]
fn fixed_replay_force_ceiling_is_preserved_before_file_access() -> anyhow::Result<()> {
    // Given a value now parseable for fader mode but invalid for a fixed mode.
    let args = Args::try_parse_from([
        "diagnostic",
        "replay",
        "--input",
        "/invalid/never-opened.jsonl",
        "--mode",
        "brake",
        "--max-force",
        "1001",
    ])?;
    // When mode-specific options are resolved by the real command.
    let error = run(args).err().context("fixed force cap must fail")?;
    // Then the original fixed-mode ceiling rejects it before opening evidence.
    assert!(error.to_string().contains("<=1000"));
    Ok(())
}

#[test]
fn incompatible_fader_and_continuous_flags_fail_before_hid() -> anyhow::Result<()> {
    // Given authorized-looking live commands with incompatible controls.
    for (mode, extra) in [
        ("fader-brake", &["--gain", "1"][..]),
        ("fader-brake", &["--side", "right"][..]),
        ("brake", &["--until-stopped"][..]),
    ] {
        let mut arguments = vec![
            "diagnostic",
            "live",
            "--out",
            "/invalid/never-created.jsonl",
            "--mode",
            mode,
            "--enable-motor",
        ];
        arguments.extend_from_slice(extra);
        let args = Args::try_parse_from(arguments)?;
        // When the real command validates the selected mode.
        let error = run(args).err().context("incompatible flag must fail")?;
        // Then rejection is specific to that flag, before file/device access.
        assert!(error.to_string().contains(extra[0]));
    }
    Ok(())
}

#[test]
fn fader_defaults_and_explicit_duration_conflicts_are_resolved() -> anyhow::Result<()> {
    // Given the supervised interactive invocation with no optional tuning.
    let args = Args::try_parse_from([
        "diagnostic",
        "live",
        "--out",
        "unused.jsonl",
        "--mode",
        "fader-brake",
        "--enable-motor",
        "--until-stopped",
    ])?;
    // When the typed CLI is resolved without running any hardware command.
    match args.command {
        Command::Live {
            control,
            until_stopped,
            ..
        } => {
            let control = control.resolve()?;
            // Then fader defaults are meaningful and continuous mode was requested.
            assert_eq!(control.max_force, 5000);
            assert_eq!(control.gain, 1);
            assert!(until_stopped);
        }
        Command::Capture { .. }
        | Command::Replay { .. }
        | Command::PlatterMeasure(_)
        | Command::PlatterDrive(_)
        | Command::PlatterRotate(_)
        | Command::PlatterRing(_) => {
            anyhow::bail!("expected live arguments");
        }
    }
    assert!(
        Args::try_parse_from([
            "diagnostic",
            "live",
            "--out",
            "unused.jsonl",
            "--mode",
            "fader-brake",
            "--enable-motor",
            "--until-stopped",
            "--seconds",
            "10",
        ])
        .is_err()
    );
    Ok(())
}
