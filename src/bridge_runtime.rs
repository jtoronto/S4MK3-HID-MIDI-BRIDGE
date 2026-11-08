//! One output owner services motors, feedback and LEDs while snapshots run apart.

mod input;
mod paced;
pub mod replay;

use anyhow::{Context as _, ensure};
use encdr::device::led_builder::LedBuilder;
use encdr::device::parser::PacketParser;
use hidapi::HidDevice;
use s4mk3_hid_midi_bridge::tension::{Config, Engine};
use std::time::{Duration, Instant};
use tracing::{debug, info};

use crate::{Args, ProbeCounts, full, input_payload, record_event};
use input::Reader;
use paced::Action;

#[derive(Clone, Copy)]
pub struct MotorSettings {
    pub tension: Config,
    pub jog: full::jog::Config,
}

impl MotorSettings {
    fn engine(self) -> anyhow::Result<Engine> {
        let mut engine =
            Engine::with_backspin_strength(self.tension, self.jog.backspin_assist_strength)?;
        engine.configure_playback(self.jog.platter_settings());
        Ok(engine)
    }
}

pub fn run(
    args: &Args,
    hid: &HidDevice,
    reader_hid: HidDevice,
    parser: &mut PacketParser,
    leds: &mut LedBuilder,
    settings: MotorSettings,
    session: &mut crate::lifecycle::Session,
) -> anyhow::Result<()> {
    let jog = settings.jog;
    let led_config = session
        .led_config
        .take()
        .context("validated LED config missing")?;
    let mut bridge = crate::screens_control::bridge(args, session, jog)?;
    session.midi_ready()?;
    let mut output = None;
    let mut counts = ProbeCounts::default();
    let mut events = Vec::with_capacity(128);
    let started = Instant::now();
    let mut engine = settings.engine()?;
    let mut service = paced::Output::new(|action| {
        match action {
            Action::Initialize => {
                output = Some(full::output::LedOutput::new(hid)?);
                crate::initialize_full_analog(hid, parser, &mut bridge)?;
            }
            Action::Motor(report) => write_motor(hid, &report)?,
            Action::Frame(frame, backspin) => {
                for index in 0..3 {
                    let report = frame.report(index);
                    if report.is_empty() {
                        continue;
                    }
                    if args.raw {
                        info!(length = report.len(), bytes = %format_args!("{report:02x?}"), "HID_RAW");
                    }
                    let Some(payload) = input_payload(report)? else {
                        continue;
                    };
                    events.clear();
                    parser.parse_from("control", payload, &mut events);
                    for event in &events {
                        record_event(event, leds, &mut counts, false)?;
                    }
                    crate::send_full_report(
                        &mut bridge,
                        &events,
                        report,
                        &mut counts,
                        started.elapsed(),
                    )?;
                }
                *backspin = bridge.backspin_assist();
            }
            Action::Playback(now_us, state) => {
                let now = Duration::from_micros(now_us);
                let (received, sent) = bridge.drain_feedback(now)?;
                counts.feedback_messages = counts
                    .feedback_messages
                    .saturating_add(u64::try_from(received)?);
                counts.midi_messages = counts.midi_messages.saturating_add(u64::try_from(sent)?);
                *state = bridge.playback_state();
                session.feedback(bridge.feedback_state().playback_received)?;
            }
            Action::Feedback(now_us) => {
                let now = Duration::from_micros(now_us);
                let writes = output
                    .as_mut()
                    .context("LED output not initialized")?
                    .tick(
                        &led_config,
                        bridge.feedback_state(),
                        &bridge.local_led_state(),
                        now,
                    )?;
                counts.led_writes = counts.led_writes.saturating_add(u64::try_from(writes)?);
            }
            Action::Screens => bridge.publish_screens(),
        }
        Ok(())
    })?;
    service.initialize()?;
    let mut reader = Reader::start(reader_hid, started)?;
    session.ready()?;
    info!(
        tension = settings.tension.value(),
        backspin_assist = jog.motor_assist_enabled(),
        backspin_assist_strength = jog.backspin_assist_strength,
        "TENSION_ACTIVE: shared physical wheels; mixer faders unchanged"
    );
    let duration_us = (!args.until_stopped).then(|| args.seconds.get().saturating_mul(1_000_000));
    let result = paced::run(
        &mut engine,
        &mut reader,
        &session.receiver,
        duration_us,
        || Ok(u64::try_from(started.elapsed().as_micros())?),
        &mut service,
    );
    let result = with_cleanup(result, session.stop.request(None), "stop status");
    // Clear force before waiting for input teardown or releasing MIDI/LED ownership.
    let result = with_cleanup(
        result,
        finish_input(&mut service, &mut reader),
        "input teardown",
    );
    drop(service);
    with_cleanup(result, bridge.finish(), "MIDI releases")?;
    crate::report_probe_counts(args, &counts)
}

fn finish_input<F: FnMut(Action<'_>) -> anyhow::Result<()>>(
    service: &mut paced::Output<F>,
    reader: &mut Reader,
) -> anyhow::Result<()> {
    let neutral = service.finish();
    let cleanup = reader.stop();
    with_cleanup(neutral, cleanup, "reader shutdown")
}

fn write_motor(hid: &HidDevice, report: &[u8; 11]) -> anyhow::Result<()> {
    let written = crate::platform::write(hid, report).context("write S4 tension report")?;
    ensure!(
        written == report.len(),
        "short S4 tension write: {written}/11"
    );
    debug!(written, bytes = ?report, "TENSION_WRITE");
    if *report == s4mk3_hid_midi_bridge::tension::neutral_report() {
        debug!("TENSION_NEUTRAL");
    }
    Ok(())
}

fn with_cleanup(
    result: anyhow::Result<()>,
    cleanup: anyhow::Result<()>,
    operation: &str,
) -> anyhow::Result<()> {
    match (result, cleanup) {
        (Ok(()), cleanup) => cleanup,
        (Err(error), Ok(())) => Err(error),
        (Err(error), Err(cleanup)) => {
            Err(error.context(format!("{operation} also failed: {cleanup:#}")))
        }
    }
}
