//! A bounded S4 MK3 HID probe with optional minimal MIDI output.

mod args;
mod bridge_runtime;
mod full;
mod input;
mod lifecycle;
mod midi;
mod midi_capture;
mod midi_layout;
mod midi_transport;
mod ni_runtime;
#[path = "jog_diagnostic/ownership.rs"]
mod ownership;
mod platform;
mod screen_motor_capture;
mod screens_control;

#[cfg(test)]
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail, ensure};
use args::{Args, MidiProfile};
use clap::Parser;
use encdr::core::descriptor::{ButtonItemDesc, HexU16, InputItemDesc, SliderItemDesc};
use encdr::device::led_builder::LedBuilder;
use encdr::device::loader::DescriptorRegistry;
use encdr::device::parser::PacketParser;
use encdr::usb::hotplug;
use encdr::{DeviceDescriptor, Event, LedValue};
use hidapi::{HidApi, HidDevice};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

const S4_VENDOR_ID: u16 = 0x17cc;
const S4_PRODUCT_ID: u16 = 0x1720;

#[derive(Debug, Default)]
struct ProbeCounts {
    input_events: u64,
    led_writes: u64,
    midi_messages: u64,
    feedback_messages: u64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let ni_tension = ni_runtime::settings(
        &args,
        std::env::var_os("S4_BRIDGE_NI_JOGT_TRIAL_FILE").is_some(),
    )?;
    ensure!(
        args.midi_capture.is_none() || args.midi_backend.port_name().is_some(),
        "--midi-capture requires --midi-backend loopbe or bloop"
    );
    ensure!(
        args.jog_config.is_none()
            || args.replay_motor_fixture.is_some()
            || args.generate_mapping.is_some()
            || matches!(args.midi, Some(MidiProfile::Full)),
        "--jog-config requires mapping export or --midi full"
    );
    ensure!(
        args.led_config.is_none() || matches!(args.midi, Some(MidiProfile::Full)),
        "--led-config requires --midi full"
    );
    ensure!(
        args.tension_config.is_none() || matches!(args.midi, Some(MidiProfile::Full)),
        "--tension-config requires --midi full"
    );
    ensure!(
        !args.screens.enabled || matches!(args.midi, Some(MidiProfile::Full)),
        "--screens requires --midi full"
    );
    ensure!(
        !args.screens.control_stdin || matches!(args.midi, Some(MidiProfile::Full)),
        "--screen-control-stdin requires --midi full"
    );
    let tension = args
        .tension_config
        .as_deref()
        .map(|path| s4mk3_hid_midi_bridge::tension::Config::load(Some(path)))
        .transpose()?;
    tracing_subscriber::fmt()
        .with_env_filter(if args.screen_motor_diagnostic_config.is_some() {
            EnvFilter::try_new("info")?
        } else {
            EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?
        })
        .with_target(false)
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .try_init()
        .map_err(|error| anyhow::anyhow!("initialize logging: {error}"))?;

    if args.replay_motor_fixture.is_some() {
        return bridge_runtime::replay::run(full::jog::Config::load(args.jog_config.as_deref())?);
    }

    if let Some(path) = &args.generate_mapping {
        full::mapping::write_for_backend(path, args.jog_config.as_deref(), args.midi_backend)?;
        for control in full::catalog::LOCAL_CONTROLS {
            info!(
                input = control.name,
                reason = control.reason,
                "LOCAL_CONTROL"
            );
        }
        for control in full::catalog::UNSUPPORTED_CONTROLS {
            info!(
                input = control.name,
                reason = control.reason,
                "UNASSIGNED_CONTROL"
            );
        }
        info!(path = %path.display(), "DJAY_MAPPING_EXPORTED");
        return Ok(());
    }
    if let Some(path) = &args.screens.generate_screen_previews {
        full::screens::write_previews(path)?;
        info!(path = %path.display(), "SCREEN_PREVIEWS_EXPORTED: no USB or MIDI opened");
        return Ok(());
    }
    if !args.list {
        args.midi_backend.validate_runtime()?;
    }
    let jog = matches!(args.midi, Some(MidiProfile::Full))
        .then(|| full::jog::Config::load(args.jog_config.as_deref()))
        .transpose()?
        .map(|jog| {
            if args.screens.ni_decks {
                jog.for_ni_bridge()
            } else {
                jog
            }
        });
    if let Some(jog) = jog {
        jog.validate()?;
        println!("{}", motor_config_receipt(&jog)?);
    }
    let tension =
        tension.or_else(|| jog.map(|_| s4mk3_hid_midi_bridge::tension::Config::default()));
    run(args, tension, jog, ni_tension)
}

