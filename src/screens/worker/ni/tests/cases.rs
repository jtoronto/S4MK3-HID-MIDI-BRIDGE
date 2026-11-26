use super::*;
use crate::screens::worker::presentation::update_phase;

fn case_fixture(case: crate::screen_motor_diag::Case) -> anyhow::Result<(Worker, Receiver<Event>)> {
    case_fixture_clock(case, Instant::now)
}

fn case_fixture_clock(
    case: crate::screen_motor_diag::Case,
    clock: impl FnMut() -> Instant + Send + 'static,
) -> anyhow::Result<(Worker, Receiver<Event>)> {
    let (events, receiver) = unbounded();
    let transport_events = events.clone();
    let worker = Worker::spawn(
        Mailbox {
            latest: Some(Request {
                id: 0,
                generation: 0,
                action: Action::Off,
            }),
            ..Mailbox::default()
        },
        move |shared| {
            lifecycle::run_clock(
                shared,
                move |identity, cancellation| {
                    transport_events.send(Event::Open(identity.serial().to_owned()))?;
                    Ok(Peer {
                        events: transport_events.clone(),
                        cancellation,
                        poll: None,
                        block: false,
                    })
                },
                false,
                &move |id, state| {
                    assert!(events.send(Event::Status(id, state)).is_ok());
                },
                case,
                clock,
            )
        },
    )?;
    worker
        .control()
        .bind_identity(Identity::selected_hid("18F73BA2")?)?;
    Ok((worker, receiver))
}

#[test]
fn diagnostic_off_never_opens_even_when_requested_on() -> anyhow::Result<()> {
    let (mut worker, events) = case_fixture(crate::screen_motor_diag::Case::Off)?;
    assert_eq!(next(&events)?, Event::Status(0, State::Inactive));
    worker.publish(pages(false));
    worker.control().request(1, Action::On);
    assert_eq!(next(&events)?, Event::Status(1, State::Inactive));
    drop(worker);
    assert!(events.try_recv().is_err());
    Ok(())
}

#[test]
fn diagnostic_acquired_holds_session_without_publishing() -> anyhow::Result<()> {
    let (mut worker, events) = case_fixture(crate::screen_motor_diag::Case::Acquired)?;
    worker.publish(pages(false));
    let control = worker.control();
    control.request(1, Action::On);
    assert_eq!(next(&events)?, Event::Status(1, State::Pending));
    assert_eq!(next(&events)?, Event::Open("18F73BA2".to_owned()));
    worker.publish(pages(true));
    control.request(2, Action::Off);
    assert_eq!(next(&events)?, Event::Released);
    assert_eq!(next(&events)?, Event::Status(2, State::Inactive));
    drop(worker);
    assert!(events.try_recv().is_err());
    Ok(())
}

#[test]
fn diagnostic_static_freezes_pair_until_new_generation() -> anyhow::Result<()> {
    let (mut worker, events) = case_fixture(crate::screen_motor_diag::Case::Static)?;
    worker.publish(pages(false));
    let control = worker.control();
    control.request(1, Action::On);
    assert_eq!(next(&events)?, Event::Status(1, State::Pending));
    assert_eq!(next(&events)?, Event::Open("18F73BA2".to_owned()));
    assert_eq!(next(&events)?, rendered(pages(false)));
    assert_eq!(next(&events)?, Event::Status(1, State::Active));
    worker.publish(pages(true));
    control.request(2, Action::Retry);
    assert_eq!(next(&events)?, Event::Released);
    assert_eq!(next(&events)?, Event::Status(2, State::Pending));
    assert_eq!(next(&events)?, Event::Open("18F73BA2".to_owned()));
    assert_eq!(next(&events)?, rendered(pages(true)));
    assert_eq!(next(&events)?, Event::Status(2, State::Active));
    drop(worker);
    assert_eq!(next(&events)?, Event::Released);
    Ok(())
}

#[test]
fn diagnostic_updates_admit_at_most_twice_per_second_without_catchup() {
    let start = Instant::now();
    let mut next = None;
    let mut bright = false;
    assert!(update_phase(start, &mut next, &mut bright));
    assert!(bright);
    assert!(!update_phase(
        start + Duration::from_millis(499),
        &mut next,
        &mut bright
    ));
    assert!(update_phase(
        start + Duration::from_millis(500),
        &mut next,
        &mut bright
    ));
    assert!(!bright);
    assert!(update_phase(
        start + Duration::from_secs(5),
        &mut next,
        &mut bright
    ));
    assert!(bright);
    assert!(!update_phase(
        start + Duration::from_secs(5),
        &mut next,
        &mut bright
    ));
}

#[test]
fn diagnostic_updates_publish_initial_real_pair() -> anyhow::Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    let start = Instant::now();
    let elapsed = Arc::new(AtomicU64::new(0));
    let clock_elapsed = Arc::clone(&elapsed);
    let (mut worker, events) =
        case_fixture_clock(crate::screen_motor_diag::Case::Updates, move || {
            start + Duration::from_millis(clock_elapsed.load(Ordering::SeqCst))
        })?;
    worker.publish(pages(false));
    worker.control().request(1, Action::On);
    assert_eq!(next(&events)?, Event::Status(1, State::Pending));
    assert_eq!(next(&events)?, Event::Open("18F73BA2".to_owned()));
    let Event::Pair(initial) = next(&events)? else {
        anyhow::bail!("missing initial pair")
    };
    assert_eq!(Event::Pair(initial.clone()), rendered(pages(false)));
    assert_eq!(next(&events)?, Event::Status(1, State::Active));
    elapsed.store(500, Ordering::SeqCst);
    worker.publish(pages(true));
    let Event::Pair(updated) = next(&events)? else {
        anyhow::bail!("missing updated pair")
    };
    let mut expected = initial;
    for frame in &mut expected {
        frame[0] ^= u16::MAX;
    }
    assert_eq!(updated, expected);
    drop(worker);
    assert_eq!(next(&events)?, Event::Released);
    Ok(())
}
