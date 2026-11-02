use super::*;

const fn settings(target_rpm: u16) -> crate::jog::platter::Settings {
    crate::jog::platter::Settings {
        target_rpm,
        overspeed_limit_rpm: Some(0),
        max_force: 20_000,
        base_force: 1650,
        force_per_rpm: 110,
        kp: 4000,
        ki: 3000,
        startup_ms: 200,
        force_rise_per_second: 200_000,
        startup_force_rise_per_second: 0,
        count_direction: 1,
    }
}

fn running() -> anyhow::Result<Engine> {
    let mut engine = Engine::new(selected(100)?);
    engine.configure_playback(settings(33));
    assert_eq!(
        engine.set_playback(PlaybackState {
            enabled: [true; 2],
            owned: [false; 2],
            continuous: [true; 2],
        }),
        None
    );
    for index in 0..=40 {
        observe(&mut engine, index, index * 1584 / 500, false)?;
    }
    Ok(engine)
}

fn observe(engine: &mut Engine, index: u32, counts: u32, touch: bool) -> anyhow::Result<[u8; 11]> {
    let at = u64::from(index) * 2000 + 1;
    let (buttons, jog) = sample(at, index * 200_000 + 1, [counts; 2], [touch; 2]);
    engine.observe(&buttons, &jog, at, at)?;
    engine.tick(at)
}

#[test]
fn captured_handsfree_ripple_keeps_regulation_without_assistance_restarts() -> anyhow::Result<()> {
    // Recorded right-wheel endpoints, 14:57:33.027–34.010 UTC on November 2.
    // Columns are host microseconds, sensor timer and absolute encoder count.
    // Mirror the same untouched movement to both sides through the real decoder.
    let mut engine = running()?;
    engine.configure_playback(crate::jog::platter::Settings {
        base_force: 7000,
        ..settings(33)
    });
    engine.clear_motion();
    for line in include_str!("handsfree-ripple.csv").lines() {
        let fields: Vec<u64> = line.split(',').map(str::parse).collect::<Result<_, _>>()?;
        let [at, timer, counter] = fields.as_slice() else {
            anyhow::bail!("invalid captured encoder row");
        };
        let (buttons, jog) = sample(
            *at,
            u32::try_from(*timer)?,
            [u32::try_from(*counter)?; 2],
            [false; 2],
        );
        engine.observe(&buttons, &jog, *at, *at)?;
        engine.tick(*at)?;
        if *at >= 22_120_000 {
            for wheel in engine.playback.as_ref().context("playback wheels")? {
                assert_eq!(
                    wheel.diagnostic(true).mode,
                    playback::Mode::Regulating,
                    "hands-free movement restarted assistance at {at}",
                );
            }
        }
    }
    Ok(())
}

#[test]
fn sustained_rim_braking_confirms_assistance_then_recovers_without_readmission()
-> anyhow::Result<()> {
    let mut engine = running()?;
    for index in 41..=100 {
        observe(&mut engine, index, index * 1584 / 500, false)?;
    }
    // An untouched step to 30 RPM crosses the 100ms threshold after 68ms in
    // this quantized fixture, rather than on its first short-window dip.
    let mut first_assist = None;
    for index in 101..=180 {
        observe(&mut engine, index, 316 + (index - 100) * 144 / 50, false)?;
        let mode = engine.playback.as_ref().context("playback wheels")?[0]
            .diagnostic(true)
            .mode;
        if mode == playback::Mode::Assisting && first_assist.is_none() {
            first_assist = Some(index);
        }
    }
    assert_eq!(first_assist, Some(134));
    for index in 181..=240 {
        let report = observe(&mut engine, index, 546, false)?;
        assert_eq!(magnitude(report, 0), 5280);
        assert_eq!(magnitude(report, 1), 5280);
    }
    // Recovery uses the short window: the still-low 100ms history cannot
    // hold assistance or admit it again once current speed has recovered.
    for index in 241..=280 {
        observe(&mut engine, index, 546 + (index - 240) * 16 / 5, false)?;
        let data = serde_json::to_value(
            engine.playback.as_ref().context("playback wheels")?[0].diagnostic(true),
        )?;
        if index == 250 {
            assert_eq!(data["mode"], "qualifying");
            assert!(data["assistance_rpm"].as_f64().context("long RPM")? < 31.0);
        }
        if index >= 250 {
            assert_ne!(data["mode"], "assisting");
        }
        if index >= 260 {
            assert_eq!(data["mode"], "regulating");
        }
    }
    Ok(())
}

