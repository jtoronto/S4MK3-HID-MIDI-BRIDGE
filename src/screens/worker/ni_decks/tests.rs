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
    Tension(u8),
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

    async fn set_jog_tension(&mut self, value: u8) -> anyhow::Result<()> {
        self.events.send(Event::Tension(value))?;
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

fn busy_worker(gate_first_pair: bool, initial_tension: Option<u8>) -> anyhow::Result<Fixture> {
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
        None,
        initial_tension,
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
    if let Some(value) = initial_tension {
        assert_eq!(events.recv_timeout(WAIT)?, Event::Tension(value));
    }
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
fn startup_tension_precedes_readiness_and_stop_waits_for_quiesce() -> anyhow::Result<()> {
    let Fixture {
        mut worker,
        control,
        events,
        states,
        resume,
    } = busy_worker(true, Some(83))?;
    assert!(!control.ni_decks_active()?);
    assert!(states.try_recv().is_err());
    assert!(resume.send(true).is_ok());
    assert_eq!(states.recv_timeout(WAIT)?, State::Active);
    assert!(control.ni_decks_active()?);
    assert_eq!(events.recv_timeout(WAIT)?, Event::Outputs(PLAYING));
    worker.stop_ni_decks()?;
    assert_eq!(events.recv_timeout(WAIT)?, Event::Quiesced);
    assert_eq!(events.recv_timeout(WAIT)?, Event::Closed);
    assert!(!control.ni_decks_active()?);
    Ok(())
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
    } = busy_worker(false, None)?;
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
fn tension_coalesces_behind_screen_pair_and_pause_barrier() -> anyhow::Result<()> {
    let Fixture {
        mut worker,
        control,
        events,
        states: _,
        resume,
    } = busy_worker(false, None)?;
    let paused = PLAYING.map(|output| DeckOutput {
        playing: false,
        ..output
    });
    control.set_ni_jog_tension(0)?;
    control.set_ni_jog_tension(255)?;
    control.queue_ni_decks(paused)?;
    assert!(resume.send(true).is_ok());
    assert_eq!(events.recv_timeout(WAIT)?, Event::Outputs(paused));
    assert_eq!(events.recv_timeout(WAIT)?, Event::Tension(255));
    worker.stop_ni_decks()?;
    assert_eq!(events.recv_timeout(WAIT)?, Event::Quiesced);
    assert_eq!(events.recv_timeout(WAIT)?, Event::Closed);
    assert!(control.set_ni_jog_tension(3).is_err());
    Ok(())
}

#[test]
fn trial_mode_rejects_managed_tension_without_touching_transport() -> anyhow::Result<()> {
    let trial = trial::Trial::new("unused".into())?;
    let mut worker = Worker::start_mixed(
        |_, _| async { Err::<Peer, _>(Error::Disconnected) },
        |_| {},
        Some(trial),
        None,
    )?;
    assert!(worker.control().set_ni_jog_tension(25).is_err());
    worker.stop_ni_decks()?;
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
    } = busy_worker(gate_first_pair, None)?;
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
        None,
        None,
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
            None,
            None,
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

#[test]
fn trial_never_drives_even_when_play_and_turntable_requested() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("s4-jogt-worker-{}", std::process::id()));
    std::fs::write(&path, b"idle")?;
    let (send, events) = bounded(16);
    let mut worker = Worker::start_mixed(
        move |_, live| async move {
            Ok(Peer {
                events: send,
                frame_gate: None,
                play_gate: None,
                live,
            })
        },
        |_| {},
        Some(trial::Trial::new(path.clone())?),
        None,
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
    assert_eq!(
        events.recv_timeout(WAIT)?,
        Event::Outputs(PLAYING.map(|deck| DeckOutput {
            playing: false,
            turntable: false,
            ..deck
        }))
    );
    worker.stop_ni_decks()?;
    assert_eq!(events.recv_timeout(WAIT)?, Event::Quiesced);
    assert_eq!(events.recv_timeout(WAIT)?, Event::Closed);
    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn changed_trial_quiesces_before_restoring_baseline() -> anyhow::Result<()> {
    let path = std::env::temp_dir().join(format!("s4-jogt-restore-{}", std::process::id()));
    std::fs::write(&path, b"idle")?;
    let mut trial = trial::Trial::new(path.clone())?;
    let paused = [DeckOutput {
        ring: Some(1),
        ..DeckOutput::default()
    }; 2];
    assert_eq!(trial.command(paused, 0)?, None);
    std::fs::write(&path, b"25")?;
    assert_eq!(trial.command(paused, 0)?, Some(25));
    trial.restored(25);
    let (send, events) = bounded(4);
    let (_keep, live) = watch::channel(false);
    let mut peer = Peer {
        events: send,
        frame_gate: None,
        play_gate: None,
        live,
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    shutdown(&runtime, &mut peer, Some(&mut trial))?;
    assert_eq!(events.recv_timeout(WAIT)?, Event::Quiesced);
    assert_eq!(events.recv_timeout(WAIT)?, Event::Tension(107));
    std::fs::remove_file(path)?;
    Ok(())
}
