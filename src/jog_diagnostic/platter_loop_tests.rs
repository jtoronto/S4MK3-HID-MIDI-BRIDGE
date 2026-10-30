use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use super::*;
use crate::jog_diagnostic::decode::tests::frame;
use crate::jog_diagnostic::motor::{neutral_report, report_for};
use crate::jog_diagnostic::platter::{CommonOptions, Polarity, Wheel};
use crate::jog_diagnostic::platter_telemetry::{Clock, StopRequest};

fn config(experiment: Experiment) -> Config {
    Config {
        common: CommonOptions {
            out: PathBuf::from("unused"),
            side: Wheel::Right,
            seconds: 1,
        },
        experiment,
        counts_per_revolution: None,
    }
}

fn drive(drive_ms: u64) -> Experiment {
    Experiment::Drive {
        polarity: Polarity::Positive,
        force: 500,
        drive_ms,
        initialize_ring: false,
    }
}

fn untouched(at: u64, timer: u32) -> super::super::capture::RawFrame {
    let mut raw = frame(at, timer, [0; 2]);
    raw.buttons[17] = 0;
    raw
}

fn arm<F: FnMut(&[u8]) -> anyhow::Result<usize>>(
    session: &mut Session<'_, F>,
) -> anyhow::Result<()> {
    session.sample(&untouched(0, 1), 0)?;
    let mut pressed = untouched(1000, 2);
    pressed.buttons[16] = 0x20;
    session.sample(&pressed, 1000)?;
    session.sample(&untouched(2000, 3), 2000)?;
    session.tick(2000)?;
    Ok(())
}

fn log_records(log: &[u8]) -> anyhow::Result<Vec<serde_json::Value>> {
    Ok(std::str::from_utf8(log)?
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()?)
}

#[test]
fn rotation_stop_and_pulse_deadline_preempt_queued_frames() -> anyhow::Result<()> {
    use crate::jog_diagnostic::platter_rotation::{
        DEFAULT_BASE_FORCE, DEFAULT_FORCE_PER_RPM, DEFAULT_FORCE_RISE_PER_SECOND, DEFAULT_KI,
        DEFAULT_KP, DEFAULT_STARTUP_FORCE_RISE_PER_SECOND, DEFAULT_STARTUP_MS, Settings,
    };

    for stop_pending in [false, true] {
        let mut settings = config(Experiment::Rotate(Settings {
            polarity: Polarity::Positive,
            count_direction: Polarity::Positive,
            target_rpm: 45,
            overspeed_limit_rpm: None,
            max_force: 25_000,
            drive_ms: 630,
            base_force: DEFAULT_BASE_FORCE,
            force_per_rpm: DEFAULT_FORCE_PER_RPM,
            kp: DEFAULT_KP,
            ki: DEFAULT_KI,
            force_rise_per_second: DEFAULT_FORCE_RISE_PER_SECOND,
            startup_ms: DEFAULT_STARTUP_MS,
            startup_force_rise_per_second: DEFAULT_STARTUP_FORCE_RISE_PER_SECOND,
            initialize_ring: false,
        }));
        settings.counts_per_revolution = Some(6000);
        let (mut reader, frames, _done) = Reader::channel_only();
        for index in 0..32_u32 {
            let at = 626_000 + index * 1000;
            frames.send(untouched(u64::from(at), at.wrapping_mul(100)))?;
        }
        let (stop_tx, stop) = crossbeam_channel::bounded(1);
        if stop_pending {
            stop_tx.send(())?;
        }
        let calls = Cell::new(0_u64);
        let mut writes = Vec::new();
        let mut log = Vec::new();
        {
            let mut session = Session::new(
                settings,
                |r| {
                    writes.push(r.to_vec());
                    Ok(r.len())
                },
                &mut log,
            );
            session.start()?;
            for at in (0..=624_000_u32).step_by(2000) {
                let mut raw = untouched(u64::from(at), at + 1);
                let timer = at.wrapping_mul(100).to_le_bytes();
                raw.jog[8..12].copy_from_slice(&timer);
                raw.jog[36..40].copy_from_slice(&timer);
                if at == 2000 {
                    raw.buttons[16] = 0x20;
                }
                session.sample(&raw, u64::from(at))?;
            }
            assert_eq!(session.model.force(), [0, 25_000]);
            session.tick(624_000)?;
            let reason = run_loop(&mut session, &mut reader, &stop, || {
                let call = calls.get();
                calls.set(call + 1);
                if call >= 20 {
                    1_000_000
                } else {
                    625_000 + call * 1000
                }
            })?;
            assert_eq!(reason, if stop_pending { "ctrl_c" } else { "duration" });
        }
        let outputs = output_records(&log)?;
        assert!(
            outputs
                .iter()
                .any(|r| r["requested_force"] == serde_json::json!([0, 25_000]))
        );
        let cutoff = if stop_pending { 625_000 } else { 634_000 };
        for record in outputs {
            if record["requested_force"] != serde_json::json!([0, 0]) {
                assert!(record["host_us"].as_u64().context("time")? < cutoff);
                assert_eq!(record["requested_force"][0], 0);
            }
        }
        assert_eq!(reader.frames.len() == 32, stop_pending);
        assert_eq!(writes.last(), Some(&neutral_report().to_vec()));
    }
    Ok(())
}

