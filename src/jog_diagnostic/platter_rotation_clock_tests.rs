use super::*;

#[test]
fn startup_step_requests_full_capped_force_without_target_ramp() -> anyhow::Result<()> {
    for elapsed in [0.002, 0.1, 0.2] {
        let mut controller = Controller::default();
        controller.update(0.0, elapsed, 0.002, settings())?;
        assert!((controller.state.commanded_rpm - 33.0).abs() < 1e-9);
        assert_eq!(controller.state.force, 8000);
    }
    Ok(())
}

#[test]
fn rim_speed_disturbance_recovers_without_rearming() -> anyhow::Result<()> {
    for disturbed_rate in [2400_u32, 4500] {
        let mut model = Model::new(config(Wheel::Left, settings()));
        arm(&mut model, Wheel::Left)?;
        let mut count_numerator = 0_u64;
        for at in (6000_u32..=900_000).step_by(2000) {
            let rate = if at > 600_000 && at <= 700_000 {
                disturbed_rate
            } else {
                3300
            };
            count_numerator += u64::from(rate) * 2000;
            let count = u32::try_from(count_numerator / 1_000_000)?;
            let sample = model.observe(
                &input(at, [count, 0], Wheel::Left, false, false),
                u64::from(at),
            )?;
            assert_eq!(sample.requested_force[1], 0);
            assert!(matches!(sample.gate, Gate::Driving { .. }));
            if at == 700_000 {
                assert_eq!(
                    sample.requested_force[0],
                    if disturbed_rate == 2400 { 8000 } else { 0 }
                );
            }
            if at == 900_000 {
                let state = sample.rotation.context("rotation")?;
                assert!((state.measured_rpm.context("speed")? - 33.0).abs() < 1e-9);
                assert!(state.force > 0 && state.force < 8000);
            }
        }
    }
    Ok(())
}

#[test]
fn force_slew_resumes_only_for_time_after_startup() -> anyhow::Result<()> {
    for (elapsed, dt, expected) in [(0.201, 0.004, 50), (0.202, 0.002, 100), (0.3, 0.002, 100)] {
        let mut controller = Controller::default();
        controller.update(0.0, elapsed, dt, settings())?;
        assert_eq!(controller.state.force, expected);
    }
    Ok(())
}

#[test]
fn faster_startup_still_coasts_immediately_on_overspeed() -> anyhow::Result<()> {
    let mut controller = Controller::default();
    controller.state.force = 8000;
    controller.update(50.0, 0.1, 0.002, settings())?;
    assert_eq!(controller.state.force, 0);
    Ok(())
}

#[test]
fn sensor_velocity_is_independent_of_arrival_jitter() -> anyhow::Result<()> {
    let mut model = Model::new(config(Wheel::Left, settings()));
    arm(&mut model, Wheel::Left)?;
    let mut settled_force = None;
    for at in (6000_u32..=600_000).step_by(2000) {
        let counts = u32::try_from(u64::from(at - 4000) * 3300 / 1_000_000)?;
        let mut raw = input(at, [counts, 0], Wheel::Left, false, false);
        raw.host_us += 900 + u64::from((at / 2000) % 3) * 600;
        let timer = at.wrapping_mul(100).to_le_bytes();
        raw.jog[8..12].copy_from_slice(&timer);
        raw.jog[36..40].copy_from_slice(&timer);
        let sample = model.observe(&raw, raw.host_us)?;
        if at >= 400_000 {
            let state = sample.rotation.context("rotation")?;
            let speed = state.measured_rpm.context("warm velocity")?;
            assert!((speed - 33.0).abs() < 1e-9, "arrival-biased speed {speed}");
            assert_eq!(state.force, *settled_force.get_or_insert(state.force));
        }
    }
    Ok(())
}

#[test]
fn implausible_sensor_time_gap_is_not_treated_as_slow_motion() -> anyhow::Result<()> {
    let mut model = Model::new(config(Wheel::Left, settings()));
    arm(&mut model, Wheel::Left)?;
    let mut raw = input(6000, [0; 2], Wheel::Left, false, false);
    let timer = 5_600_000_u32.to_le_bytes();
    raw.jog[8..12].copy_from_slice(&timer);
    raw.jog[36..40].copy_from_slice(&timer);
    assert!(model.observe(&raw, raw.host_us).is_err());
    Ok(())
}

#[test]
fn sensor_clock_wrap_preserves_constant_velocity() -> anyhow::Result<()> {
    let mut model = Model::new(config(Wheel::Left, settings()));
    let origin = u32::MAX - 500_000;
    for at in (0_u32..=600_000).step_by(2000) {
        let counts = u32::try_from(u64::from(at) * 3300 / 1_000_000)?;
        let mut raw = input(at, [counts, 0], Wheel::Left, at == 2000, false);
        let timer = origin.wrapping_add(at.wrapping_mul(100)).to_le_bytes();
        raw.jog[8..12].copy_from_slice(&timer);
        raw.jog[36..40].copy_from_slice(&timer);
        let sample = model.observe(&raw, raw.host_us)?;
        if at >= 400_000 {
            let speed = sample
                .rotation
                .context("rotation")?
                .measured_rpm
                .context("speed")?;
            assert!((speed - 33.0).abs() < 1e-9);
        }
    }
    Ok(())
}

#[test]
fn pi_correction_is_linear_near_target_and_integral_does_not_wind_up() -> anyhow::Result<()> {
    let mut controller = Controller::default();
    let options = settings();
    for _ in 0..100 {
        controller.update(0.0, 1.0, 0.02, options)?;
    }
    assert_eq!(controller.state.force, 8000);
    assert!(controller.state.integral_force.abs() < f64::EPSILON);

    controller.update(30.0, 1.0, 0.02, options)?;
    assert_eq!(controller.state.force, 6639); // 5280 + 450*3 + 150*3*0.02.
    assert!((controller.state.integral_force - 9.0).abs() < 1e-9);
    controller.update(50.0, 1.0, 0.02, options)?;
    assert_eq!(controller.state.force, 0); // Coast immediately, never reverse.
    assert!((controller.state.integral_force - 9.0).abs() < 1e-9);
    Ok(())
}
