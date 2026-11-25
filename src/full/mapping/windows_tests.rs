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
fn windows_stem_fx_mapping_uses_disjoint_deck_addresses() -> anyhow::Result<()> {
    for backend in [Backend::Loopbe, Backend::Bloop] {
        let mapping = generate(&super::super::jog::DEFAULT, backend)?;
        let root = mapping.as_dictionary().context("root")?;
        for deck in 0_u8..4 {
            let turntable = deck + 1;
            for slot in 0_u8..3 {
                for (route, (action, state)) in crate::full::stem_fx::VARIANTS.iter().enumerate() {
                    let route = u8::try_from(route)?;
                    for (field, key, expected) in [
                        (
                            "controls",
                            format!("turntable{turntable}.fx{}Routing.{action}", slot + 1),
                            (8 + deck, 1, 80 + slot * 5 + route),
                        ),
                        (
                            "outputs",
                            format!("turntable{turntable}.fx{}.routing.{state}", slot + 1),
                            (12 + deck, 3, 51 + slot * 5 + route),
                        ),
                    ] {
                        let rows = root[field].as_array().context("mapping rows")?;
                        let matches: Vec<_> = rows
                            .iter()
                            .filter(|row| {
                                row.as_dictionary()
                                    .and_then(|d| d.get("keyPath"))
                                    .and_then(Value::as_string)
                                    == Some(key.as_str())
                            })
                            .collect();
                        assert_eq!(matches.len(), 1, "{key}");
                        assert_eq!(address(matches[0])?, expected, "{key}");
                    }
                }
            }
            for (index, suffix) in crate::full::stem_fx::LAYOUTS.iter().enumerate() {
                let key =
                    format!("turntable{turntable}.view.state.hasUnmixerToolTrackMode{suffix}");
                let rows = root["outputs"].as_array().context("outputs")?;
                let matches: Vec<_> = rows
                    .iter()
                    .filter(|row| {
                        row.as_dictionary()
                            .and_then(|d| d.get("keyPath"))
                            .and_then(Value::as_string)
                            == Some(key.as_str())
                    })
                    .collect();
                assert_eq!(matches.len(), 1, "{key}");
                assert_eq!(
                    address(matches[0])?,
                    (12 + deck, 3, 66 + u8::try_from(index)?)
                );
            }
        }
    }
    Ok(())
}

#[test]
fn windows_mapping_outputs_are_disjoint_and_reach_real_decoder_cache() -> anyhow::Result<()> {
    for backend in [Backend::Loopbe, Backend::Bloop] {
        mapping_outputs_reach_decoder_cache(backend)?;
    }
    Ok(())
}

fn assert_mute_fx_addresses(controls: &[Value], outputs: &[Value]) -> anyhow::Result<()> {
    for (rows, key, expected) in [
        (controls, "mixer.toggleUnmixerMuteFxEnabled", (4, 1, 4)),
        (outputs, "turntable1.unmixer.muteWithFx", (6, 3, 9)),
    ] {
        let matches: Vec<_> = rows
            .iter()
            .filter(|row| {
                row.as_dictionary()
                    .and_then(|fields| fields.get("keyPath"))
                    .and_then(Value::as_string)
                    == Some(key)
            })
            .collect();
        assert_eq!(matches.len(), 1, "{key}");
        assert_eq!(address(matches[0])?, expected, "{key}");
    }
    Ok(())
}

fn mapping_outputs_reach_decoder_cache(backend: Backend) -> anyhow::Result<()> {
    let mapping = generate(&super::super::jog::DEFAULT, backend)?;
    let mut xml = Vec::new();
    mapping.to_writer_xml(&mut xml)?;
    let mapping = Value::from_reader(std::io::Cursor::new(xml))?;
    let root = mapping.as_dictionary().context("root")?;
    assert_eq!(root["endpointName"].as_string(), backend.port_name());
    let controls = root["controls"].as_array().context("controls")?;
    let commands: BTreeSet<_> = controls
        .iter()
        .map(address)
        .collect::<anyhow::Result<_>>()?;
    let outputs = root["outputs"].as_array().context("outputs")?;
    assert_mute_fx_addresses(controls, outputs)?;
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
                backend,
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
        let expected_channel = match (channel, family, number) {
            (12..=15, 3, 51..=69) => channel,
            (12..=15, _, _) => channel - 12,
            (7, _, _) => 4,
            (6, _, _) => 6,
            _ => anyhow::bail!("unexpected feedback channel"),
        };
        let mut packets = Vec::new();
        decoder.feed(&[status, number, 127], |message| packets.push(message));
        assert_eq!(packets.len(), 1);
        let normalized = crate::midi_layout::normalize(backend, packets[0])
            .context("mapping output accepted")?;
        assert_eq!(normalized.status & 0x0f, expected_channel);
        state.apply(normalized, Duration::ZERO);
    }
    assert_eq!(commands.len(), 643);
    assert_eq!(feedback.len(), 426);
    assert_eq!(state.mute_fx, Some(true));
    assert_eq!(state.playing, [Some(true); 4]);
    assert_eq!(state.internal, [Some(true); 4]);
    assert_eq!(state.thru, [Some(true); 4]);
    assert_eq!(state.meters, [Some(127); 4]);
    assert!(state.playback_received);
    assert_playback_resets(backend, &mut state, &mut decoder)
}

fn assert_playback_resets(
    backend: Backend,
    state: &mut FeedbackState,
    decoder: &mut Decoder,
) -> anyhow::Result<()> {
    for number in 4..=7 {
        let mut packets = Vec::new();
        decoder.feed(&[0xb6, number, 0], |message| packets.push(message));
        assert_eq!(packets.len(), 1);
        for message in packets {
            let normalized = crate::midi_layout::normalize(backend, message)
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
        let native_controls = controls(&native)?;
        let windows_controls = controls(&windows)?;
        let native_shared: Vec<_> = native_controls
            .iter()
            .filter(|control| {
                !control
                    .as_dictionary()
                    .and_then(|fields| fields.get("keyPath"))
                    .and_then(Value::as_string)
                    .is_some_and(|key| key.contains("Routing.routing"))
            })
            .collect();
        let windows_shared: Vec<_> = windows_controls
            .into_iter()
            .filter(|control| {
                !control
                    .as_dictionary()
                    .and_then(|fields| fields.get("keyPath"))
                    .and_then(Value::as_string)
                    .is_some_and(|key| key.contains("Routing.routing"))
            })
            .collect();
        assert_eq!(native_shared.len(), windows_shared.len());
        for (native, windows) in native_shared.into_iter().zip(windows_shared) {
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
