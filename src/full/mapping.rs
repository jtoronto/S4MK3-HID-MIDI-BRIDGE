//! Generates a native Djay MIDI mapping plist from the shared catalog.

use std::collections::BTreeSet;

use anyhow::{Result, ensure};
use plist::{Dictionary, Value};

use super::catalog::{
    Binding, DECK_BINDINGS, FX_LEFT_BINDINGS, FX_RIGHT_BINDINGS, Kind, MIXER_BINDINGS, PadMode,
    QUICK_BINDINGS, pad_binding,
};
use super::feedback::{PAD_COLOR_TOKENS, PAD_WHITE_TOKEN};

pub const PORT_NAME: &str = "S4 MK3 MIDI Full";
const DECK_MARKER: &str = "{deck}";

pub fn generate() -> Result<Value> {
    let mut controls = Vec::new();
    let mut addresses = BTreeSet::new();
    let mut outputs = Vec::new();
    let mut output_addresses = BTreeSet::new();

    for binding in DECK_BINDINGS {
        for deck in 1_u8..=4 {
            push_deck(&mut controls, &mut addresses, *binding, deck)?;
        }
    }
    for bindings in [FX_LEFT_BINDINGS, FX_RIGHT_BINDINGS, QUICK_BINDINGS] {
        for binding in bindings {
            for deck in 1_u8..=4 {
                push_deck(&mut controls, &mut addresses, *binding, deck)?;
            }
        }
    }
    for mode in [PadMode::Hotcue, PadMode::Samples, PadMode::Stems] {
        for index in 0_u8..8 {
            let binding = pad_binding(mode, index)
                .ok_or_else(|| anyhow::anyhow!("pad catalog omitted index {index}"))?;
            for deck in 1_u8..=4 {
                push_deck(&mut controls, &mut addresses, binding, deck)?;
            }
        }
    }
    for binding in MIXER_BINDINGS {
        push_control(&mut controls, &mut addresses, *binding, binding.target, 4)?;
    }
    for deck in 1_u8..=4 {
        for (suffix, data, message_type, meter) in [
            ("monoMeter", deck - 1, 3, true),
            ("playing", deck + 3, 3, false),
            ("song.loadingSuccess", deck - 1, 1, false),
        ] {
            let key = format!("turntable{deck}.{suffix}");
            push_output(
                &mut outputs,
                &mut output_addresses,
                OutputSpec {
                    key_path: &key,
                    channel: 6,
                    data,
                    message_type,
                    meter,
                },
            )?;
        }
    }
    for deck in 1_u8..=4 {
        let selected = super::catalog::deck_selection_binding(deck)
            .ok_or_else(|| anyhow::anyhow!("deck selector missing"))?;
        push_control(&mut controls, &mut addresses, selected, selected.target, 5)?;
        for state in 0_u8..=2 {
            let assignment = super::catalog::assignment_binding(deck, state)
                .ok_or_else(|| anyhow::anyhow!("crossfader selector missing"))?;
            push_control(
                &mut controls,
                &mut addresses,
                assignment,
                assignment.target,
                4,
            )?;
        }
    }
    let curve = super::catalog::CURVE_BINDING;
    push_control(&mut controls, &mut addresses, curve, curve.target, 4)?;

    let mut root = Dictionary::new();
    root.insert(
        String::from("endpointName"),
        Value::String(PORT_NAME.to_owned()),
    );
    root.insert(String::from("schemeVersion"), Value::Integer(1.into()));
    root.insert(String::from("version"), Value::Integer(0.into()));
    root.insert(String::from("controls"), Value::Array(controls));
    root.insert(String::from("outputs"), Value::Array(outputs));
    let mut user_info = Dictionary::new();
    user_info.insert(String::from("padColorOff"), Value::Integer(0.into()));
    user_info.insert(
        String::from("padColorWhiteActive"),
        Value::Integer(i64::from(PAD_WHITE_TOKEN).into()),
    );
    user_info.insert(
        String::from("padColorsActive"),
        Value::Array(
            PAD_COLOR_TOKENS
                .iter()
                .map(|token| Value::Integer(i64::from(*token).into()))
                .collect(),
        ),
    );
    root.insert(String::from("userInfo"), Value::Dictionary(user_info));
    Ok(Value::Dictionary(root))
}

