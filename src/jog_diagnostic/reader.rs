//! A bounded snapshot producer, independent of the motor writer's deadline.

use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, TryRecvError, TrySendError, bounded};
use hidapi::HidDevice;
use tracing::warn;

use super::capture::RawFrame;
use super::continuity::OUTPUT_INTERVAL_US;
use super::reader_telemetry::{Operation, PollEvent, Snapshot, Telemetry};

pub struct Reader {
    pub frames: Receiver<RawFrame>,
    pub done: Receiver<anyhow::Result<()>>,
    cancel: Sender<()>,
    handle: Option<JoinHandle<()>>,
    telemetry: Option<Arc<Mutex<Telemetry>>>,
    pub completed: bool,
}

impl Reader {
    #[cfg(test)]
    pub fn channel_only() -> (Self, Sender<RawFrame>, Sender<anyhow::Result<()>>) {
        let (frames_tx, frames) = bounded(32);
        let (done_tx, done) = bounded(1);
        let (cancel, _) = bounded(1);
        (
            Self {
                frames,
                done,
                cancel,
                handle: None,
                telemetry: None,
                completed: false,
            },
            frames_tx,
            done_tx,
        )
    }

    pub fn start(hid: HidDevice, started: Instant, include_analog: bool) -> anyhow::Result<Self> {
        Self::start_inner(hid, started, include_analog, None)
    }

    pub fn start_instrumented(
        hid: HidDevice,
        started: Instant,
        include_analog: bool,
    ) -> anyhow::Result<Self> {
        Self::start_inner(
            hid,
            started,
            include_analog,
            Some(Arc::new(Mutex::new(Telemetry::new(started)))),
        )
    }

    pub fn telemetry(&self) -> anyhow::Result<Option<Snapshot>> {
        self.telemetry
            .as_ref()
            .map(|telemetry| {
                let mut telemetry = telemetry
                    .lock()
                    .map_err(|_| anyhow::anyhow!("reader telemetry lock poisoned"))?;
                Ok(telemetry.snapshot(self.frames.len()))
            })
            .transpose()
    }

    fn start_inner(
        hid: HidDevice,
        started: Instant,
        include_analog: bool,
        telemetry: Option<Arc<Mutex<Telemetry>>>,
    ) -> anyhow::Result<Self> {
        let (frame_tx, frames) = bounded(32);
        let (done_tx, done) = bounded(1);
        let (cancel, cancel_rx) = bounded(1);
        let worker_telemetry = telemetry.clone();
        let handle = std::thread::Builder::new()
            .name("jog-diagnostic-input".to_owned())
            .spawn(move || {
                let result = run_reader(
                    &hid,
                    started,
                    include_analog,
                    worker_telemetry.as_ref(),
                    &frame_tx,
                    &cancel_rx,
                );
                // Completion means native handle teardown has also finished.
                drop(hid);
                if let Err(error) = done_tx.try_send(result) {
                    warn!(%error, "DIAGNOSTIC_READER_RESULT_UNDELIVERED");
                }
            })
            .context("start snapshot reader")?;
        Ok(Self {
            frames,
            done,
            cancel,
            handle: Some(handle),
            telemetry,
            completed: false,
        })
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        match self.cancel.try_send(()) {
            Ok(()) | Err(crossbeam_channel::TrySendError::Disconnected(())) => {}
            Err(error) => bail!("cancel snapshot reader: {error}"),
        }
        if !self.completed {
            let result = self
                .done
                .recv_timeout(Duration::from_millis(50))
                .context("reader did not stop within 50ms; exit diagnostic process")?;
            self.completed = true;
            result?;
        }
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("snapshot reader panicked"))?;
        }
        Ok(())
    }
}