fn run(
    args: Args,
    tension: Option<s4mk3_hid_midi_bridge::tension::Config>,
    jog: Option<full::jog::Config>,
    ni_tension: Option<u8>,
) -> anyhow::Result<()> {
    let capture = screen_motor_capture::Session::start(&args)?;
    let result =
        screen_motor_capture::await_endpoint(args.managed.prepare_screen_motor_diagnostics)
            .and_then(|begin| {
                if begin {
                    run_owned(args, tension, jog, ni_tension)
                } else {
                    Ok(())
                }
            });
    capture.finish(result)
}

fn run_owned(
    mut args: Args,
    tension: Option<s4mk3_hid_midi_bridge::tension::Config>,
    jog: Option<full::jog::Config>,
    ni_tension: Option<u8>,
) -> anyhow::Result<()> {
    let mut capture = args
        .midi_capture
        .as_deref()
        .map(midi_capture::Capture::start)
        .transpose()?;
    if let Some(capture) = &capture {
        args.capture_handle = capture.handle();
    }
    let mut session = (!args.list)
        .then(|| lifecycle::Session::new(&args, ni_tension))
        .transpose()?;
    let result = open_and_run(&args, tension, jog, session.as_mut());
    let result = match session {
        Some(session) => session.finish(result),
        None => result,
    };
    let cleanup = capture
        .as_mut()
        .map(midi_capture::Capture::finish)
        .transpose();
    if let Err(error) = cleanup {
        if let Err(runtime) = result {
            return Err(runtime.context(format!("MIDI capture close also failed: {error:#}")));
        }
        return Err(error);
    }
    result
}

fn motor_config_receipt(config: &full::jog::Config) -> anyhow::Result<String> {
    Ok(format!(
        "S4_MOTOR_CONFIG {}",
        serde_json::to_string(&serde_json::json!({ "jog": config }))?
    ))
}