fn output_records(log: &[u8]) -> anyhow::Result<Vec<serde_json::Value>> {
    Ok(log_records(log)?
        .into_iter()
        .filter(|record| record["kind"] == "platter_output")
        .collect())
}

#[test]
fn ctrl_c_preempts_saturated_frames_and_sends_final_neutral() -> anyhow::Result<()> {
    // Given active right drive, a full FIFO, and Ctrl-C already signaled.
    let (mut reader, frames, _done) = Reader::channel_only();
    for index in 0..32 {
        frames.send(untouched(3000 + u64::from(index), index + 4))?;
    }
    let (stop_tx, stop) = crossbeam_channel::bounded(1);
    stop_tx.send(())?;
    let mut writes = Vec::new();
    let mut log = Vec::new();
    {
        let mut session = Session::new(
            config(drive(2000)),
            |r| {
                writes.push(r.to_vec());
                Ok(r.len())
            },
            &mut log,
        );
        session.start()?;
        arm(&mut session)?;
        // When the real reader loop selects from ready stop and frames.
        assert_eq!(
            run_loop(&mut session, &mut reader, &stop, || 3000)?,
            "ctrl_c"
        );
    }
    // Then Stop wins without consuming frames and final neutral is unconditional.
    assert_eq!(reader.frames.len(), 32);
    assert_eq!(writes.last(), Some(&neutral_report().to_vec()));
    assert!(writes.contains(&report_for([0, 500]).to_vec()));
    Ok(())
}

#[test]
fn saturated_frames_cannot_extend_pulse_or_starve_input() -> anyhow::Result<()> {
    // Given a 5ms pulse and continuously ready fresh input.
    let (mut reader, frames, _done) = Reader::channel_only();
    for index in 0..32 {
        frames.send(untouched(3000 + u64::from(index) * 1000, index + 4))?;
    }
    let (_stop_tx, stop) = crossbeam_channel::bounded(1);
    let calls = Cell::new(0_u64);
    let mut log = Vec::new();
    let writes = RefCell::new(Vec::new());
    {
        let mut session = Session::new(
            config(drive(5)),
            |r| {
                writes.borrow_mut().push(r.to_vec());
                Ok(r.len())
            },
            &mut log,
        );
        session.start()?;
        arm(&mut session)?;
        // When fake time reaches pulse expiry and then the overall deadline.
        let reason = run_loop(&mut session, &mut reader, &stop, || {
            let call = calls.get();
            calls.set(call + 1);
            if call >= 14 {
                1_000_000
            } else {
                3000 + call * 1000
            }
        })?;
        assert_eq!(reason, "duration");
    }
    // Then frames were consumed, but no nonzero write occurred at/after 7ms.
    assert!(reader.frames.len() < 32);
    for record in output_records(&log)? {
        if record["requested_force"] != serde_json::json!([0, 0]) {
            assert!(record["host_us"].as_u64().context("output timestamp")? < 7000);
        }
    }
    assert_eq!(writes.borrow().last(), Some(&neutral_report().to_vec()));
    Ok(())
}

#[test]
fn duplicate_fifo_traffic_cannot_defeat_watchdog_neutral() -> anyhow::Result<()> {
    // Given active drive and a saturated FIFO of duplicate sensor snapshots.
    let (mut reader, frames, _done) = Reader::channel_only();
    for index in 0..32_u32 {
        frames.send(untouched(3000 + u64::from(index) * 1000, 3))?;
    }
    let (_stop_tx, stop) = crossbeam_channel::bounded(1);
    let now = Cell::new(3000_u64);
    let mut log = Vec::new();
    let writes = RefCell::new(Vec::new());
    {
        let mut session = Session::new(
            config(drive(2000)),
            |r| {
                writes.borrow_mut().push(r.to_vec());
                Ok(r.len())
            },
            &mut log,
        );
        session.start()?;
        arm(&mut session)?;
        // When only host timestamps advance, the real loop must fail closed.
        assert!(
            run_loop(&mut session, &mut reader, &stop, || {
                let at = now.get();
                now.set(at + 1000);
                at
            })
            .is_err()
        );
    }
    // Then pending frames did not refresh the sensor deadline.
    assert!(!reader.frames.is_empty());
    assert_eq!(writes.borrow().last(), Some(&neutral_report().to_vec()));
    for record in output_records(&log)? {
        if record["requested_force"] != serde_json::json!([0, 0]) {
            assert!(record["host_us"].as_u64().context("output timestamp")? < 52_000);
        }
    }
    Ok(())
}

