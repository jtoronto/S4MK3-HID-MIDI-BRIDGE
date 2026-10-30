//! Generates a native Djay MIDI mapping plist from the shared catalog.

use std::collections::BTreeSet;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::Path;

use anyhow::{Context as _, Result, ensure};
use plist::{Dictionary, Value};

use super::catalog::{
    Binding, DECK_BINDINGS, EQ_MODE_NOTE, EXT_BINDINGS, EXT_THRU_CC, FX_LEFT_BINDINGS,
    FX_RIGHT_BINDINGS, INTERNAL_MODE_CC, Kind, MIXER_BINDINGS, PLATTER_POSITION_BINDING, PadMode,
    QUICK_BINDINGS, pad_binding,
};
use super::feedback::{PAD_COLOR_TOKENS, PAD_WHITE_TOKEN};
use super::jog;

pub const PORT_NAME: &str = "S4 MK3 MIDI Full";
const DECK_MARKER: &str = "{deck}";

pub fn write(path: &Path, config_path: Option<&Path>) -> Result<()> {
    let mapping = match config_path {
        Some(path) => generate_with_jog(&jog::Config::load(Some(path))?)?,
        None => generate()?,
    };
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("create mapping {}; choose a new file path", path.display()))?;
    mapping
        .to_writer_xml(&mut output)
        .context("serialize Djay mapping")?;
    output.flush().context("flush Djay mapping")?;
    Ok(())
}

pub fn generate() -> Result<Value> {
    generate_with_jog(&jog::Config::default())
}

