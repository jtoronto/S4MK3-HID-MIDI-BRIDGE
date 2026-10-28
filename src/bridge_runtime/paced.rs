//! Input-edge priority, a 2ms output clock, and unconditional neutral cleanup.

use anyhow::{Context as _, bail};
use crossbeam_channel::{Receiver, TryRecvError, select_biased};
use s4mk3_hid_midi_bridge::{
    jog::decode::FRESHNESS_US,
    tension::{Engine, PlaybackState, neutral_report},
};
use std::time::Duration;
use tracing::{info, warn};

use super::input::{Frame, INTERVAL_US, Reader};

pub enum Action<'a> {
    Initialize,
    Motor([u8; 11]),
    Frame(&'a Frame, &'a mut [bool; 2]),
    Playback(u64, &'a mut PlaybackState),
    Feedback(u64),
    Screens,
}

pub struct Output<F: FnMut(Action<'_>) -> anyhow::Result<()>> {
    service: F,
    finished: bool,
    recovery_since_us: Option<u64>,
    input_seen: bool,
}

impl<F: FnMut(Action<'_>) -> anyhow::Result<()>> Output<F> {
    pub fn new(service: F) -> anyhow::Result<Self> {
        let mut output = Self {
            service,
            finished: false,
            recovery_since_us: None,
            input_seen: false,
        };
        (output.service)(Action::Motor(neutral_report()))?;
        Ok(output)
    }

    pub fn initialize(&mut self) -> anyhow::Result<()> {
        (self.service)(Action::Initialize)
    }

    pub fn finish(&mut self) -> anyhow::Result<()> {
        if !self.finished {
            (self.service)(Action::Motor(neutral_report()))?;
            self.finished = true;
            info!("TENSION_FINAL_NEUTRAL: native write accepted");
        }
        Ok(())
    }

    fn expire_tension(&mut self, engine: &mut Engine, now_us: u64) -> anyhow::Result<()> {
        let deadline_us = engine.input_deadline_us();
        if self.recovery_since_us.is_none() && now_us >= deadline_us {
            (self.service)(Action::Motor(neutral_report()))?;
            engine.clear_motion();
            self.recovery_since_us = Some(now_us);
            warn!(
                now_us,
                deadline_us,
                "TENSION_SUSPENDED: input freshness timeout; force cleared; waiting for fresh input"
            );
        }
        Ok(())
    }

    pub(super) fn sample(
        &mut self,
        engine: &mut Engine,
        frame: &Frame,
        now_us: u64,
    ) -> anyhow::Result<()> {
        self.expire_tension(engine, now_us)?;
        if self.recovery_since_us.is_none_or(|suspended_us| {
            frame.host_us > suspended_us && now_us.saturating_sub(frame.host_us) <= FRESHNESS_US
        }) {
            if let Some(report) =
                engine.observe(frame.report(0), frame.report(1), frame.host_us, now_us)?
            {
                (self.service)(Action::Motor(report))?;
            }
            if self.recovery_since_us.take().is_some() {
                info!(
                    host_us = frame.host_us,
                    "TENSION_INPUT_RECOVERED: fresh baseline; force requires new qualified movement"
                );
            }
        }
        let mut backspin = [false; 2];
        (self.service)(Action::Frame(frame, &mut backspin))?;
        if let Some(report) = engine.set_backspin(backspin) {
            (self.service)(Action::Motor(report))?;
        }
        self.playback(engine, now_us)?;
        self.input_seen = true;
        Ok(())
    }

    fn playback(&mut self, engine: &mut Engine, now_us: u64) -> anyhow::Result<()> {
        let mut state = PlaybackState::default();
        (self.service)(Action::Playback(now_us, &mut state))?;
        if let Some(report) = engine.set_playback(state) {
            (self.service)(Action::Motor(report))?;
        }
        Ok(())
    }

    pub(super) fn tick(&mut self, engine: &mut Engine, tick_us: u64) -> anyhow::Result<()> {
        self.expire_tension(engine, tick_us)?;
        self.playback(engine, tick_us)?;
        let report = if self.recovery_since_us.is_some() {
            neutral_report()
        } else {
            engine.tick(tick_us)?
        };
        (self.service)(Action::Motor(report))?;
        (self.service)(Action::Feedback(tick_us))?;
        if self.input_seen {
            (self.service)(Action::Screens)?;
        }
        Ok(())
    }
}