fn open_and_run(
    args: &Args,
    tension: Option<s4mk3_hid_midi_bridge::tension::Config>,
    jog: Option<full::jog::Config>,
    session: Option<&mut lifecycle::Session>,
) -> anyhow::Result<()> {
    let mut registry = DescriptorRegistry::new();
    registry.load_builtins().context("load Encdr descriptors")?;
    let devices: Vec<_> = hotplug::scan_devices(&registry)
        .into_iter()
        .filter(|device| {
            device.descriptor.vendor_id.0 == S4_VENDOR_ID
                && device.descriptor.product_id.0 == S4_PRODUCT_ID
        })
        .collect();

    for device in &devices {
        info!(device = %device.device_id, "S4_USB_FOUND: 17cc:1720");
    }
    if args.list {
        info!(
            count = devices.len(),
            "S4 enumeration complete; no interfaces opened"
        );
        return Ok(());
    }
    let session = session.context("runtime session missing")?;
    if !session.receiver.is_empty() {
        return Ok(());
    }
    let mut devices = devices.into_iter();
    let device = devices.next().context(
        "no S4 MK3 found; connect USB and the S4 power adapter, then run the probe again",
    )?;
    ensure!(
        devices.next().is_none(),
        "multiple S4 MK3 devices found; connect just one for this probe"
    );
    let descriptor = if matches!(args.midi, Some(MidiProfile::Full)) {
        full_control_descriptor(&device.descriptor)?
    } else {
        control_descriptor(&device.descriptor)
    };
    let names = registry.intern_descriptor_names(&descriptor);
    let mut parser = PacketParser::new(device.device_id, &descriptor, names);
    let api = HidApi::new().context("initialize native HID access")?;
    let collection = platform::collection(&api)?;
    s4mk3_hid_midi_bridge::screen_motor_diag::record(
        s4mk3_hid_midi_bridge::screen_motor_diag::Kind::Lifecycle,
        [
            10,
            u64::from(collection.usage_page()),
            u64::from(collection.usage()),
            0,
            0,
            0,
            0,
            0,
        ],
        collection.serial_number().unwrap_or("").as_bytes(),
    );
    #[cfg(windows)]
    bind_windows_screens(args, session, collection.serial_number())?;
    #[cfg(target_os = "macos")]
    if args.screens.ni_decks {
        ni_runtime::bind(session, collection.serial_number())?;
    }
    ownership::check()?;
    let hid = api
        .open_path(collection.path())
        .context("open S4 HID collection")?;
    session.milestone(|ready| ready.hid = true)?;
    let tension = tension
        .map(|config| {
            api.open_path(collection.path())
                .context("open S4 tension snapshot reader")
                .map(|reader| (reader, config))
        })
        .transpose()?;
    let interface = descriptor
        .interface_by_id("control")
        .context("control interface missing")?;
    let buttons = descriptor
        .leds
        .first()
        .context("button LED layout missing")?;
    let mut leds = LedBuilder::new(buttons, interface);
    info!("S4_HID_OPEN: native HID access; audio interfaces are not claimed");
    if let Some((reader, config)) = tension {
        bridge_runtime::run(
            args,
            &hid,
            reader,
            &mut parser,
            &mut leds,
            bridge_runtime::MotorSettings {
                tension: config,
                jog: jog.context("motor output requires the full MIDI profile")?,
            },
            session,
        )
    } else {
        run_probe(args, &hid, &mut parser, &mut leds, session)
    }
}

#[cfg(windows)]
fn bind_windows_screens(
    args: &Args,
    session: &mut lifecycle::Session,
    serial: Option<&str>,
) -> anyhow::Result<()> {
    if !matches!(args.midi, Some(MidiProfile::Full)) {
        return Ok(());
    }
    if args.screens.ni_decks {
        return ni_runtime::bind(session, serial);
    }
    if !args.screens.enabled && !args.screens.control_stdin {
        return Ok(());
    }
    // Bind the uniquely selected HID before initialized input can start NI I/O.
    // An absent/rejected serial fails only screen acquisition, preserving controls.
    if session
        .screens
        .as_ref()
        .and_then(full::screens::Output::control)
        .is_none()
    {
        session.screens = Some(full::screens::Output::managed(args.screens.enabled)?);
    }
    if let Some(control) = session
        .screens
        .as_ref()
        .and_then(full::screens::Output::control)
    {
        if let Some(serial) = serial {
            if let Err(error) = control.bind_selected_hid(serial) {
                tracing::warn!(%error, "SCREENS_DISABLED: selected HID identity rejected");
            }
        } else {
            tracing::warn!("SCREENS_DISABLED: selected HID serial unavailable");
        }
    }
    Ok(())
}

