//! Opted-in native HID experiments; optional ring initialization is isolated.

use std::fs::OpenOptions;
use std::io::{BufWriter, Write as _};
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context as _, ensure};
use crossbeam_channel::{Receiver, bounded, select_biased};
use hidapi::{HidApi, HidDevice};
use tracing::{info, warn};

use super::ControlArgs;
use super::capture::{LogRecord, emit};
use super::decode::FRESHNESS_US;
use super::ownership;
use super::reader::Reader;
use super::session::{Event, Session};

pub fn capture(path: &Path, seconds: u64, analog: bool) -> anyhow::Result<()> {
    ownership::check()?;
    let mut log = create_log(path)?;
    emit(&mut log, &LogRecord::session("capture_no_writes", None))?;
    let (hid, _) = open(false)?;
    let stop = stop_signal()?;
    let started = Instant::now();
    let mut reader = Reader::start(hid, started, analog)?;
    info!(
        analog,
        "DIAGNOSTIC_CAPTURE: no output reports; Ctrl-C stops capture"
    );
    let result = capture_loop(&mut reader, (&stop, started, seconds), &mut log);
    let cleanup = reader.stop();
    log.flush()?;
    result?;
    cleanup
}

fn capture_loop(
    reader: &mut Reader,
    timing: (&Receiver<()>, Instant, u64),
    log: &mut impl std::io::Write,
) -> anyhow::Result<()> {
    let mut samples = 0_u64;
    loop {
        let remaining = Duration::from_secs(timing.2).saturating_sub(timing.1.elapsed());
        if remaining.is_zero() {
            break;
        }
        select_biased! {
            recv(timing.0) -> _ => break,
            recv(reader.done) -> result => {
                reader.completed = true;
                result.context("reader completion channel closed")??;
                anyhow::bail!("reader ended before capture stopped");
            },
            recv(reader.frames) -> frame => {
                emit(log, &LogRecord::Raw { frame: frame.context("snapshot channel closed")? })?;
                samples += 1;
            },
            default(remaining) => break,
        }
    }
    emit(log, &LogRecord::CaptureEnd { samples })?;
    Ok(())
}

#[derive(Clone, Copy)]
pub struct TrialOptions {
    pub seconds: Option<u64>,
    pub initialize_rings: bool,
    pub start_after_neutral: bool,
}

pub fn trial(path: &Path, options: TrialOptions, control: ControlArgs) -> anyhow::Result<()> {
    ownership::check()?;
    let mut log = create_log(path)?;
    emit(
        &mut log,
        &LogRecord::session("live_motor_trial", Some(control)),
    )?;
    let (hid, writer) = open(true)?;
    let writer = writer.context("motor writer handle missing")?;
    let stop = stop_signal()?;
    let started = Instant::now();
    let mut session = Session::new(control, |report| Ok(writer.write(report)?), &mut log)?;
    if options.start_after_neutral {
        match options.seconds {
            Some(seconds) => session.prepare(seconds)?,
            None => session.prepare_manual()?,
        }
    }
    if options.initialize_rings {
        session.initialize_rings()?;
    }
    let mut reader = Reader::start(hid, started, control.mode.uses_fader())?;
    info!(
        ?control,
        seconds = ?options.seconds,
        initialize_rings = options.initialize_rings,
        start_after_neutral = options.start_after_neutral,
        "DIAGNOSTIC_MOTOR_ENABLED: uncalibrated torque; Ctrl-C neutralizes"
    );
    let result = if control.mode.is_paced() {
        super::paced::run(&mut session, &mut reader, (&stop, started, options.seconds))
    } else {
        trial_loop(&mut session, &mut reader, (&stop, started, options.seconds))
    };
    // Explicit neutral precedes cancellation/join, even when input is stuck.
    let neutral = session.finish("error");
    let cleanup = reader.stop();
    result?;
    neutral?;
    cleanup
}

fn trial_loop<F: FnMut(&[u8]) -> anyhow::Result<usize>>(
    session: &mut Session<'_, F>,
    reader: &mut Reader,
    timing: (&Receiver<()>, Instant, Option<u64>),
) -> anyhow::Result<()> {
    loop {
        let now_us = u64::try_from(timing.1.elapsed().as_micros())?;
        let remaining = timing
            .2
            .and_then(|seconds| session.remaining_duration_us(now_us, seconds))
            .map_or(Duration::MAX, Duration::from_micros);
        let event = if remaining.is_zero() {
            Event::Deadline
        } else {
            let timeout = remaining.min(Duration::from_micros(FRESHNESS_US));
            select_biased! {
                recv(timing.0) -> _ => Event::Stop,
                recv(reader.done) -> result => {
                    reader.completed = true;
                    Event::ReaderError(result.context("reader completion channel closed")
                        .and_then(|result| result)
                        .err().unwrap_or_else(|| anyhow::anyhow!("reader stopped unexpectedly")))
                },
                recv(reader.frames) -> frame => match frame {
                    Ok(frame) => Event::Sample(frame),
                    Err(error) => Event::ReaderError(error.into()),
                },
                default(timeout) => {
                    let now_us = u64::try_from(timing.1.elapsed().as_micros())?;
                    if timing.2.is_some_and(|seconds| session.remaining_duration_us(now_us, seconds) == Some(0)) {
                        Event::Deadline
                    } else {
                        Event::InputTimeout
                    }
                },
            }
        };
        // Check again after receiving: never apply a queued sample past duration.
        let now_us = u64::try_from(timing.1.elapsed().as_micros())?;
        let event = if timing
            .2
            .is_some_and(|seconds| session.remaining_duration_us(now_us, seconds) == Some(0))
        {
            Event::Deadline
        } else {
            event
        };
        if !session.handle(event, now_us)? {
            return Ok(());
        }
    }
}

pub(super) fn create_log(path: &Path) -> anyhow::Result<BufWriter<std::fs::File>> {
    Ok(BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .with_context(|| format!("create new capture {}", path.display()))?,
    ))
}

pub(super) fn open(motor: bool) -> anyhow::Result<(HidDevice, Option<HidDevice>)> {
    let api = HidApi::new().context("initialize native HID")?;
    let mut devices = api
        .device_list()
        .filter(|device| device.vendor_id() == 0x17cc && device.product_id() == 0x1720);
    let device = devices.next().context("no S4 MK3 HID collection found")?;
    ensure!(
        devices.next().is_none(),
        "multiple S4 HID collections; connect only one S4"
    );
    let reader = api
        .open_path(device.path())
        .context("open S4 snapshot reader")?;
    let writer = motor
        .then(|| {
            api.open_path(device.path())
                .context("open separate S4 motor writer")
        })
        .transpose()?;
    info!(
        motor,
        "DIAGNOSTIC_HID_OPEN: native control collection; no audio interfaces claimed"
    );
    Ok((reader, writer))
}

pub(super) fn stop_signal() -> anyhow::Result<Receiver<()>> {
    stop_signal_observed(|| {})
}

pub(super) fn stop_signal_observed(
    mut observe: impl FnMut() + Send + 'static,
) -> anyhow::Result<Receiver<()>> {
    let (tx, rx) = bounded(1);
    ctrlc::set_handler(move || {
        observe();
        if let Err(error) = tx.try_send(()) {
            warn!(%error, "DIAGNOSTIC_STOP_SIGNAL_FAILED");
        }
    })
    .context("install diagnostic Ctrl-C handler")?;
    Ok(rx)
}

#[cfg(test)]
#[path = "live_tests.rs"]
mod tests;