fn run_reader(
    hid: &HidDevice,
    started: Instant,
    include_analog: bool,
    telemetry: Option<&Arc<Mutex<Telemetry>>>,
    frame_tx: &Sender<RawFrame>,
    cancel_rx: &Receiver<()>,
) -> anyhow::Result<()> {
    let mut buffers = [[0_u8; 128]; 3];
    let mut next_poll_us = 0;
    let clock = || -> anyhow::Result<u64> { Ok(u64::try_from(started.elapsed().as_micros())?) };
    let wait = |remaining_us| match cancel_rx.recv_timeout(Duration::from_micros(remaining_us)) {
        Ok(()) | Err(RecvTimeoutError::Disconnected) => true,
        Err(RecvTimeoutError::Timeout) => false,
    };
    let poll_observed = |event| {
        if let Some(telemetry) = telemetry {
            let mut telemetry = telemetry
                .lock()
                .map_err(|_| anyhow::anyhow!("reader telemetry lock poisoned"))?;
            match event {
                PollEvent::WaitStarted { .. } | PollEvent::WaitReturned { .. } => {
                    telemetry.poll_event(event);
                }
                PollEvent::PollAdmitted { lateness_us, .. } => {
                    telemetry.poll_lateness(Duration::from_micros(lateness_us));
                    telemetry.poll_event(event);
                }
            }
        }
        Ok(())
    };
    while poll_slot_observed(
        &mut next_poll_us,
        cancel_rx,
        (clock, wait),
        poll_observed,
        telemetry.is_some(),
    )? {
        let (analog, lengths) = acquire_snapshot(hid, include_analog, &mut buffers, telemetry)?;
        let frame = RawFrame {
            host_us: u64::try_from(started.elapsed().as_micros())?,
            buttons: buffers[0][..lengths[0]].to_vec(),
            jog: buffers[1][..lengths[1]].to_vec(),
            analog,
        };
        let publication_started = Instant::now();
        if !publish_frame(frame, (frame_tx, cancel_rx))? {
            return Ok(());
        }
        if let Some(telemetry) = telemetry {
            let mut telemetry = telemetry
                .lock()
                .map_err(|_| anyhow::anyhow!("reader telemetry lock poisoned"))?;
            telemetry.publication_finished(publication_started.elapsed(), frame_tx.len());
        }
    }
    Ok(())
}

fn acquire_snapshot(
    hid: &HidDevice,
    include_analog: bool,
    buffers: &mut [[u8; 128]; 3],
    telemetry: Option<&Arc<Mutex<Telemetry>>>,
) -> anyhow::Result<(Option<Vec<u8>>, [usize; 3])> {
    let batch_started = Instant::now();
    let acquired: anyhow::Result<(Option<Vec<u8>>, [usize; 3])> = (|| {
        let analog = if include_analog {
            Some(match telemetry {
                Some(telemetry) => {
                    let mut observer = ReaderTelemetryObserver { telemetry };
                    crate::input::read_analog_with_observer(
                        hid,
                        Some(&mut observer),
                        "request S4 analog report 2",
                        "unexpected S4 analog snapshot framing",
                    )?
                    .to_vec()
                }
                None => crate::input::read_analog_with_observer(
                    hid,
                    None,
                    "request S4 analog report 2",
                    "unexpected S4 analog snapshot framing",
                )?
                .to_vec(),
            })
        } else {
            None
        };
        let lengths = match telemetry {
            Some(telemetry) => {
                let mut observer = ReaderTelemetryObserver { telemetry };
                crate::input::read_batch_with_observer(hid, true, buffers, Some(&mut observer))?
            }
            None => crate::input::read_batch(hid, true, buffers)?,
        };
        Ok((analog, lengths))
    })();
    let (analog, lengths) = acquired?;
    if let Some(telemetry) = telemetry {
        let mut telemetry = telemetry
            .lock()
            .map_err(|_| anyhow::anyhow!("reader telemetry lock poisoned"))?;
        telemetry.batch_finished(batch_started.elapsed());
    }
    Ok((analog, lengths))
}

struct ReaderTelemetryObserver<'a> {
    telemetry: &'a Arc<Mutex<Telemetry>>,
}

impl crate::input::ReadObserver for ReaderTelemetryObserver<'_> {
    fn begin(&mut self, operation: crate::input::ReadOperation) -> anyhow::Result<()> {
        let mut telemetry = self
            .telemetry
            .lock()
            .map_err(|_| anyhow::anyhow!("reader telemetry lock poisoned"))?;
        telemetry.begin(map_operation(operation));
        drop(telemetry);
        Ok(())
    }

    fn end(
        &mut self,
        operation: crate::input::ReadOperation,
        duration: Duration,
        failed: bool,
    ) -> anyhow::Result<()> {
        let mut telemetry = self
            .telemetry
            .lock()
            .map_err(|_| anyhow::anyhow!("reader telemetry lock poisoned"))?;
        telemetry.end(map_operation(operation), duration, failed);
        drop(telemetry);
        Ok(())
    }
}

const fn map_operation(operation: crate::input::ReadOperation) -> Operation {
    match operation {
        crate::input::ReadOperation::Get1 => Operation::Get1,
        crate::input::ReadOperation::Get3 => Operation::Get3,
        crate::input::ReadOperation::InterruptRead => Operation::InterruptRead,
        crate::input::ReadOperation::Get2Analog => Operation::Get2Analog,
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        // Never wait on a possibly stalled native call during unwinding.
        if self.handle.is_some() {
            match self.cancel.try_send(()) {
                Ok(()) | Err(crossbeam_channel::TrySendError::Disconnected(())) => {}
                Err(error) => warn!(%error, "DIAGNOSTIC_READER_CANCEL_FAILED"),
            }
        }
    }
}

