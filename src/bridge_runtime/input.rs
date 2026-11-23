//! Bounded, explicitly paced full-profile snapshots; interrupts stay in order.

use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use crossbeam_channel::{Receiver, Sender, TryRecvError, bounded};
use hidapi::HidDevice;
use s4mk3_hid_midi_bridge::screen_motor_diag::{self as diag, Kind, Role};
use tracing::warn;

pub const INTERVAL_US: u64 = 2_000;

#[derive(Clone)]
pub struct Frame {
    pub host_us: u64,
    pub buffers: [[u8; 128]; 3],
    pub lengths: [usize; 3],
}

impl Frame {
    pub fn report(&self, index: usize) -> &[u8] {
        &self.buffers[index][..self.lengths[index]]
    }
}

pub struct Reader {
    pub frames: Receiver<Frame>,
    pub done: Receiver<anyhow::Result<()>>,
    pub completed: bool,
    cancel: Sender<()>,
    handle: Option<JoinHandle<()>>,
}

impl Reader {
    pub fn start(hid: HidDevice, started: Instant) -> anyhow::Result<Self> {
        let (tx, frames) = bounded(32);
        let (done_tx, done) = bounded(1);
        let (cancel, cancel_rx) = bounded(1);
        let handle = std::thread::Builder::new()
            .name("s4-snapshots".to_owned())
            .spawn(move || {
                let _trace = diag::enter(Role::Input);
                let result = (|| {
                    let mut next_poll = started;
                    loop {
                        let remaining = next_poll.saturating_duration_since(Instant::now());
                        match cancel_rx.recv_timeout(remaining) {
                            Ok(()) | Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                                return Ok(());
                            }
                            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                        }
                        // Actual admission, never catch up missed snapshot slots.
                        next_poll = Instant::now() + Duration::from_micros(INTERVAL_US);
                        let mut buffers = [[0; 128]; 3];
                        let lengths = crate::input::read_batch(&hid, true, &mut buffers)?;
                        match cancel_rx.try_recv() {
                            Ok(()) | Err(TryRecvError::Disconnected) => return Ok(()),
                            Err(TryRecvError::Empty) => {}
                        }
                        let host_us = u64::try_from(started.elapsed().as_micros())?;
                        let published = tx.try_send(Frame {
                            host_us,
                            buffers,
                            lengths,
                        });
                        diag::record(
                            Kind::BatchPublished,
                            [
                                host_us,
                                u64::from(published.is_ok()),
                                u64::try_from(tx.len())?,
                                u64::try_from(lengths[0])?,
                                u64::try_from(lengths[1])?,
                                u64::try_from(lengths[2])?,
                                0,
                                0,
                            ],
                            &[],
                        );
                        published
                            .context("full-profile snapshot queue overflow or consumer stopped")?;
                    }
                })();
                drop(hid);
                if let Err(error) = done_tx.try_send(result) {
                    warn!(%error, "SNAPSHOT_RESULT_UNDELIVERED");
                }
            })
            .context("start full-profile snapshot reader")?;
        Ok(Self {
            frames,
            done,
            completed: false,
            cancel,
            handle: Some(handle),
        })
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.cancel()?;
        if !self.completed {
            let result = self
                .done
                .recv_timeout(Duration::from_millis(50))
                .context("snapshot reader did not stop within 50ms; exiting bridge")?;
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

    pub(super) fn cancel(&self) -> anyhow::Result<()> {
        match self.cancel.try_send(()) {
            Ok(()) | Err(crossbeam_channel::TrySendError::Disconnected(())) => Ok(()),
            Err(error) => bail!("cancel snapshot reader: {error}"),
        }
    }

    #[cfg(test)]
    pub fn channels() -> (Self, Sender<Frame>, Sender<anyhow::Result<()>>) {
        let (tx, frames) = bounded(32);
        let (done_tx, done) = bounded(1);
        let (cancel, _cancel_rx) = bounded(1);
        (
            Self {
                frames,
                done,
                completed: false,
                cancel,
                handle: None,
            },
            tx,
            done_tx,
        )
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        if self.handle.is_some()
            && let Err(error) = self.cancel()
        {
            warn!(%error, "SNAPSHOT_CANCEL_FAILED");
        }
    }
}
