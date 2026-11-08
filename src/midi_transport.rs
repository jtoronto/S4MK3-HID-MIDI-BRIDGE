//! Concrete midir connections; platform selection never opens a fallback backend.

use anyhow::{Context as _, ensure};
use clap::ValueEnum;
use midir::{MidiIO, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use serde::Serialize;

pub const LOOPBE_PORT: &str = "LoopBe Internal MIDI";

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    CoreMidi,
    Loopbe,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::CoreMidi => "core-midi",
            Self::Loopbe => "loopbe",
        })
    }
}

impl Default for Backend {
    fn default() -> Self {
        if cfg!(windows) {
            Self::Loopbe
        } else {
            Self::CoreMidi
        }
    }
}

impl Backend {
    pub fn validate_runtime(self) -> anyhow::Result<()> {
        ensure!(
            matches!(self, Self::CoreMidi) && cfg!(target_os = "macos")
                || matches!(self, Self::Loopbe) && cfg!(windows),
            "MIDI backend {self:?} is unavailable on this platform"
        );
        Ok(())
    }
}

fn exact_port<T: MidiIO>(io: &T, direction: &str) -> anyhow::Result<T::Port> {
    let ports = io.ports();
    let identities = ports
        .iter()
        .map(|port| io.port_name(port))
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("enumerate {direction} MIDI port names"))?;
    let index = exact_index(&identities, direction)?;
    ports
        .into_iter()
        .nth(index)
        .context("selected MIDI port disappeared")
}

fn exact_index(names: &[String], direction: &str) -> anyhow::Result<usize> {
    let matches: Vec<_> = names
        .iter()
        .enumerate()
        .filter(|(_, name)| name.as_str() == LOOPBE_PORT)
        .map(|(index, _)| index)
        .collect();
    ensure!(
        matches.len() == 1,
        "expected exactly one {direction} named {LOOPBE_PORT:?}; matches={matches:?}, candidates={names:?}"
    );
    Ok(matches[0])
}

pub struct Output {
    connection: MidiOutputConnection,
    capture: crate::midi_capture::Handle,
}

impl Output {
    pub fn send(&mut self, bytes: &[u8]) -> Result<(), midir::SendError> {
        send_captured(&self.capture, bytes, |bytes| self.connection.send(bytes))
    }
}

fn send_captured<E>(
    capture: &crate::midi_capture::Handle,
    bytes: &[u8],
    send: impl FnOnce(&[u8]) -> Result<(), E>,
) -> Result<(), E> {
    capture.output(bytes);
    send(bytes)
}

pub fn output(
    backend: Backend,
    name: &str,
    capture: crate::midi_capture::Handle,
) -> anyhow::Result<Output> {
    backend.validate_runtime()?;
    let output = MidiOutput::new("S4 MK3 bridge").context("create MIDI output client")?;
    let connection = match backend {
        Backend::CoreMidi => {
            #[cfg(target_os = "macos")]
            {
                use midir::os::unix::VirtualOutput as _;
                output
                    .create_virtual(name)
                    .map_err(|error| anyhow::anyhow!("create virtual MIDI source: {error}"))
            }
            #[cfg(not(target_os = "macos"))]
            anyhow::bail!("CoreMIDI unavailable: {name}")
        }
        Backend::Loopbe => {
            let port = exact_port(&output, "output")?;
            output
                .connect(&port, name)
                .map_err(|error| anyhow::anyhow!("connect LoopBe MIDI output: {error}"))
        }
    }?;
    Ok(Output {
        connection,
        capture,
    })
}

