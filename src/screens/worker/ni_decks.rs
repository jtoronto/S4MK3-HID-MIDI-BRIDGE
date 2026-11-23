//! One non-retrying NI session for rasters, motors, and rings.

use std::future::Future;
use std::time::{Duration, Instant};

use tokio::{runtime::Runtime, sync::watch};

use super::*;
use crate::screen_motor_diag::Case;
use crate::screens::ni_service::{Cancellation, DeckOutput, Identity};

const POLL: Duration = Duration::from_millis(50);
// At 500 ms per scalar, allow 8 s for initialization or a two-deck
// change, then 6 s for quiesce, with margin for scheduling.
const STOP_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Default)]
pub(super) struct DeckMailbox {
    desired: [DeckOutput; 2],
    dirty: bool,
    pause: [bool; 2],
    failure: Option<String>,
}

impl Control {
    /// Queue the latest physical left/right output snapshot without waiting for I/O.
    ///
    /// # Errors
    /// Returns a terminal worker failure, or rejects a stopped or non-mixed worker.
    #[cfg(windows)]
    pub fn set_ni_decks(&self, desired: [DeckOutput; 2]) -> anyhow::Result<()> {
        self.queue_ni_decks(desired)
    }

    fn queue_ni_decks(&self, desired: [DeckOutput; 2]) -> anyhow::Result<()> {
        let mut mailbox = lock(&self.shared)?;
        let stopped = mailbox.stop;
        let decks = mailbox.decks.as_mut().context("NI deck mode not enabled")?;
        decks.check_failure()?;
        anyhow::ensure!(!stopped, "NI deck worker stopped");
        if decks.desired == desired {
            return Ok(());
        }
        for (side, output) in desired.iter().enumerate() {
            let old = decks.desired[side];
            decks.pause[side] |=
                old.playing && (!output.playing || old.turntable != output.turntable);
        }
        decks.desired = desired;
        decks.dirty = true;
        drop(mailbox);
        self.shared.1.notify_one();
        Ok(())
    }
}

impl DeckMailbox {
    fn check_failure(&self) -> anyhow::Result<()> {
        if let Some(error) = &self.failure {
            anyhow::bail!("NI deck worker failed; motor Pause and ring clear unconfirmed: {error}");
        }
        Ok(())
    }
}

impl Worker {
    /// Start the Windows-only shared NI session instead of the Encdr screen helper.
    ///
    /// # Errors
    /// Returns a thread start failure. Bind the selected HID before publishing
    /// initialized pages; asynchronous failures surface on set or stop.
    #[cfg(windows)]
    pub fn start_ni_decks() -> anyhow::Result<Self> {
        use crate::screens::ni_service::NiService;
        println!("S4_SCREEN_STATUS 0 pending");
        Self::start_mixed(NiService::connect, |state| {
            println!("S4_SCREEN_STATUS 0 {}", state.label());
        })
    }

    fn start_mixed<T, F, O, S>(open: F, status: S) -> anyhow::Result<Self>
    where
        T: Transport,
        F: FnOnce(Identity, Cancellation) -> O + Send + 'static,
        O: Future<Output = Result<T, crate::screens::ni_service::Error>>,
        S: Fn(State) + Send + 'static,
    {
        let shared = Arc::new((
            Mutex::new(Mailbox {
                decks: Some(DeckMailbox::default()),
                ..Mailbox::default()
            }),
            Condvar::new(),
        ));
        let task_shared = Arc::clone(&shared);
        let (finished, completion) = crossbeam_channel::bounded(1);
        let thread = thread::Builder::new()
            .name(String::from("s4-ni-decks"))
            .spawn(move || {
                let result = run(&task_shared, open, &status);
                let failed = result.is_err();
                if let Ok(mut mailbox) = lock(&task_shared) {
                    mailbox.stop = true;
                    if let (Some(decks), Err(error)) = (&mut mailbox.decks, result) {
                        decks.failure = Some(format!("{error:#}"));
                    }
                }
                if failed {
                    status(State::Failed);
                }
                if finished.send(()).is_err() {
                    tracing::debug!("NI deck worker completion receiver dropped");
                }
            })
            .context("start shared NI deck worker")?;
        Ok(Self {
            shared,
            thread: Some(thread),
            last: None,
            deck_completion: Some(completion),
        })
    }

