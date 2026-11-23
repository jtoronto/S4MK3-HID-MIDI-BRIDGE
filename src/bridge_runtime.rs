//! One output owner services motors, feedback and LEDs while snapshots run apart.

mod diagnostics;
mod input;
mod paced;
pub mod replay;

use anyhow::{Context as _, ensure};
use encdr::device::led_builder::LedBuilder;
use encdr::device::parser::PacketParser;
use hidapi::HidDevice;
use s4mk3_hid_midi_bridge::tension::{Config, Engine};
use std::time::Duration;
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
    let led_config = session
        .led_config
        .take()
        .context("validated LED config missing")?;
    let mut bridge = crate::screens_control::bridge(args, session, settings.jog)?;
    session.midi_ready()?;
    let mut counts = ProbeCounts::default();
    let mut events = Vec::with_capacity(128);
    let started = crate::screen_motor_capture::bridge_epoch();
    let mut engine = settings.engine()?;
    let ni_decks = args.screens.ni_decks;
    #[cfg(windows)]
    let mut last_wheel_snapshot = None;
    let mut output = None;
    let mut service = paced::Output::with_native_motor(!ni_decks, |action| {
        match action {
            Action::Initialize => {
                output = Some(full::output::LedOutput::new(hid, !ni_decks)?);
                crate::initialize_full_analog(hid, parser, &mut bridge)?;
            }
            Action::Motor(report) => write_motor(hid, &report)?,
            Action::Frame(frame, backspin) => {
                #[cfg(windows)]
                if ni_decks && frame.report(1).first() == Some(&3) {
                    last_wheel_snapshot = Some(frame.host_us);
                }
                for report in (0..3)
                    .map(|index| frame.report(index))
                    .filter(|report| !report.is_empty())
                {
                    if !report_input(args.raw, report, parser, &mut events)? {
                        continue;
                    }
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
                let (received, sent) = bridge.drain_feedback(Duration::from_micros(now_us))?;
                count_feedback(&mut counts, received, sent)?;
                *state = bridge.playback_state();
                #[cfg(windows)]
                if ni_decks {
                    let fresh = last_wheel_snapshot.is_some_and(|sample| {
                        now_us.saturating_sub(sample)
                            < s4mk3_hid_midi_bridge::jog::decode::FRESHNESS_US
                    });
                    bridge.update_ni_decks(&led_config, fresh)?;
                }
                diagnostics::feedback(&bridge, now_us);
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
    let result = run_paced(
        args,
        settings,
        &mut engine,
        &mut reader,
        started,
        session,
        &mut service,
    );
    // The callback borrows the bridge; give it back before quiescing the NI session.
    drop(service);
    #[cfg(windows)]
    let result = if ni_decks {
        with_cleanup(result, bridge.stop_ni_decks(), "NI deck shutdown")
    } else {
        result
    };
    let result = with_cleanup(result, reader.stop(), "reader shutdown");
    with_cleanup(result, bridge.finish(), "MIDI releases")?;
    crate::report_probe_counts(args, &counts)
}

fn count_feedback(counts: &mut ProbeCounts, received: usize, sent: usize) -> anyhow::Result<()> {
    counts.feedback_messages = counts
        .feedback_messages
        .saturating_add(u64::try_from(received)?);
    counts.midi_messages = counts.midi_messages.saturating_add(u64::try_from(sent)?);
    Ok(())
}

fn run_paced<F: FnMut(Action<'_>) -> anyhow::Result<()>>(
    args: &Args,
    settings: MotorSettings,
    engine: &mut Engine,
    reader: &mut Reader,
    started: std::time::Instant,
    session: &crate::lifecycle::Session,
    service: &mut paced::Output<F>,
) -> anyhow::Result<()> {
    log_output_mode(args.screens.ni_decks, settings);
    let duration_us = (!args.until_stopped).then(|| args.seconds.get().saturating_mul(1_000_000));
    let result = paced::run(
        engine,
        reader,
        &session.receiver,
        duration_us,
        || Ok(u64::try_from(started.elapsed().as_micros())?),
        service,
    );
    let result = with_cleanup(result, session.stop.request(None), "stop status");
    let result = with_cleanup(result, service.finish(), "motor neutral");
    if args.screens.ni_decks {
        // Stop admitting snapshots while waiting for the NI worker's final Pause.
        with_cleanup(result, reader.cancel(), "request snapshot reader stop")
    } else {
        result
    }
}

fn report_input(
    raw: bool,
    report: &[u8],
    parser: &mut PacketParser,
    events: &mut Vec<encdr::Event>,
) -> anyhow::Result<bool> {
    if raw {
        info!(length = report.len(), bytes = %format_args!("{report:02x?}"), "HID_RAW");
    }
    events.clear();
    let Some(payload) = input_payload(report)? else {
        return Ok(false);
    };
    parser.parse_from("control", payload, events);
    Ok(true)
}

fn log_output_mode(ni_decks: bool, settings: MotorSettings) {
    if ni_decks {
        info!("NI_DECK_OUTPUT_ACTIVE: raw platter and ring writes disabled");
    } else {
        info!(
            tension = settings.tension.value(),
            backspin_assist = settings.jog.motor_assist_enabled(),
            backspin_assist_strength = settings.jog.backspin_assist_strength,
            "TENSION_ACTIVE: shared physical wheels; mixer faders unchanged"
        );
    }
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
