use std::cell::RefCell;
use std::path::PathBuf;

use anyhow::Context as _;
use clap::Parser as _;
use num_traits::ToPrimitive as _;

use super::*;
use crate::jog_diagnostic::decode::tests::frame;
use crate::jog_diagnostic::motor::{neutral_report, report_for};
use crate::jog_diagnostic::platter::{Gate, Model, Wheel};
use crate::jog_diagnostic::platter_session::Session;
use crate::jog_diagnostic::{Args, Command};

const BASE_ROTATE_ARGS: [&str; 12] = [
    "diag",
    "platter-rotate",
    "--side",
    "left",
    "--out",
    "/invalid/never-created",
    "--polarity",
    "positive",
    "--count-direction",
    "positive",
    "--counts-per-revolution",
    "2880",
];

#[path = "platter_rotation_clock_tests.rs"]
mod clock_tests;

fn settings() -> Settings {
    Settings {
        polarity: Polarity::Positive,
        count_direction: Polarity::Positive,
        target_rpm: 33,
        overspeed_limit_rpm: None,
        max_force: 8000,
        drive_ms: 2000,
        base_force: DEFAULT_BASE_FORCE,
        force_per_rpm: DEFAULT_FORCE_PER_RPM,
        kp: DEFAULT_KP,
        ki: DEFAULT_KI,
        force_rise_per_second: DEFAULT_FORCE_RISE_PER_SECOND,
        startup_ms: DEFAULT_STARTUP_MS,
        startup_force_rise_per_second: DEFAULT_STARTUP_FORCE_RISE_PER_SECOND,
        initialize_ring: false,
    }
}

fn config(side: Wheel, settings: Settings) -> Config {
    Config {
        common: CommonOptions {
            out: PathBuf::from("unused"),
            side,
            seconds: 3,
        },
        experiment: Experiment::Rotate(settings),
        counts_per_revolution: Some(6000),
    }
}

fn input(
    at: u32,
    counts: [u32; 2],
    side: Wheel,
    pressed: bool,
    touched: bool,
) -> super::super::capture::RawFrame {
    let mut raw = frame(u64::from(at), at + 1, counts);
    let timer = at.wrapping_mul(100).to_le_bytes();
    raw.jog[8..12].copy_from_slice(&timer);
    raw.jog[36..40].copy_from_slice(&timer);
    raw.buttons[17] = 0;
    match side {
        Wheel::Left => {
            raw.buttons[7] = u8::from(pressed) * 4;
            raw.buttons[17] = u8::from(touched) * 0x10;
        }
        Wheel::Right => {
            raw.buttons[16] = u8::from(pressed) * 0x20;
            raw.buttons[17] = u8::from(touched) * 0x20;
        }
    }
    raw
}

fn arm(model: &mut Model, side: Wheel) -> anyhow::Result<()> {
    for (at, pressed) in [(0, false), (2000, true), (4000, false)] {
        model.observe(&input(at, [0; 2], side, pressed, false), u64::from(at))?;
    }
    Ok(())
}

fn rest_until(model: &mut Model, end: u32) -> anyhow::Result<State> {
    let side = model.config.common.side;
    let mut state = State::default();
    for at in (6000..=end).step_by(2000) {
        state = model
            .observe(&input(at, [0; 2], side, false, false), u64::from(at))?
            .rotation
            .context("rotation state")?;
    }
    Ok(state)
}

#[test]
fn cli_requires_explicit_calibration_direction_and_motor_opt_in() -> anyhow::Result<()> {
    assert!(Args::try_parse_from(BASE_ROTATE_ARGS).is_ok());
    // Missing opt-in is rejected before ownership checks, log creation or HID.
    assert!(crate::jog_diagnostic::run(Args::try_parse_from(BASE_ROTATE_ARGS)?).is_err());
    for removed in ["--polarity", "--count-direction", "--counts-per-revolution"] {
        let mut args = BASE_ROTATE_ARGS.to_vec();
        let index = args.iter().position(|v| *v == removed).context("flag")?;
        args.drain(index..index + 2);
        assert!(Args::try_parse_from(args).is_err());
    }
    Ok(())
}

