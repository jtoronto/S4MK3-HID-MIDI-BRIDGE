use std::cell::{Cell, RefCell};
use std::io::{self, Write};

use anyhow::{Context as _, bail};
use clap::Parser as _;

use super::*;
use crate::jog_diagnostic::decode::tests::frame;
use crate::jog_diagnostic::motor::{neutral_report, report_for};
use crate::jog_diagnostic::platter_session::{Session, combine};
use crate::jog_diagnostic::{Args, Command};

fn config(experiment: Experiment, side: Wheel) -> Config {
    Config {
        common: CommonOptions {
            out: PathBuf::from("unused.jsonl"),
            side,
            seconds: 1,
        },
        experiment,
        counts_per_revolution: Some(1000),
    }
}

fn drive() -> Experiment {
    Experiment::Drive {
        polarity: Polarity::Negative,
        force: 500,
        drive_ms: 5,
        initialize_ring: false,
    }
}

fn input(at: u64, timer: u32, counts: [u32; 2], pressed: [bool; 2], touch: [bool; 2]) -> RawFrame {
    let mut raw = frame(at, timer, counts);
    raw.buttons[7] = if pressed[0] { 4 } else { 0 };
    raw.buttons[16] = if pressed[1] { 0x20 } else { 0 };
    raw.buttons[17] = u8::from(touch[0]) * 0x10 + u8::from(touch[1]) * 0x20;
    raw
}

fn plain(at: u64, timer: u32, counts: [u32; 2]) -> RawFrame {
    input(at, timer, counts, [false; 2], [false; 2])
}

fn arm(model: &mut Model, wheel: Wheel) -> anyhow::Result<()> {
    let mut pressed = [false; 2];
    pressed[wheel.index()] = true;
    model.observe(&plain(0, 1, [0; 2]), 0)?;
    model.observe(&input(1000, 2, [0; 2], pressed, [false; 2]), 1000)?;
    model.observe(&plain(2000, 3, [0; 2]), 2000)?;
    Ok(())
}

#[test]
fn parsing_enforces_single_wheel_calibration_force_and_duration() -> anyhow::Result<()> {
    // Given valid new commands and every bounded numeric option at its edges.
    for (command, extra) in [
        ("platter-measure", vec![]),
        ("platter-ring", vec!["--counts-per-revolution", "1000"]),
        (
            "platter-drive",
            vec!["--polarity", "positive", "--enable-motor"],
        ),
    ] {
        let mut args = vec!["diag", command, "--out", "unused", "--side", "left"];
        args.extend(extra);
        // When parsing without dispatching to HID.
        assert!(Args::try_parse_from(&args).is_ok());
        // Then neither both wheels nor a missing wheel can be selected.
        args[5] = "both";
        assert!(Args::try_parse_from(&args).is_err());
    }
    for (flag, value) in [
        ("--force", "0"),
        ("--force", "2001"),
        ("--drive-ms", "0"),
        ("--drive-ms", "2001"),
        ("--seconds", "0"),
        ("--seconds", "121"),
        ("--counts-per-revolution", "0"),
        ("--counts-per-revolution", "1000000001"),
    ] {
        assert!(
            Args::try_parse_from([
                "diag",
                "platter-drive",
                "--out",
                "unused",
                "--side",
                "right",
                "--polarity",
                "negative",
                "--enable-motor",
                flag,
                value,
            ])
            .is_err()
        );
    }
    assert!(
        Args::try_parse_from(["diag", "platter-ring", "--out", "unused", "--side", "left",])
            .is_err()
    );
    let args = Args::try_parse_from([
        "diag",
        "platter-drive",
        "--out",
        "/invalid/never-created",
        "--side",
        "left",
        "--polarity",
        "positive",
    ])?;
    assert!(crate::jog_diagnostic::run(args).is_err());
    Ok(())
}

#[test]
fn resolved_cli_preserves_maximum_values_and_raw_polarity() -> anyhow::Result<()> {
    // Given explicit maximum permitted values.
    let args = Args::try_parse_from([
        "diag",
        "platter-drive",
        "--out",
        "unused",
        "--side",
        "right",
        "--polarity",
        "negative",
        "--force",
        "2000",
        "--drive-ms",
        "2000",
        "--seconds",
        "120",
        "--enable-motor",
        "--counts-per-revolution",
        "12345",
    ])?;
    // When boundary options resolve, then no default scale or direction replaces them.
    let Command::PlatterDrive(options) = args.command else {
        bail!("wrong command")
    };
    let resolved = options.resolve()?;
    assert_eq!(resolved.common.seconds, 120);
    assert_eq!(resolved.counts_per_revolution, Some(12345));
    assert!(matches!(
        resolved.experiment,
        Experiment::Drive {
            polarity: Polarity::Negative,
            force: 2000,
            drive_ms: 2000,
            ..
        }
    ));
    Ok(())
}

