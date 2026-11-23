use super::*;
use crate::screens::{ni_service::Error, route_fingerprint};
use crossbeam_channel::{Receiver, Sender, bounded};

const WAIT: Duration = Duration::from_secs(3);

#[derive(Debug, PartialEq, Eq)]
enum Event {
    Connected,
    PreparingPlay,
    Outputs([DeckOutput; 2]),
    Frames([u64; 2]),
    Quiesced,
    Closed,
}

struct Peer {
    events: Sender<Event>,
    frame_gate: Option<(usize, tokio::sync::oneshot::Receiver<bool>)>,
    play_gate: Option<tokio::sync::oneshot::Receiver<()>>,
    live: Cancellation,
}

impl Transport for Peer {
    async fn publish(&mut self, frames: &[Frame; 2]) -> anyhow::Result<()> {
        self.events.send(Event::Frames(
            frames
                .each_ref()
                .map(|frame| route_fingerprint(&frame.pixels)),
        ))?;
        if let Some((remaining, _)) = &mut self.frame_gate {
            if *remaining == 0 {
                let (_, gate) = self.frame_gate.take().context("frame gate missing")?;
                if !tokio::time::timeout(WAIT, gate).await?? {
                    return Err(Error::ControlLost.into());
                }
            } else {
                *remaining -= 1;
            }
        }
        Ok(())
    }

    async fn apply(
        &mut self,
        mut desired: [DeckOutput; 2],
        allow_play: impl Fn(usize) -> bool + Send + Sync,
    ) -> anyhow::Result<[DeckOutput; 2]> {
        if desired.iter().any(|output| output.playing)
            && let Some(gate) = self.play_gate.take()
        {
            self.events.send(Event::PreparingPlay)?;
            tokio::time::timeout(WAIT, gate).await??;
        }
        for (side, output) in desired.iter_mut().enumerate() {
            output.playing &= allow_play(side);
        }
        self.events.send(Event::Outputs(desired))?;
        Ok(desired)
    }