#[test]
fn cli_rejects_rotation_parameters_outside_their_bounds() {
    for extra in [
        ("--target-rpm", "0"),
        ("--target-rpm", "46"),
        ("--target-rpm", "NaN"),
        ("--overspeed-limit-rpm", "251"),
        ("--max-force", "0"),
        ("--max-force", "25001"),
        ("--drive-ms", "0"),
        ("--drive-ms", "15001"),
        ("--base-force", "25001"),
        ("--force-per-rpm", "25001"),
        ("--kp", "25001"),
        ("--ki", "25001"),
        ("--force-rise-per-second", "1000001"),
        ("--startup-ms", "2001"),
        ("--startup-force-rise-per-second", "1000001"),
    ] {
        let mut args = BASE_ROTATE_ARGS.to_vec();
        args.extend(<[&str; 2]>::from(extra));
        assert!(Args::try_parse_from(args).is_err());
    }
}

#[test]
fn cli_resolves_default_and_custom_rotation_tuning() -> anyhow::Result<()> {
    let mut args = BASE_ROTATE_ARGS.to_vec();
    args.extend([
        "--enable-motor",
        "--max-force",
        "25000",
        "--target-rpm",
        "45",
        "--overspeed-limit-rpm",
        "90",
        "--drive-ms",
        "15000",
        "--base-force",
        "1200",
        "--force-per-rpm",
        "100",
        "--kp",
        "600",
        "--ki",
        "200",
        "--force-rise-per-second",
        "75000",
        "--startup-ms",
        "300",
        "--startup-force-rise-per-second",
        "25000",
    ]);
    let Command::PlatterRotate(options) = Args::try_parse_from(args)?.command else {
        anyhow::bail!("wrong command");
    };
    let config = options.resolve()?;
    assert_eq!(config.counts_per_revolution, Some(2880));
    assert!(matches!(
        config.experiment,
        Experiment::Rotate(Settings {
            target_rpm: 45,
            overspeed_limit_rpm: Some(90),
            max_force: 25000,
            drive_ms: 15000,
            base_force: 1200,
            force_per_rpm: 100,
            kp: 600,
            ki: 200,
            force_rise_per_second: 75000,
            startup_ms: 300,
            startup_force_rise_per_second: 25000,
            ..
        })
    ));
    let mut defaults = BASE_ROTATE_ARGS.to_vec();
    defaults.push("--enable-motor");
    let Command::PlatterRotate(options) = Args::try_parse_from(defaults)?.command else {
        anyhow::bail!("wrong command");
    };
    assert!(matches!(
        options.resolve()?.experiment,
        Experiment::Rotate(Settings {
            target_rpm: 33,
            overspeed_limit_rpm: None,
            max_force: 8000,
            drive_ms: 2000,
            base_force: DEFAULT_BASE_FORCE,
            force_per_rpm: DEFAULT_FORCE_PER_RPM,
            kp: DEFAULT_KP,
            ki: DEFAULT_KI,
            force_rise_per_second: DEFAULT_FORCE_RISE_PER_SECOND,
            startup_ms: DEFAULT_STARTUP_MS,
            startup_force_rise_per_second: DEFAULT_STARTUP_FORCE_RISE_PER_SECOND,
            ..
        })
    ));
    let mut edge = BASE_ROTATE_ARGS.to_vec();
    edge.extend(["--enable-motor", "--target-rpm", "1", "--max-force", "1"]);
    let Command::PlatterRotate(options) = Args::try_parse_from(edge)?.command else {
        anyhow::bail!("wrong command");
    };
    assert!(matches!(
        options.resolve()?.experiment,
        Experiment::Rotate(Settings {
            target_rpm: 1,
            max_force: 1,
            ..
        })
    ));
    let mut disabled = BASE_ROTATE_ARGS.to_vec();
    disabled.extend(["--enable-motor", "--overspeed-limit-rpm", "0"]);
    let Command::PlatterRotate(options) = Args::try_parse_from(disabled)?.command else {
        anyhow::bail!("wrong command");
    };
    assert!(matches!(
        options.resolve()?.experiment,
        Experiment::Rotate(Settings {
            overspeed_limit_rpm: Some(0),
            ..
        })
    ));
    Ok(())
}