fn run_probe(
    args: &Args,
    hid: &HidDevice,
    parser: &mut PacketParser,
    leds: &mut LedBuilder,
    session: &lifecycle::Session,
) -> anyhow::Result<()> {
    let mut midi = matches!(args.midi, Some(MidiProfile::Minimal))
        .then(|| midi::MidiBridge::new(args.midi_backend, args.capture_handle.clone()))
        .transpose()?;
    session.midi_ready()?;
    session.ready()?;
    info!(
        seconds = args.seconds.get(),
        until_stopped = args.until_stopped,
        "Press/release left Play; move a fader and jog."
    );
    let started = Instant::now();
    let mut counts = ProbeCounts::default();
    let mut buffers = [[0_u8; 128]; 3];
    let mut events = Vec::with_capacity(128);
    while args.until_stopped || started.elapsed() < Duration::from_secs(args.seconds.get()) {
        match session.receiver.try_recv() {
            Ok(()) => break,
            Err(crossbeam_channel::TryRecvError::Empty) => {}
            Err(crossbeam_channel::TryRecvError::Disconnected) => bail!("stop channel closed"),
        }
        let lengths = input::read_batch(hid, false, &mut buffers)?;
        for (buffer, length) in buffers.iter().zip(lengths) {
            if length == 0 {
                continue;
            }
            let report = buffer
                .get(..length)
                .context("HID report exceeds input buffer")?;
            debug!(length, report_id = ?report.first(), "HID_REPORT");
            if args.raw {
                info!(length, bytes = %format_args!("{report:02x?}"), "HID_RAW");
            }
            let Some(payload) = input_payload(report)? else {
                continue;
            };
            events.clear();
            parser.parse_from("control", payload, &mut events);
            for event in &events {
                record_event(event, leds, &mut counts, true)?;
                if let Some(bridge) = &mut midi
                    && bridge.send(event)?
                {
                    counts.midi_messages = counts.midi_messages.saturating_add(1);
                }
            }
            if let Some(report) = leds.flush() {
                let written =
                    platform::write(hid, &report).context("write S4 button LED report")?;
                ensure!(
                    written == report.len(),
                    "short S4 LED write: {written} bytes"
                );
                counts.led_writes = counts.led_writes.saturating_add(1);
                info!(
                    written,
                    "LED_WRITE_OK: native HID accepted report; confirm light physically"
                );
            }
        }
    }
    session.stop.request(None)?;
    if let Some(midi) = &mut midi {
        midi.finish()?;
    }
    report_probe_counts(args, &counts)
}

fn initialize_full_analog(
    hid: &HidDevice,
    parser: &mut PacketParser,
    bridge: &mut full::FullBridge,
) -> anyhow::Result<()> {
    let report = input::read_analog(hid)?;
    let mut events = Vec::with_capacity(48);
    // Seed the real parser once, including zero-valued sliders, without sending
    // startup FX, gain, filter, crossfader-position or tempo commands.
    parser.parse_from("control", &report[1..], &mut events);
    bridge.seed_analog(&events)
}

fn send_full_report(
    bridge: &mut full::FullBridge,
    events: &[Event],
    report: &[u8],
    counts: &mut ProbeCounts,
    now: Duration,
) -> anyhow::Result<()> {
    let (received, sent) = bridge.send_report(events, report, now)?;
    counts.feedback_messages = counts
        .feedback_messages
        .saturating_add(u64::try_from(received)?);
    counts.midi_messages = counts.midi_messages.saturating_add(u64::try_from(sent)?);
    Ok(())
}

fn report_probe_counts(args: &Args, counts: &ProbeCounts) -> anyhow::Result<()> {
    info!(
        input_events = counts.input_events,
        led_writes = counts.led_writes,
        midi_messages = counts.midi_messages,
        feedback_messages = counts.feedback_messages,
        "Probe ended; LED visibility and audio coexistence need physical confirmation"
    );
    // Continuous service use may be idle; bounded probes still require activity.
    if args.until_stopped || args.managed.stdin || args.screens.control_stdin {
        return Ok(());
    }
    ensure!(
        counts.input_events > 0,
        "no decoded S4 input received during the probe"
    );
    ensure!(
        args.midi.is_none()
            || counts.midi_messages > 0
            || (matches!(args.midi, Some(MidiProfile::Full)) && counts.feedback_messages > 0),
        "no mapped MIDI controls received; press Play/Cue or move a channel fader/crossfader"
    );
    Ok(())
}

