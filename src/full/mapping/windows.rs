//! Windows output declarations use explicit addresses on a disjoint shared bus.

use anyhow::Context as _;
use plist::Value;

use crate::{midi_layout, midi_transport::Backend};

pub fn generate(jog: &super::jog::Config, backend: Backend) -> anyhow::Result<Value> {
    let Some(port_name) = backend.port_name() else {
        return super::generate_with_jog(jog);
    };
    let mut mapping = super::generate_controls(jog, false)?;
    let root = mapping.as_dictionary_mut().context("mapping root")?;
    root.insert(
        "endpointName".to_owned(),
        Value::String(port_name.to_owned()),
    );
    for item in root
        .get_mut("controls")
        .and_then(Value::as_array_mut)
        .context("controls")?
    {
        let control = item.as_dictionary_mut().context("control")?;
        let channel = control
            .get("midiChannel")
            .and_then(Value::as_unsigned_integer)
            .context("channel")?;
        let data = control.get("midiData").context("data")?.clone();
        let family = control.get("midiMessageType").context("family")?.clone();
        if let Some(output) = control.get_mut("output").and_then(Value::as_dictionary_mut) {
            let channel = midi_layout::feedback_channel(u8::try_from(channel)?)
                .context("feedback channel")?;
            output.insert("midiChannel".to_owned(), Value::Integer(channel.into()));
            output.insert("midiData".to_owned(), data);
            output.insert("midiMessageType".to_owned(), family);
        }
    }
    for item in root
        .get_mut("outputs")
        .and_then(Value::as_array_mut)
        .context("outputs")?
    {
        let output = item.as_dictionary_mut().context("output")?;
        for deck in 1..=4 {
            if output.get("keyPath").and_then(Value::as_string)
                == Some(format!("turntable{deck}.playing").as_str())
            {
                output.insert(
                    "keyPath".to_owned(),
                    Value::String(format!("turntable{deck}.isPlaying")),
                );
            }
        }
        let channel = output
            .get("midiChannel")
            .and_then(Value::as_unsigned_integer)
            .context("channel")?;
        let channel =
            midi_layout::feedback_channel(u8::try_from(channel)?).context("feedback channel")?;
        output.insert("midiChannel".to_owned(), Value::Integer(channel.into()));
    }
    Ok(mapping)
}

#[cfg(test)]
#[path = "windows_tests.rs"]
mod tests;