#[test]
fn speed_probe_warms_then_steps_to_cap_with_steady_slew_preserved() -> anyhow::Result<()> {
    for cap in [1, 8000, 25000] {
        let mut options = settings();
        options.max_force = cap;
        if cap == 25000 {
            options.target_rpm = 45;
        }
        let mut model = Model::new(config(Wheel::Left, options));
        arm(&mut model, Wheel::Left)?;
        let mut previous = 0;
        for at in (6000..=1_900_000).step_by(2000) {
            let sample =
                model.observe(&input(at, [0; 2], Wheel::Left, false, false), u64::from(at))?;
            let state = sample.rotation.context("state")?;
            assert_eq!(sample.requested_force[1], 0);
            assert!(state.force <= cap);
            assert_eq!(state.at_force_cap, state.force == cap);
            if at > 204_000 {
                assert!(state.force <= previous + 100);
            }
            assert!(state.commanded_rpm <= f64::from(options.target_rpm));
            match at.cmp(&24_000) {
                std::cmp::Ordering::Less => {
                    assert_eq!(state.force, 0);
                    assert!(state.measured_rpm.is_none());
                }
                std::cmp::Ordering::Equal => {
                    assert_eq!(state.force, cap);
                    assert!((state.commanded_rpm - f64::from(options.target_rpm)).abs() < 1e-9);
                }
                std::cmp::Ordering::Greater => {}
            }
            previous = state.force;
        }
        assert_eq!(model.force(), [i32::from(cap), 0]);
    }
    Ok(())
}

#[test]
fn matched_thirty_three_rpm_has_expected_feedforward_command() -> anyhow::Result<()> {
    let mut model = Model::new(config(Wheel::Left, settings()));
    arm(&mut model, Wheel::Left)?;
    let mut sample = None;
    for at in (6000..=2_000_000).step_by(2000) {
        let count = u32::try_from(u64::from(at) * 3300 / 1_000_000)?;
        sample = Some(model.observe(
            &input(at, [count, 0], Wheel::Left, false, false),
            u64::from(at),
        )?);
    }
    let state = sample.context("sample")?.rotation.context("state")?;
    assert!((state.measured_rpm.context("measured speed")? - 33.0).abs() < 1e-9);
    assert!((state.commanded_rpm - 33.0).abs() < 1e-9);
    assert!((state.feed_forward_force - 5280.0).abs() < 1e-9);
    assert!((state.unclamped_force - 5280.0 - state.integral_force).abs() < 1e-9);
    assert_eq!(
        state.force,
        state.unclamped_force.round().to_u16().context("force")?
    );
    Ok(())
}

#[test]
fn overspeed_limit_defaults_customizes_and_can_be_disabled() -> anyhow::Result<()> {
    let mut maximum_target = settings();
    maximum_target.target_rpm = 45;
    assert_eq!(maximum_target.effective_overspeed_limit_rpm(), Some(95));

    for (override_limit, expected_limit, should_error) in [
        (None, Some(71), true),
        (Some(90), Some(90), true),
        (Some(101), Some(101), false),
        (Some(0), None, false),
    ] {
        let mut options = settings();
        options.overspeed_limit_rpm = override_limit;
        assert_eq!(options.effective_overspeed_limit_rpm(), expected_limit);

        let mut controller = Controller::default();
        controller.observe(0, 0, 0, 2880, options)?;
        let result = controller.observe(2_000_000, 20_000, 96, 2880, options);
        assert_eq!(result.is_err(), should_error);
        if let Err(error) = result {
            let limit = expected_limit.context("enabled overspeed limit")?;
            let message = format!("{error:#}");
            assert!(message.contains("measured 100.000 RPM"));
            assert!(message.contains(&format!("exceeds {limit} RPM")));
        }
    }
    Ok(())
}