#[test]
fn integrated_motion_wraps_without_assuming_encoder_scale() -> anyhow::Result<()> {
    // Given independently wrapping counters and no calibration.
    for side in [Wheel::Left, Wheel::Right] {
        let mut settings = config(Experiment::Measure, side);
        settings.counts_per_revolution = None;
        let mut model = Model::new(settings);
        model.observe(&plain(0, u32::MAX - 1, [u32::MAX - 1, 1]), 0)?;
        // When the real decoder unwraps both sensor and motion counters.
        let sample = model.observe(&plain(2000, 1, [1, u32::MAX - 1]), 2000)?;
        // Then total displacement, travel, and speed retain the raw sign.
        let expected = match side {
            Wheel::Left => 3,
            Wheel::Right => -3,
        };
        assert_eq!(sample.relative_counts, expected);
        assert_eq!(sample.travel_counts, 3);
        assert!(
            f64::from(i32::try_from(expected)?)
                .mul_add(-500.0, sample.counts_per_second.context("speed")?)
                .abs()
                < 1e-9
        );
        assert!(sample.rpm.is_none());
    }
    Ok(())
}

#[test]
fn calibrated_speed_uses_elapsed_host_time_not_led_period() -> anyhow::Result<()> {
    // Given a deliberately non-2880 calibration.
    let mut model = Model::new(config(Experiment::Measure, Wheel::Left));
    model.observe(&plain(0, 1, [0; 2]), 0)?;
    // When ten counts occur across 2ms.
    let sample = model.observe(&plain(2000, 2, [10, 0]), 2000)?;
    // Then the calibrated speed is 300 RPM, independently of LED units.
    assert!((sample.rpm.context("rpm")? - 300.0).abs() < 1e-9);
    Ok(())
}

#[test]
fn selected_move_gesture_arms_once_and_touch_requires_new_gesture() -> anyhow::Result<()> {
    // Given each independently selected wheel, with the other MOVE held.
    for side in [Wheel::Left, Wheel::Right] {
        let mut model = Model::new(config(drive(), side));
        let other = 1 - side.index();
        let mut pressed = [false; 2];
        pressed[other] = true;
        model.observe(&input(0, 1, [0; 2], pressed, [false; 2]), 0)?;
        model.observe(&plain(1000, 2, [0; 2]), 1000)?;
        assert_eq!(model.force(), [0; 2]);
        pressed = [false; 2];
        pressed[side.index()] = true;
        model.observe(&input(2000, 3, [0; 2], pressed, [false; 2]), 2000)?;
        model.observe(&plain(3000, 4, [0; 2]), 3000)?;
        assert_eq!(model.force()[side.index()], -500);
        assert_eq!(model.force()[other], 0);
        // When touching interrupts, then release alone cannot re-arm.
        let mut touch = [false; 2];
        touch[side.index()] = true;
        model.observe(&input(4000, 5, [0; 2], [false; 2], touch), 4000)?;
        model.observe(&plain(5000, 6, [0; 2]), 5000)?;
        assert_eq!(model.force(), [0; 2]);
        model.observe(&input(6000, 7, [0; 2], pressed, [false; 2]), 6000)?;
        model.observe(&plain(7000, 8, [0; 2]), 7000)?;
        assert_eq!(model.force()[side.index()], -500);
    }
    Ok(())
}

#[test]
fn startup_held_button_and_touched_gestures_do_not_arm() -> anyhow::Result<()> {
    // Given MOVE already held at startup, followed by a gesture under touch.
    let mut model = Model::new(config(drive(), Wheel::Left));
    for (at, pressed, touch) in [
        (0, true, false),
        (1000, false, false),
        (2000, true, true),
        (3000, false, true),
        (4000, false, false),
    ] {
        // When these snapshots are observed, then force remains neutral.
        model.observe(
            &input(
                at,
                u32::try_from(at + 1)?,
                [0; 2],
                [pressed, false],
                [touch, false],
            ),
            at,
        )?;
        assert_eq!(model.force(), [0; 2]);
    }
    Ok(())
}

