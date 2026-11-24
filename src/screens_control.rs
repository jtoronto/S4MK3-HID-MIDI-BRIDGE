//! Managed stdin belongs to a separate blocking reader, not the HID service.

use std::io::{self, BufRead as _};

use s4mk3_hid_midi_bridge::screens::{Control, ControlAction};

fn parse(line: &str) -> Option<(u64, ControlAction)> {
    let mut parts = line.split(' ');
    let ("S4_SCREEN_CONTROL", Some(id), Some(action), None) =
        (parts.next()?, parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    if id.is_empty() || !id.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let action = match action {
        "ON" => ControlAction::On,
        "OFF" => ControlAction::Off,
        "RETRY" => ControlAction::Retry,
        _ => return None,
    };
    Some((id.parse().ok()?, action))
}

pub fn parse_stop(line: &str) -> Option<u64> {
    let mut parts = line.split(' ');
    let ("S4_BRIDGE_CONTROL", Some(id), Some("STOP"), None) =
        (parts.next()?, parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    if id.is_empty() || !id.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    id.parse().ok()
}

fn parse_marker(line: &str) -> Option<(u64, &str)> {
    let rest = line.strip_prefix("S4_MIDI_CAPTURE_MARK ")?;
    let (id, label) = rest.split_once(' ')?;
    if id.is_empty() || !id.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some((id.parse().ok()?, label))
}

fn parse_ni_jog_tension(line: &str) -> Option<u8> {
    let value = line.strip_prefix("ni_jog_tension ")?;
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

fn serve(
    reader: impl io::Read,
    control: Option<&Control>,
    ni_tension: impl Fn(u8) -> anyhow::Result<()>,
    stop: impl Fn(Option<u64>) -> anyhow::Result<()>,
    capture: &crate::midi_capture::Handle,
) -> anyhow::Result<()> {
    for line in io::BufReader::new(reader).lines() {
        let line = line?;
        if let Some(id) = parse_stop(&line) {
            stop(Some(id))?;
        } else if let Some((id, label)) = parse_marker(&line) {
            capture.marker(label).emit(id)?;
        } else if let Some(value) = parse_ni_jog_tension(&line) {
            if let Err(error) = ni_tension(value) {
                tracing::warn!(%error, "NI jog tension command rejected");
            }
        } else if let (Some(control), Some((id, action))) = (control, parse(&line)) {
            control.request(id, action);
        } else {
            tracing::warn!("invalid S4 screen control command");
        }
    }
    // Closing the app's stdin follows the same stop path as Ctrl-C, including
    // motor neutral and MIDI release. An idle stdin read may block until process
    // exit, so this reader is detached rather than joined during normal Stop.
    stop(None)
}

pub fn start(
    control: Option<Control>,
    ni_control: Option<Control>,
    stop: crate::lifecycle::Stop,
    capture: crate::midi_capture::Handle,
) -> anyhow::Result<()> {
    std::thread::Builder::new()
        .name("s4-screen-stdin".to_owned())
        .spawn(move || {
            if let Err(error) = serve(
                io::stdin(),
                control.as_ref(),
                |value| {
                    #[cfg(windows)]
                    if let Some(control) = &ni_control {
                        return control.set_ni_jog_tension(value);
                    }
                    let _ = (&ni_control, value);
                    anyhow::bail!("managed NI decks are not enabled")
                },
                |id| stop.request(id),
                &capture,
            ) {
                tracing::warn!(%error, "managed stdin failed; stopping bridge");
                if let Err(error) = stop.request(None) {
                    tracing::debug!(%error, "managed stdin stop already requested");
                }
            }
        })?;
    Ok(())
}

pub fn bridge(
    args: &crate::Args,
    session: &mut crate::lifecycle::Session,
    jog: crate::full::jog::Config,
) -> anyhow::Result<crate::full::FullBridge> {
    crate::full::FullBridge::new(
        session
            .screens
            .take()
            .ok_or_else(|| anyhow::anyhow!("full screen output missing"))?,
        jog,
        args.midi_backend,
        args.capture_handle.clone(),
        session.feedback_input.take(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use s4mk3_hid_midi_bridge::screens::Worker;

    #[test]
    fn managed_marker_records_before_stop_without_taking_bridge_ownership() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(format!("s4-midi-stdin-{}.jsonl", std::process::id()));
        let mut capture = crate::midi_capture::Capture::start(&path)?;
        let (tx, rx) = crossbeam_channel::bounded(2);
        serve(
            b"S4_MIDI_CAPTURE_MARK 41 before_play\nS4_BRIDGE_CONTROL 42 STOP\n".as_slice(),
            None,
            |_| anyhow::bail!("NI decks not enabled"),
            |id| Ok(tx.try_send(id)?),
            &capture.handle(),
        )?;
        capture.finish()?;
        let result = (|| -> anyhow::Result<()> {
            assert_eq!(rx.try_iter().collect::<Vec<_>>(), [Some(42), None]);
            let rows = std::fs::read_to_string(&path)?
                .lines()
                .map(serde_json::from_str::<serde_json::Value>)
                .collect::<Result<Vec<_>, _>>()?;
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[1]["marker"], "before_play");
            assert_eq!(rows[2]["counts"]["written"], 1);
            Ok(())
        })();
        std::fs::remove_file(path)?;
        result
    }

    #[test]
    fn marker_command_retains_request_identity_and_rejects_malformed_ids() {
        assert_eq!(
            parse_marker("S4_MIDI_CAPTURE_MARK 18446744073709551615 before_play"),
            Some((u64::MAX, "before_play"))
        );
        for invalid in [
            "S4_MIDI_CAPTURE_MARK +1 before_play",
            "S4_MIDI_CAPTURE_MARK -1 before_play",
            "S4_MIDI_CAPTURE_MARK 18446744073709551616 before_play",
            "S4_MIDI_CAPTURE_MARK 1",
            "S4_MIDI_CAPTURE_MARK  1 before_play",
        ] {
            assert_eq!(parse_marker(invalid), None);
        }
        // Labels with whitespace still reach validation and receive rejection.
        assert_eq!(
            parse_marker("S4_MIDI_CAPTURE_MARK 7 two words"),
            Some((7, "two words"))
        );
    }

    #[test]
    fn decoder_accepts_only_exact_machine_commands() {
        assert_eq!(
            parse("S4_SCREEN_CONTROL 18446744073709551615 RETRY"),
            Some((u64::MAX, ControlAction::Retry))
        );
        for input in [
            "S4_SCREEN_CONTROL -1 ON",
            "S4_SCREEN_CONTROL +1 ON",
            "S4_SCREEN_CONTROL 1 on",
            "S4_SCREEN_CONTROL 1 OFF extra",
            " S4_SCREEN_CONTROL 1 ON",
            "S4_SCREEN_CONTROL 1  ON",
            "S4_SCREEN_CONTROL 18446744073709551616 OFF",
        ] {
            assert!(parse(input).is_none(), "{input}");
        }
    }

    #[test]
    fn managed_jog_tension_dispatches_only_valid_unsigned_bytes() -> anyhow::Result<()> {
        let (tx, rx) = crossbeam_channel::unbounded();
        serve(
            b"ni_jog_tension 0\nni_jog_tension 255\nni_jog_tension 256\nni_jog_tension +1\nni_jog_tension 1 extra\n".as_slice(),
            None,
            |value| Ok(tx.send(value)?),
            |_| Ok(()),
            &crate::midi_capture::Handle::default(),
        )?;
        assert_eq!(rx.try_iter().collect::<Vec<_>>(), [0, 255]);
        Ok(())
    }

    #[test]
    fn managed_stdin_eof_uses_normal_stop_channel() -> anyhow::Result<()> {
        // Given a managed screen worker and a subscribed stop receiver.
        let worker = Worker::start_managed(false)?;
        let (stop_tx, stop_rx) = crossbeam_channel::bounded(1);
        // When the app's pipe reaches EOF without any screen commands.
        serve(
            io::empty(),
            Some(&worker.control()),
            |_| anyhow::bail!("NI decks not enabled"),
            |_| Ok(stop_tx.try_send(())?),
            &crate::midi_capture::Handle::default(),
        )?;
        // Then the existing runtime stop channel receives the shutdown event.
        assert!(stop_rx.try_recv().is_ok());
        Ok(())
    }

    #[test]
    fn bridge_stop_grammar_is_exact_and_dispatches_before_eof() -> anyhow::Result<()> {
        for invalid in [
            "S4_BRIDGE_CONTROL +1 STOP",
            "S4_BRIDGE_CONTROL -1 STOP",
            "S4_BRIDGE_CONTROL 18446744073709551616 STOP",
            "S4_BRIDGE_CONTROL 1  STOP",
            "S4_BRIDGE_CONTROL 1 STOP extra",
            "S4_BRIDGE_CONTROL 1 stop",
            "S4_BRIDGE_STATUS 1 STOP",
            "S4_BRIDGE_CONTROL 1 READY enable motor",
        ] {
            assert_eq!(parse_stop(invalid), None, "{invalid}");
        }
        let (tx, rx) = crossbeam_channel::unbounded();
        serve(
            "S4_BRIDGE_CONTROL 18446744073709551615 STOP\n".as_bytes(),
            None,
            |_| anyhow::bail!("NI decks not enabled"),
            |id| Ok(tx.send(id)?),
            &crate::midi_capture::Handle::default(),
        )?;
        assert_eq!(rx.try_iter().collect::<Vec<_>>(), [Some(u64::MAX), None]);
        Ok(())
    }
}