#[test]
fn custom_feedforward_and_pi_parameters_change_the_force_law() -> anyhow::Result<()> {
    let mut options = settings();
    options.base_force = 1200;
    options.force_per_rpm = 100;
    options.kp = 600;
    options.ki = 200;
    options.force_rise_per_second = 100_000;
    options.startup_ms = 0;
    let mut controller = Controller::default();

    controller.update(30.0, 0.3, 0.002, options)?;

    assert!((controller.state.feed_forward_force - 4500.0).abs() < 1e-9);
    assert!((controller.state.integral_force - 1.2).abs() < 1e-9);
    assert!((controller.state.unclamped_force - 6301.2).abs() < 1e-9);
    assert_eq!(controller.state.force, 200);
    Ok(())
}

#[test]
fn custom_startup_force_slew_uses_the_configured_interval() -> anyhow::Result<()> {
    let mut options = settings();
    options.startup_ms = 300;
    options.startup_force_rise_per_second = 25_000;
    let mut controller = Controller::default();

    controller.update(0.0, 0.1, 0.002, options)?;
    assert_eq!(controller.state.force, 50);
    controller.update(0.0, 0.302, 0.004, options)?;

    assert_eq!(controller.state.force, 200);
    Ok(())
}

#[test]
fn acceleration_toward_target_reduces_force_below_cap() -> anyhow::Result<()> {
    let mut model = Model::new(config(Wheel::Left, settings()));
    arm(&mut model, Wheel::Left)?;
    rest_until(&mut model, 1_000_000)?;
    assert_eq!(model.force(), [8000, 0]);
    let mut sample = None;
    for at in (1_002_000..=1_060_000).step_by(2000) {
        let count = u32::try_from(u64::from(at - 1_000_000) * 3000 / 1_000_000)?;
        sample = Some(model.observe(
            &input(at, [count, 0], Wheel::Left, false, false),
            u64::from(at),
        )?);
    }
    let state = sample.context("sample")?.rotation.context("state")?;
    assert!((state.measured_rpm.context("measured speed")? - 30.0).abs() < 1e-9);
    assert!(state.force < 8000);
    assert!(state.force > 5280);
    Ok(())
}

#[test]
fn overspeed_coasts_immediately_with_unselected_wheel_neutral() -> anyhow::Result<()> {
    let mut model = Model::new(config(Wheel::Right, settings()));
    arm(&mut model, Wheel::Right)?;
    assert_eq!(rest_until(&mut model, 1_000_000)?.force, 8000);
    let mut sample = None;
    for at in (1_002_000..=1_022_000).step_by(2000) {
        let count = u32::try_from(u64::from(at - 1_000_000) * 5000 / 1_000_000)?;
        sample = Some(model.observe(
            &input(at, [0, count], Wheel::Right, false, false),
            u64::from(at),
        )?);
    }
    let state = sample.context("sample")?.rotation.context("state")?;
    assert!((state.measured_rpm.context("measured speed")? - 50.0).abs() < 0.1);
    assert_eq!(model.force(), [0; 2]);
    Ok(())
}

#[test]
fn raised_force_cap_reaches_native_report_high_byte() -> anyhow::Result<()> {
    let mut options = settings();
    options.max_force = 25000;
    options.target_rpm = 45;
    let mut model = Model::new(config(Wheel::Right, options));
    arm(&mut model, Wheel::Right)?;
    assert_eq!(rest_until(&mut model, 1_000_000)?.force, 25000);
    let report = report_for(model.force());
    assert_eq!(&report[4..6], &[0, 0]);
    assert_eq!(&report[9..11], &[0xa8, 0x61]);
    Ok(())
}

#[test]
fn signs_and_wheel_selection_are_independent() -> anyhow::Result<()> {
    for side in [Wheel::Left, Wheel::Right] {
        for polarity in [Polarity::Positive, Polarity::Negative] {
            let mut options = settings();
            options.polarity = polarity;
            options.count_direction = Polarity::Negative;
            let mut model = Model::new(config(side, options));
            arm(&mut model, side)?;
            assert!(rest_until(&mut model, 100_000)?.force > 0);
            assert_eq!(model.force()[1 - side.index()], 0);
            assert_eq!(model.force()[side.index()].signum(), polarity.sign());
            let mut counts = [0; 2];
            counts[side.index()] = u32::MAX - 1;
            // A negative raw wrap is positive speed in the specified count direction.
            model.observe(&input(102_000, counts, side, false, false), 102_000)?;
        }
    }
    Ok(())
}