#[test]
fn pulse_deadline_is_not_renewed_by_motion_or_button_changes() -> anyhow::Result<()> {
    // Given a 5ms pulse armed at 2ms.
    let mut model = Model::new(config(drive(), Wheel::Left));
    arm(&mut model, Wheel::Left)?;
    model.observe(&plain(6000, 4, [10, 0]), 6000)?;
    // When the exact pulse deadline arrives, then force ends without a new sample.
    assert_eq!(model.pulse_deadline_us(), 7000);
    model.check_time(7000)?;
    assert_eq!(model.force(), [0; 2]);
    model.observe(&plain(8000, 5, [20, 0]), 8000)?;
    assert_eq!(model.force(), [0; 2]);
    Ok(())
}

#[test]
fn duplicate_selected_sensor_cannot_refresh_or_arm_from_other_wheel() -> anyhow::Result<()> {
    // Given a baseline with continuing fresh motion only on the other wheel.
    let mut model = Model::new(config(drive(), Wheel::Left));
    model.observe(&plain(0, 1, [0; 2]), 0)?;
    for (at, pressed) in [(10_000, true), (20_000, false), (49_999, false)] {
        let mut raw = input(at, 1, [0; 2], [pressed, false], [false; 2]);
        raw.jog[36..40].copy_from_slice(&u32::try_from(at)?.to_le_bytes());
        model.observe(&raw, at)?;
    }
    // When the selected wheel's exact freshness deadline arrives.
    // Then duplicate snapshots and another wheel's progress have not kept it alive.
    assert_eq!(model.force(), [0; 2]);
    assert!(model.check_time(50_000).is_err());
    Ok(())
}

#[test]
fn overall_deadline_rejects_queued_arming_before_any_output() -> anyhow::Result<()> {
    // Given a frame generated before the deadline but consumed at the deadline.
    let mut model = Model::new(config(drive(), Wheel::Left));
    // When checking overall time, then duration wins over even stale sensor state.
    assert_eq!(model.check_time(1_000_000)?, Some("duration"));
    assert!(
        model
            .observe(&plain(999_999, 1, [0; 2]), 1_000_000)
            .is_err()
    );
    assert_eq!(model.force(), [0; 2]);
    Ok(())
}

#[test]
fn ring_wraps_both_directions_and_touch_holds_relative_phase() -> anyhow::Result<()> {
    // Given a 1000-count revolution, independent from the 2880-unit ring.
    for reverse_ring in [false, true] {
        let mut model = Model::new(config(Experiment::Ring { reverse_ring }, Wheel::Right));
        model.observe(&plain(0, 1, [0; 2]), 0)?;
        let negative = model.observe(&plain(1000, 2, [0, u32::MAX - 249]), 1000)?;
        assert_eq!(
            negative.ring_position,
            if reverse_ring { 720 } else { 2160 }
        );
        // When touch holds while raw movement continues through release.
        model.observe(
            &input(2000, 3, [0, u32::MAX - 149], [false; 2], [false, true]),
            2000,
        )?;
        let released = model.observe(&plain(3000, 4, [0, u32::MAX - 49]), 3000)?;
        // Then raw counts include held travel, but the displayed phase is held.
        assert_eq!(released.relative_counts, -50);
        assert_eq!(released.ring_position, negative.ring_position);
        let resumed = model.observe(&plain(4000, 5, [0, 0]), 4000)?;
        assert_eq!(resumed.ring_position, if reverse_ring { 576 } else { 2304 });
        assert!(!resumed.ring_held);
    }
    Ok(())
}

#[test]
fn ring_phase_reduces_extreme_integrals_before_scaling() -> anyhow::Result<()> {
    // Given extreme signed integration on either side of the integer range.
    for total in [i64::MIN, i64::MAX, -1000, 1000, 0] {
        let mut model = Model::new(config(Experiment::Ring { reverse_ring: true }, Wheel::Left));
        model.ring_counts = total;
        // When mapping to a ring position, then Euclidean wrapping stays bounded.
        assert!(model.ring_position()? < 2880);
    }
    Ok(())
}

#[test]
fn passive_session_never_writes_even_during_error_cleanup() -> anyhow::Result<()> {
    // Given a passive session and a transport that would fail if invoked.
    let mut log = Vec::new();
    let mut session = Session::new(
        config(Experiment::Measure, Wheel::Left),
        |_| bail!("unexpected write"),
        &mut log,
    );
    // When setup, malformed input, and final cleanup run.
    session.start()?;
    let mut malformed = plain(1000, 1, [0; 2]);
    malformed.buttons.truncate(2);
    assert!(session.sample(&malformed, 1000).is_err());
    // Then cleanup remains completely output-free.
    session.finish(1000, "test_error")?;
    Ok(())
}