#[test]
fn passive_loop_duration_and_ctrl_c_never_write_output() -> anyhow::Result<()> {
    // Given passive measurement with either a signaled Stop or expired duration.
    for stop_now in [false, true] {
        let (mut reader, _frames, _done) = Reader::channel_only();
        let (stop_tx, stop) = crossbeam_channel::bounded(1);
        if stop_now {
            stop_tx.send(())?;
        }
        let mut log = Vec::new();
        let mut session = Session::new(
            config(Experiment::Measure),
            |_| bail!("passive output"),
            &mut log,
        );
        session.start()?;
        // When the real loop ends without needing a native reader or wait.
        let reason = run_loop(&mut session, &mut reader, &stop, || {
            if stop_now { 0 } else { 1_000_000 }
        })?;
        // Then both exit paths remain no-write.
        assert_eq!(reason, if stop_now { "ctrl_c" } else { "duration" });
    }
    Ok(())
}

#[test]
fn reader_failure_neutralizes_before_reader_cleanup() -> anyhow::Result<()> {
    // Given active drive and a native reader failure ready for consumption.
    let (mut reader, _frames, done) = Reader::channel_only();
    done.send(Err(anyhow::anyhow!("native reader failed")))?;
    let (_stop_tx, stop) = crossbeam_channel::bounded(1);
    let mut writes = Vec::new();
    let mut log = Vec::new();
    {
        let mut session = Session::new(
            config(drive(2000)),
            |r| {
                writes.push(r.to_vec());
                Ok(r.len())
            },
            &mut log,
        );
        session.start()?;
        arm(&mut session)?;
        // When failure wins the biased select, cleanup already has neutral.
        assert!(run_loop(&mut session, &mut reader, &stop, || 3000).is_err());
    }
    // Then native cancellation/join can follow the completed loop safely.
    assert_eq!(writes.last(), Some(&neutral_report().to_vec()));
    assert!(reader.completed);
    reader.stop()?;
    Ok(())
}

#[test]
fn overall_deadline_neutralizes_active_drive_before_consuming_queued_frame() -> anyhow::Result<()> {
    // Given drive that would outlast the overall duration and a queued snapshot.
    let (mut reader, frames, _done) = Reader::channel_only();
    frames.send(untouched(999_999, 4))?;
    let (_stop_tx, stop) = crossbeam_channel::bounded(1);
    let mut writes = Vec::new();
    let mut log = Vec::new();
    {
        let mut session = Session::new(
            config(drive(2000)),
            |r| {
                writes.push(r.to_vec());
                Ok(r.len())
            },
            &mut log,
        );
        session.start()?;
        arm(&mut session)?;
        // When the overall deadline arrives before any other event.
        assert_eq!(
            run_loop(&mut session, &mut reader, &stop, || 1_000_000)?,
            "duration"
        );
    }
    // Then no queued input is applied, and active force ends in final neutral.
    assert_eq!(reader.frames.len(), 1);
    assert_eq!(writes.last(), Some(&neutral_report().to_vec()));
    Ok(())
}

#[test]
fn short_or_failed_drive_write_preserves_error_and_neutralizes() -> anyhow::Result<()> {
    // Given an already active drive and a failing next nonzero write.
    for short in [false, true] {
        let (mut reader, frames, _done) = Reader::channel_only();
        frames.send(untouched(3000, 4))?;
        let (_stop_tx, stop) = crossbeam_channel::bounded(1);
        let failed = Cell::new(false);
        let now = Cell::new(3000);
        let writes = RefCell::new(Vec::new());
        let mut log = Vec::new();
        {
            let mut session = Session::new(
                config(drive(2000)),
                |r| {
                    writes.borrow_mut().push(r.to_vec());
                    if failed.get() && r == report_for([0, 500]) {
                        if short {
                            return Ok(10);
                        }
                        bail!("drive transport failed");
                    }
                    Ok(r.len())
                },
                &mut log,
            );
            session.start()?;
            arm(&mut session)?;
            failed.set(true);
            // When the real output slot attempts that write, then its error survives.
            let error = run_loop(&mut session, &mut reader, &stop, || {
                let at = now.get();
                now.set(at + 1000);
                at
            })
            .err()
            .context("write must fail")?;
            assert!(format!("{error:#}").contains(if short {
                "short platter write"
            } else {
                "drive transport failed"
            }));
        }
        // Then final neutral follows the failed drive, with the actual short length logged.
        assert_eq!(writes.borrow().last(), Some(&neutral_report().to_vec()));
        if short {
            assert!(
                output_records(&log)?
                    .iter()
                    .any(|record| record["written"] == 10 && !record["error"].is_null())
            );
        }
    }
    Ok(())
}