#[test]
fn touch_and_expiry_reset_history_and_require_new_move_gesture() -> anyhow::Result<()> {
    let side = Wheel::Right;
    let mut model = Model::new(config(side, settings()));
    arm(&mut model, side)?;
    assert!(rest_until(&mut model, 200_000)?.force > 0);
    let touched = model.observe(&input(202_000, [0; 2], side, false, true), 202_000)?;
    assert_eq!(touched.requested_force, [0; 2]);
    assert_eq!(touched.rotation.context("state")?.force, 0);
    model.observe(&input(204_000, [0; 2], side, false, false), 204_000)?;
    assert_eq!(model.force(), [0; 2]);
    model.observe(&input(206_000, [0; 2], side, true, false), 206_000)?;
    let rearmed = model.observe(&input(208_000, [0; 2], side, false, false), 208_000)?;
    assert!(rearmed.rotation.context("state")?.measured_rpm.is_none());
    for at in (210_000..=2_208_000).step_by(2000) {
        model.observe(&input(at, [0; 2], side, false, false), u64::from(at))?;
    }
    assert!(!matches!(model.gate, Gate::Driving { .. }));
    assert_eq!(model.force(), [0; 2]);
    Ok(())
}

#[test]
fn fifteen_second_pulse_expires_and_rearms_only_after_a_fresh_move() -> anyhow::Result<()> {
    let side = Wheel::Left;
    let mut options = settings();
    options.drive_ms = 15_000;
    let mut config = config(side, options);
    config.common.seconds = 60;
    let mut model = Model::new(config);
    arm(&mut model, side)?;
    let mut sample = None;
    for at in (6000_u32..15_004_000).step_by(2000) {
        sample = Some(model.observe(&input(at, [0; 2], side, false, false), u64::from(at))?);
    }
    let before_expiry = sample.context("pre-expiry sample")?;
    assert_eq!(
        before_expiry.gate,
        Gate::Driving {
            until_us: 15_004_000
        }
    );
    assert!(before_expiry.requested_force[0] > 0);

    let expired = model.observe(&input(15_004_000, [0; 2], side, false, false), 15_004_000)?;
    assert!(!matches!(expired.gate, Gate::Driving { .. }));
    assert_eq!(expired.requested_force, [0; 2]);

    model.observe(&input(15_006_000, [0; 2], side, true, false), 15_006_000)?;
    let rearmed = model.observe(&input(15_008_000, [0; 2], side, false, false), 15_008_000)?;
    assert_eq!(
        rearmed.gate,
        Gate::Driving {
            until_us: 30_008_000
        }
    );
    assert_eq!(rearmed.requested_force, [0; 2]);
    Ok(())
}

#[test]
fn duplicate_snapshots_cannot_update_rotation_or_refresh_watchdog() -> anyhow::Result<()> {
    let mut model = Model::new(config(Wheel::Left, settings()));
    arm(&mut model, Wheel::Left)?;
    let before = rest_until(&mut model, 100_000)?;
    for at in (102_000_u32..150_000).step_by(2000) {
        let mut duplicate = input(100_000, [0; 2], Wheel::Left, false, false);
        duplicate.host_us = u64::from(at);
        let sample = model.observe(&duplicate, u64::from(at))?;
        assert_eq!(sample.rotation.context("state")?.force, before.force);
    }
    assert!(
        model
            .observe(&input(150_000, [0; 2], Wheel::Left, false, false), 150_000)
            .is_err()
    );
    Ok(())
}

