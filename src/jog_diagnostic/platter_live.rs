//! Native plumbing reused from legacy diagnostics, without MIDI or audio.

use std::time::{Duration, Instant};

use anyhow::{Context as _, bail};
use crossbeam_channel::{Receiver, select_biased};

use super::live::{create_log, open, stop_signal_observed};
use super::ownership;
use super::platter::{Config, Experiment};
use super::platter_session::{Session, combine};
use super::platter_telemetry::{Clock, StopRequest};
use super::reader::Reader;

pub fn run(config: Config) -> anyhow::Result<()> {
    ownership::check()?;
    let mut log = create_log(&config.common.out)?;
    let output = !matches!(config.experiment, Experiment::Measure);
    let (hid, writer) = open(output)?;
    let started = Instant::now();
    let clock = Clock::Live(started);
    let stop_request = StopRequest::new(clock.clone());
    // No fallible setup between acquiring the writer and installing its guard.
    let mut session = Session::with_telemetry(
        config,
        |report| {
            Ok(writer
                .as_ref()
                .context("output writer missing")?
                .write(report)?)
        },
        &mut log,
        clock,
        stop_request.clone(),
    );
    let mut reader = None;
    let result = (|| {
        let stop = stop_signal_observed(move || stop_request.request())?;
        session.start()?;
        reader = Some(Reader::start_instrumented(hid, started, false)?);
        run_loop(
            &mut session,
            reader.as_mut().context("reader missing")?,
            &stop,
            || micros(started),
        )
    })();
    let now_us = micros(started);
    let reason = match &result {
        Ok(reason) => (*reason).to_owned(),
        Err(error) => format!("error: {error:#}"),
    };
    // Neutral is attempted before input cancellation/join and log cleanup,
    // including initialization or native-reader setup failures.
    let neutral = session.finish(now_us, &reason);
    let cleanup = reader.as_mut().map_or(Ok(()), Reader::stop);
    combine(result.map(|_| ()), combine(neutral, cleanup))
}

fn micros(started: Instant) -> u64 {
    // Saturating time forces deadline expiry even on a clock beyond u64 us.
    u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX)
}

fn run_loop<F: FnMut(&[u8]) -> anyhow::Result<usize>>(
    session: &mut Session<'_, F>,
    reader: &mut Reader,
    stop: &Receiver<()>,
    mut clock: impl FnMut() -> u64,
) -> anyhow::Result<&'static str> {
    let result = run_events(session, reader, stop, &mut clock);
    let result = combine(result, session.capture_reader_health(reader));
    let reason = match &result {
        Ok(reason) => (*reason).to_owned(),
        Err(error) => format!("error: {error:#}"),
    };
    let neutral = session.finish(clock(), &reason);
    combine(result, neutral)
}

fn run_events<F: FnMut(&[u8]) -> anyhow::Result<usize>>(
    session: &mut Session<'_, F>,
    reader: &mut Reader,
    stop: &Receiver<()>,
    clock: &mut impl FnMut() -> u64,
) -> anyhow::Result<&'static str> {
    let mut next_tick = 0;
    loop {
        session.capture_reader_health(reader)?;
        let now_us = clock();
        if now_us >= session.model.deadline_us() {
            return Ok("duration");
        }
        let wake = next_tick
            .min(session.model.pulse_deadline_us())
            .min(session.model.sensor_deadline_us())
            .min(session.model.deadline_us());
        session.model.check_time(now_us)?;
        select_biased! {
            recv(stop) -> _ => return Ok("ctrl_c"),
            recv(reader.done) -> result => {
                reader.completed = true;
                result.context("reader completion channel closed")??;
                bail!("reader stopped unexpectedly");
            },
            // Every received frame is followed by a due-output check below.
            default(Duration::from_micros(wake.saturating_sub(now_us))) => {},
            recv(reader.frames) -> frame => {
                let frame = frame.context("snapshot channel closed")?;
                let now_us = clock();
                session.consumed(frame.host_us, now_us);
                if now_us >= session.model.deadline_us() {
                    return Ok("duration");
                }
                session.model.check_time(now_us)?;
                session.sample(&frame, now_us)?;
            },
        }
        let now_us = clock();
        if now_us >= wake {
            if let Some(reason) = session.tick(now_us)? {
                return Ok(reason);
            }
            next_tick = now_us.saturating_add(2000);
        }
    }
}

#[cfg(test)]
#[path = "platter_loop_tests.rs"]
mod tests;