    /// Stop accepting updates and confirm transport quiescence before returning.
    ///
    /// # Errors
    /// Reports failed or timed-out shutdown as unconfirmed, never as motor-off.
    pub fn stop_ni_decks(&mut self) -> anyhow::Result<()> {
        let completion = self
            .deck_completion
            .as_ref()
            .context("NI deck mode not enabled")?;
        {
            let mut mailbox = lock(&self.shared)?;
            mailbox.stop = true;
            if let Some(setup) = &mailbox.cancellation {
                setup.send_replace(true);
            }
            drop(mailbox);
            self.shared.1.notify_one();
        }
        if let Some(thread) = self.thread.take() {
            let failure = match completion.recv_timeout(STOP_TIMEOUT) {
                Ok(()) => {
                    if thread.join().is_ok() {
                        None
                    } else {
                        Some("worker panicked")
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => Some("shutdown timed out"),
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    Some("worker exited unexpectedly")
                }
            };
            if let Some(failure) = failure {
                let mut mailbox = lock(&self.shared)?;
                mailbox
                    .decks
                    .as_mut()
                    .context("NI deck mode not enabled")?
                    .failure
                    .get_or_insert_with(|| failure.to_owned());
            }
        }
        lock(&self.shared)?
            .decks
            .as_ref()
            .context("NI deck mode not enabled")?
            .check_failure()
    }
}

pub(super) trait Transport {
    fn publish(&mut self, frames: &[Frame; 2]) -> impl Future<Output = anyhow::Result<()>>;
    fn apply(
        &mut self,
        desired: [DeckOutput; 2],
        allow_play: impl Fn(usize) -> bool + Send + Sync,
    ) -> impl Future<Output = anyhow::Result<[DeckOutput; 2]>>;
    fn poll(&mut self) -> impl Future<Output = anyhow::Result<()>>;
    fn quiesce(&mut self) -> impl Future<Output = anyhow::Result<()>>;
}

fn lock(shared: &Shared) -> anyhow::Result<std::sync::MutexGuard<'_, Mailbox>> {
    shared.0.lock().map_err(|error| anyhow::anyhow!("{error}"))
}