#[test]
fn wrong_direction_or_overspeed_neutralizes_before_returning_error() -> anyhow::Result<()> {
    for reverse in [false, true] {
        let writes = RefCell::new(Vec::new());
        let mut log = Vec::new();
        {
            let mut session = Session::new(
                config(Wheel::Left, settings()),
                |r| {
                    writes.borrow_mut().push(r.to_vec());
                    Ok(r.len())
                },
                &mut log,
            );
            session.start()?;
            arm(&mut session.model, Wheel::Left)?;
            rest_until(&mut session.model, 200_000)?;
            session.tick(200_000)?;
            session.sample(&input(202_000, [0; 2], Wheel::Left, false, false), 202_000)?;
            let mut result = Ok(());
            for at in (204_000..=226_000).step_by(2000) {
                let movement = (at - 202_000) / 1000 * 8;
                let counts = if reverse {
                    0_u32.wrapping_sub(movement)
                } else {
                    movement
                };
                result = session.sample(
                    &input(at, [counts, 0], Wheel::Left, false, false),
                    u64::from(at),
                );
                if result.is_err() {
                    break;
                }
            }
            assert!(result.is_err());
            assert_eq!(writes.borrow().last(), Some(&neutral_report().to_vec()));
            session.finish(224_000, "guard")?;
        }
    }
    Ok(())
}

#[test]
fn tuning_record_logs_selected_controller_parameters() -> anyhow::Result<()> {
    let mut options = settings();
    options.overspeed_limit_rpm = Some(90);
    options.base_force = 1200;
    options.force_per_rpm = 100;
    options.kp = 600;
    options.ki = 200;
    options.force_rise_per_second = 75_000;
    options.startup_ms = 300;
    options.startup_force_rise_per_second = 25_000;
    let mut log = Vec::new();
    {
        let mut session = Session::new(
            config(Wheel::Left, options),
            |report| Ok(report.len()),
            &mut log,
        );
        session.start()?;
        session.finish(0, "test")?;
    }
    let records: Vec<serde_json::Value> = std::str::from_utf8(&log)?
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    let tuning = records
        .iter()
        .find(|record| record["kind"] == "platter_rotation_tuning")
        .context("rotation tuning record")?;
    assert_eq!(tuning["algorithm"], "feedforward_pi_tunable_v7");
    assert_eq!(tuning["speed_window_us"], WINDOW_US);
    assert_eq!(tuning["overspeed_limit_rpm"], serde_json::json!(90));
    assert_eq!(tuning["base_force"], serde_json::json!(1200));
    assert_eq!(tuning["force_per_rpm"], serde_json::json!(100));
    assert_eq!(
        tuning["sensor_ticks_per_us"],
        serde_json::json!(SENSOR_TICKS_PER_US)
    );
    assert_eq!(tuning["proportional_force_per_rpm"], serde_json::json!(600));
    assert_eq!(
        tuning["integral_force_per_rpm_second"],
        serde_json::json!(200)
    );
    assert_eq!(tuning["target_step"], true);
    assert_eq!(tuning["force_rise_per_second"], serde_json::json!(75_000));
    assert_eq!(tuning["startup_ms"], serde_json::json!(300));
    assert_eq!(
        tuning["startup_force_rise_per_second"],
        serde_json::json!(25_000)
    );
    assert_eq!(tuning["startup_force_step"], false);
    Ok(())
}

#[test]
fn latched_stop_prevents_a_new_rotation_tick() -> anyhow::Result<()> {
    let mut writes = Vec::new();
    let mut log = Vec::new();
    {
        let mut session = Session::new(
            config(Wheel::Left, settings()),
            |report| {
                writes.push(report.to_vec());
                Ok(report.len())
            },
            &mut log,
        );
        session.start()?;
        arm(&mut session.model, Wheel::Left)?;
        rest_until(&mut session.model, 200_000)?;
        session.tick(200_000)?;
        session.stop_request.request();
        assert_eq!(session.tick(202_000)?, Some("ctrl_c"));
        session.finish(202_000, "ctrl_c")?;
    }
    // No second non-neutral command can slip past a Stop latched during logging.
    assert_eq!(writes.len(), 3);
    assert_ne!(writes[1], neutral_report());
    assert_eq!(writes[2], neutral_report());
    Ok(())
}
