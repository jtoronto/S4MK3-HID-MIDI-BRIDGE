//! A bounded S4 MK3 HID probe with optional minimal MIDI output.

mod full;
mod input;
mod midi;

use std::fs::OpenOptions;
use std::io::Write as _;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context as _, bail, ensure};
use clap::{Parser, ValueEnum};
use encdr::core::descriptor::{ButtonItemDesc, HexU16, InputItemDesc, SliderItemDesc};
use encdr::device::led_builder::LedBuilder;
use encdr::device::loader::DescriptorRegistry;
use encdr::device::parser::PacketParser;
use encdr::usb::hotplug;
use encdr::{DeviceDescriptor, Event, LedValue};
use hidapi::{HidApi, HidDevice};
use tracing::{debug, info, warn};
use tracing_subscriber::EnvFilter;

const S4_VENDOR_ID: u16 = 0x17cc;
const S4_PRODUCT_ID: u16 = 0x1720;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum MidiProfile {
    Minimal,
    Full,
}

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// List attached S4 MK3 devices without opening their USB interfaces.
    #[arg(long)]
    list: bool,

    /// Log every complete input HID report in hex, including its report ID.
    #[arg(long)]
    raw: bool,

    /// Publish virtual MIDI; bare --midi retains the verified minimal profile.
    #[arg(long, value_enum, num_args = 0..=1, default_missing_value = "minimal", conflicts_with_all = ["list", "generate_mapping"])]
    midi: Option<MidiProfile>,

    /// JSON LED preferences for the full profile; restart after editing.
    #[arg(long, value_name = "PATH", requires = "midi", conflicts_with_all = ["list", "generate_mapping"])]
    led_config: Option<PathBuf>,

    /// Export the full native Djay mapping without opening HID or MIDI.
    #[arg(long, value_name = "PATH", conflicts_with_all = ["midi", "list", "raw"])]
    generate_mapping: Option<PathBuf>,

    /// Maximum probe duration; stop earlier with Ctrl-C.
    #[arg(long, default_value = "30")]
    seconds: NonZeroU64,

    /// Run until Ctrl-C instead of stopping after a fixed duration.
    #[arg(long, conflicts_with = "seconds")]
    until_stopped: bool,
}

#[derive(Debug, Default)]
struct ProbeCounts {
    input_events: u64,
    led_writes: u64,
    midi_messages: u64,
    feedback_messages: u64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    ensure!(
        args.led_config.is_none() || matches!(args.midi, Some(MidiProfile::Full)),
        "--led-config requires --midi full"
    );
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?)
        .with_target(false)
        .with_ansi(false)
        .try_init()
        .map_err(|error| anyhow::anyhow!("initialize logging: {error}"))?;

    if let Some(path) = &args.generate_mapping {
        let mapping = full::mapping::generate()?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .with_context(|| {
                format!("create mapping {}; choose a new file path", path.display())
            })?;
        mapping
            .to_writer_xml(&mut output)
            .context("serialize Djay mapping")?;
        output.flush().context("flush Djay mapping")?;
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
    let api = HidApi::new().context("initialize macOS-native HID access")?;
    let mut collections = api.device_list().filter(|collection| {
        collection.vendor_id() == S4_VENDOR_ID && collection.product_id() == S4_PRODUCT_ID
    });
    let collection = collections.next().context("no S4 HID collection found")?;
    ensure!(
        collections.next().is_none(),
        "multiple S4 HID collections found; collection selection requires investigation"
    );
    let hid = api
        .open_path(collection.path())
        .context("open S4 HID collection")?;
    let interface = descriptor
        .interface_by_id("control")
        .context("control interface missing")?;
    let buttons = descriptor
        .leds
        .first()
        .context("button LED layout missing")?;
    let mut leds = LedBuilder::new(buttons, interface);
    info!("S4_HID_OPEN: native HID access; audio interfaces are not claimed");
    run_probe(&args, &hid, &mut parser, &mut leds)
}

fn run_probe(
    args: &Args,
    hid: &HidDevice,
    parser: &mut PacketParser,
    leds: &mut LedBuilder,
) -> anyhow::Result<()> {
    let led_config = matches!(args.midi, Some(MidiProfile::Full))
        .then(|| full::leds::LedConfig::load(args.led_config.as_deref()))
        .transpose()?;
    let stop_rx = stop_signal()?;
    let mut midi = matches!(args.midi, Some(MidiProfile::Minimal))
        .then(midi::MidiBridge::new)
        .transpose()?;
    let mut full = matches!(args.midi, Some(MidiProfile::Full))
        .then(full::FullBridge::new)
        .transpose()?;
    let mut full_leds = full
        .as_ref()
        .map(|_| full::output::LedOutput::new(hid))
        .transpose()?;
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
        match stop_rx.try_recv() {
            Ok(()) => break,
            Err(crossbeam_channel::TryRecvError::Empty) => {}
            Err(crossbeam_channel::TryRecvError::Disconnected) => bail!("stop channel closed"),
        }
        let lengths = input::read_batch(hid, full.is_some(), &mut buffers)?;
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
                record_event(event, leds, &mut counts, full.is_none())?;
                if let Some(bridge) = &mut midi
                    && bridge.send(event)?
                {
                    counts.midi_messages = counts.midi_messages.saturating_add(1);
                }
            }
            if let Some(bridge) = &mut full {
                let buttons = if report.first() == Some(&1) {
                    Some(
                        payload
                            .try_into()
                            .context("full button payload must have 22 bytes")?,
                    )
                } else {
                    None
                };
                let sent = bridge.send_report(&events, buttons)?;
                counts.midi_messages = counts.midi_messages.saturating_add(u64::try_from(sent)?);
            }
            if full.is_none()
                && let Some(report) = leds.flush()
            {
                let written = hid.write(&report).context("write S4 button LED report")?;
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
        if let (Some(bridge), Some(output), Some(config)) = (&mut full, &mut full_leds, &led_config)
        {
            let now = started.elapsed();
            let (received, sent) = bridge.drain_feedback(now)?;
            counts.feedback_messages = counts
                .feedback_messages
                .saturating_add(u64::try_from(received)?);
            counts.midi_messages = counts.midi_messages.saturating_add(u64::try_from(sent)?);
            let writes = output.tick(
                config,
                bridge.feedback_state(),
                &bridge.local_led_state(),
                now,
            )?;
            counts.led_writes = counts.led_writes.saturating_add(u64::try_from(writes)?);
        }
    }
    report_probe_counts(args, &counts)
}

fn stop_signal() -> anyhow::Result<crossbeam_channel::Receiver<()>> {
    let (sender, receiver) = crossbeam_channel::bounded(1);
    ctrlc::set_handler(move || {
        if let Err(error) = sender.try_send(()) {
            warn!(%error, "could not enqueue stop signal");
        }
    })
    .context("install Ctrl-C handler")?;
    Ok(receiver)
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
    if args.until_stopped {
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
            info!(control = name, value, "SLIDER");
        }
        Event::Encoder { name, delta, .. } => {
            info!(control = name, delta, "ENCODER");
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
    fn continuous_run_is_explicit_and_conflicts_with_a_duration() -> anyhow::Result<()> {
        assert!(!Args::try_parse_from(["probe"])?.until_stopped);
        assert!(
            Args::try_parse_from(["probe", "--midi", "full", "--until-stopped"])?.until_stopped
        );
        assert!(Args::try_parse_from(["probe", "--until-stopped", "--seconds", "60"]).is_err());
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
