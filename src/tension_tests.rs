use super::*;

#[path = "tension_tests/backspin.rs"]
mod backspin;
#[path = "tension_tests/continuous.rs"]
mod continuous;

fn sample(at: u64, timer: u32, counts: [u32; 2], touch: [bool; 2]) -> ([u8; 23], [u8; 60]) {
    let mut buttons = [0; 23];
    buttons[0] = 1;
    buttons[17] = (if touch[0] { 0x10 } else { 0 }) | (if touch[1] { 0x20 } else { 0 });
    let mut jog = [0; 60];
    jog[0] = 3;
    for (offset, value) in [(8, timer), (12, counts[0]), (36, timer), (40, counts[1])] {
        jog[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    let _ = at;
    (buttons, jog)
}

fn selected(value: i32) -> anyhow::Result<Config> {
    Ok(serde_json::from_str(&format!(r#"{{"tension":{value}}}"#))?)
}

fn prime(engine: &mut Engine, left_step: u32, right_step: u32) -> anyhow::Result<[u8; 11]> {
    prime_until(engine, left_step, right_step, 40)
}

fn prime_until(
    engine: &mut Engine,
    left_step: u32,
    right_step: u32,
    last: u32,
) -> anyhow::Result<[u8; 11]> {
    let mut report = neutral_report();
    for i in 0..=last {
        let at = u64::from(i) * 2000;
        let (buttons, jog) = sample(
            at,
            i + 1,
            [1000 + i * left_step, 2000 + i * right_step],
            [true, true],
        );
        engine.observe(&buttons, &jog, at, at)?;
        report = engine.tick(at)?;
    }
    Ok(report)
}

fn magnitude(report: [u8; 11], side: usize) -> u16 {
    u16::from_le_bytes([report[4 + side * 5], report[5 + side * 5]])
}

#[test]
fn continuous_tt_keeps_bounded_motor_assist_through_touch_and_scratch() -> anyhow::Result<()> {
    let mut engine = Engine::new(selected(-100)?);
    engine.configure_playback(crate::jog::platter::Settings {
        target_rpm: 33,
        overspeed_limit_rpm: None,
        max_force: 20_000,
        base_force: 1_650,
        force_per_rpm: 110,
        kp: 4_000,
        ki: 3_000,
        startup_ms: 200,
        force_rise_per_second: 200_000,
        startup_force_rise_per_second: 0,
        count_direction: 1,
    });
    let state = PlaybackState {
        enabled: [true, false],
        owned: [false; 2],
        continuous: [true, false],
    };
    assert_eq!(engine.set_playback(state), None);
    // Given fresh sensor time, touch alone must not bypass the recovery window.
    for index in 0..10_u32 {
        let at = 1 + u64::from(index) * 2000;
        let (buttons, jog) = sample(at, 1 + index * 200_000, [0; 2], [true, false]);
        engine.observe(&buttons, &jog, at, at)?;
        assert_eq!(magnitude(engine.tick(at)?, 0), 0);
    }
    let (buttons, jog) = sample(20_001, 2_000_001, [0; 2], [true, false]);
    engine.observe(&buttons, &jog, 20_001, 20_001)?;
    let assist = magnitude(engine.tick(20_001)?, 0);
    assert_eq!(assist, 5280);

    let owned = PlaybackState {
        owned: [true, false],
        ..state
    };
    assert_eq!(engine.set_playback(owned), None);
    let (buttons, jog) = sample(22_001, 2_200_001, [0; 2], [false, false]);
    engine.observe(&buttons, &jog, 22_001, 22_001)?;
    assert_eq!(magnitude(engine.tick(22_001)?, 0), assist);

    let stopped = PlaybackState {
        enabled: [false; 2],
        ..owned
    };
    let neutral = engine
        .set_playback(stopped)
        .context("stop gate neutralizes continuous motor")?;
    assert_eq!(magnitude(neutral, 0), 0);
    for index in 1..=5 {
        let at = 22_001 + u64::from(index) * 2000;
        let (buttons, jog) = sample(at, 2_200_001 + index * 200_000, [index * 2; 2], [true; 2]);
        engine.observe(&buttons, &jog, at, at)?;
        assert_eq!(magnitude(engine.tick(at)?, 0), 0);
    }
    Ok(())
}

#[test]
fn config_rejects_noninteger_unknown_and_out_of_range_selection() -> anyhow::Result<()> {
    // Given defaults and all boundary values, when JSON is deserialized,
    // then only a valid saved integer survives.
    assert_eq!(Config::load(None)?.value(), 0);
    assert_eq!(serde_json::from_str::<Config>("{}")?.value(), 0);
    for (input, value) in [(r#"{"tension":-100}"#, -100), (r#"{"tension":100}"#, 100)] {
        assert_eq!(serde_json::from_str::<Config>(input)?.value(), value);
    }
    for input in [
        r#"{"tension":-101}"#,
        r#"{"tension":101}"#,
        r#"{"tension":null}"#,
        r#"{"tension":1.0}"#,
        r#"{"tension":"5"}"#,
        r#"{"extra":true}"#,
        r#"{"tension":1,"tension":2}"#,
        "null",
        "{",
    ] {
        assert!(serde_json::from_str::<Config>(input).is_err(), "{input}");
    }
    Ok(())
}

#[test]
fn load_reads_saved_config_without_fader_or_hardware() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("s4mk3-tension-test-{}.json", std::process::id()));
    // Given a valid saved value, when the file is loaded, then the value
    // is independent of any controller report.
    std::fs::write(&path, r#"{"tension":-37}"#)?;
    let loaded = Config::load(Some(&path));
    std::fs::remove_file(&path)?;
    assert_eq!(loaded?.value(), -37);
    Ok(())
}

#[test]
fn signed_caps_are_independent_for_both_wheels() -> anyhow::Result<()> {
    for (setting, cap, negative) in [(-100, 7000, false), (100, 30000, true), (0, 0, false)] {
        let mut engine = Engine::new(selected(setting)?);
        // Given independently moving wheels at full scale, when both tick,
        // then each request stays within the selected signed cap.
        let output = prime_until(&mut engine, 2, 3, 200)?;
        assert_eq!(magnitude(output, 0), cap);
        assert_eq!(magnitude(output, 1), cap);
        assert_eq!(output[3] == 0xfe, negative && cap > 0);
        assert_eq!(output[8] == 0xfe, negative && cap > 0);
    }
    let mut half = Engine::new(selected(-50)?);
    assert_eq!(magnitude(prime_until(&mut half, 2, 0, 200)?, 0), 3500);
    let mut half = Engine::new(selected(50)?);
    assert_eq!(magnitude(prime_until(&mut half, 2, 0, 200)?, 0), 15000);
    Ok(())
}

#[test]
fn release_reversal_and_rest_zero_only_affected_wheel() -> anyhow::Result<()> {
    let mut engine = Engine::new(selected(-100)?);
    assert!(magnitude(prime(&mut engine, 2, 2)?, 0) > 0);
    // Given both moving, when left touch releases, then left zeroes
    // immediately and right retains qualified assistance.
    let (mut buttons, jog) = sample(82000, 42, [1082, 2082], [false, true]);
    let partial = engine
        .observe(&buttons, &jog, 82000, 82000)?
        .context("release gate")?;
    assert_eq!(magnitude(partial, 0), 0);
    assert!(magnitude(partial, 1) > 0);
    assert!(magnitude(engine.tick(82000)?, 1) > 0);
    // When released motion stops and touch returns without movement,
    // then retouch alone never reactivates left force.
    buttons[17] = 0x30;
    let (_, still) = sample(84000, 43, [1082, 2084], [true, true]);
    engine.observe(&buttons, &still, 84000, 84000)?;
    assert_eq!(magnitude(engine.tick(84000)?, 0), 0);
    let (_, reverse) = sample(86000, 44, [1082, 2082], [true, true]);
    let partial = engine
        .observe(&buttons, &reverse, 86000, 86000)?
        .context("reversal gate")?;
    assert_eq!(magnitude(partial, 1), 0);
    assert_eq!(magnitude(engine.tick(86000)?, 1), 0);
    Ok(())
}

#[test]
fn stale_frames_and_watchdog_reject_without_refreshing_state() -> anyhow::Result<()> {
    let mut engine = Engine::new(selected(100)?);
    let (buttons, jog) = sample(0, 1, [0, 0], [true, true]);
    assert!(engine.observe(&buttons, &jog, 0, 1).is_ok());
    assert!(engine.observe(&buttons, &jog, 1, 0).is_err());
    assert!(engine.observe(&buttons, &jog, 1, 50002).is_err());
    assert_eq!(engine.input_deadline_us(), 50000);
    assert_eq!(engine.tick(50000)?, neutral_report());
    assert!(engine.tick(50001).is_err());
    Ok(())
}

#[test]
fn new_engine_cannot_replay_prior_motion_or_stationary_retouch() -> anyhow::Result<()> {
    let config = selected(-100)?;
    let mut previous = Engine::new(config);
    assert!(magnitude(prime(&mut previous, 2, 0)?, 0) > 0);
    // Given a new engine at the same saved setting, when stationary
    // snapshots arrive, then no old motion or output history is retained.
    let mut restarted = Engine::new(config);
    for i in 0..=10 {
        let at = u64::from(i) * 2000;
        let (buttons, jog) = sample(at, i + 1, [1080, 2000], [true, true]);
        assert_eq!(restarted.observe(&buttons, &jog, at, at)?, None);
        assert_eq!(restarted.tick(at)?, neutral_report());
    }
    Ok(())
}

#[test]
fn clearing_motion_requires_a_new_baseline_and_preserves_saved_force_cap() -> anyhow::Result<()> {
    // Given qualified assistance at a saved half-strength setting.
    let mut engine = Engine::new(selected(-50)?);
    assert!(magnitude(prime(&mut engine, 2, 2)?, 0) > 0);
    // When the output owner clears history after neutralizing both wheels.
    engine.clear_motion();
    // Then clearing state does not make stale input fresh, and a distant
    // counter/timer baseline or stationary touch cannot restore old force.
    assert!(engine.tick(140_001).is_err());
    for i in 0..=10 {
        let at = 140_001 + u64::from(i) * 2000;
        let (buttons, jog) = sample(at, i + 1, [9000, 3000], [true, true]);
        assert_eq!(engine.observe(&buttons, &jog, at, at)?, None);
        assert_eq!(engine.tick(at)?, neutral_report());
    }
    let mut last = neutral_report();
    for i in 0..=200 {
        let at = 162_001 + u64::from(i) * 2000;
        let (buttons, jog) = sample(at, i + 12, [9000 + i * 2, 3000], [true, true]);
        engine.observe(&buttons, &jog, at, at)?;
        last = engine.tick(at)?;
        assert_eq!(magnitude(last, 1), 0);
        assert!(magnitude(last, 0) <= 3500);
    }
    assert_eq!(magnitude(last, 0), 3500);
    Ok(())
}

#[test]
fn rest_and_duplicate_do_not_extend_qualified_force() -> anyhow::Result<()> {
    let mut engine = Engine::new(selected(100)?);
    assert!(magnitude(prime(&mut engine, 2, 0)?, 0) > 0);
    // Given qualified braking, when sensor time advances without counts,
    // then the independent 8 ms inactivity cutoff bypasses smoothing.
    for (at, timer) in [(82000, 42), (84000, 43), (86000, 44)] {
        let (buttons, jog) = sample(at, timer, [1080, 2000], [false, false]);
        engine.observe(&buttons, &jog, at, at)?;
        assert!(magnitude(engine.tick(at)?, 0) > 0);
    }
    let (buttons, jog) = sample(88000, 45, [1080, 2000], [true, true]);
    let immediate = engine
        .observe(&buttons, &jog, 88000, 88000)?
        .context("rest gate")?;
    assert_eq!(immediate, neutral_report());
    assert_eq!(engine.tick(88000)?, neutral_report());
    Ok(())
}

#[test]
fn malformed_snapshots_fail_before_decoder_field_access() -> anyhow::Result<()> {
    let mut engine = Engine::new(selected(-100)?);
    let (buttons, jog) = sample(0, 1, [0, 0], [true, true]);
    assert!(engine.observe(&buttons[..22], &jog, 0, 0).is_err());
    assert!(engine.observe(&buttons, &jog[..59], 0, 0).is_err());
    let mut bad_id = jog;
    bad_id[0] = 2;
    assert!(engine.observe(&buttons, &bad_id, 0, 0).is_err());
    assert_eq!(engine.tick(0)?, neutral_report());
    Ok(())
}
