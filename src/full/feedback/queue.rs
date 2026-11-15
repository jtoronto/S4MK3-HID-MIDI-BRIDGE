//! Callback ownership and bounded delivery into the real feedback cache.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use super::{Decoder, FeedbackState, Message};
use crate::{midi_layout, midi_transport::Backend};
use anyhow::ensure;
use crossbeam_channel::{Receiver, Sender, TrySendError};

const CAPACITY: usize = 4096;

pub(super) struct Queue {
    pub(super) receiver: Receiver<Message>,
    overflow: Arc<AtomicBool>,
}

pub(super) struct Ingress {
    sender: Sender<Message>,
    overflow: Arc<AtomicBool>,
    backend: Backend,
}

impl Queue {
    pub(super) fn new(backend: Backend) -> (Self, Ingress) {
        let (sender, receiver) = crossbeam_channel::bounded(CAPACITY);
        let overflow = Arc::new(AtomicBool::new(false));
        (
            Self {
                receiver,
                overflow: Arc::clone(&overflow),
            },
            Ingress {
                sender,
                overflow,
                backend,
            },
        )
    }

    pub(super) fn drain(&self, state: &mut FeedbackState, now: Duration) -> anyhow::Result<usize> {
        self.check()?;
        let mut count = 0;
        for message in self.receiver.try_iter().take(CAPACITY) {
            state.apply(message, now);
            count += 1;
        }
        self.check()?;
        Ok(count)
    }

    fn check(&self) -> anyhow::Result<()> {
        ensure!(
            !self.overflow.load(Ordering::Relaxed),
            "MIDI feedback queue overflow; restart to synchronize software state"
        );
        Ok(())
    }
}

impl Ingress {
    pub(super) fn feed(&self, decoder: &mut Decoder, bytes: &[u8]) {
        decoder.feed(bytes, |message| {
            let Some(message) = midi_layout::normalize(self.backend, message) else {
                return;
            };
            tracing::debug!(?message, "MIDI_FEEDBACK");
            match self.sender.try_send(message) {
                Ok(()) => {}
                Err(TrySendError::Full(_)) => self.overflow.store(true, Ordering::Relaxed),
                Err(TrySendError::Disconnected(_)) => {
                    tracing::warn!("MIDI feedback receiver closed");
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_echoes_never_authorize_cache_and_genuine_split_feedback_does() -> anyhow::Result<()>
    {
        for backend in [Backend::Loopbe, Backend::Bloop] {
            command_echoes_and_split_feedback(backend)?;
        }
        Ok(())
    }

    fn command_echoes_and_split_feedback(backend: Backend) -> anyhow::Result<()> {
        let (queue, ingress) = Queue::new(backend);
        let mut decoder = Decoder::default();
        let mut state = FeedbackState::default();
        ingress.feed(&mut decoder, &[0x90, 0, 127, 0xb0, 32, 127, 0xb8, 32, 100]);
        assert_eq!(queue.drain(&mut state, Duration::ZERO)?, 0);
        assert_eq!(state.notes[0][0], None);
        assert_eq!(state.fx_values[0][0], None);
        assert!(!state.playback_received);
        ingress.feed(&mut decoder, &[0x9c, 40]);
        ingress.feed(
            &mut decoder,
            &[0xf8, 8, 0x8c, 0, 64, 0xbc, 22, 127, 0xb6, 4, 0],
        );
        assert_eq!(queue.drain(&mut state, Duration::ZERO)?, 4);
        assert_eq!(state.notes[0][40], Some(8));
        assert_eq!(state.notes[0][0], Some(0));
        assert_eq!(state.internal[0], Some(true));
        assert_eq!(state.playing[0], Some(false));
        assert!(state.playback_received);
        Ok(())
    }

    #[test]
    fn queue_capacity_is_exact_and_overflow_stays_failed_after_drain() -> anyhow::Result<()> {
        for backend in [Backend::Loopbe, Backend::Bloop] {
            queue_capacity_and_overflow(backend)?;
        }
        Ok(())
    }

    fn queue_capacity_and_overflow(backend: Backend) -> anyhow::Result<()> {
        let (queue, ingress) = Queue::new(backend);
        let mut decoder = Decoder::default();
        let mut state = FeedbackState::default();
        for _ in 0..4096 {
            ingress.feed(&mut decoder, &[0xb6, 4, 0]);
        }
        assert_eq!(queue.drain(&mut state, Duration::ZERO)?, 4096);
        for _ in 0..=4096 {
            ingress.feed(&mut decoder, &[0xb6, 4, 127]);
        }
        assert!(queue.drain(&mut state, Duration::ZERO).is_err());
        assert_eq!(queue.receiver.try_iter().count(), 4096);
        assert!(queue.drain(&mut state, Duration::ZERO).is_err());
        Ok(())
    }

    #[test]
    fn startup_echo_burst_does_not_displace_real_feedback() -> anyhow::Result<()> {
        for backend in [Backend::Loopbe, Backend::Bloop] {
            let (queue, ingress) = Queue::new(backend);
            let mut decoder = Decoder::default();
            let mut state = FeedbackState::default();
            for _ in 0..CAPACITY {
                ingress.feed(&mut decoder, &[0x90, 0, 127, 0xb0, 32, 127]);
            }
            for deck in 0..4 {
                ingress.feed(&mut decoder, &[0xb6, 4 + deck, 127]);
            }
            assert_eq!(queue.drain(&mut state, Duration::ZERO)?, 4);
            assert_eq!(state.playing, [Some(true); 4]);
            assert!(state.playback_received);
            assert_eq!(state.notes[0][0], None);
            assert_eq!(state.fx_values[0][0], None);
        }
        Ok(())
    }
}