    async fn poll(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn quiesce(&mut self) -> anyhow::Result<()> {
        anyhow::ensure!(
            !*self.live.borrow() && self.live.has_changed().is_ok(),
            "transport canceled before quiesce"
        );
        self.events.send(Event::Quiesced)?;
        Ok(())
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        drop(self.events.send(Event::Closed));
    }
}

struct Fixture {
    worker: Worker,
    control: Control,
    events: Receiver<Event>,
    states: Receiver<State>,
    resume: tokio::sync::oneshot::Sender<bool>,
}

const PLAYING: [DeckOutput; 2] = [
    DeckOutput {
        turntable: true,
        playing: true,
        ring: Some(3),
    },
    DeckOutput {
        turntable: true,
        playing: true,
        ring: Some(27),
    },
];

fn busy_worker(gate_first_pair: bool) -> anyhow::Result<Fixture> {
    let (send, events) = bounded(16);
    let (state_tx, states) = bounded(4);
    let (resume, first_frame) = tokio::sync::oneshot::channel();
    let mut worker = Worker::start_mixed(
        move |_, live| async move {
            send.send(Event::Connected)
                .map_err(|_| Error::Disconnected)?;
            Ok(Peer {
                events: send,
                frame_gate: Some((usize::from(!gate_first_pair), first_frame)),
                play_gate: None,
                live,
            })
        },
        move |state| assert!(state_tx.send(state).is_ok()),
    )?;
    let control = worker.control();
    control.bind_identity(Identity::selected_hid("12345678901234")?)?;
    control.queue_ni_decks(PLAYING)?;
    assert!(events.try_recv().is_err()); // Deck output alone does not acquire NI.
    let pages = [
        page::Page::Info(page::Info {
            jump: Some("8"),
            ..page::Info::default()
        }),
        page::Page::Empty,
    ];
    worker.publish(pages);
    assert_eq!(events.recv_timeout(WAIT)?, Event::Connected);
    assert_eq!(
        events.recv_timeout(WAIT)?,
        Event::Outputs([DeckOutput::default(); 2])
    );
    let hashes = |pages: [page::Page; 2]| {
        pages.map(|page| {
            let mut frame = Frame::default();
            page::render(&page, &mut frame);
            route_fingerprint(&frame.pixels)
        })
    };
    let expected = hashes(pages);
    assert_ne!(expected[0], expected[1]);
    assert_eq!(events.recv_timeout(WAIT)?, Event::Frames(expected));
    if !gate_first_pair {
        assert_eq!(states.recv_timeout(WAIT)?, State::Active);
        assert_eq!(events.recv_timeout(WAIT)?, Event::Outputs(PLAYING));
        let next = [pages[1], pages[0]];
        worker.publish(next);
        assert_eq!(events.recv_timeout(WAIT)?, Event::Frames(hashes(next)));
    }
    Ok(Fixture {
        worker,
        control,
        events,
        states,
        resume,
    })
}

#[test]
fn mixed_worker_preserves_pause_then_quiesces_before_closing_healthy_transport()
-> anyhow::Result<()> {
    let Fixture {
        mut worker,
        control,
        events,
        states: _,
        resume,
    } = busy_worker(false)?;
    let paused = PLAYING.map(|output| DeckOutput {
        playing: false,
        ..output
    });
    let resumed = [
        DeckOutput {
            ring: Some(9),
            ..PLAYING[0]
        },
        DeckOutput {
            ring: Some(33),
            ..PLAYING[1]
        },
    ];
    // Both snapshots arrive while the same transport is busy with screens.
    control.queue_ni_decks(paused)?;
    control.queue_ni_decks(resumed)?;
    assert!(resume.send(true).is_ok());
    assert_eq!(events.recv_timeout(WAIT)?, Event::Outputs(paused));
    assert_eq!(events.recv_timeout(WAIT)?, Event::Outputs(resumed));
    worker.stop_ni_decks()?;
    assert_eq!(events.recv_timeout(WAIT)?, Event::Quiesced);
    assert_eq!(events.recv_timeout(WAIT)?, Event::Closed);
    assert!(control.queue_ni_decks(PLAYING).is_err());
    Ok(())
}

#[test]
fn mixed_worker_reports_lost_control_without_quiescing_or_replaying_play() -> anyhow::Result<()> {
    for gate_first_pair in [true, false] {
        assert_loss(gate_first_pair)?;
    }
    Ok(())
}

fn assert_loss(gate_first_pair: bool) -> anyhow::Result<()> {
    let Fixture {
        mut worker,
        control,
        events,
        states,
        resume,
    } = busy_worker(gate_first_pair)?;
    let pending = PLAYING.map(|output| DeckOutput {
        ring: Some(9),
        ..output
    });
    control.queue_ni_decks(pending)?;
    assert!(resume.send(false).is_ok());
    assert_eq!(states.recv_timeout(WAIT)?, State::Failed);
    assert_eq!(events.recv_timeout(WAIT)?, Event::Closed);
    let set = control
        .queue_ni_decks(PLAYING)
        .err()
        .context("must reject live updates")?;
    let stop = worker
        .stop_ni_decks()
        .err()
        .context("must report unconfirmed shutdown")?;
    assert!(set.to_string().contains("ControlLost"));
    assert_eq!(stop.to_string(), set.to_string());
    assert_eq!(
        events.try_recv(),
        Err(crossbeam_channel::TryRecvError::Disconnected)
    );
    Ok(())
}

#[test]
fn stop_inhibits_play_from_an_in_flight_apply() -> anyhow::Result<()> {
    let (send, events) = bounded(16);
    let (resume, play_gate) = tokio::sync::oneshot::channel();
    let mut worker = Worker::start_mixed(
        move |_, live| async move {
            Ok(Peer {
                events: send,
                frame_gate: None,
                play_gate: Some(play_gate),
                live,
            })
        },
        |_| {},
    )?;
    let control = worker.control();
    control.bind_identity(Identity::selected_hid("12345678901234")?)?;
    control.queue_ni_decks(PLAYING)?;
    worker.publish([page::Page::Empty; 2]);
    assert_eq!(
        events.recv_timeout(WAIT)?,
        Event::Outputs([DeckOutput::default(); 2])
    );
    assert!(matches!(events.recv_timeout(WAIT)?, Event::Frames(_)));
    assert_eq!(events.recv_timeout(WAIT)?, Event::PreparingPlay);

    thread::scope(|scope| -> anyhow::Result<()> {
        let stop = scope.spawn(|| worker.stop_ni_decks());
        let (mailbox, _) = control
            .shared
            .1
            .wait_timeout_while(lock(&control.shared)?, WAIT, |mailbox| !mailbox.stop)
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        assert!(mailbox.stop);
        drop(mailbox);
        assert!(resume.send(()).is_ok());
        stop.join().map_err(|_| anyhow::anyhow!("stop panicked"))?
    })?;

    assert_eq!(
        events.recv_timeout(WAIT)?,
        Event::Outputs(PLAYING.map(|output| DeckOutput {
            playing: false,
            ..output
        }))
    );
    assert_eq!(events.recv_timeout(WAIT)?, Event::Quiesced);
    assert_eq!(events.recv_timeout(WAIT)?, Event::Closed);
    Ok(())
}

#[test]
fn in_flight_play_cannot_enable_a_superseded_mode() -> anyhow::Result<()> {
    for turntable in [false, true] {
        let (send, events) = bounded(16);
        let (resume, play_gate) = tokio::sync::oneshot::channel();
        let mut worker = Worker::start_mixed(
            move |_, live| async move {
                Ok(Peer {
                    events: send,
                    frame_gate: None,
                    play_gate: Some(play_gate),
                    live,
                })
            },
            |_| {},
        )?;
        let control = worker.control();
        let initial = PLAYING.map(|output| DeckOutput {
            turntable,
            ..output
        });
        let next = PLAYING.map(|output| DeckOutput {
            turntable: !turntable,
            ..output
        });
        control.bind_identity(Identity::selected_hid("12345678901234")?)?;
        control.queue_ni_decks(initial)?;
        worker.publish([page::Page::Empty; 2]);
        assert_eq!(
            events.recv_timeout(WAIT)?,
            Event::Outputs([DeckOutput::default(); 2])
        );
        assert!(matches!(events.recv_timeout(WAIT)?, Event::Frames(_)));
        assert_eq!(events.recv_timeout(WAIT)?, Event::PreparingPlay);
        control.queue_ni_decks(next)?;
        assert!(resume.send(()).is_ok());
        assert_eq!(
            events.recv_timeout(WAIT)?,
            Event::Outputs(initial.map(|output| DeckOutput {
                playing: false,
                ..output
            }))
        );
        assert_eq!(events.recv_timeout(WAIT)?, Event::Outputs(next));
        worker.stop_ni_decks()?;
        assert_eq!(events.recv_timeout(WAIT)?, Event::Quiesced);
        assert_eq!(events.recv_timeout(WAIT)?, Event::Closed);
    }
    Ok(())
}
