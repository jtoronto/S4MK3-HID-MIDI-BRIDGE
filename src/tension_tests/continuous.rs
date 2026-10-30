use super::*;

fn running() -> anyhow::Result<Engine> {
    let mut engine = Engine::new(selected(100)?);
    engine.configure_playback(crate::jog::platter::Settings {
        target_rpm: 33,
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
    });
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
