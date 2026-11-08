use super::*;
use crate::full::feedback::{Decoder, FeedbackState, Message};
use std::{collections::BTreeSet, time::Duration};

fn address(value: &Value) -> anyhow::Result<(u8, u8, u8)> {
    let dictionary = value.as_dictionary().context("address dictionary")?;
    let field = |name| {
        dictionary
            .get(name)
            .and_then(Value::as_unsigned_integer)
            .context("address field")
            .and_then(|value| Ok(u8::try_from(value)?))
    };
    Ok((
        field("midiChannel")?,
        field("midiMessageType")?,
        field("midiData")?,
    ))
}

#[test]
fn windows_mapping_outputs_are_disjoint_and_reach_real_decoder_cache() -> anyhow::Result<()> {
    let mapping = generate(&super::super::jog::DEFAULT, Backend::Loopbe)?;
    let mut xml = Vec::new();
    mapping.to_writer_xml(&mut xml)?;
    let mapping = Value::from_reader(std::io::Cursor::new(xml))?;
    let root = mapping.as_dictionary().context("root")?;
    assert_eq!(root["endpointName"].as_string(), Some(LOOPBE_PORT));
    let controls = root["controls"].as_array().context("controls")?;
    let commands: BTreeSet<_> = controls
        .iter()
        .map(address)
        .collect::<anyhow::Result<_>>()?;
    let outputs = root["outputs"].as_array().context("outputs")?;
    for deck in 1_u8..=4 {
        let matches: Vec<_> = outputs
            .iter()
            .filter(|output| address(output).ok() == Some((6, 3, deck + 3)))
            .collect();
        assert_eq!(matches.len(), 1);
        assert_eq!(
            matches[0].as_dictionary().context("playback output")?["keyPath"].as_string(),
            Some(format!("turntable{deck}.isPlaying").as_str())
        );
    }
    let mut feedback = BTreeSet::new();
    let mut state = FeedbackState::default();
    let mut decoder = Decoder::default();
    for control in controls {
        let (channel, family, number) = address(control)?;
        let status = (if family == 1 { 0x90 } else { 0xb0 }) | channel;
        assert_eq!(
            crate::midi_layout::normalize(
                Backend::Loopbe,
                Message {
                    status,
                    number,
                    value: 127
                }
            ),
            None
        );
    }
    for output in outputs.iter().chain(
        controls
            .iter()
            .filter_map(|control| control.as_dictionary()?.get("output")),
    ) {
        let (channel, family, number) = address(output)?;
        assert!(feedback.insert((channel, family, number)));
        assert!(!commands.contains(&(channel, family, number)));
        let status = (if family == 1 { 0x90 } else { 0xb0 }) | channel;
        let expected_channel = match channel {
            12..=15 => channel - 12,
            7 => 4,
            6 => 6,
            _ => anyhow::bail!("unexpected feedback channel"),
        };
        let mut packets = Vec::new();
        decoder.feed(&[status, number, 127], |message| packets.push(message));
        assert_eq!(packets.len(), 1);
        let normalized = crate::midi_layout::normalize(Backend::Loopbe, packets[0])
            .context("mapping output accepted")?;
        assert_eq!(normalized.status & 0x0f, expected_channel);
        state.apply(normalized, Duration::ZERO);
    }
    assert_eq!(commands.len(), 582);
    assert_eq!(feedback.len(), 345);
    assert_eq!(state.playing, [Some(true); 4]);
    assert_eq!(state.internal, [Some(true); 4]);
    assert_eq!(state.thru, [Some(true); 4]);
    assert_eq!(state.meters, [Some(127); 4]);
    assert!(state.playback_received);
    for number in 4..=7 {
        let mut packets = Vec::new();
        decoder.feed(&[0xb6, number, 0], |message| packets.push(message));
        assert_eq!(packets.len(), 1);
        for message in packets {
            let normalized = crate::midi_layout::normalize(Backend::Loopbe, message)
                .context("playback feedback accepted")?;
            state.apply(normalized, Duration::ZERO);
        }
    }
    assert_eq!(state.playing, [Some(false); 4]);
    Ok(())
}

#[test]
fn native_mapping_and_jog_semantics_survive_windows_export() -> anyhow::Result<()> {
    let mut custom = super::super::jog::DEFAULT;
    custom.scratch_speed = 9.0;
    custom.pitch_bend_reaction = 31;
    custom.scratch_direction = super::super::jog::ScratchDirection::Reversed;
    for jog in [super::super::jog::DEFAULT, custom] {
        let native = super::super::generate_with_jog(&jog)?;
        assert_eq!(generate(&jog, Backend::CoreMidi)?, native);
        let windows = generate(&jog, Backend::Loopbe)?;
        let controls = |value: &Value| {
            value
                .as_dictionary()
                .and_then(|root| root.get("controls"))
                .and_then(Value::as_array)
                .cloned()
                .context("controls")
        };
        for (native, windows) in controls(&native)?.iter().zip(controls(&windows)?) {
            let mut expected = native.as_dictionary().context("native control")?.clone();
            let mut actual = windows.as_dictionary().context("Windows control")?.clone();
            let expected_output = expected.remove("output");
            let actual_output = actual.remove("output");
            assert_eq!(expected, actual);
            match (expected_output, actual_output) {
                (Some(expected), Some(mut actual)) => {
                    let output = actual.as_dictionary_mut().context("embedded output")?;
                    for key in ["midiChannel", "midiData", "midiMessageType"] {
                        output.remove(key);
                    }
                    assert_eq!(expected, actual);
                }
                (None, None) => {}
                _ => anyhow::bail!("feedback declaration lost"),
            }
        }
    }
    Ok(())
}
