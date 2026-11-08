use super::*;
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};

#[derive(Debug, PartialEq, Eq)]
enum Event {
    Open(String),
    Pair([Vec<u16>; 2]),
    Cancelled,
    Released,
    Status(u64, State),
}

struct Peer {
    events: Sender<Event>,
    cancellation: Cancellation,
    poll: Option<Receiver<()>>,
    block: bool,
}

impl Drop for Peer {
    fn drop(&mut self) {
        assert!(
            self.events.send(Event::Released).is_ok(),
            "owned NI test session release lost"
        );
    }
}

impl Transport for Peer {
    fn publish(&mut self, frames: &[Frame; 2]) -> anyhow::Result<()> {
        self.events.send(Event::Pair([
            frames[0].pixels.clone(),
            frames[1].pixels.clone(),
        ]))?;
        if self.block {
            self.block = false;
            // The same watch cancellation boundary used by production setup/I/O.
            tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()?
                .block_on(async {
                    tokio::time::timeout(Duration::from_secs(3), async {
                        while !*self.cancellation.borrow_and_update() {
                            if self.cancellation.changed().await.is_err() {
                                break;
                            }
                        }
                    })
                    .await
                })?;
            self.events.send(Event::Cancelled)?;
            anyhow::bail!("cancelled NI test publication");
        }
        Ok(())
    }

    fn poll_control(&mut self) -> anyhow::Result<()> {
        if let Some(poll) = &self.poll
            && poll.try_recv().is_ok()
        {
            anyhow::bail!("control lost");
        }
        Ok(())
    }
}

fn next(events: &Receiver<Event>) -> anyhow::Result<Event> {
    Ok(events.recv_timeout(Duration::from_secs(3))?)
}

fn pages(color: bool) -> [page::Page; 2] {
    [page::Page::Stems {
        shifted: false,
        fx: page::FxWarning::default(),
        backspin: None,
        pads: [page::PadState {
            active: Some(color),
            held: false,
        }; 8],
    }; 2]
}

fn rendered(pages: [page::Page; 2]) -> Event {
    Event::Pair(pages.map(|page| {
        let mut frame = Frame::default();
        page::render(&page, &mut frame);
        frame.pixels
    }))
}

fn fixture(block: bool, poll: Option<Receiver<()>>) -> anyhow::Result<(Worker, Receiver<Event>)> {
    let (events, receiver) = unbounded();
    let transport_events = events.clone();
    let worker = Worker::start_ni(
        move |identity, cancellation| {
            transport_events.send(Event::Open(identity.serial().to_owned()))?;
            Ok(Peer {
                events: transport_events.clone(),
                cancellation,
                poll: poll.clone(),
                block,
            })
        },
        false,
        move |id, state| {
            assert!(
                events.send(Event::Status(id, state)).is_ok(),
                "NI status lost"
            );
        },
    )?;
    worker
        .control()
        .bind_identity(Identity::selected_hid("18F73BA2")?)?;
    Ok((worker, receiver))
}

#[test]
fn off_closes_without_overwrite_and_retry_repaints_latest_pair() -> anyhow::Result<()> {
    let (mut worker, events) = fixture(false, None)?;
    worker.publish(pages(false));
    let control = worker.control();
    control.request(1, Action::On);
    assert_eq!(next(&events)?, Event::Status(1, State::Pending));
    assert_eq!(next(&events)?, Event::Open("18F73BA2".to_owned()));
    assert_eq!(next(&events)?, rendered(pages(false)));
    assert_eq!(next(&events)?, Event::Status(1, State::Active));
    control.request(2, Action::Off);
    assert_eq!(next(&events)?, Event::Released);
    assert_eq!(next(&events)?, Event::Status(2, State::Inactive));
    worker.publish(pages(true));
    control.request(3, Action::Retry);
    assert_eq!(next(&events)?, Event::Status(3, State::Pending));
    assert_eq!(next(&events)?, Event::Open("18F73BA2".to_owned()));
    assert_eq!(next(&events)?, rendered(pages(true)));
    assert_eq!(next(&events)?, Event::Status(3, State::Active));
    drop(worker);
    assert_eq!(next(&events)?, Event::Released);
    Ok(())
}

#[test]
fn distinct_pages_reach_the_corresponding_frame_slots() -> anyhow::Result<()> {
    let (mut worker, events) = fixture(false, None)?;
    let distinct = [page::Page::Empty, pages(true)[1]];
    worker.publish(distinct);
    worker.control().request(4, Action::On);
    assert_eq!(next(&events)?, Event::Status(4, State::Pending));
    assert_eq!(next(&events)?, Event::Open("18F73BA2".to_owned()));
    assert_eq!(next(&events)?, rendered(distinct));
    assert_eq!(next(&events)?, Event::Status(4, State::Active));
    drop(worker);
    assert_eq!(next(&events)?, Event::Released);
    Ok(())
}

