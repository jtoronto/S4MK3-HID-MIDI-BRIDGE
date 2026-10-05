//! Minimal fixed-deck profile. No deck switching, feedback, or jog translation.

use anyhow::Context as _;
use encdr::Event;
use midir::os::unix::VirtualOutput as _;
use midir::{MidiOutput, MidiOutputConnection};
use num_traits::ToPrimitive as _;
use tracing::{info, warn};

pub const PORT_NAME: &str = "S4 MK3 MIDI";

pub struct MidiBridge {
    connection: MidiOutputConnection,
}

impl MidiBridge {
    pub fn new() -> anyhow::Result<Self> {
        let connection = MidiOutput::new("S4 MK3 Bridge")?
            .create_virtual(PORT_NAME)
            .map_err(|error| anyhow::anyhow!("create virtual MIDI source: {error}"))?;
        info!(
            port = PORT_NAME,
            "MIDI_READY: fixed decks 1/2; mixer channels 1-4"
        );
        Ok(Self { connection })
    }

    pub fn send(&mut self, event: &Event) -> anyhow::Result<bool> {
        let message = match event {
            Event::Button { name, pressed, .. } => button_message(name, *pressed),
            Event::Slider { name, value, .. } => slider_message(name, *value)?,
            Event::DeviceConnected { .. }
            | Event::DeviceDisconnected { .. }
            | Event::Encoder { .. }
            | Event::EncoderFine { .. }
            | Event::Touch { .. }
            | Event::Grid { .. } => None,
        };
        let Some(message) = message else {
            return Ok(false);
        };
        self.connection
            .send(&message)
            .context("send virtual MIDI message")?;
        info!(bytes = %format_args!("{message:02x?}"), "MIDI_SENT");
        Ok(true)
    }
}

impl Drop for MidiBridge {
    fn drop(&mut self) {
        // Release any mapped buttons still held when the bounded bridge stops.
        for status in [0x80, 0x81] {
            for note in [0, 1] {
                if let Err(error) = self.connection.send(&[status, note, 0]) {
                    warn!(%error, "could not release MIDI button during shutdown");
                }
            }
        }
    }
}

fn button_message(name: &str, pressed: bool) -> Option<[u8; 3]> {
    let (channel, note) = match name {
        "left_play" => (0, 0),
        "left_cue" => (0, 1),
        "right_play" => (1, 0),
        "right_cue" => (1, 1),
        _ => return None,
    };
    Some([
        if pressed { 0x90 } else { 0x80 } | channel,
        note,
        if pressed { 127 } else { 0 },
    ])
}

fn slider_message(name: &str, value: f32) -> anyhow::Result<Option<[u8; 3]>> {
    let cc = match name {
        "ch1_fader" => 16,
        "ch2_fader" => 17,
        "ch3_fader" => 18,
        "ch4_fader" => 19,
        "crossfader" => 20,
        _ => return Ok(None),
    };
    // MIDI is an output boundary: never emit non-finite or out-of-range data.
    anyhow::ensure!(
        (0.0..=1.0).contains(&value),
        "invalid normalized {name}: {value}"
    );
    let value = (value * 127.0)
        .round()
        .to_u8()
        .context("quantize normalized fader to MIDI")?;
    Ok(Some([0xb0, cc, value]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_buttons_emit_distinct_notes_and_releases() {
        // Given the fixed two-deck profile.
        for (name, status, note) in [
            ("left_play", 0x90, 0),
            ("left_cue", 0x90, 1),
            ("right_play", 0x91, 0),
            ("right_cue", 0x91, 1),
        ] {
            // When a button is translated in each physical state.
            let messages = [button_message(name, true), button_message(name, false)];
            // Then press and release address the same distinct deck/note.
            assert_eq!(
                messages,
                [Some([status, note, 127]), Some([status & 0x8f, note, 0])]
            );
        }
    }

    #[test]
    fn faders_have_distinct_ccs_and_reach_both_endpoints() -> anyhow::Result<()> {
        // Given all mapped faders and representative positions.
        for (name, cc) in [
            ("ch1_fader", 16),
            ("ch2_fader", 17),
            ("ch3_fader", 18),
            ("ch4_fader", 19),
            ("crossfader", 20),
        ] {
            for (position, expected) in [(0.0, 0), (0.5, 64), (1.0, 127)] {
                // When a position is translated.
                let message = slider_message(name, position)?;
                // Then its CC and quantized absolute value are correct.
                assert_eq!(message, Some([0xb0, cc, expected]));
            }
        }
        Ok(())
    }

    #[test]
    fn unrelated_controls_do_not_enter_the_minimal_profile() -> anyhow::Result<()> {
        // Given controls intentionally outside this stage.
        // When they are translated.
        let messages = [
            button_message("left_sync", true),
            slider_message("left_tempo_fader", 0.5)?,
        ];
        // Then neither produces MIDI.
        assert_eq!(messages, [None, None]);
        Ok(())
    }

    #[test]
    fn invalid_fader_values_cannot_produce_midi_data() {
        for value in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            // Given an invalid position at the MIDI boundary.
            // When the position is translated.
            let result = slider_message("crossfader", value);
            // Then no invalid MIDI data escapes.
            assert!(result.is_err());
        }
    }

    #[test]
    fn virtual_source_delivers_translated_message_to_a_real_receiver() -> anyhow::Result<()> {
        // Given a real CoreMIDI source and receiver subscribed before sending.
        let name = format!("S4 MIDI integration {}", std::process::id());
        let mut output = MidiOutput::new("S4 MIDI test output")?
            .create_virtual(&name)
            .map_err(|error| anyhow::anyhow!("create test source: {error}"))?;
        let input = midir::MidiInput::new("S4 MIDI test receiver")?;
        let mut selected = None;
        for port in input.ports() {
            if input.port_name(&port)? == name {
                selected = Some(port);
                break;
            }
        }
        let port = selected.context("test source absent from CoreMIDI")?;
        let (sender, receiver) = crossbeam_channel::unbounded();
        let _connection = input
            .connect(
                &port,
                "S4 MIDI test connection",
                move |_, bytes, ()| {
                    if let Err(error) = sender.send(bytes.to_vec()) {
                        warn!(%error, "test receiver closed");
                    }
                },
                (),
            )
            .map_err(|error| anyhow::anyhow!("connect test receiver: {error}"))?;
        let message = button_message("right_cue", true).context("cue mapping missing")?;

        // When the translated message is sent across CoreMIDI.
        output.send(&message)?;

        // Then an independent subscribed receiver sees the exact bytes.
        let packet = receiver.recv_timeout(std::time::Duration::from_secs(2))?;
        assert_eq!(packet, message);
        Ok(())
    }
}