impl<F: FnMut(Action<'_>) -> anyhow::Result<()>> Drop for Output<F> {
    fn drop(&mut self) {
        if !self.finished
            && let Err(error) = self.finish()
        {
            warn!(%error, "TENSION_NEUTRAL_FAILED");
        }
    }
}

pub fn run<C, F>(
    engine: &mut Engine,
    reader: &mut Reader,
    stop: &Receiver<()>,
    duration_us: Option<u64>,
    clock: C,
    output: &mut Output<F>,
) -> anyhow::Result<()>
where
    C: Fn() -> anyhow::Result<u64>,
    F: FnMut(Action<'_>) -> anyhow::Result<()>,
{
    let mut next_tick_us = INTERVAL_US;
    loop {
        if terminal(reader, stop, duration_us, clock()?)? {
            return Ok(());
        }
        let now_us = clock()?;
        output.expire_tension(engine, now_us)?;
        if now_us >= next_tick_us {
            // A queued touch release cannot sit behind an overdue force write.
            for _ in 0..32 {
                if terminal(reader, stop, duration_us, clock()?)? {
                    return Ok(());
                }
                match reader.frames.try_recv() {
                    Ok(frame) => output.sample(engine, &frame, clock()?)?,
                    Err(TryRecvError::Disconnected) => bail!("snapshot channel closed"),
                    Err(TryRecvError::Empty) => break,
                }
            }
            if terminal(reader, stop, duration_us, clock()?)? {
                return Ok(());
            }
            let tick_us = clock()?;
            output.tick(engine, tick_us)?;
            // No bursts to catch up slots consumed by MIDI, logging or native I/O.
            next_tick_us = tick_us.saturating_add(INTERVAL_US);
            continue;
        }
        let wait_us = next_tick_us
            .min(
                output
                    .recovery_since_us
                    .map_or_else(|| engine.input_deadline_us(), |_| u64::MAX),
            )
            .min(duration_us.unwrap_or(u64::MAX))
            .saturating_sub(now_us);
        if wait_us == 0 {
            continue;
        }
        select_biased! {
            recv(stop) -> _ => return Ok(()),
            recv(reader.done) -> result => {
                reader.completed = true;
                result.context("snapshot completion channel closed")??;
                bail!("snapshot reader stopped unexpectedly");
            },
            recv(reader.frames) -> frame => {
                let frame = frame.context("snapshot channel closed")?;
                let now_us = clock()?;
                if terminal(reader, stop, duration_us, now_us)? {
                    return Ok(());
                }
                output.sample(engine, &frame, now_us)?;
            },
            default(Duration::from_micros(wait_us)) => {}
        }
    }
}

fn terminal(
    reader: &mut Reader,
    stop: &Receiver<()>,
    duration_us: Option<u64>,
    now_us: u64,
) -> anyhow::Result<bool> {
    match stop.try_recv() {
        Ok(()) | Err(TryRecvError::Disconnected) => return Ok(true),
        Err(TryRecvError::Empty) => {}
    }
    match reader.done.try_recv() {
        Ok(result) => {
            reader.completed = true;
            result?;
            bail!("snapshot reader stopped unexpectedly");
        }
        Err(TryRecvError::Disconnected) => bail!("snapshot completion channel closed"),
        Err(TryRecvError::Empty) => {}
    }
    Ok(duration_us.is_some_and(|deadline| now_us >= deadline))
}

#[cfg(test)]
mod tests;