fn run<T: Transport, O: Future<Output = Result<T, crate::screens::ni_service::Error>>>(
    shared: &Shared,
    open: impl FnOnce(Identity, Cancellation) -> O,
    status: &impl Fn(State),
) -> anyhow::Result<()> {
    let (identity, mut stop_setup) = {
        let mut mailbox = lock(shared)?;
        while !mailbox.stop && !mailbox.initialized {
            mailbox = shared
                .1
                .wait(mailbox)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
        }
        if mailbox.stop {
            return Ok(());
        }
        let identity = mailbox
            .identity
            .clone()
            .context("selected HID serial not bound to NI decks")?;
        let (sender, receiver) = watch::channel(false);
        mailbox.cancellation = Some(sender);
        drop(mailbox);
        (identity, receiver)
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    // Setup may be interrupted, but the cancellation receiver retained by
    // the connected transport must stay false through its final quiesce.
    let (_keep_live, live) = watch::channel(false);
    let opened = runtime.block_on(async {
        tokio::select! {
            biased;
            _ = stop_setup.changed() => None,
            result = open(identity, live) => Some(result),
        }
    });
    let stopping = {
        let mut mailbox = lock(shared)?;
        mailbox.cancellation = None;
        mailbox.stop
    };
    let Some(opened) = opened else { return Ok(()) };
    let mut session = opened.context("acquire shared NI session")?;
    if !stopping {
        // Initialize in a safe state before consuming a potentially stale Play.
        let _initialized = runtime
            .block_on(session.apply([DeckOutput::default(); 2], |_| false))
            .context("initialize shared NI decks")?;
    }
    drive(shared, &runtime, &mut session, status)
}

fn drive(
    shared: &Shared,
    runtime: &Runtime,
    session: &mut impl Transport,
    status: &impl Fn(State),
) -> anyhow::Result<()> {
    let mut presentation = ni::Presentation::new(Case::Normal);
    let first_pages = {
        let mut mailbox = lock(shared)?;
        if mailbox.stop {
            drop(mailbox);
            return runtime
                .block_on(session.quiesce())
                .context("quiesce shared NI decks");
        }
        mailbox.pending.take().context("initial NI pages missing")?
    };
    let mut current = first_pages;
    presentation.publish(first_pages, Instant::now(), |frames| {
        runtime
            .block_on(session.publish(frames))
            .context("publish first shared NI screen pair")
    })?;
    status(State::Active);
    // Take a fresh output snapshot only after both frame acknowledgements.
    let mut applied = [DeckOutput::default(); 2];
    let allow_play = |side: usize, turntable: bool| {
        lock(shared).is_ok_and(|mailbox| {
            !mailbox.stop
                && mailbox.decks.as_ref().is_some_and(|decks| {
                    let desired = decks.desired[side];
                    !decks.pause[side] && desired.turntable == turntable && desired.playing
                })
        })
    };
    loop {
        let (desired, pause) = {
            let wait = presentation.deadline().map_or(POLL, |flash| {
                flash.saturating_duration_since(Instant::now()).min(POLL)
            });
            let (mut mailbox, _) = shared
                .1
                .wait_timeout_while(lock(shared)?, wait, |mailbox| {
                    !mailbox.stop
                        && mailbox.pending.is_none()
                        && mailbox.decks.as_ref().is_some_and(|decks| !decks.dirty)
                })
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            if mailbox.stop {
                drop(mailbox);
                return runtime
                    .block_on(session.quiesce())
                    .context("quiesce shared NI decks");
            }
            current = mailbox.pending.take().unwrap_or(current);
            let decks = mailbox.decks.as_mut().context("NI deck mode not enabled")?;
            let pause = std::mem::take(&mut decks.pause);
            // Leave a fresh snapshot pending until the Pause barrier is sent.
            decks.dirty = pause.contains(&true);
            let desired = decks.desired;
            drop(mailbox);
            (desired, pause)
        };
        if pause.contains(&true) {
            let mut paused = applied;
            for (output, pause) in paused.iter_mut().zip(pause) {
                if pause {
                    output.playing = false;
                }
            }
            if paused != applied {
                applied = runtime
                    .block_on(
                        session.apply(paused, |side| allow_play(side, paused[side].turntable)),
                    )
                    .context("apply NI Pause transition")?;
            }
            continue;
        }
        if desired == applied {
            runtime
                .block_on(session.poll())
                .context("poll shared NI control")?;
        } else {
            applied = runtime
                .block_on(session.apply(desired, |side| allow_play(side, desired[side].turntable)))
                .context("apply NI deck outputs")?;
        }
        // Give queued deck transitions or shutdown precedence over the next pair.
        let deck_transition_pending = {
            let mailbox = lock(shared)?;
            mailbox.stop
                || mailbox
                    .decks
                    .as_ref()
                    .is_some_and(|decks| decks.pause.contains(&true))
        };
        if deck_transition_pending {
            continue;
        }
        presentation.publish(current, Instant::now(), |frames| {
            runtime
                .block_on(session.publish(frames))
                .context("publish shared NI screens")
        })?;
    }
}

#[cfg(windows)]
mod native;
#[cfg(test)]
mod tests;
