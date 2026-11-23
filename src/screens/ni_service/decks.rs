//! Reply-free NI platter commands on the acquired display instance.

use super::{Connector, DeckOutput, Error, Pipe, Session, bounded, cancelled, protocol};
use std::time::Duration;
use tokio::time::Instant;

const MOTOR: u32 = 0x036a_736d;
const TICKS: u32 = 0x036a_7378;
const MODE: u32 = 0x036a_734d;
const TIMELINE: u32 = 0x036a_7354;
const RING_COLOR: u32 = 0x036a_7343;
const RING_MODE: u32 = 0x036a_734c;
const TRUE: u32 = 0x7472_7565;

impl<C: Connector> Session<C> {
    // These setters have no replies; never consume a display acknowledgement.
    pub(super) async fn scalar_write(
        &mut self,
        side: u32,
        (command, value): (u32, u32),
    ) -> Result<(), Error> {
        let mut resources = self.resources.take().ok_or(Error::Disconnected)?;
        let deadline = Instant::now() + Duration::from_millis(500);
        Self::inspect(
            &mut resources,
            &self.identity.serial,
            &mut self.cancel,
            deadline,
        )
        .await?;
        let payload = protocol::words(&[command, side, value]);
        bounded(
            &mut self.cancel,
            deadline,
            resources.instance.request.write(&payload),
        )
        .await?;
        self.resources = Some(resources);
        Ok(())
    }

    async fn mode(&mut self, side: u32, turntable: bool) -> Result<(), Error> {
        for command in [
            (TICKS, 0),
            (MODE, if turntable { 2 } else { 1 }),
            (TIMELINE, TRUE),
        ] {
            self.scalar_write(side, command).await?;
        }
        Ok(())
    }

    async fn ring(&mut self, side: u32, color: Option<u8>) -> Result<(), Error> {
        let commands = color.map_or([(RING_MODE, 0), (RING_COLOR, 0)], |value| {
            [(RING_COLOR, u32::from(value)), (RING_MODE, 1)]
        });
        for command in commands {
            self.scalar_write(side, command).await?;
        }
        Ok(())
    }

    pub(super) async fn quiesce_decks(&mut self) -> Result<(), Error> {
        // Both pauses precede every other shutdown command, even if cached off.
        for side in 0..2 {
            self.scalar_write(side, (MOTOR, 0)).await?;
        }
        for side in 0..2 {
            self.ring(side, None).await?;
            self.mode(side, false).await?;
        }
        self.decks = Some([DeckOutput::default(); 2]);
        Ok(())
    }

    pub(super) async fn set_deck_output(&mut self, desired: [DeckOutput; 2]) -> Result<(), Error> {
        self.set_deck_output_guarded(desired, |_| true).await?;
        Ok(())
    }

    pub(super) async fn set_deck_output_guarded(
        &mut self,
        desired: [DeckOutput; 2],
        allow_play: impl Fn(usize) -> bool + Send + Sync,
    ) -> Result<[DeckOutput; 2], Error> {
        if self.decks.is_none() {
            self.quiesce_decks().await?;
        }
        if !self.deck_speeds_initialized {
            for side in 0..2 {
                // Captured global default and nominal per-side 33 1/3 RPM.
                self.scalar_write(side, (0x036a_7373, 0x4205_5555)).await?;
                self.scalar_write(side, (0x036a_7353, 0x4205_5555)).await?;
            }
            self.deck_speeds_initialized = true;
        }
        let previous = self.decks.ok_or(Error::Disconnected)?;
        let mut applied = desired;
        for (side, (old, new)) in [0_u32, 1]
            .into_iter()
            .zip(previous.into_iter().zip(desired))
        {
            if old.turntable != new.turntable || (old.playing && !new.playing) {
                self.scalar_write(side, (MOTOR, 0)).await?;
            }
        }
        for (index, (side, (old, new))) in [0_u32, 1]
            .into_iter()
            .zip(previous.into_iter().zip(desired))
            .enumerate()
        {
            let was_playing = old.playing;
            let will_play = new.playing;
            if old.turntable != new.turntable {
                self.mode(side, new.turntable).await?;
            }
            if old.ring != new.ring {
                self.ring(side, new.ring).await?;
            }
            if will_play && (!was_playing || old.turntable != new.turntable) {
                let mut resources = self.resources.take().ok_or(Error::Disconnected)?;
                let deadline = Instant::now() + Duration::from_millis(500);
                Self::inspect(
                    &mut resources,
                    &self.identity.serial,
                    &mut self.cancel,
                    deadline,
                )
                .await?;
                let payload = protocol::words(&[MOTOR, side, TRUE]);
                let guard = || allow_play(index);
                let sent = tokio::select! {
                    biased;
                    () = cancelled(&mut self.cancel) => Err(Error::Cancelled),
                    result = tokio::time::timeout_at(deadline, resources.instance.request.write_guarded(&payload, &guard)) =>
                        result.map_err(|_| Error::Timeout)?,
                }?;
                self.resources = Some(resources);
                if !sent {
                    applied[index].playing = false;
                }
            }
        }
        self.decks = Some(applied);
        Ok(applied)
    }
}