pub fn input<T: Send + 'static>(
    backend: Backend,
    name: &str,
    callback: impl FnMut(u64, &[u8], &mut T) + Send + 'static,
    data: T,
    capture: crate::midi_capture::Handle,
) -> anyhow::Result<MidiInputConnection<T>> {
    backend.validate_runtime()?;
    // Preserve midir's input policy. In the locked 0.11.0 both backends default
    // to Ignore::None; enabling capture must not change callback admission.
    let input = MidiInput::new("S4 MK3 feedback").context("create MIDI input client")?;
    let callback = captured_callback(capture, callback);
    match backend {
        Backend::CoreMidi => {
            #[cfg(target_os = "macos")]
            {
                use midir::os::unix::VirtualInput as _;
                input
                    .create_virtual(name, callback, data)
                    .map_err(|error| anyhow::anyhow!("create virtual MIDI destination: {error}"))
            }
            #[cfg(not(target_os = "macos"))]
            anyhow::bail!("CoreMIDI unavailable: {name}")
        }
        Backend::Loopbe => {
            let port = exact_port(&input, "input")?;
            input
                .connect(&port, name, callback, data)
                .map_err(|error| anyhow::anyhow!("connect LoopBe MIDI input: {error}"))
        }
    }
}

fn captured_callback<T>(
    capture: crate::midi_capture::Handle,
    mut callback: impl FnMut(u64, &[u8], &mut T),
) -> impl FnMut(u64, &[u8], &mut T) {
    move |timestamp, bytes: &[u8], data: &mut T| {
        capture.input(timestamp, bytes);
        callback(timestamp, bytes, data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn system_message_packets(segmented: bool) -> Vec<&'static [u8]> {
        let mut packets: Vec<&[u8]> = vec![&[0x90, 40, 1], &[0xf8], &[0xfe], &[41]];
        if segmented {
            // WinMM may deliver a long message in separate callback chunks.
            packets.extend([&[0xf0, 0x7d, 1][..], &[2, 3], &[0xf7]]);
        } else {
            // CoreMIDI reassembles its segmented SysEx before invoking us.
            packets.push(&[0xf0, 0x7d, 1, 2, 3, 0xf7]);
        }
        packets.extend([
            &[42, 2][..],
            &[0x90, 43],
            &[0xf8, 0xfe, 3],
            &[0xf1, 7],
            &[44, 4],
            &[0x90, 45, 5],
        ]);
        packets
    }

    #[test]
    fn capture_preserves_default_callback_stream_and_decoder_sysex_boundaries() -> anyhow::Result<()>
    {
        use crate::full::feedback::{Decoder, Message};

        // Given each backend's SysEx callback framing and the real decoder.
        for segmented in [false, true] {
            let packets = system_message_packets(segmented);
            for enabled in [false, true] {
                let path = std::env::temp_dir().join(format!(
                    "s4-midi-policy-{}-{segmented}.jsonl",
                    std::process::id()
                ));
                let mut capture = enabled
                    .then(|| crate::midi_capture::Capture::start(&path))
                    .transpose()?;
                let handle = capture.as_ref().map_or_else(
                    crate::midi_capture::Handle::default,
                    crate::midi_capture::Capture::handle,
                );
                let mut received = Vec::new();
                let mut state = (Decoder::default(), Vec::new());
                // When the adapter receives every packet with capture off/on.
                {
                    let mut callback = captured_callback(
                        handle,
                        |timestamp, bytes, state: &mut (Decoder, Vec<Message>)| {
                            received.push((timestamp, bytes.to_vec()));
                            state.0.feed(bytes, |message| state.1.push(message));
                        },
                    );
                    for bytes in &packets {
                        callback(900, bytes, &mut state);
                    }
                }
                // Then no classes disappear and system-common/SysEx clear
                // partial running status, while realtime leaves it untouched.
                assert_eq!(
                    received,
                    packets
                        .iter()
                        .map(|bytes| (900, bytes.to_vec()))
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    state.1,
                    [(40, 1), (43, 3), (45, 5)].map(|(number, value)| Message {
                        status: 0x90,
                        number,
                        value,
                    })
                );
                if let Some(capture) = &mut capture {
                    capture.finish()?;
                    let result = (|| -> anyhow::Result<()> {
                        let rows = std::fs::read_to_string(&path)?
                            .lines()
                            .map(serde_json::from_str::<serde_json::Value>)
                            .collect::<Result<Vec<_>, _>>()?;
                        assert_eq!(rows.len(), packets.len() + 2);
                        for (row, raw) in rows[1..=packets.len()].iter().zip(&packets) {
                            assert_eq!(row["bytes"], serde_json::json!(raw));
                            assert_eq!(row["callback_us"], 900);
                        }
                        Ok(())
                    })();
                    std::fs::remove_file(path)?;
                    result?;
                }
            }
        }
        Ok(())
    }

    #[test]
    fn capture_precedes_input_filtering_and_failed_output_send() -> anyhow::Result<()> {
        // Given the actual transport adapters and real file writers, without MIDI devices.
        for incoming in [true, false] {
            let path = std::env::temp_dir().join(format!(
                "s4-midi-transport-{}-{incoming}.jsonl",
                std::process::id()
            ));
            let mut capture = crate::midi_capture::Capture::start(&path)?;
            let handle = capture.handle();
            let bytes = [0xf8, 0x90, 0, 127, 0xf0, 1, 0xf7];
            let mut observed = Vec::new();
            let mut boundary = |received: &[u8]| -> anyhow::Result<()> {
                // Closing here proves capture happened before the real consumer/send.
                capture.finish()?;
                observed = std::fs::read(&path)?;
                assert_eq!(received, bytes);
                Ok(())
            };
            // When raw input reaches a filtering callback, or a send fails.
            if incoming {
                let mut callback_result = Ok(());
                {
                    let mut callback = captured_callback(
                        handle,
                        |_, raw, decoder: &mut crate::full::feedback::Decoder| {
                            callback_result = boundary(raw);
                            decoder.feed(raw, |_| {});
                        },
                    );
                    callback(123, &bytes, &mut crate::full::feedback::Decoder::default());
                }
                callback_result?;
            } else {
                let result = send_captured(&handle, &bytes, |raw| {
                    boundary(raw)?;
                    Err::<(), _>(anyhow::anyhow!("injected send failure"))
                });
                assert_eq!(
                    result.err().context("send must fail")?.to_string(),
                    "injected send failure"
                );
            }
            // Then the evidence already contained exactly the unchanged raw bytes.
            let result = (|| -> anyhow::Result<()> {
                let rows = std::str::from_utf8(&observed)?
                    .lines()
                    .map(serde_json::from_str::<serde_json::Value>)
                    .collect::<Result<Vec<_>, _>>()?;
                assert_eq!(rows.len(), 3);
                assert_eq!(rows[1]["bytes"], serde_json::json!(bytes));
                assert_eq!(
                    rows[1]["kind"],
                    if incoming { "input" } else { "output_attempt" }
                );
                assert_eq!(
                    rows[1]["callback_us"],
                    if incoming {
                        serde_json::json!(123)
                    } else {
                        serde_json::Value::Null
                    }
                );
                Ok(())
            })();
            std::fs::remove_file(path)?;
            result?;
        }
        Ok(())
    }

    #[test]
    fn port_selection_requires_one_exact_name_per_direction() -> anyhow::Result<()> {
        let names = ["Other", LOOPBE_PORT, "LoopBe Internal MIDI 2"].map(str::to_owned);
        assert_eq!(exact_index(&names, "input")?, 1);
        for names in [
            vec![],
            vec!["loopbe internal midi".to_owned()],
            vec!["LoopBe Internal MIDI 2".to_owned()],
            vec![LOOPBE_PORT.to_owned(), LOOPBE_PORT.to_owned()],
        ] {
            for direction in ["input", "output"] {
                let error = exact_index(&names, direction)
                    .err()
                    .context("ambiguous or absent port must fail")?
                    .to_string();
                assert!(error.contains(direction));
                assert!(error.contains("candidates="));
            }
        }
        Ok(())
    }
}