fn record_event(
    event: &Event,
    leds: &mut LedBuilder,
    counts: &mut ProbeCounts,
    mirror_play: bool,
) -> anyhow::Result<()> {
    match event {
        Event::DeviceConnected { id, .. } => {
            info!(device = %id, "device event; waiting for actual controls");
            return Ok(());
        }
        Event::DeviceDisconnected { id } => {
            bail!("S4 disconnected: {id}");
        }
        Event::Button { name, pressed, .. } => {
            info!(control = name, pressed, "BUTTON");
            if mirror_play && *name == "left_play" {
                ensure!(
                    leds.set(
                        name,
                        if *pressed {
                            LedValue::Single(127)
                        } else {
                            LedValue::Off
                        }
                    ),
                    "left_play LED missing from Encdr layout"
                );
            }
        }
        Event::Slider { name, value, .. } => {
            debug!(control = name, value, "SLIDER");
        }
        Event::Encoder { name, delta, .. } => {
            debug!(control = name, delta, "ENCODER");
        }
        Event::EncoderFine { name, delta, .. } => {
            debug!(control = name, delta, "ENCODER_FINE");
        }
        Event::Touch { name, touched, .. } => {
            info!(control = name, touched, "TOUCH");
        }
        Event::Grid {
            name,
            index,
            pressure,
            ..
        } => {
            info!(control = name, index, pressure, "GRID");
        }
    }
    counts.input_events = counts.input_events.saturating_add(1);
    Ok(())
}

fn control_descriptor(original: &DeviceDescriptor) -> DeviceDescriptor {
    let mut descriptor = original.clone();
    descriptor
        .interfaces
        .retain(|interface| interface.id == "control");
    descriptor.screens.clear();
    descriptor.leds.retain(|group| group.id == "buttons");
    // Payload sizes from this S4's HID report descriptor (report ID excluded).
    // The pinned Encdr descriptor has incorrect sizes for buttons and jogs.
    for packet in &mut descriptor.input_packets {
        match packet.id.as_str() {
            "buttons" => packet.size = 22,
            "jogwheels" => packet.size = 59,
            _ => {}
        }
        // S4 mixer faders use 12-bit values in 16-bit fields (Mixxx: 4095).
        // Correct only this stage's mapped faders, not the broader descriptor.
        for item in &mut packet.items {
            if let InputItemDesc::Slider(slider) = item
                && matches!(
                    slider.name.as_str(),
                    "ch1_fader" | "ch2_fader" | "ch3_fader" | "ch4_fader" | "crossfader"
                )
            {
                slider.max_value = Some(4095);
            }
        }
    }
    descriptor
}

fn input_payload(report: &[u8]) -> anyhow::Result<Option<&[u8]>> {
    let (id, payload) = report.split_first().context("empty HID report")?;
    let expected = match id {
        1 => 22,
        2 => 78,
        3 => 59,
        _ => return Ok(None),
    };
    ensure!(
        payload.len() == expected,
        "unexpected S4 report {id} payload length: {} (expected {expected})",
        payload.len()
    );
    Ok(Some(payload))
}

