use super::*;
use anyhow::Context as _;

#[test]
fn in_flight_and_failed_operation_are_visible() -> anyhow::Result<()> {
    // Given an instrumented reader operation that has entered native code.
    let mut telemetry = Telemetry::new(Instant::now());
    telemetry.begin(Operation::Get3);

    // When a watchdog samples before the native operation returns.
    let in_flight = telemetry.snapshot(0);

    // Then the operation and its monotonic start point are available.
    assert_eq!(in_flight.in_flight_operation, Some(Operation::Get3));
    assert!(in_flight.in_flight_started_us.is_some());

    // When the operation returns an error.
    telemetry.end(Operation::Get3, Duration::from_micros(17), true);
    let completed = telemetry.snapshot(0);
    let timing = completed
        .operations
        .iter()
        .find(|timing| timing.operation == Operation::Get3)
        .context("GET3 timing missing")?;

    // Then the failed attempt contributes timing and failure counts.
    assert_eq!(completed.in_flight_operation, None);
    assert_eq!(timing.count, 1);
    assert_eq!(timing.failures, 1);
    assert_eq!(timing.latest_duration_us, Some(17));
    assert_eq!(timing.maximum_duration_us, 17);
    assert!(timing.latest_started_us.is_some());
    assert!(timing.latest_ended_us.is_some());

    // The owned snapshot remains serializable for the parent health record.
    let value = serde_json::to_value(completed)?;
    assert_eq!(value["publication_count"], 0);
    Ok(())
}

#[test]
fn queue_high_water_tracks_sampled_and_published_depth() {
    // Given an instrumented queue whose first sampled depth is three.
    let mut telemetry = Telemetry::new(Instant::now());
    let first = telemetry.snapshot(3);

    // When publication observes a shallower queue and a later sample is empty.
    telemetry.publication_finished(Duration::from_micros(5), 2);
    let latest = telemetry.snapshot(0);

    // Then sampled high-water is retained while depth and publication timing advance.
    assert_eq!(first.queue_depth, 3);
    assert_eq!(latest.queue_depth, 0);
    assert_eq!(latest.queue_high_water, 3);
    assert_eq!(latest.publication_count, 1);
    assert_eq!(latest.latest_publication_duration_us, Some(5));
    assert_eq!(latest.maximum_publication_duration_us, 5);
    assert!(latest.last_publication_us.is_some());
}

#[test]
fn batch_and_poll_maxima_are_bounded_counters() {
    // Given deterministic measured durations without a wall-clock wait.
    let mut telemetry = Telemetry::new(Instant::now());
    telemetry.batch_finished(Duration::from_micros(20));
    telemetry.batch_finished(Duration::from_micros(8));
    telemetry.poll_lateness(Duration::from_micros(4));
    telemetry.poll_lateness(Duration::from_micros(9));

    // When a reader snapshot is taken.
    let snapshot = telemetry.snapshot(0);

    // Then latest values and maxima remain distinct.
    assert_eq!(snapshot.latest_batch_duration_us, Some(8));
    assert_eq!(snapshot.maximum_batch_duration_us, 20);
    assert_eq!(snapshot.latest_poll_lateness_us, Some(9));
    assert_eq!(snapshot.maximum_poll_lateness_us, 9);
}

#[test]
fn wait_snapshot_exposes_in_flight_and_bounded_latest_maximum() {
    // Given a deterministic wait with an observable scheduled deadline.
    let mut telemetry = Telemetry::new(Instant::now());
    telemetry.poll_event(PollEvent::WaitStarted {
        requested_us: 100,
        scheduled_deadline_us: 500,
        started_us: 400,
    });

    // When sampled before return and after two completed waits.
    let in_flight = telemetry.snapshot(0);
    telemetry.poll_event(PollEvent::WaitReturned {
        returned_us: 525,
        cancelled: false,
    });
    telemetry.poll_event(PollEvent::WaitStarted {
        requested_us: 200,
        scheduled_deadline_us: 900,
        started_us: 700,
    });
    telemetry.poll_event(PollEvent::WaitReturned {
        returned_us: 960,
        cancelled: false,
    });
    telemetry.poll_event(PollEvent::PollAdmitted {
        started_us: 980,
        lateness_us: 80,
        post_wait_us: Some(20),
    });
    telemetry.poll_event(PollEvent::PollAdmitted {
        started_us: 990,
        lateness_us: 90,
        post_wait_us: Some(10),
    });
    let completed = telemetry.snapshot(0);

    // Then the active wait and latest/max/counter values are independently visible.
    assert_eq!(
        in_flight.in_flight_wait,
        Some(InFlightWait {
            requested: Duration::from_micros(100),
            scheduled_deadline_us: 500,
            started_us: 400,
        })
    );
    assert_eq!(completed.in_flight_wait, None);
    assert_eq!(completed.waits.count, 2);
    assert_eq!(completed.waits.latest_requested_us, Some(200));
    assert_eq!(completed.waits.latest_started_us, Some(700));
    assert_eq!(completed.waits.latest_returned_us, Some(960));
    assert_eq!(completed.waits.latest_elapsed_us, Some(260));
    assert_eq!(completed.waits.latest_overshoot_us, Some(60));
    assert_eq!(completed.waits.maximum_elapsed_us, 260);
    assert_eq!(completed.waits.maximum_overshoot_us, 60);
    assert_eq!(completed.latest_poll_admitted_us, Some(990));
    assert_eq!(completed.latest_post_wait_us, Some(10));
    assert_eq!(completed.maximum_post_wait_us, 20);
    assert_eq!(completed.post_wait_count, 2);
}