fn push_output(
    outputs: &mut Vec<Value>,
    addresses: &mut BTreeSet<(u8, u8, u8)>,
    spec: OutputSpec<'_>,
) -> Result<()> {
    let OutputSpec {
        key_path,
        channel,
        data,
        message_type,
        meter,
    } = spec;
    ensure!(
        channel <= 15 && data <= 127 && !key_path.contains('{'),
        "invalid generated MIDI output"
    );
    ensure!(
        addresses.insert((channel, message_type, data)),
        "duplicate MIDI output address: channel {channel}, type {message_type}, data {data}"
    );
    let mut entry = Dictionary::new();
    entry.insert(String::from("keyPath"), Value::String(key_path.to_owned()));
    entry.insert(
        String::from("midiChannel"),
        Value::Integer(i64::from(channel).into()),
    );
    entry.insert(
        String::from("midiData"),
        Value::Integer(i64::from(data).into()),
    );
    entry.insert(
        String::from("midiMessageType"),
        Value::Integer(i64::from(message_type).into()),
    );
    if meter {
        entry.insert(
            String::from("controlType"),
            Value::String(String::from("control")),
        );
    }
    entry.insert(String::from("midiMinValue"), Value::Integer(0.into()));
    entry.insert(String::from("midiMaxValue"), Value::Integer(127.into()));
    outputs.push(Value::Dictionary(entry));
    Ok(())
}

#[derive(Clone, Copy)]
struct OutputSpec<'a> {
    key_path: &'a str,
    channel: u8,
    data: u8,
    message_type: u8,
    meter: bool,
}

fn push_deck(
    controls: &mut Vec<Value>,
    addresses: &mut BTreeSet<(u8, u8, u8)>,
    binding: Binding,
    deck: u8,
) -> Result<()> {
    let deck_number = deck.to_string();
    let bank_number = if deck % 2 == 1 { "1" } else { "2" };
    let target = binding
        .target
        .replace(DECK_MARKER, &deck_number)
        .replace("{bank}", bank_number);
    push_control(
        controls,
        addresses,
        binding,
        target.as_str(),
        deck.saturating_sub(1),
    )?;
    if let Some(shifted) = binding.shifted {
        let shifted_target = shifted
            .replace(DECK_MARKER, &deck_number)
            .replace("{bank}", bank_number);
        push_control(
            controls,
            addresses,
            binding,
            shifted_target.as_str(),
            deck.saturating_add(7),
        )?;
    }
    Ok(())
}