#[test]
fn midi_ownership_cannot_bypass_missing_or_exact_threshold_long_window() -> anyhow::Result<()> {
    let mut engine = running()?;
    engine.clear_motion();
    assert_eq!(
        engine.set_playback(PlaybackState {
            enabled: [true; 2],
            owned: [true; 2],
            continuous: [true; 2],
        }),
        None
    );
    let mut counter = 0;
    for index in 0_u32..=8 {
        if index > 0 {
            counter += [47, 47, 46, 46][usize::try_from((index - 1) % 4)?];
        }
        let at = 1_000_001 + u64::from(index) * 31_250;
        let (buttons, jog) = sample(at, 1 + index * 3_125_000, [counter; 2], [false; 2]);
        engine.observe(&buttons, &jog, at, at)?;
        engine.tick(at)?;
        let data = serde_json::to_value(
            engine.playback.as_ref().context("playback wheels")?[0].diagnostic(true),
        )?;
        if index > 0 {
            assert_eq!(data["mode"], "regulating");
        }
        if index < 4 {
            assert!(data["assistance_rpm"].is_null());
        } else {
            assert!((data["assistance_rpm"].as_f64().context("long RPM")? - 31.0).abs() < 1e-9);
        }
    }
    Ok(())
}

#[test]
fn recovered_short_threshold_and_reset_reject_stale_long_braking_evidence() -> anyhow::Result<()> {
    let mut engine = running()?;
    engine.configure_playback(settings(32));
    engine.clear_motion();
    assert_eq!(
        engine.set_playback(PlaybackState {
            enabled: [true; 2],
            owned: [true; 2],
            continuous: [true; 2],
        }),
        None
    );
    // 25 RPM qualifies sustained assistance, then 33.33 RPM releases it.
    // The next exact 30 RPM short window must not re-enter with a low long RPM.
    for (index, counter) in [0, 30, 60, 90, 120, 160, 196].into_iter().enumerate() {
        let index = u32::try_from(index)?;
        let at = 1_000_001 + u64::from(index) * 25_000;
        let timer = (u32::MAX - 1_000_000).wrapping_add(index * 2_500_000);
        let (buttons, jog) = sample(at, timer, [counter; 2], [false; 2]);
        engine.observe(&buttons, &jog, at, at)?;
        engine.tick(at)?;
        let data = serde_json::to_value(
            engine.playback.as_ref().context("playback wheels")?[0].diagnostic(true),
        )?;
        if index == 4 {
            assert_eq!(data["mode"], "assisting");
        }
        if index == 6 {
            assert_eq!(data["mode"], "regulating");
            assert!((data["motor_rpm"].as_f64().context("short RPM")? - 30.0).abs() < 1e-9);
            assert!(data["assistance_rpm"].as_f64().context("long RPM")? < 30.0);
        }
    }
    engine.clear_motion();
    for index in 0_u32..=2 {
        let at = 2_000_001 + u64::from(index) * 25_000;
        let (buttons, jog) = sample(at, 1 + index * 2_500_000, [index * 36; 2], [false; 2]);
        engine.observe(&buttons, &jog, at, at)?;
        engine.tick(at)?;
        let data = serde_json::to_value(
            engine.playback.as_ref().context("playback wheels")?[0].diagnostic(true),
        )?;
        assert!(data["assistance_rpm"].is_null());
        if index > 0 {
            assert_eq!(data["mode"], "regulating");
        }
    }
    Ok(())
}

#[test]
fn upper_scratch_without_touch_keeps_speed_regulation() -> anyhow::Result<()> {
    // Given steady TT regulation and an upper-speed MIDI Scratch owner.
    let mut engine = running()?;
    assert_eq!(
        engine.set_playback(PlaybackState {
            enabled: [true; 2],
            owned: [true; 2],
            continuous: [true; 2],
        }),
        None
    );
    // When a released forward nudge remains above target without physical touch.
    let mut report = neutral_report();
    for index in 41..=90 {
        report = observe(&mut engine, index, 126 + (index - 40) * 6, false)?;
    }
    // Then ownership alone cannot replace regulation with constant forward force.
    assert_eq!(magnitude(report, 0), 0);
    assert_eq!(magnitude(report, 1), 0);
    Ok(())
}

#[test]
fn touched_overspeed_and_release_never_select_forward_assistance() -> anyhow::Result<()> {
    let mut engine = running()?;
    for index in 41..=110 {
        let report = observe(&mut engine, index, 126 + (index - 40) * 6, index <= 90)?;
        if index >= 55 {
            assert_eq!(magnitude(report, 0), 0, "sample {index}");
            assert_eq!(magnitude(report, 1), 0, "sample {index}");
        }
    }
    Ok(())
}

#[test]
fn touched_near_target_tapers_force_from_the_measured_sensor_window() -> anyhow::Result<()> {
    let mut engine = running()?;
    let mut report = neutral_report();
    for index in 41..=90 {
        report = observe(&mut engine, index, 126 + (index - 40) * 31 / 10, true)?;
    }
    // 31 counts / (0.020 seconds * 48 counts/RPM) = 32.291666... RPM.
    // The remaining 0.708333... RPM over the 2 RPM taper gives 1870.
    assert_eq!(magnitude(report, 0), 1870);
    assert_eq!(magnitude(report, 1), 1870);
    Ok(())
}