pub fn generate_with_jog(jog: &jog::Config) -> Result<Value> {
    jog.validate()?;
    let mut controls = Vec::new();
    let mut addresses = BTreeSet::new();
    let mut outputs = Vec::new();
    let mut output_addresses = BTreeSet::new();

    push_jog_controls(jog, &mut controls, &mut addresses)?;
    for bindings in [FX_LEFT_BINDINGS, FX_RIGHT_BINDINGS, QUICK_BINDINGS] {
        push_deck_bindings(bindings, &mut controls, &mut addresses)?;
    }
    push_fx_browsing(&mut controls, &mut addresses)?;
    push_jump_restores(&mut controls, &mut addresses)?;
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
    push_fx_outputs(&mut outputs, &mut output_addresses)?;
    push_ext_outputs(&mut outputs, &mut output_addresses)?;
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
    push_info_outputs(&mut outputs, &mut output_addresses)?;
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

fn push_deck_bindings(
    bindings: &[Binding],
    controls: &mut Vec<Value>,
    addresses: &mut BTreeSet<(u8, u8, u8)>,
) -> Result<()> {
    for binding in bindings {
        for deck in 1_u8..=4 {
            push_deck(controls, addresses, *binding, deck)?;
        }
    }
    Ok(())
}

fn push_jog_controls(
    jog: &jog::Config,
    controls: &mut Vec<Value>,
    addresses: &mut BTreeSet<(u8, u8, u8)>,
) -> Result<()> {
    for binding in DECK_BINDINGS.iter().chain(&EXT_BINDINGS) {
        let mut binding = *binding;
        (binding.sensitivity, binding.acceleration) = match binding.input {
            "jog_scratch" => (Some(jog.scratch_speed), Some(jog.scratch_reaction)),
            "jog_bend" => (Some(jog.pitch_bend_speed), Some(jog.pitch_bend_reaction)),
            _ => (binding.sensitivity, binding.acceleration),
        };
        if binding.input == "jog_scratch" {
            binding.flipped = jog.scratch_direction == super::jog::ScratchDirection::Reversed;
        }
        for deck in 1_u8..=4 {
            push_deck(controls, addresses, binding, deck)?;
        }
    }
    let platter_position = Binding {
        sensitivity: Some(jog.platter_position_sensitivity),
        acceleration: Some(jog.scratch_reaction),
        ..PLATTER_POSITION_BINDING
    };
    for deck in 1_u8..=4 {
        push_deck(controls, addresses, platter_position, deck)?;
    }
    Ok(())
}

fn push_fx_browsing(
    controls: &mut Vec<Value>,
    addresses: &mut BTreeSet<(u8, u8, u8)>,
) -> Result<()> {
    for binding in super::catalog::FX_BROWSE_BINDINGS {
        for deck in 1_u8..=4 {
            let target = binding.target.replace(DECK_MARKER, &deck.to_string());
            push_control(controls, addresses, *binding, &target, deck + 7)?;
        }
    }
    Ok(())
}

fn push_jump_restores(
    controls: &mut Vec<Value>,
    addresses: &mut BTreeSet<(u8, u8, u8)>,
) -> Result<()> {
    for index in 0..super::feedback::info::JUMP_INTERVALS.len() {
        let binding = super::catalog::jump_restore_binding(index)
            .context("jump restoration binding missing")?;
        push_deck_bindings(std::slice::from_ref(&binding), controls, addresses)?;
    }
    Ok(())
}

fn push_ext_outputs(
    outputs: &mut Vec<Value>,
    addresses: &mut BTreeSet<(u8, u8, u8)>,
) -> Result<()> {
    for deck in 1_u8..=4 {
        for (suffix, data) in [
            ("controlModeIsThru", EXT_THRU_CC),
            ("controlModeIsInternal", INTERNAL_MODE_CC),
        ] {
            push_output(
                outputs,
                addresses,
                OutputSpec {
                    key_path: &format!("turntable{deck}.{suffix}"),
                    channel: deck - 1,
                    data,
                    message_type: 3,
                    meter: false,
                },
            )?;
        }
    }
    Ok(())
}

fn push_fx_outputs(outputs: &mut Vec<Value>, addresses: &mut BTreeSet<(u8, u8, u8)>) -> Result<()> {
    for deck in 1_u8..=4 {
        for (index, suffix) in [
            "fx1.parameterValue",
            "fx2.parameterValue",
            "fx3.parameterValue",
            "fx1.wetDryValue",
            "fx2.wetDryValue",
            "fx3.wetDryValue",
        ]
        .iter()
        .enumerate()
        {
            push_output(
                outputs,
                addresses,
                OutputSpec {
                    key_path: &format!("turntable{deck}.{suffix}"),
                    channel: deck - 1,
                    data: 32 + u8::try_from(index)?,
                    message_type: 3,
                    meter: true,
                },
            )?;
        }
    }
    Ok(())
}

fn push_info_outputs(
    outputs: &mut Vec<Value>,
    output_addresses: &mut BTreeSet<(u8, u8, u8)>,
) -> Result<()> {
    for deck in 1_u8..=4 {
        for (index, (interval, _)) in super::feedback::info::JUMP_INTERVALS.iter().enumerate() {
            let key = format!("turntable{deck}.skipping{interval}BeatInterval");
            push_output(
                outputs,
                output_addresses,
                OutputSpec {
                    key_path: &key,
                    channel: deck - 1,
                    data: 80 + u8::try_from(index)?,
                    message_type: 3,
                    meter: false,
                },
            )?;
        }
        for (index, (interval, _)) in super::feedback::info::LOOP_INTERVALS.iter().enumerate() {
            let key = format!("turntable{deck}.autoLoop{interval}BeatIntervalActive");
            push_output(
                outputs,
                output_addresses,
                OutputSpec {
                    key_path: &key,
                    channel: deck - 1,
                    data: 90 + u8::try_from(index)?,
                    message_type: 3,
                    meter: false,
                },
            )?;
        }
        let key = format!("turntable{deck}.displayKeyIndex");
        push_output(
            outputs,
            output_addresses,
            OutputSpec {
                key_path: &key,
                channel: deck - 1,
                data: 112,
                message_type: 3,
                meter: true,
            },
        )?;
    }
    push_output(
        outputs,
        output_addresses,
        OutputSpec {
            key_path: "userDefaults.DJTempoSliderRange",
            channel: 6,
            data: 8,
            message_type: 3,
            meter: true,
        },
    )?;
    Ok(())
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
        Kind::Absolute | Kind::Relative | Kind::RotaryAbsolute => 3,
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
    let rotary_type = match binding.kind {
        Kind::Relative => Some("rotary-64"),
        Kind::RotaryAbsolute => Some("rotary-absolute"),
        Kind::Note | Kind::Absolute => None,
    };
    if let Some(rotary_type) = rotary_type {
        control.insert(
            String::from("controlType"),
            Value::String(rotary_type.to_owned()),
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
    if let Some(acceleration) = binding.acceleration {
        control.insert(
            String::from("rotaryAcceleration"),
            Value::Integer(i64::from(acceleration).into()),
        );
    }
    let has_feedback = binding.kind == Kind::Note
        && ((channel < 4
            && (binding.number == EQ_MODE_NOTE
                || matches!(binding.number, 0..=4 | 6 | 8 | 15..=19 | 32..=35 | 40..=63)))
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
    fn eq_mode_input_and_feedback_match_each_deck_address() -> anyhow::Result<()> {
        let controls = generated_controls()?;
        for deck in 1_u8..=4 {
            let key_path = format!("turntable{deck}.toggleUnmixerEQMode");
            let channel = u64::from(deck - 1);
            let control = controls
                .iter()
                .find(|control| {
                    field(control, "keyPath").and_then(Value::as_string) == Some(key_path.as_str())
                        && field(control, "midiChannel").and_then(Value::as_unsigned_integer)
                            == Some(channel)
                        && field(control, "midiMessageType").and_then(Value::as_unsigned_integer)
                            == Some(1)
                        && field(control, "midiData").and_then(Value::as_unsigned_integer)
                            == Some(u64::from(EQ_MODE_NOTE))
                })
                .context("EQ mode input missing")?;
            assert!(
                field(control, "output")
                    .and_then(Value::as_dictionary)
                    .is_some()
            );
        }
        Ok(())
    }

    #[test]
    fn ext_actions_use_only_their_normal_deck_note_addresses() -> anyhow::Result<()> {
        let controls = generated_controls()?;
        let expected = [(27_u64, "controlModeThru"), (28_u64, "controlModeInternal")];
        for deck in 1_u8..=4 {
            let channel = u64::from(deck - 1);
            for (data, action) in expected {
                let key = format!("turntable{deck}.{action}");
                let matches: Vec<_> = controls
                    .iter()
                    .filter(|control| {
                        field(control, "keyPath").and_then(Value::as_string) == Some(key.as_str())
                    })
                    .collect();
                assert_eq!(matches.len(), 1);
                let control = matches[0];
                assert_eq!(
                    field(control, "midiChannel").and_then(Value::as_unsigned_integer),
                    Some(channel)
                );
                assert_eq!(
                    field(control, "midiMessageType").and_then(Value::as_unsigned_integer),
                    Some(1)
                );
                assert_eq!(
                    field(control, "midiData").and_then(Value::as_unsigned_integer),
                    Some(data)
                );
            }
        }
        for control in &controls {
            let data = field(control, "midiData").and_then(Value::as_unsigned_integer);
            let channel = field(control, "midiChannel").and_then(Value::as_unsigned_integer);
            if field(control, "midiMessageType").and_then(Value::as_unsigned_integer) == Some(1)
                && matches!(data, Some(27 | 28))
            {
                assert!(matches!(channel, Some(0..=3)));
            }
        }
        Ok(())
    }

    #[test]
    fn mode_feedback_is_boolean_and_deck_addressed() -> anyhow::Result<()> {
        let outputs = generated_outputs()?;
        for deck in 1_u8..=4 {
            for (suffix, data) in [
                ("controlModeIsThru", EXT_THRU_CC),
                ("controlModeIsInternal", INTERNAL_MODE_CC),
            ] {
                let key = format!("turntable{deck}.{suffix}");
                let matches: Vec<_> = outputs
                    .iter()
                    .filter(|output| {
                        field(output, "keyPath").and_then(Value::as_string) == Some(key.as_str())
                    })
                    .collect();
                assert_eq!(matches.len(), 1);
                let output = matches[0];
                assert_eq!(
                    field(output, "midiChannel").and_then(Value::as_unsigned_integer),
                    Some(u64::from(deck - 1))
                );
                assert_eq!(
                    field(output, "midiMessageType").and_then(Value::as_unsigned_integer),
                    Some(3)
                );
                assert_eq!(
                    field(output, "midiData").and_then(Value::as_unsigned_integer),
                    Some(u64::from(data))
                );
                assert_eq!(
                    field(output, "midiMinValue").and_then(Value::as_unsigned_integer),
                    Some(0)
                );
                assert_eq!(
                    field(output, "midiMaxValue").and_then(Value::as_unsigned_integer),
                    Some(127)
                );
                assert!(field(output, "controlType").is_none());
            }
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
    fn platter_position_maps_absolute_rotary_to_scratch_on_every_deck() -> anyhow::Result<()> {
        // Given the generated mapping with physical jog and platter-position inputs.
        let controls = generated_controls()?;
        for deck in 1_u8..=4 {
            let key = format!("turntable{deck}.scratchingMove");
            let control = controls
                .iter()
                .find(|control| {
                    field(control, "keyPath").and_then(Value::as_string) == Some(key.as_str())
                        && field(control, "midiChannel").and_then(Value::as_unsigned_integer)
                            == Some(u64::from(deck - 1))
                        && field(control, "midiData").and_then(Value::as_unsigned_integer)
                            == Some(u64::from(PLATTER_POSITION_BINDING.number))
                })
                .context("absolute platter-position control missing")?;
            // When absolute angle values arrive, Djay receives them as Scratch CC.
            assert_eq!(
                field(control, "controlType").and_then(Value::as_string),
                Some("rotary-absolute")
            );
            assert_eq!(
                field(control, "rotarySensitivity").and_then(Value::as_real),
                Some(jog::DEFAULT.platter_position_sensitivity)
            );
            let scratching_mode = format!("turntable{deck}.scratchingMode");
            assert!(controls.iter().any(|control| {
                field(control, "keyPath").and_then(Value::as_string)
                    == Some(scratching_mode.as_str())
                    && field(control, "midiChannel").and_then(Value::as_unsigned_integer)
                        == Some(u64::from(deck - 1))
                    && field(control, "midiMessageType").and_then(Value::as_unsigned_integer)
                        == Some(1)
                    && field(control, "midiData").and_then(Value::as_unsigned_integer)
                        == Some(u64::from(super::super::catalog::SCRATCHING_MODE_NOTE))
            }));
        }
        Ok(())
    }

    #[test]
    fn numeric_info_outputs_use_the_declared_deck_and_global_addresses() -> anyhow::Result<()> {
        let outputs = generated_outputs()?;
        let mut addresses = BTreeSet::new();
        for output in &outputs {
            let channel = field(output, "midiChannel")
                .and_then(Value::as_unsigned_integer)
                .context("output channel")?;
            let data = field(output, "midiData")
                .and_then(Value::as_unsigned_integer)
                .context("output data")?;
            let kind = field(output, "midiMessageType")
                .and_then(Value::as_unsigned_integer)
                .context("output message type")?;
            assert!(addresses.insert((channel, kind, data)));
        }
        for deck in 1_u8..=4 {
            let channel = u64::from(deck - 1);
            for (offset, (interval, _)) in super::super::feedback::info::JUMP_INTERVALS
                .iter()
                .enumerate()
            {
                let key = format!("turntable{deck}.skipping{interval}BeatInterval");
                let data = 80 + u64::try_from(offset)?;
                assert!(outputs.iter().any(|output| {
                    field(output, "keyPath").and_then(Value::as_string) == Some(key.as_str())
                        && field(output, "midiChannel").and_then(Value::as_unsigned_integer)
                            == Some(channel)
                        && field(output, "midiMessageType").and_then(Value::as_unsigned_integer)
                            == Some(3)
                        && field(output, "midiData").and_then(Value::as_unsigned_integer)
                            == Some(data)
                }));
            }
            for (offset, (interval, _)) in super::super::feedback::info::LOOP_INTERVALS
                .iter()
                .enumerate()
            {
                let key = format!("turntable{deck}.autoLoop{interval}BeatIntervalActive");
                let data = 90 + u64::try_from(offset)?;
                assert!(outputs.iter().any(|output| {
                    field(output, "keyPath").and_then(Value::as_string) == Some(key.as_str())
                        && field(output, "midiChannel").and_then(Value::as_unsigned_integer)
                            == Some(channel)
                        && field(output, "midiMessageType").and_then(Value::as_unsigned_integer)
                            == Some(3)
                        && field(output, "midiData").and_then(Value::as_unsigned_integer)
                            == Some(data)
                }));
            }
            let key = format!("turntable{deck}.displayKeyIndex");
            assert!(outputs.iter().any(|output| {
                field(output, "keyPath").and_then(Value::as_string) == Some(key.as_str())
                    && field(output, "midiChannel").and_then(Value::as_unsigned_integer)
                        == Some(channel)
                    && field(output, "midiData").and_then(Value::as_unsigned_integer) == Some(112)
            }));
        }
        assert!(outputs.iter().any(|output| {
            field(output, "keyPath").and_then(Value::as_string)
                == Some("userDefaults.DJTempoSliderRange")
                && field(output, "midiChannel").and_then(Value::as_unsigned_integer) == Some(6)
                && field(output, "midiData").and_then(Value::as_unsigned_integer) == Some(8)
                && field(output, "controlType").and_then(Value::as_string) == Some("control")
        }));
        Ok(())
    }

    #[test]
    fn jog_overrides_change_only_native_speed_and_reaction_fields() -> anyhow::Result<()> {
        // Given defaults and a user configuration distinct from them.
        let defaults = generated_controls()?;
        let config = jog::Config {
            scratch_speed: 4.5,
            scratch_reaction: 60,
            platter_position_sensitivity: 23.4,
            pitch_bend_speed: 1.6,
            pitch_bend_reaction: 25,
            backspin_enabled: false,
            backspin_assist_enabled: true,
            backspin_assist_strength: 37,
            backspin_acceleration: 250_000.0,
            backspin_speed_increase: 2.0,
            ..jog::DEFAULT
        };
        // When the real generated mapping is serialized and read back.
        let mapping = generate_with_jog(&config)?;
        let mut bytes = Vec::new();
        mapping.to_writer_xml(&mut bytes)?;
        let parsed = Value::from_reader(Cursor::new(bytes))?;
        let controls = field(&parsed, "controls")
            .and_then(Value::as_array)
            .context("generated controls missing")?;
        assert_eq!(controls.len(), defaults.len());
        let mut changed = 0;
        // Then manual CC 5 scratch and absolute CC 8 platter sensitivity stay independent.
        for (control, baseline) in controls.iter().zip(&defaults) {
            let key = field(control, "keyPath")
                .and_then(Value::as_string)
                .context("control key missing")?;
            let data = field(control, "midiData")
                .and_then(Value::as_unsigned_integer)
                .context("control data missing")?;
            let settings = if key.ends_with(".scratchingMove") && data == 5 {
                Some((config.scratch_speed, config.scratch_reaction))
            } else if key.ends_with(".scratchingMove") && data == 8 {
                Some((config.platter_position_sensitivity, config.scratch_reaction))
            } else if key.ends_with(".pitchBendMove") {
                Some((config.pitch_bend_speed, config.pitch_bend_reaction))
            } else {
                None
            };
            if let Some((speed, reaction)) = settings {
                let actual = field(control, "rotarySensitivity")
                    .and_then(Value::as_real)
                    .context("jog speed missing")?;
                assert!((actual - speed).abs() < f64::EPSILON);
                assert_eq!(
                    field(control, "rotaryAcceleration").and_then(Value::as_unsigned_integer),
                    Some(u64::from(reaction))
                );
                let mut restored = control
                    .as_dictionary()
                    .context("control dictionary")?
                    .clone();
                for name in ["rotarySensitivity", "rotaryAcceleration"] {
                    restored.insert(
                        name.to_owned(),
                        field(baseline, name).context("default jog field")?.clone(),
                    );
                }
                assert_eq!(Value::Dictionary(restored), *baseline);
                changed += 1;
            } else {
                assert_eq!(control, baseline);
            }
        }
        assert_eq!(changed, 12);
        let base = generate()?;
        assert_eq!(field(&parsed, "outputs"), field(&base, "outputs"));
        assert_eq!(field(&parsed, "userInfo"), field(&base, "userInfo"));
        Ok(())
    }

    #[test]
    fn scratch_direction_flip_affects_touch_scratch_but_not_platter_position() -> anyhow::Result<()>
    {
        // Given a configuration that reverses only manual top-touch scratch.
        let config = jog::Config {
            scratch_direction: jog::ScratchDirection::Reversed,
            ..jog::DEFAULT
        };
        let controls = {
            let mapping = generate_with_jog(&config)?;
            let mut bytes = Vec::new();
            mapping.to_writer_xml(&mut bytes)?;
            let parsed = Value::from_reader(Cursor::new(bytes))?;
            field(&parsed, "controls")
                .and_then(Value::as_array)
                .context("generated controls missing")?
                .clone()
        };
        // When CC 5 and CC 8 target the same Scratch action.
        for (data, flipped) in [(5_u64, Some(true)), (8, None)] {
            let control = controls
                .iter()
                .find(|control| {
                    field(control, "keyPath")
                        .and_then(Value::as_string)
                        .is_some_and(|key| key.ends_with(".scratchingMove"))
                        && field(control, "midiData").and_then(Value::as_unsigned_integer)
                            == Some(data)
                })
                .context("Scratch input missing")?;
            // Then polarity is configurable for touch only, not platter angle.
            assert_eq!(
                field(control, "flipped").and_then(Value::as_boolean),
                flipped
            );
        }
        Ok(())
    }

    #[test]
    fn invalid_jog_parameters_cannot_generate_native_controls() {
        // Given invalid speed/reaction values at the export boundary.
        for config in [
            jog::Config {
                scratch_speed: 0.0,
                ..jog::DEFAULT
            },
            jog::Config {
                pitch_bend_speed: -1.0,
                ..jog::DEFAULT
            },
            jog::Config {
                scratch_speed: f64::NAN,
                ..jog::DEFAULT
            },
            jog::Config {
                pitch_bend_speed: f64::INFINITY,
                ..jog::DEFAULT
            },
            jog::Config {
                scratch_reaction: 151,
                ..jog::DEFAULT
            },
            jog::Config {
                pitch_bend_reaction: 255,
                ..jog::DEFAULT
            },
            jog::Config {
                backspin_acceleration: 0.0,
                ..jog::DEFAULT
            },
            jog::Config {
                backspin_acceleration: f64::NAN,
                ..jog::DEFAULT
            },
            jog::Config {
                backspin_speed_increase: -1.0,
                ..jog::DEFAULT
            },
            jog::Config {
                backspin_speed_increase: f64::INFINITY,
                ..jog::DEFAULT
            },
        ] {
            // When export is requested, then no mapping is returned.
            assert!(generate_with_jog(&config).is_err());
        }
    }

    #[test]
    fn encoder_actions_match_runtime_addresses_on_every_deck() -> anyhow::Result<()> {
        // Given the serialized native mapping, not a separate mock catalog.
        let controls = generated_controls()?;
        // When each normal, selection, and fixed one-beat route is resolved.
        for deck in 0_u8..4 {
            for (channel, kind, data, action) in [
                (deck, 3, 2, "skipRotary"),
                (deck, 3, 7, "skipDurationRotary"),
                (deck + 8, 1, 5, "skipForward1Beat"),
                (deck + 8, 1, 14, "skipBackward1Beat"),
                (deck, 1, 6, "autoLoopOnOff"),
                (deck + 8, 1, 6, "reloop"),
                (deck, 1, 3, "reverseHold"),
            ] {
                let key = format!("turntable{}.{action}", deck + 1);
                // Then exactly that endpoint/channel/type/data address has that action.
                assert!(controls.iter().any(|control| {
                    field(control, "keyPath").and_then(Value::as_string) == Some(key.as_str())
                        && field(control, "midiChannel").and_then(Value::as_unsigned_integer)
                            == Some(u64::from(channel))
                        && field(control, "midiMessageType").and_then(Value::as_unsigned_integer)
                            == Some(kind)
                        && field(control, "midiData").and_then(Value::as_unsigned_integer)
                            == Some(data)
                }));
            }
        }
        assert!(!controls.iter().any(|control| {
            field(control, "keyPath")
                .and_then(Value::as_string)
                .is_some_and(|key| {
                    [".loopIn", ".loopOut", ".loopInOut"]
                        .iter()
                        .any(|action| key.ends_with(action))
                })
        }));
        Ok(())
    }

    #[test]
    fn browse_polarity_is_flipped_once_on_normal_layer() -> anyhow::Result<()> {
        // Given the real generated mapping for ordinary browsing.
        let controls = generated_controls()?;
        // When every deck and modifier address is examined.
        for channel in [0_u64, 1, 2, 3] {
            let control = controls
                .iter()
                .find(|control| {
                    field(control, "midiChannel").and_then(Value::as_unsigned_integer)
                        == Some(channel)
                        && field(control, "midiMessageType").and_then(Value::as_unsigned_integer)
                            == Some(3)
                        && field(control, "midiData").and_then(Value::as_unsigned_integer)
                            == Some(3)
                })
                .context("Browse mapping missing")?;
            // Then Djay performs the single inversion; the wire remains rotary-64.
            assert_eq!(
                field(control, "flipped").and_then(Value::as_boolean),
                Some(true)
            );
            assert_eq!(
                field(control, "controlType").and_then(Value::as_string),
                Some("rotary-64")
            );
            assert_eq!(
                field(control, "keyPath").and_then(Value::as_string),
                Some("musicLibrary.libraryRotary")
            );
        }
        // Shift uses directional button pulses, never section-navigation CCs.
        assert!(!controls.iter().any(|control| {
            field(control, "keyPath").and_then(Value::as_string)
                == Some("musicLibrary.sectionRotary")
        }));
        Ok(())
    }

    #[test]
    fn cue_layers_and_library_note_swap_resolve_on_every_deck() -> anyhow::Result<()> {
        // Given the real generated mapping.
        let controls = generated_controls()?;
        for deck in 0_u64..4 {
            // When the requested note addresses are resolved.
            for (channel, number, key) in [
                (
                    deck,
                    1,
                    format!(
                        "turntable{}.cuePositionOrJumpConsideringPlayState1",
                        deck + 1
                    ),
                ),
                (deck + 8, 1, format!("turntable{}.gotoZero", deck + 1)),
                (
                    deck,
                    10,
                    String::from("musicLibrary.markUnmarkSelectedSongs"),
                ),
                (deck, 11, String::from("musicLibrary.toggleLibrarySource")),
                (deck + 8, 25, String::from("musicLibrary.nextSource")),
                (deck + 8, 26, String::from("musicLibrary.previousSource")),
            ] {
                let control = controls
                    .iter()
                    .find(|control| {
                        field(control, "midiChannel").and_then(Value::as_unsigned_integer)
                            == Some(channel)
                            && field(control, "midiMessageType")
                                .and_then(Value::as_unsigned_integer)
                                == Some(1)
                            && field(control, "midiData").and_then(Value::as_unsigned_integer)
                                == Some(number)
                    })
                    .context("requested note mapping missing")?;
                // Then each note targets the requested native action, preserving normal Cue.
                assert_eq!(
                    field(control, "keyPath").and_then(Value::as_string),
                    Some(key.as_str())
                );
            }
        }
        Ok(())
    }

    #[test]
    fn sync_layers_resolve_to_deck_sync_and_global_range_cycle() -> anyhow::Result<()> {
        // Given the serialized generated mapping.
        let controls = generated_controls()?;
        // When both Sync layers are resolved on all four decks.
        for deck in 0_u64..4 {
            for (channel, key) in [
                (deck, format!("turntable{}.bpmSync", deck + 1)),
                (deck + 8, String::from("application.tempoSliderRangeNext")),
            ] {
                let control = controls
                    .iter()
                    .find(|control| {
                        field(control, "midiChannel").and_then(Value::as_unsigned_integer)
                            == Some(channel)
                            && field(control, "midiMessageType")
                                .and_then(Value::as_unsigned_integer)
                                == Some(1)
                            && field(control, "midiData").and_then(Value::as_unsigned_integer)
                                == Some(2)
                    })
                    .context("Sync mapping missing")?;
                // Then normal Sync remains deck-specific and Shift cycles the shared range.
                assert_eq!(
                    field(control, "keyPath").and_then(Value::as_string),
                    Some(key.as_str())
                );
            }
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
            assert!(message_type != 1 || !(0..=3).contains(&channel) || !(36..=39).contains(&data));
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
    fn shifted_fx_browsing_maps_relative_selectors_without_pickup() -> anyhow::Result<()> {
        // Given the serialized mapping that Djay consumes.
        let controls = generated_controls()?;
        // When resolving each emitted browse address for all target decks.
        for deck in 1_u64..=4 {
            for slot in 1_u64..=3 {
                let matches: Vec<_> = controls
                    .iter()
                    .filter(|control| {
                        field(control, "midiChannel").and_then(Value::as_unsigned_integer)
                            == Some(deck + 7)
                            && field(control, "midiMessageType")
                                .and_then(Value::as_unsigned_integer)
                                == Some(3)
                            && field(control, "midiData").and_then(Value::as_unsigned_integer)
                                == Some(31 + slot)
                    })
                    .collect();
                // Then precisely one native stepped selector owns the rotary-64 address.
                assert_eq!(matches.len(), 1);
                let control = matches[0];
                assert_eq!(
                    field(control, "keyPath").and_then(Value::as_string),
                    Some(format!("turntable{deck}.fx{slot}SelectRotary").as_str())
                );
                assert_eq!(
                    field(control, "controlType").and_then(Value::as_string),
                    Some("rotary-64")
                );
                assert!(field(control, "pickupMode").is_none());
                assert!(field(control, "output").is_none());
            }
        }
        Ok(())
    }

    #[test]
    fn fx_absolute_feedback_has_one_canonical_address_with_pickup_on_both_strips()
    -> anyhow::Result<()> {
        // Given the serialized mapping consumed by Djay.
        let controls = generated_controls()?;
        let generated = generate()?;
        let outputs = field(&generated, "outputs")
            .and_then(Value::as_array)
            .context("outputs")?;
        // When each FX parameter and wet/dry alias is resolved on every deck.
        for channel in 0_u64..4 {
            for data in 32_u64..38 {
                let output = outputs
                    .iter()
                    .find(|output| {
                        field(output, "midiChannel").and_then(Value::as_unsigned_integer)
                            == Some(channel)
                            && field(output, "midiData").and_then(Value::as_unsigned_integer)
                                == Some(data)
                    })
                    .context("canonical FX model value output missing")?;
                let index = data - 32;
                let key = format!(
                    "turntable{}.fx{}.{}",
                    channel + 1,
                    index % 3 + 1,
                    if index < 3 {
                        "parameterValue"
                    } else {
                        "wetDryValue"
                    }
                );
                assert_eq!(
                    field(output, "keyPath").and_then(Value::as_string),
                    Some(key.as_str())
                );
                assert_eq!(
                    field(output, "controlType").and_then(Value::as_string),
                    Some("control")
                );
                for address in [data, data + 8] {
                    let control = controls
                        .iter()
                        .find(|control| {
                            field(control, "midiChannel").and_then(Value::as_unsigned_integer)
                                == Some(channel)
                                && field(control, "midiMessageType")
                                    .and_then(Value::as_unsigned_integer)
                                    == Some(3)
                                && field(control, "midiData").and_then(Value::as_unsigned_integer)
                                    == Some(address)
                        })
                        .context("FX absolute mapping missing")?;
                    // Then both aliases use native pickup but only canonical CCs return values.
                    assert_eq!(
                        field(control, "pickupMode").and_then(Value::as_boolean),
                        Some(true)
                    );
                    assert!(field(control, "output").is_none());
                }
            }
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