fn push_control(
    controls: &mut Vec<Value>,
    addresses: &mut BTreeSet<(u8, u8, u8)>,
    binding: Binding,
    key_path: &str,
    channel: u8,
) -> Result<()> {
    let message_type = match binding.kind {
        Kind::Note => 1,
        Kind::Absolute | Kind::Relative => 3,
    };
    ensure!(
        channel <= 15 && binding.number <= 127 && !key_path.contains('{'),
        "invalid generated MIDI binding"
    );
    ensure!(
        addresses.insert((channel, message_type, binding.number)),
        "duplicate MIDI address: channel {channel}, type {message_type}, data {}",
        binding.number
    );

    let mut control = Dictionary::new();
    control.insert(String::from("keyPath"), Value::String(key_path.to_owned()));
    control.insert(
        String::from("midiChannel"),
        Value::Integer(i64::from(channel).into()),
    );
    control.insert(
        String::from("midiData"),
        Value::Integer(i64::from(binding.number).into()),
    );
    control.insert(
        String::from("midiMessageType"),
        Value::Integer(message_type.into()),
    );
    if binding.kind == Kind::Relative {
        control.insert(
            String::from("controlType"),
            Value::String(String::from("rotary-64")),
        );
    }
    if binding.pickup {
        control.insert(String::from("pickupMode"), Value::Boolean(true));
    }
    if binding.flipped {
        control.insert(String::from("flipped"), Value::Boolean(true));
    }
    if let Some(sensitivity) = binding.sensitivity {
        control.insert(String::from("rotarySensitivity"), Value::Real(sensitivity));
    }
    let has_feedback = binding.kind == Kind::Note
        && ((channel < 4 && matches!(binding.number, 0..=4 | 6 | 8 | 15..=19 | 32..=35 | 40..=63))
            || (channel == 4 && binding.number < 4));
    if has_feedback {
        let mut output = Dictionary::new();
        if !(40..=47).contains(&binding.number) {
            output.insert(String::from("midiMinValue"), Value::Integer(0.into()));
            output.insert(String::from("midiMaxValue"), Value::Integer(127.into()));
        }
        control.insert(String::from("output"), Value::Dictionary(output));
    }
    controls.push(Value::Dictionary(control));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context as _;
    use std::io::Cursor;

    fn field<'a>(control: &'a Value, key: &str) -> Option<&'a Value> {
        control
            .as_dictionary()
            .and_then(|dictionary| dictionary.get(key))
    }

    fn generated_plist() -> anyhow::Result<Value> {
        let mapping = generate()?;
        let mut bytes = Vec::new();
        plist::to_writer_xml(&mut bytes, &mapping).context("serialize generated mapping")?;
        plist::from_reader(Cursor::new(bytes)).context("parse serialized mapping")
    }

    fn generated_controls() -> anyhow::Result<Vec<Value>> {
        let mapping = generated_plist()?;
        let root = mapping.as_dictionary().context("root is a dictionary")?;
        let controls = root
            .get("controls")
            .and_then(Value::as_array)
            .context("controls is an array")?;
        Ok(controls.clone())
    }

    fn generated_outputs() -> anyhow::Result<Vec<Value>> {
        let mapping = generated_plist()?;
        let root = mapping.as_dictionary().context("root is a dictionary")?;
        let outputs = root
            .get("outputs")
            .and_then(Value::as_array)
            .context("outputs is an array")?;
        Ok(outputs.clone())
    }

    #[test]
    fn generated_mapping_has_native_root_fields_and_deck_targets() -> anyhow::Result<()> {
        let mapping = generated_plist()?;
        let root = mapping.as_dictionary().context("root is a dictionary")?;
        assert_eq!(
            root.get("endpointName").and_then(Value::as_string),
            Some(PORT_NAME)
        );
        assert_eq!(
            root.get("schemeVersion")
                .and_then(Value::as_unsigned_integer),
            Some(1)
        );
        assert_eq!(
            root.get("version").and_then(Value::as_unsigned_integer),
            Some(0)
        );
        let controls = generated_controls()?;
        assert!(controls.iter().any(|control| {
            field(control, "keyPath").and_then(Value::as_string) == Some("turntable1.playPause")
        }));
        assert!(controls.iter().any(|control| {
            field(control, "keyPath").and_then(Value::as_string) == Some("turntable4.playPause")
        }));
        Ok(())
    }

    #[test]
    fn generated_mapping_contains_native_fx_and_resolved_mixer_targets() -> anyhow::Result<()> {
        let controls = generated_controls()?;
        for key_path in [
            "turntable1.fx1ParameterValue",
            "turntable4.fx3WetDryValue",
            "turntable3.instantFx4",
            "turntable2.gain",
        ] {
            assert!(controls.iter().any(|control| {
                field(control, "keyPath").and_then(Value::as_string) == Some(key_path)
            }));
        }
        Ok(())
    }

    #[test]
    fn generated_mapping_addresses_are_unique() -> anyhow::Result<()> {
        let controls = generated_controls()?;
        let mut addresses = BTreeSet::new();
        for control in controls {
            let channel = field(&control, "midiChannel")
                .and_then(Value::as_unsigned_integer)
                .context("midiChannel is an integer")?;
            let message_type = field(&control, "midiMessageType")
                .and_then(Value::as_unsigned_integer)
                .context("midiMessageType is an integer")?;
            let data = field(&control, "midiData")
                .and_then(Value::as_unsigned_integer)
                .context("midiData is an integer")?;
            assert!(addresses.insert((channel, message_type, data)));
        }
        Ok(())
    }

    #[test]
    fn generated_controls_keep_relative_and_sampler_address_semantics() -> anyhow::Result<()> {
        let controls = generated_controls()?;
        assert!(controls.iter().any(|control| {
            field(control, "keyPath").and_then(Value::as_string)
                == Some("turntable2.autoLoopDurationRotary")
                && field(control, "controlType").and_then(Value::as_string) == Some("rotary-64")
        }));
        assert!(controls.iter().any(|control| {
            field(control, "keyPath").and_then(Value::as_string)
                == Some("sampler.turntable1.player8.playingConsideringHoldSetting")
                && field(control, "midiData").and_then(Value::as_unsigned_integer) == Some(55)
        }));
        assert!(controls.iter().any(|control| {
            field(control, "keyPath").and_then(Value::as_string)
                == Some("sampler.turntable2.player1.playingConsideringHoldSetting")
                && field(control, "midiData").and_then(Value::as_unsigned_integer) == Some(48)
        }));
        Ok(())
    }

    #[test]
    fn outputs_exclude_shifted_duplicate_and_local_addresses() -> anyhow::Result<()> {
        let outputs: Vec<_> = generated_controls()?
            .into_iter()
            .filter(|control| field(control, "output").is_some())
            .collect();
        let mut addresses = BTreeSet::new();
        for output in &outputs {
            let channel = field(output, "midiChannel")
                .and_then(Value::as_unsigned_integer)
                .context("output midiChannel is an integer")?;
            let message_type = field(output, "midiMessageType")
                .and_then(Value::as_unsigned_integer)
                .context("output midiMessageType is an integer")?;
            let data = field(output, "midiData")
                .and_then(Value::as_unsigned_integer)
                .context("output midiData is an integer")?;
            assert!(addresses.insert((channel, message_type, data)));
            assert!(!(8..=11).contains(&channel));
            assert!(!(0..=3).contains(&channel) || !(36..=39).contains(&data));
            assert!(!(channel == 4 && data >= 80));
            assert!(!(channel == 5 && data <= 3));
        }
        for channel in 0..=3 {
            for data in 0..=4 {
                assert!(addresses.contains(&(channel, 1, data)));
            }
            for data in [6, 8, 15, 16, 17, 18, 19, 32, 33, 34, 35] {
                assert!(addresses.contains(&(channel, 1, data)));
            }
            for data in 40..=63 {
                assert!(addresses.contains(&(channel, 1, data)));
            }
        }
        for data in 0..=3 {
            assert!(addresses.contains(&(4, 1, data)));
        }
        Ok(())
    }

    #[test]
    fn binary_and_colored_pad_outputs_have_distinct_value_configuration() -> anyhow::Result<()> {
        let outputs: Vec<_> = generated_controls()?
            .into_iter()
            .filter(|control| field(control, "output").is_some())
            .collect();
        for output in outputs {
            let data = field(&output, "midiData")
                .and_then(Value::as_unsigned_integer)
                .context("output midiData is an integer")?;
            let channel = field(&output, "midiChannel")
                .and_then(Value::as_unsigned_integer)
                .context("output midiChannel is an integer")?;
            if (40..=47).contains(&data) && channel <= 3 {
                let config = field(&output, "output")
                    .and_then(Value::as_dictionary)
                    .context("hotcue output configuration is a dictionary")?;
                assert!(config.is_empty());
            } else if field(&output, "controlType").and_then(Value::as_string) != Some("control") {
                let config = field(&output, "output")
                    .and_then(Value::as_dictionary)
                    .context("binary output configuration is a dictionary")?;
                assert_eq!(
                    config
                        .get("midiMinValue")
                        .and_then(Value::as_unsigned_integer),
                    Some(0)
                );
                assert_eq!(
                    config
                        .get("midiMaxValue")
                        .and_then(Value::as_unsigned_integer),
                    Some(127)
                );
            }
        }
        Ok(())
    }

    #[test]
    fn meter_load_and_playback_outputs_route_by_deck() -> anyhow::Result<()> {
        let outputs = generated_outputs()?;
        for deck in 1_u8..=4 {
            let meter_key = format!("turntable{deck}.monoMeter");
            let load_key = format!("turntable{deck}.song.loadingSuccess");
            let playing_key = format!("turntable{deck}.playing");
            let playing = outputs
                .iter()
                .find(|output| {
                    field(output, "keyPath").and_then(Value::as_string) == Some(&playing_key)
                })
                .context("raw playing output exists")?;
            assert_eq!(
                field(playing, "midiData").and_then(Value::as_unsigned_integer),
                Some(u64::from(deck + 3))
            );
            let meter = outputs
                .iter()
                .find(|output| {
                    field(output, "keyPath").and_then(Value::as_string) == Some(&meter_key)
                })
                .context("deck meter output exists")?;
            let load = outputs
                .iter()
                .find(|output| {
                    field(output, "keyPath").and_then(Value::as_string) == Some(&load_key)
                })
                .context("deck load output exists")?;
            for (output, message_type, data) in [
                (meter, 3, deck - 1),
                (load, 1, deck - 1),
                (playing, 3, deck + 3),
            ] {
                assert_eq!(
                    field(output, "midiChannel").and_then(Value::as_unsigned_integer),
                    Some(6)
                );
                assert_eq!(
                    field(output, "midiData").and_then(Value::as_unsigned_integer),
                    Some(u64::from(data))
                );
                assert_eq!(
                    field(output, "midiMessageType").and_then(Value::as_unsigned_integer),
                    Some(message_type)
                );
                assert_eq!(
                    field(output, "midiMinValue").and_then(Value::as_unsigned_integer),
                    Some(0)
                );
                assert_eq!(
                    field(output, "midiMaxValue").and_then(Value::as_unsigned_integer),
                    Some(127)
                );
            }
            assert_eq!(
                field(meter, "controlType").and_then(Value::as_string),
                Some("control")
            );
        }
        Ok(())
    }

    #[test]
    fn serialized_root_contains_native_pad_palette() -> anyhow::Result<()> {
        let mapping = generated_plist()?;
        let root = mapping.as_dictionary().context("root is a dictionary")?;
        let user_info = root
            .get("userInfo")
            .and_then(Value::as_dictionary)
            .context("userInfo is a dictionary")?;
        assert_eq!(
            user_info
                .get("padColorOff")
                .and_then(Value::as_unsigned_integer),
            Some(0)
        );
        assert_eq!(
            user_info
                .get("padColorWhiteActive")
                .and_then(Value::as_unsigned_integer),
            Some(u64::from(PAD_WHITE_TOKEN))
        );
        let colors = user_info
            .get("padColorsActive")
            .and_then(Value::as_array)
            .context("padColorsActive is an array")?;
        assert_eq!(
            colors
                .iter()
                .map(|value| value
                    .as_unsigned_integer()
                    .context("palette token is integer"))
                .collect::<anyhow::Result<Vec<_>>>()?,
            PAD_COLOR_TOKENS
                .iter()
                .map(|token| u64::from(*token))
                .collect::<Vec<_>>()
        );
        Ok(())
    }
}