#[test]
fn reverse_and_held_stop_keep_constant_assistance_without_other_motor_owners() -> anyhow::Result<()>
{
    // Given qualified TT regulation above maximum positive normal tension.
    let mut engine = running()?;
    // When an untouched wheel reverses then remains stopped for one second.
    let mut held_count = 126_u32;
    for index in 41..=650 {
        if index <= 100 {
            held_count = held_count.wrapping_sub(4);
        }
        let report = observe(&mut engine, index, held_count, false)?;
        // Then measured braking selects only bounded forward assistance, never tension.
        if index >= 55 {
            assert_eq!(magnitude(report, 0), 5280);
            assert_eq!(magnitude(report, 1), 5280);
        }
    }
    Ok(())
}

#[test]
fn fresh_recovery_under_touch_requires_a_new_sensor_window() -> anyhow::Result<()> {
    // Given prior drive and cleared stale history with physical touch still held.
    let mut engine = running()?;
    engine.clear_motion();
    // When fresh stationary sensor reports establish a new baseline/window.
    for index in 100..=110 {
        let report = observe(&mut engine, index, 126, true)?;
        // Then old force cannot escape before the full new 20ms window.
        assert_eq!(magnitude(report, 0), if index == 110 { 5280 } else { 0 });
    }
    Ok(())
}

#[test]
fn duplicate_contact_reports_cannot_extend_the_force_deadline() -> anyhow::Result<()> {
    // Given constant assistance from a physically held wheel.
    let mut engine = running()?;
    observe(&mut engine, 41, 126, true)?;
    let deadline = engine.input_deadline_us();
    let (buttons, jog) = sample(82_001, 8_200_001, [126; 2], [true; 2]);
    // When host reports continue but both sensor timer/counter values are frozen.
    for index in 42_u32..=60 {
        let at = u64::from(index) * 2000 + 1;
        engine.observe(&buttons, &jog, at, at)?;
    }
    // Then the original sensor deadline remains decisive even under contact.
    assert_eq!(engine.input_deadline_us(), deadline);
    assert!(engine.tick(deadline + 1).is_err());
    Ok(())
}

#[test]
fn regulator_payload_reports_real_windows_and_clears_inactive_pi_state() -> anyhow::Result<()> {
    // Given a real decoder/regulator at the fixture's nominal rotation.
    let mut engine = running()?;
    let regulated = serde_json::to_value(
        engine.playback.as_ref().context("playback wheels")?[0].diagnostic(true),
    )?;
    assert_eq!(regulated["mode"], "regulating");
    assert_eq!(regulated["window_counts"], 31);
    assert_eq!(regulated["window_ticks"], 2_000_000);
    // Then exact window travel explains RPM and the actual PI force components.
    let rpm = regulated["window_counts"]
        .as_f64()
        .context("window counts")?
        * 6_000_000_000.0
        / regulated["window_ticks"].as_f64().context("window ticks")?
        / 2880.0;
    assert!((regulated["motor_rpm"].as_f64().context("motor RPM")? - rpm).abs() < 1e-9);
    let regulator = &regulated["regulator"];
    let error = 33.0 - regulator["measured_rpm"].as_f64().context("PI RPM")?;
    assert!(
        error
            .mul_add(
                -4000.0,
                regulator["proportional_force"]
                    .as_f64()
                    .context("P force")?
            )
            .abs()
            < 1e-9
    );
    assert_eq!(regulator["feed_forward_force"], 5280.0);
    let components = regulator["feed_forward_force"]
        .as_f64()
        .context("baseline")?
        + regulator["proportional_force"]
            .as_f64()
            .context("proportional")?
        + regulator["integral_force"].as_f64().context("integral")?;
    assert!(
        (regulator["unclamped_force"].as_f64().context("force sum")? - components).abs() < 1e-9
    );
    assert_eq!(regulator["force"], magnitude(engine.tick(80_001)?, 0));
    // When contact transfers force to assistance, old PI terms must disappear.
    let mut report = neutral_report();
    for index in 41..=90 {
        report = observe(&mut engine, index, 126 + (index - 40) * 31 / 10, true)?;
    }
    let held = serde_json::to_value(
        engine.playback.as_ref().context("playback wheels")?[0].diagnostic(true),
    )?;
    assert_eq!(held["mode"], "assisting");
    assert_eq!(held["window_counts"], 31);
    assert_eq!(held["window_ticks"], 2_000_000);
    assert_eq!(magnitude(report, 0), 1870);
    assert!(
        held["regulator"].is_null(),
        "inactive PI state must not look current"
    );
    Ok(())
}