#[test]
fn superseding_generation_cancels_blocked_io_and_suppresses_stale_active() -> anyhow::Result<()> {
    let (mut worker, events) = fixture(true, None)?;
    worker.publish(pages(false));
    let control = worker.control();
    control.request(11, Action::On);
    assert_eq!(next(&events)?, Event::Status(11, State::Pending));
    assert_eq!(next(&events)?, Event::Open("18F73BA2".to_owned()));
    assert!(matches!(next(&events)?, Event::Pair(_)));
    control.request(12, Action::Off);
    assert_eq!(next(&events)?, Event::Cancelled);
    assert_eq!(next(&events)?, Event::Released);
    assert_eq!(next(&events)?, Event::Status(12, State::Inactive));
    drop(worker);
    assert!(events.try_recv().is_err());
    Ok(())
}

#[test]
fn idle_control_loss_releases_before_failure_and_preserves_retry_pages() -> anyhow::Result<()> {
    let (lose, poll) = bounded(1);
    let (mut worker, events) = fixture(false, Some(poll))?;
    worker.publish(pages(true));
    let control = worker.control();
    control.request(21, Action::On);
    assert_eq!(next(&events)?, Event::Status(21, State::Pending));
    assert_eq!(next(&events)?, Event::Open("18F73BA2".to_owned()));
    let initial = next(&events)?;
    assert!(matches!(initial, Event::Pair(_)));
    assert_eq!(next(&events)?, Event::Status(21, State::Active));
    lose.send(())?;
    assert_eq!(next(&events)?, Event::Released);
    assert_eq!(next(&events)?, Event::Status(21, State::Failed));
    control.request(22, Action::Retry);
    assert_eq!(next(&events)?, Event::Status(22, State::Pending));
    assert_eq!(next(&events)?, Event::Open("18F73BA2".to_owned()));
    assert_eq!(next(&events)?, initial);
    assert_eq!(next(&events)?, Event::Status(22, State::Active));
    drop(worker);
    assert_eq!(next(&events)?, Event::Released);
    Ok(())
}

#[test]
fn drop_cancels_publication_before_join_and_sends_no_shutdown_frame() -> anyhow::Result<()> {
    let (mut worker, events) = fixture(true, None)?;
    worker.publish(pages(false));
    worker.control().request(25, Action::On);
    assert_eq!(next(&events)?, Event::Status(25, State::Pending));
    assert_eq!(next(&events)?, Event::Open("18F73BA2".to_owned()));
    assert!(matches!(next(&events)?, Event::Pair(_)));
    drop(worker);
    assert_eq!(next(&events)?, Event::Cancelled);
    assert_eq!(next(&events)?, Event::Released);
    assert!(events.try_recv().is_err());
    Ok(())
}

#[test]
fn missing_or_rebound_identity_never_opens_service() -> anyhow::Result<()> {
    let (events, received) = unbounded();
    let opened = events.clone();
    let mut worker = Worker::start_ni(
        move |_, cancellation| {
            opened.send(Event::Open("unexpected".to_owned()))?;
            Ok(Peer {
                events: opened.clone(),
                cancellation,
                poll: None,
                block: false,
            })
        },
        false,
        move |id, state| {
            assert!(events.send(Event::Status(id, state)).is_ok());
        },
    )?;
    let control = worker.control();
    control.bind_identity(Identity::selected_hid("18F73BA2")?)?;
    assert!(
        control
            .bind_identity(Identity::selected_hid("OTHER123")?)
            .is_err()
    );
    // An independent unbound worker must not fall back to the first service unit.
    drop(worker);
    let (events, received_unbound) = unbounded();
    let opened = events.clone();
    worker = Worker::start_ni(
        move |_, cancellation| {
            opened.send(Event::Open("unexpected".to_owned()))?;
            Ok(Peer {
                events: opened.clone(),
                cancellation,
                poll: None,
                block: false,
            })
        },
        false,
        move |id, state| {
            assert!(events.send(Event::Status(id, state)).is_ok());
        },
    )?;
    worker.publish(pages(false));
    worker.control().request(31, Action::On);
    assert_eq!(next(&received_unbound)?, Event::Status(31, State::Pending));
    assert_eq!(next(&received_unbound)?, Event::Status(31, State::Failed));
    assert!(received.try_recv().is_err());
    drop(worker);
    Ok(())
}
