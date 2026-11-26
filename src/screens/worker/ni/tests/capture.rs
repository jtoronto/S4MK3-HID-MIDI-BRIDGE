use super::*;

#[test]
fn diagnostic_capture_correlates_real_transactions_and_pair_hashes() -> anyhow::Result<()> {
    use crate::screen_motor_diag::{Capture, Case, Role};
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "s4-screen-diag-{}-{unique}.jsonl",
        std::process::id()
    ));
    let capture = Capture::start(&path, Case::Normal)?;
    let thread_capture = capture.clone();
    let (events, received) = unbounded();
    let transport_events = events.clone();
    let mut worker = Worker::spawn(
        Mailbox {
            latest: Some(Request {
                id: 0,
                generation: 0,
                action: Action::Off,
            }),
            ..Mailbox::default()
        },
        move |shared| {
            let _guard = thread_capture.enter(Role::Screen);
            lifecycle::run_case(
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
                Case::Normal,
            )
        },
    )?;
    worker
        .control()
        .bind_identity(Identity::selected_hid("18F73BA2")?)?;
    worker.publish(pages(false));
    worker.control().request(1, Action::On);
    assert_eq!(next(&received)?, Event::Status(1, State::Pending));
    assert_eq!(next(&received)?, Event::Open("18F73BA2".to_owned()));
    let Event::Pair(pair) = next(&received)? else {
        anyhow::bail!("missing published pair")
    };
    assert_eq!(next(&received)?, Event::Status(1, State::Active));
    drop(worker);
    assert_eq!(next(&received)?, Event::Released);
    capture.finish(true)?;
    let contents = std::fs::read_to_string(&path)?;
    std::fs::remove_file(path)?;
    let rows = contents
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()?;
    let find = |kind: &str| {
        rows.iter()
            .find(|row| row["kind"] == kind)
            .context("missing event")
    };
    for (begin, end) in [
        ("screen_connect_begin", "screen_ready"),
        ("screen_render_begin", "screen_render_end"),
        ("screen_publish_begin", "screen_publish_end"),
        ("screen_release_begin", "screen_release_end"),
    ] {
        assert_eq!(find(end)?["values"][0], find(begin)?["seq"]);
    }
    assert_eq!(find("screen_publish_end")?["values"][2], 1);
    assert_eq!(
        find("screen_publish_end")?["values"][4],
        crate::screens::route_fingerprint(&pair[0])
    );
    assert_eq!(
        find("screen_publish_end")?["values"][5],
        crate::screens::route_fingerprint(&pair[1])
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["kind"] == "screen_case_ready")
            .count(),
        1
    );
    Ok(())
}

#[test]
fn diagnostic_failed_open_records_failure_without_readiness() -> anyhow::Result<()> {
    use crate::screen_motor_diag::{Capture, Case, Role};
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "s4-screen-diag-failure-{}-{unique}.jsonl",
        std::process::id()
    ));
    let capture = Capture::start(&path, Case::Acquired)?;
    let thread_capture = capture.clone();
    let (events, received) = unbounded();
    let mut worker = Worker::spawn(
        Mailbox {
            latest: Some(Request {
                id: 0,
                generation: 0,
                action: Action::Off,
            }),
            ..Mailbox::default()
        },
        move |shared| {
            let _guard = thread_capture.enter(Role::Screen);
            lifecycle::run_case(
                shared,
                |_, _| -> anyhow::Result<Peer> { anyhow::bail!("grant rejected") },
                false,
                &move |id, state| {
                    assert!(events.send(Event::Status(id, state)).is_ok());
                },
                Case::Acquired,
            )
        },
    )?;
    worker
        .control()
        .bind_identity(Identity::selected_hid("18F73BA2")?)?;
    worker.publish(pages(false));
    worker.control().request(1, Action::On);
    assert_eq!(next(&received)?, Event::Status(1, State::Pending));
    assert_eq!(next(&received)?, Event::Status(1, State::Failed));
    drop(worker);
    capture.finish(true)?;
    let contents = std::fs::read_to_string(&path)?;
    std::fs::remove_file(path)?;
    let rows = contents
        .lines()
        .map(serde_json::from_str::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>()?;
    let end = rows
        .iter()
        .find(|row| row["kind"] == "screen_ready")
        .context("missing open outcome")?;
    assert_eq!(end["values"][2], 0);
    assert!(!rows.iter().any(|row| row["kind"] == "screen_case_ready"));
    Ok(())
}