#[cfg(test)]
fn poll_slot<C, W>(
    next_poll_us: &mut u64,
    cancel: &Receiver<()>,
    timing: (C, W),
) -> anyhow::Result<bool>
where
    C: FnMut() -> anyhow::Result<u64>,
    W: FnMut(u64) -> bool,
{
    poll_slot_observed(next_poll_us, cancel, timing, |_| Ok(()), false)
}

fn poll_slot_observed<C, W, O>(
    next_poll_us: &mut u64,
    cancel: &Receiver<()>,
    mut timing: (C, W),
    mut observe_poll: O,
    instrumented: bool,
) -> anyhow::Result<bool>
where
    C: FnMut() -> anyhow::Result<u64>,
    W: FnMut(u64) -> bool,
    O: FnMut(PollEvent) -> anyhow::Result<()>,
{
    let mut last_wait_return_us = None;
    loop {
        match cancel.try_recv() {
            Ok(()) | Err(TryRecvError::Disconnected) => return Ok(false),
            Err(TryRecvError::Empty) => {}
        }
        let now_us = timing.0()?;
        if now_us >= *next_poll_us {
            if instrumented {
                observe_poll(PollEvent::PollAdmitted {
                    started_us: now_us,
                    lateness_us: now_us.saturating_sub(*next_poll_us),
                    post_wait_us: last_wait_return_us
                        .map(|returned| now_us.saturating_sub(returned)),
                })?;
            }
            // Schedule from this actual start, not missed deadlines. An
            // available interrupt cannot create a catch-up polling burst.
            *next_poll_us = now_us.saturating_add(OUTPUT_INTERVAL_US);
            return Ok(true);
        }
        let requested_us = *next_poll_us - now_us;
        if instrumented {
            observe_poll(PollEvent::WaitStarted {
                requested_us,
                scheduled_deadline_us: *next_poll_us,
                started_us: now_us,
            })?;
        }
        let cancelled = timing.1(requested_us);
        if instrumented {
            let returned_us = timing.0()?;
            observe_poll(PollEvent::WaitReturned {
                returned_us,
                cancelled,
            })?;
            last_wait_return_us = Some(returned_us);
        }
        if cancelled {
            return Ok(false);
        }
    }
}

fn publish_frame(
    frame: RawFrame,
    channels: (&Sender<RawFrame>, &Receiver<()>),
) -> anyhow::Result<bool> {
    match channels.1.try_recv() {
        Ok(()) | Err(TryRecvError::Disconnected) => return Ok(false),
        Err(TryRecvError::Empty) => {}
    }
    match channels.0.try_send(frame) {
        Ok(()) => Ok(true),
        Err(TrySendError::Full(frame)) => bail!(
            "snapshot queue overflow: capacity={:?}, rejected_snapshot_us={}",
            channels.0.capacity(),
            frame.host_us
        ),
        Err(TrySendError::Disconnected(frame)) => bail!(
            "snapshot consumer disconnected: rejected_snapshot_us={}",
            frame.host_us
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jog_diagnostic::decode::tests::frame;

    #[test]
    fn cancellation_after_snapshot_prevents_full_queue_publication() -> anyhow::Result<()> {
        // Given a full snapshot queue and cancellation while the native read ran.
        let (frames_tx, _frames_rx) = bounded(1);
        frames_tx.send(frame(0, 1, [0, 0]))?;
        let (cancel_tx, cancel_rx) = bounded(1);
        cancel_tx.send(())?;
        // When the completed snapshot is offered to the real publication path.
        let continued = publish_frame(frame(2000, 2, [0, 0]), (&frames_tx, &cancel_rx))?;
        // Then cancellation wins instead of reporting overflow on normal Stop.
        assert!(!continued);
        Ok(())
    }

    #[test]
    fn channel_only_reader_keeps_telemetry_disabled() -> anyhow::Result<()> {
        // Given the hardware-independent reader fixture.
        let (reader, _frames, _done) = Reader::channel_only();

        // When telemetry is queried.
        let snapshot = reader.telemetry()?;

        // Then the legacy/test construction allocates no telemetry state.
        assert!(snapshot.is_none());
        Ok(())
    }
}

#[cfg(test)]
#[path = "reader_pacing_tests.rs"]
mod pacing_tests;