#[test]
fn output_startup_and_cleanup_are_selected_and_neutral_bracketed() -> anyhow::Result<()> {
    // Given each ring side with the real native report builders.
    for side in [Wheel::Left, Wheel::Right] {
        let mut writes = Vec::new();
        let mut log = Vec::new();
        {
            let mut session = Session::new(
                config(
                    Experiment::Ring {
                        reverse_ring: false,
                    },
                    side,
                ),
                |report| {
                    writes.push(report.to_vec());
                    Ok(report.len())
                },
                &mut log,
            );
            // When one ring tick and normal shutdown run.
            session.start()?;
            session.sample(&plain(0, 1, [0; 2]), 0)?;
            session.tick(0)?;
            session.finish(1000, "ctrl_c")?;
        }
        // Then only the selected ring is changed and neutral precedes ring clearing.
        assert_eq!(writes[0], neutral_report());
        assert_eq!(writes[1], ring_initialization(side));
        assert_eq!(writes[2], ring_report(side, 0));
        assert_eq!(writes[3], neutral_report());
        assert_eq!(&writes[4][..3], &[0x32, side.wire(), 0]);
    }
    Ok(())
}

struct FailingLog<'a>(&'a Cell<bool>);

impl Write for FailingLog<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0.get() {
            Err(io::Error::other("log failed"))
        } else {
            Ok(bytes.len())
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.0.get() {
            Err(io::Error::other("flush failed"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn touch_neutral_precedes_failing_log_and_release_does_not_drive() -> anyhow::Result<()> {
    // Given an armed session whose logger fails at the interrupting touch.
    let failed = Cell::new(false);
    let mut log = FailingLog(&failed);
    let writes = RefCell::new(Vec::new());
    {
        let mut session = Session::new(
            config(drive(), Wheel::Left),
            |r| {
                writes.borrow_mut().push(r.to_vec());
                Ok(r.len())
            },
            &mut log,
        );
        session.start()?;
        arm(&mut session.model, Wheel::Left)?;
        session.tick(2000)?;
        failed.set(true);
        // When touch is observed and logging fails.
        assert!(
            session
                .sample(&input(3000, 4, [0; 2], [false; 2], [true, false]), 3000)
                .is_err()
        );
        // Then neutral was sent before the error, and failing cleanup attempts it again.
        assert_eq!(writes.borrow().last(), Some(&neutral_report().to_vec()));
        assert!(session.finish(3000, "log_error").is_err());
    }
    assert_eq!(writes.borrow().last(), Some(&neutral_report().to_vec()));
    assert!(writes.borrow().contains(&report_for([-500, 0]).to_vec()));
    Ok(())
}

#[test]
fn failed_short_startup_and_unwinding_always_attempt_neutral() {
    // Given every startup write boundary failing or returning a short write.
    for failed_write in [0, 1] {
        for short in [false, true] {
            let writes = RefCell::new(Vec::new());
            let mut log = Vec::new();
            {
                let mut session = Session::new(
                    config(
                        Experiment::Ring {
                            reverse_ring: false,
                        },
                        Wheel::Right,
                    ),
                    |r| {
                        let index = writes.borrow().len();
                        writes.borrow_mut().push(r.to_vec());
                        if index == failed_write {
                            if short {
                                return Ok(r.len() - 1);
                            }
                            bail!("transport failed");
                        }
                        Ok(r.len())
                    },
                    &mut log,
                );
                // When fallible startup exits with an error and the guard drops.
                assert!(session.start().is_err());
            }
            // Then a real neutral attempt follows the failed setup.
            assert_eq!(writes.borrow().last(), Some(&neutral_report().to_vec()));
        }
    }
}

#[test]
fn logging_and_transport_cleanup_errors_are_both_preserved() -> anyhow::Result<()> {
    // Given independent operation, transport, and logger failures.
    let failed = Cell::new(true);
    let mut log = FailingLog(&failed);
    let mut session = Session::new(
        config(drive(), Wheel::Left),
        |_| bail!("neutral transport failed"),
        &mut log,
    );
    // When cleanup is combined with the triggering operation error.
    let error = combine::<()>(
        Err(anyhow::anyhow!("reader failed")),
        session.finish(0, "error"),
    )
    .err()
    .context("must fail")?;
    // Then no failure is discarded.
    let message = format!("{error:#}");
    assert!(message.contains("reader failed"));
    assert!(message.contains("neutral transport failed"));
    assert!(message.contains("log failed"));
    assert!(message.contains("flush failed"));
    Ok(())
}
