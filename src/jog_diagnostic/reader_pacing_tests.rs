use super::*;
use crate::jog_diagnostic::reader_telemetry::{PollEvent, Telemetry};
use anyhow::ensure;
use std::cell::Cell;
use std::time::Instant;

#[test]
fn ready_interrupts_cannot_accelerate_snapshot_polling() -> anyhow::Result<()> {
    let (_cancel_tx, cancel) = bounded(1);
    let now = Cell::new(0_u64);
    let calls = Cell::new(0_u16);
    let mut next = 0;
    let mut reads = Vec::new();
    // Given native acquisition completing quickly because interrupts are ready.
    for _ in 0..8 {
        // When the actual reader's poll gate schedules another acquisition.
        assert!(poll_slot(
            &mut next,
            &cancel,
            (
                || {
                    calls.set(calls.get() + 1);
                    ensure!(calls.get() < 100, "poll gate made no progress");
                    Ok(now.get())
                },
                |wait| {
                    now.set(now.get() + wait);
                    false
                }
            )
        )?);
        reads.push(now.get());
        now.set(now.get() + 100);
    }
    // Then starts remain on the intended rate rather than racing at 100us.
    assert_eq!(
        reads,
        (0..8).map(|i| i * OUTPUT_INTERVAL_US).collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn delayed_acquisition_does_not_catch_up_in_a_burst() -> anyhow::Result<()> {
    let (_cancel_tx, cancel) = bounded(1);
    let now = Cell::new(9999_u64);
    let mut next = 2000;
    let mut reads = Vec::new();
    // Given a delayed acquisition after several missed periods.
    for _ in 0..3 {
        // When subsequent acquisitions are immediately available.
        assert!(poll_slot(
            &mut next,
            &cancel,
            (
                || Ok(now.get()),
                |wait| {
                    now.set(now.get() + wait);
                    false
                }
            )
        )?);
        reads.push(now.get());
    }
    // Then no catch-up reads or compressed periods are issued.
    assert_eq!(reads, [9999, 11999, 13999]);
    Ok(())
}

#[test]
fn delayed_poll_reports_lateness_without_changing_schedule() -> anyhow::Result<()> {
    let (_cancel_tx, cancel) = bounded(1);
    let mut next = 2000;
    let clock_calls = Cell::new(0);
    let now = Cell::new(0_u64);
    let mut telemetry = Telemetry::new(Instant::now());
    // Given a late wait return followed by injected delay before poll admission.
    let acquired = poll_slot_observed(
        &mut next,
        &cancel,
        (
            || {
                let call = clock_calls.get();
                clock_calls.set(call + 1);
                Ok(match call {
                    0 => 1000,
                    1 => now.get(),
                    _ => now.get() + 500,
                })
            },
            |requested| {
                assert_eq!(requested, 1000);
                now.set(2000 + 275);
                false
            },
        ),
        |event| {
            if let PollEvent::PollAdmitted { lateness_us, .. } = event {
                telemetry.poll_lateness(std::time::Duration::from_micros(lateness_us));
            }
            telemetry.poll_event(event);
            Ok(())
        },
        true,
    )?;

    // When the actual acquisition gate admits the poll.
    assert!(acquired);

    // Then wait overshoot and post-wait delay are separately measurable.
    let snapshot = telemetry.snapshot(0);
    assert_eq!(snapshot.waits.latest_requested_us, Some(1000));
    assert_eq!(snapshot.waits.latest_scheduled_deadline_us, Some(2000));
    assert_eq!(snapshot.waits.latest_elapsed_us, Some(1275));
    assert_eq!(snapshot.waits.latest_overshoot_us, Some(275));
    assert_eq!(snapshot.latest_poll_lateness_us, Some(775));
    assert_eq!(snapshot.latest_post_wait_us, Some(500));
    assert_eq!(snapshot.latest_poll_admitted_us, Some(2775));
    assert_eq!(snapshot.post_wait_count, 1);
    // Scheduling remains relative to the actual admitted poll start.
    assert_eq!(next, 4775);
    Ok(())
}

#[test]
fn cancellation_wins_before_or_during_the_poll_wait() -> anyhow::Result<()> {
    use std::time::Instant;

    // Given either a queued cancel or a cancel reported by the wait itself.
    for prequeued in [false, true] {
        let (tx, cancel) = bounded(1);
        if prequeued {
            tx.send(())?;
        }
        let waited = Cell::new(false);
        let mut next = 2000;
        let mut telemetry = Telemetry::new(Instant::now());
        // When the reader is waiting for its next acquisition slot.
        let acquire = poll_slot_observed(
            &mut next,
            &cancel,
            (
                || Ok(0),
                |_| {
                    waited.set(true);
                    true
                },
            ),
            |event| {
                telemetry.poll_event(event);
                Ok(())
            },
            true,
        )?;
        // Then no snapshot read is admitted after cancellation.
        assert!(!acquire);
        assert_eq!(waited.get(), !prequeued);
        let snapshot = telemetry.snapshot(0);
        let recorded_wait = u64::from(!prequeued);
        assert_eq!(snapshot.waits.count, recorded_wait);
        assert_eq!(snapshot.waits.cancellations, recorded_wait);
        assert_eq!(
            snapshot.waits.latest_cancelled,
            (!prequeued).then_some(true)
        );
    }
    Ok(())
}

#[test]
fn eligible_poll_does_not_wait() -> anyhow::Result<()> {
    let (_cancel_tx, cancel) = bounded(1);
    let mut next = 0;
    let waited = Cell::new(false);
    let admitted = poll_slot_observed(
        &mut next,
        &cancel,
        (
            || Ok(42),
            |_| {
                waited.set(true);
                false
            },
        ),
        |_| Ok(()),
        true,
    )?;

    assert!(admitted);
    assert!(!waited.get());
    assert_eq!(next, 2042);
    Ok(())
}

#[test]
fn genuine_overflow_and_disconnection_remain_fatal_with_snapshot_time() -> anyhow::Result<()> {
    use crate::jog_diagnostic::decode::tests::frame;

    // Given either a genuinely full queue or a lost consumer.
    for disconnected in [false, true] {
        let (tx, rx) = bounded(32);
        for index in 0..32 {
            tx.send(frame(u64::from(index) * 2000, index + 1, [index, 0]))?;
        }
        let (_cancel_tx, cancel) = bounded(1);
        let consumer = if disconnected {
            drop(rx);
            None
        } else {
            Some(rx)
        };
        // When the actual publisher cannot enqueue the next snapshot.
        let error = publish_frame(frame(64_000, 33, [32, 0]), (&tx, &cancel))
            .err()
            .context("publication must fail")?;
        // Then it fails rather than dropping/retrying; timing distinguishes a future stall.
        assert!(error.to_string().contains("rejected_snapshot_us=64000"));
        if let Some(rx) = consumer {
            assert!(error.to_string().contains("capacity=Some(32)"));
            assert_eq!(rx.try_recv()?.host_us, 0);
        } else {
            assert!(error.to_string().contains("snapshot consumer disconnected"));
        }
    }
    Ok(())
}