#[test]
fn stop_during_inflight_write_is_timestamped_without_hiding_transport_error() -> anyhow::Result<()>
{
    // Given a writer whose nonzero call cannot return until the test releases it.
    for fail_write in [false, true] {
        let now = Arc::new(AtomicU64::new(0));
        let clock = Clock::Manual(Arc::clone(&now));
        let stop_request = StopRequest::new(clock.clone());
        let request = stop_request.clone();
        let (entered_tx, entered_rx) = crossbeam_channel::bounded(1);
        let (release_tx, release_rx) = crossbeam_channel::bounded(1);
        let (stop_tx, stop_rx) = crossbeam_channel::bounded(1);
        let worker_now = Arc::clone(&now);
        let worker = std::thread::spawn(move || -> anyhow::Result<_> {
            let (mut reader, _frames, _done) = Reader::channel_only();
            let mut writes = Vec::new();
            let mut log = Vec::new();
            let result = {
                let mut session = Session::with_telemetry(
                    config(drive(2000)),
                    |report| {
                        writes.push(report.to_vec());
                        if report == report_for([0, 500]) {
                            entered_tx.send(())?;
                            release_rx
                                .recv_timeout(Duration::from_secs(2))
                                .context("release in-flight writer")?;
                            if fail_write {
                                bail!("injected write failure");
                            }
                        }
                        Ok(report.len())
                    },
                    &mut log,
                    clock,
                    stop_request,
                );
                session.start()?;
                session.sample(&untouched(0, 1), 0)?;
                let mut pressed = untouched(1000, 2);
                pressed.buttons[16] = 0x20;
                session.sample(&pressed, 1000)?;
                session.sample(&untouched(2000, 3), 2000)?;
                worker_now.store(3000, Ordering::Release);
                run_loop(&mut session, &mut reader, &stop_rx, || {
                    worker_now.load(Ordering::Acquire)
                })
                .map(str::to_owned)
            };
            Ok((result, log, writes))
        });

        // When Stop is latched during that call, then the call returns on our signal.
        entered_rx
            .recv_timeout(Duration::from_secs(2))
            .context("writer entered")?;
        now.store(4500, Ordering::Release);
        request.request();
        stop_tx.send(())?;
        now.store(6000, Ordering::Release);
        release_tx.send(())?;
        let (result, log, writes) = worker
            .join()
            .map_err(|_| anyhow::anyhow!("diagnostic test worker panicked"))??;

        // Then the original result survives, no new force is sent, and neutral follows.
        if fail_write {
            assert!(
                format!(
                    "{:#}",
                    result.err().context("transport failure must survive")?
                )
                .contains("injected write failure")
            );
        } else {
            assert_eq!(result?, "ctrl_c");
        }
        assert_eq!(
            writes
                .iter()
                .filter(|r| r.as_slice() == report_for([0, 500]))
                .count(),
            1
        );
        assert_eq!(
            writes
                .iter()
                .filter(|r| r.as_slice() == neutral_report())
                .count(),
            2 // One startup and exactly one final-neutral attempt.
        );
        assert_eq!(writes.last(), Some(&neutral_report().to_vec()));
        let records = log_records(&log)?;
        let output = records
            .iter()
            .find(|r| r["requested_force"] == serde_json::json!([0, 500]))
            .context("force output record")?;
        assert_eq!(output["io_started_us"], 3000);
        assert_eq!(output["io_completed_us"], 6000);
        assert_eq!(output["io_elapsed_us"], 3000);
        assert_eq!(output["stop_requested_at_us"], 4500);
        assert_eq!(output["error"].is_null(), !fail_write);
        let end = records.last().context("end record")?;
        assert_eq!(end["kind"], "platter_end");
        assert_eq!(end["stop_requested_at_us"], 4500);
    }
    Ok(())
}