fn full_control_descriptor(original: &DeviceDescriptor) -> anyhow::Result<DeviceDescriptor> {
    let mut descriptor = control_descriptor(original);
    descriptor.leds.extend(
        original
            .leds
            .iter()
            .filter(|group| matches!(group.id.as_str(), "vu_meters" | "wheel_leds"))
            .cloned(),
    );
    for packet in &mut descriptor.input_packets {
        for item in &mut packet.items {
            match item {
                InputItemDesc::Slider(slider) => slider.max_value = Some(4095),
                InputItemDesc::Button(button) => {
                    // Captured physical presses: only the LEFT names are reversed.
                    match button.name.as_str() {
                        "left_loop_encoder_press" => {
                            "left_move_encoder_press".clone_into(&mut button.name);
                        }
                        "left_move_encoder_press" => {
                            "left_loop_encoder_press".clone_into(&mut button.name);
                        }
                        _ => {}
                    }
                    if button.name.starts_with("left_pad_") || button.name.starts_with("right_pad_")
                    {
                        let index = button
                            .name
                            .rsplit('_')
                            .next()
                            .context("pad number missing")?
                            .parse::<usize>()?
                            .checked_sub(1)
                            .context("pad numbers start at 1")?;
                        let bit = [5_u8, 4, 7, 6, 3, 2, 1, 0]
                            .get(index)
                            .context("pad number outside 1-8")?;
                        button.mask = HexU16(1_u16 << bit);
                    }
                }
                InputItemDesc::Encoder(encoder) => {
                    // All four rotation names are reversed; preserve wire polarity.
                    let name = match encoder.name.as_str() {
                        "left_loop_encoder" => Some("left_move_encoder"),
                        "left_move_encoder" => Some("left_loop_encoder"),
                        "right_loop_encoder" => Some("right_move_encoder"),
                        "right_move_encoder" => Some("right_loop_encoder"),
                        _ => None,
                    };
                    if let Some(name) = name {
                        name.clone_into(&mut encoder.name);
                    }
                }
                InputItemDesc::EncoderFine(_) | InputItemDesc::Touch(_) => {}
            }
        }
        if packet.id == "buttons" {
            for (name, byte, mask) in [
                ("mixer_ch3_ext", 2, 0x02),
                ("mixer_ch1_ext", 11, 0x01),
                ("mixer_ch2_ext", 11, 0x02),
                ("mixer_ch4_ext", 11, 0x04),
            ] {
                packet.items.push(InputItemDesc::Button(ButtonItemDesc {
                    name: name.to_owned(),
                    byte,
                    mask: HexU16(mask),
                    category: None,
                }));
            }
        }
        if packet.id == "sliders" {
            packet.items.push(InputItemDesc::Slider(SliderItemDesc {
                name: "headphone_gain".to_owned(),
                byte: None,
                bytes: Some(vec![26, 27]),
                bits: 16,
                normalize: true,
                max_value: Some(4095),
            }));
        }
    }
    Ok(descriptor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trial_requires_ni_decks_before_direct_or_recovery_startup() -> anyhow::Result<()> {
        for arguments in [
            vec!["bridge"],
            vec!["bridge", "--midi", "full"],
            vec!["bridge", "--list"],
        ] {
            let args = Args::try_parse_from(arguments)?;
            assert!(ni_runtime::settings(&args, true).is_err());
            assert_eq!(ni_runtime::settings(&args, false)?, None);
        }
        Ok(())
    }

    #[test]
    fn motor_config_receipt_contains_the_complete_loaded_jog_config() -> anyhow::Result<()> {
        let mut config = full::jog::DEFAULT;
        config.platter_target_rpm = 45;
        config.tt_pitch_bend_gain = 4.0;

        let receipt = motor_config_receipt(&config)?;
        let payload = receipt
            .strip_prefix("S4_MOTOR_CONFIG ")
            .context("receipt prefix missing")?;
        let value: serde_json::Value = serde_json::from_str(payload)?;

        assert_eq!(value["jog"], serde_json::to_value(config)?);
        Ok(())
    }

    #[test]
    fn midi_profiles_preserve_the_bare_flag_and_export_is_exclusive() -> anyhow::Result<()> {
        // Given the old bare MIDI command and the new explicit full profile.
        let minimal = Args::try_parse_from(["probe", "--midi"])?;
        let full = Args::try_parse_from(["probe", "--midi", "full"])?;
        // Then both select their intended profile.
        assert!(matches!(minimal.midi, Some(MidiProfile::Minimal)));
        assert!(matches!(full.midi, Some(MidiProfile::Full)));
        // Export cannot accidentally also open the controller.
        assert!(
            Args::try_parse_from([
                "probe",
                "--generate-mapping",
                "mapping.djayMidiMapping",
                "--midi"
            ])
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn full_descriptor_corrects_all_pots_and_physical_pad_order() -> anyhow::Result<()> {
        // Given the pinned hardware layout.
        let mut registry = DescriptorRegistry::new();
        registry.load_builtins()?;
        let original = registry
            .find(S4_VENDOR_ID, S4_PRODUCT_ID)
            .context("S4 descriptor missing")?;
        // When the full profile prepares it.
        let descriptor = full_control_descriptor(original)?;
        let mut pads = 0;
        let mut fields = 0;
        // Then every pot uses full ADC range and pad masks follow physical order.
        for item in descriptor
            .input_packets
            .iter()
            .flat_map(|packet| &packet.items)
        {
            fields += 1;
            match item {
                InputItemDesc::Slider(slider) => assert_eq!(slider.max_value, Some(4095)),
                InputItemDesc::Button(button)
                    if button.name.starts_with("left_pad_")
                        || button.name.starts_with("right_pad_") =>
                {
                    let index = button
                        .name
                        .rsplit('_')
                        .next()
                        .context("pad suffix missing")?
                        .parse::<usize>()?
                        - 1;
                    assert_eq!(button.mask.0, 1 << [5, 4, 7, 6, 3, 2, 1, 0][index]);
                    pads += 1;
                }
                _ => {}
            }
        }
        assert_eq!(pads, 16);
        assert_eq!(fields, 147);
        Ok(())
    }

    #[test]
    fn mapped_mixer_faders_use_the_s4_adc_range() -> anyhow::Result<()> {
        // Given the pinned S4 descriptor.
        let mut registry = DescriptorRegistry::new();
        registry.load_builtins()?;
        let original = registry
            .find(S4_VENDOR_ID, S4_PRODUCT_ID)
            .context("S4 descriptor missing")?;
        // When the probe prepares its control descriptor.
        let descriptor = control_descriptor(original);
        // Then the five mapped faders normalize 12-bit full scale to 1.0.
        let mut checked = 0_u8;
        for item in descriptor
            .input_packets
            .iter()
            .flat_map(|packet| &packet.items)
        {
            if let InputItemDesc::Slider(slider) = item
                && matches!(
                    slider.name.as_str(),
                    "ch1_fader" | "ch2_fader" | "ch3_fader" | "ch4_fader" | "crossfader"
                )
            {
                assert!(slider.normalize);
                assert_eq!(slider.bits, 16);
                assert_eq!(slider.max_value, Some(4095));
                checked = checked.saturating_add(1);
            }
        }
        assert_eq!(checked, 5);
        Ok(())
    }

    #[test]
    fn raw_logging_is_enabled_only_when_requested() -> anyhow::Result<()> {
        for (arguments, expected) in [(&["probe"][..], false), (&["probe", "--raw"][..], true)] {
            // Given command-line arguments with or without the diagnostic flag.
            // When the CLI parses them.
            let args = Args::try_parse_from(arguments)?;
            // Then raw logging is enabled only by the flag.
            assert_eq!(args.raw, expected);
        }
        Ok(())
    }

    #[test]
    fn midi_capture_cli_is_opt_in_and_requires_a_midi_runtime() -> anyhow::Result<()> {
        let defaults = Args::try_parse_from(["bridge"])?;
        assert!(defaults.midi_capture.is_none());
        assert_eq!(
            defaults.midi_backend,
            if cfg!(windows) {
                midi_transport::Backend::Bloop
            } else {
                midi_transport::Backend::CoreMidi
            }
        );
        for (name, backend) in [
            ("loopbe", midi_transport::Backend::Loopbe),
            ("bloop", midi_transport::Backend::Bloop),
        ] {
            let args = Args::try_parse_from([
                "bridge",
                "--midi",
                "full",
                "--midi-backend",
                name,
                "--managed-stdin",
                "--midi-capture",
                "/tmp/diagnostic.jsonl",
            ])?;
            assert_eq!(
                args.midi_capture,
                Some(PathBuf::from("/tmp/diagnostic.jsonl"))
            );
            assert_eq!(args.midi_backend, backend);
            assert!(args.managed.stdin);
        }
        for invalid in [
            vec!["bridge", "--midi-capture", "/tmp/diagnostic.jsonl"],
            vec![
                "bridge",
                "--midi",
                "full",
                "--midi-capture",
                "/tmp/diagnostic.jsonl",
                "--list",
            ],
            vec![
                "bridge",
                "--midi",
                "full",
                "--midi-capture",
                "/tmp/diagnostic.jsonl",
                "--generate-mapping",
                "out.plist",
            ],
        ] {
            assert!(Args::try_parse_from(invalid).is_err());
        }
        Ok(())
    }

    #[test]
    fn continuous_run_is_explicit_and_conflicts_with_a_duration() -> anyhow::Result<()> {
        assert!(!Args::try_parse_from(["probe"])?.until_stopped);
        assert!(
            Args::try_parse_from(["probe", "--midi", "full", "--until-stopped"])?.until_stopped
        );
        assert!(Args::try_parse_from(["probe", "--until-stopped", "--seconds", "60"]).is_err());
        Ok(())
    }

    #[test]
    fn managed_screen_stdin_accepts_full_profile_with_or_without_initial_screens()
    -> anyhow::Result<()> {
        for enabled in [false, true] {
            // Given an explicitly requested full controller runtime.
            let mut arguments = vec!["probe", "--midi", "full", "--screen-control-stdin"];
            if enabled {
                arguments.push("--screens");
            }
            // When the CLI parses the managed screen invocation.
            let args = Args::try_parse_from(arguments)?;
            // Then both enabled and initially-off generations are admitted.
            assert!(args.screens.control_stdin);
            assert_eq!(args.screens.enabled, enabled);
        }
        Ok(())
    }

    #[test]
    fn jog_preferences_are_accepted_for_export_and_full_runtime() -> anyhow::Result<()> {
        // Given a jog configuration supplied to export and runtime commands.
        let valid = [
            "probe",
            "--generate-mapping",
            "new.plist",
            "--jog-config",
            "jog.json",
        ];
        // When the real argument parser accepts the supported invocation.
        let args = Args::try_parse_from(valid)?;
        // Then export and full runtime retain the path, while bare probe use fails.
        assert_eq!(args.jog_config, Some(PathBuf::from("jog.json")));
        assert!(Args::try_parse_from(["probe", "--jog-config", "jog.json"]).is_err());
        let full = Args::try_parse_from(["probe", "--midi", "full", "--jog-config", "jog.json"])?;
        assert_eq!(full.jog_config, Some(PathBuf::from("jog.json")));
        Ok(())
    }

    #[test]
    fn continuous_stop_accepts_idle_counts_but_bounded_probe_requires_activity()
    -> anyhow::Result<()> {
        let continuous = Args::try_parse_from(["probe", "--midi", "full", "--until-stopped"])?;
        assert!(report_probe_counts(&continuous, &ProbeCounts::default()).is_ok());
        let bounded = Args::try_parse_from(["probe", "--midi", "full", "--seconds", "60"])?;
        assert!(report_probe_counts(&bounded, &ProbeCounts::default()).is_err());
        Ok(())
    }

    #[test]
    fn probe_restricts_usb_access_and_output_to_controls() -> anyhow::Result<()> {
        // Given the actual descriptor from the pinned dependency.
        let mut registry = DescriptorRegistry::new();
        registry.load_builtins()?;
        let original = registry
            .find(S4_VENDOR_ID, S4_PRODUCT_ID)
            .context("S4 descriptor missing")?;

        // When the probe narrows it for this stage.
        let descriptor = control_descriptor(original);

        // Then audio/screens/motor outputs cannot be claimed or written.
        assert_eq!(descriptor.interfaces.len(), 1);
        let interface = descriptor
            .interfaces
            .first()
            .context("control interface missing")?;
        assert_eq!(interface.number, 3);
        assert!(descriptor.screens.is_empty());
        assert_eq!(descriptor.leds.len(), 1);
        assert!(descriptor.leds.iter().all(|group| group.id == "buttons"));
        assert!(
            descriptor
                .input_packets
                .iter()
                .all(|packet| packet.interface == interface.id)
        );
        assert!(descriptor.quirks.init_writes.is_empty());
        assert!(descriptor.quirks.feature_report_leds.is_none());
        Ok(())
    }

    #[test]
    fn numbered_reports_are_stripped_and_checked_before_parsing() -> anyhow::Result<()> {
        // Given numbered S4 reports with their descriptor-defined payload sizes.
        for (id, size) in [(1, 22), (2, 78), (3, 59)] {
            let mut report = vec![id];
            report.resize(size + 1, 0x40);

            // When framing is decoded.
            let payload = input_payload(&report)?.context("known report ignored")?;

            // Then only the payload is exposed to Encdr.
            assert_eq!(payload, vec![0x40; size]);
        }
        Ok(())
    }

    #[test]
    fn malformed_known_reports_fail_instead_of_silently_losing_input() {
        // Given a truncated button report.
        let report = [1, 0];
        // When framing is decoded.
        let result = input_payload(&report);
        // Then malformed input is rejected.
        assert!(result.is_err());
    }
}
