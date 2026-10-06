//! Bounded MIDI feedback ingestion; HID output stays on the bridge thread.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context as _, ensure};
use crossbeam_channel::{Receiver, TrySendError};
use midir::os::unix::VirtualInput as _;
use midir::{MidiInput, MidiInputConnection};
use tracing::{debug, info, warn};

const QUEUE_CAPACITY: usize = 4096;
pub const PAD_COLOR_TOKENS: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
pub const PAD_WHITE_TOKEN: u8 = 9;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message {
    pub status: u8,
    pub number: u8,
    pub value: u8,
}

#[derive(Clone, Debug)]
pub struct FeedbackState {
    pub notes: [[Option<u8>; 128]; 5],
    pub meters: [Option<u8>; 4],
    pub meter_updated: [Option<Duration>; 4],
    pub loaded: [Option<bool>; 4],
    pub playing: [Option<bool>; 4],
}

impl Default for FeedbackState {
    fn default() -> Self {
        Self {
            notes: [[None; 128]; 5],
            meters: [None; 4],
            meter_updated: [None; 4],
            loaded: [None; 4],
            playing: [None; 4],
        }
    }
}

impl FeedbackState {
    pub fn apply(&mut self, message: Message, now: Duration) -> bool {
        let channel = usize::from(message.status & 0x0f);
        let kind = message.status & 0xf0;
        let number = usize::from(message.number);
        if channel == 6 && kind == 0xb0 && (4..8).contains(&number) {
            let deck = number - 4;
            let playing = message.value != 0;
            let changed = self.playing[deck] != Some(playing);
            self.playing[deck] = Some(playing);
            return changed;
        }
        if channel == 6 && number < 4 {
            match kind {
                0xb0 => {
                    let changed = self.meters[number] != Some(message.value);
                    self.meters[number] = Some(message.value);
                    self.meter_updated[number] = Some(now);
                    return changed;
                }
                0x80 | 0x90 => {
                    let loaded = kind == 0x90 && message.value != 0;
                    let changed = self.loaded[number] != Some(loaded);
                    self.loaded[number] = Some(loaded);
                    if !loaded {
                        self.notes[number][40..48].fill(None);
                        self.meters[number] = None;
                        self.meter_updated[number] = None;
                        self.playing[number] = None;
                    }
                    return changed;
                }
                _ => return false,
            }
        }
        if !matches!(kind, 0x80 | 0x90) || channel >= 5 {
            return false;
        }
        let supported = if channel == 4 {
            number < 4
        } else {
            matches!(number, 0..=4 | 6 | 8 | 15..=19 | 32..=35 | 40..=63)
        };
        if !supported {
            return false;
        }
        let value = if kind == 0x80 { 0 } else { message.value };
        let old = self.notes[channel][number];
        self.notes[channel][number] = Some(value);
        // A/C and B/D share sampler banks, including changes on a hidden deck.
        if channel < 4 && (48..=55).contains(&number) {
            self.notes[channel ^ 2][number] = Some(value);
        }
        old != Some(value)
    }
}

/// MIDI packet streams may include running status and realtime interleaving.
#[derive(Default)]
pub struct Decoder {
    status: Option<u8>,
    data: [u8; 2],
    received: usize,
}

impl Decoder {
    pub fn feed(&mut self, bytes: &[u8], mut emit: impl FnMut(Message)) {
        for &byte in bytes {
            if byte >= 0xf8 {
                continue;
            }
            if byte & 0x80 != 0 {
                self.status = (byte < 0xf0).then_some(byte);
                self.received = 0;
                continue;
            }
            let Some(status) = self.status else {
                continue;
            };
            let length = if matches!(status & 0xf0, 0xc0 | 0xd0) {
                1
            } else {
                2
            };
            self.data[self.received] = byte;
            self.received += 1;
            if self.received == length {
                if matches!(status & 0xf0, 0x80 | 0x90 | 0xb0) {
                    emit(Message {
                        status,
                        number: self.data[0],
                        value: self.data[1],
                    });
                }
                self.received = 0;
            }
        }
    }
}

pub struct FeedbackInput {
    _connection: MidiInputConnection<Decoder>,
    receiver: Receiver<Message>,
    overflow: Arc<AtomicBool>,
}

impl FeedbackInput {
    pub fn new(port_name: &str) -> anyhow::Result<Self> {
        let (sender, receiver) = crossbeam_channel::bounded(QUEUE_CAPACITY);
        let overflow = Arc::new(AtomicBool::new(false));
        let callback_overflow = Arc::clone(&overflow);
        let connection = MidiInput::new("S4 MK3 LED feedback")
            .context("create feedback MIDI client")?
            .create_virtual(
                port_name,
                move |timestamp, bytes, decoder: &mut Decoder| {
                    decoder.feed(bytes, |message| {
                        debug!(timestamp, ?message, "MIDI_FEEDBACK");
                        match sender.try_send(message) {
                            Ok(()) => {}
                            Err(TrySendError::Full(_)) => {
                                callback_overflow.store(true, Ordering::Relaxed);
                            }
                            Err(TrySendError::Disconnected(_)) => {
                                warn!("MIDI feedback receiver closed");
                            }
                        }
                    });
                },
                Decoder::default(),
            )
            .map_err(|error| anyhow::anyhow!("create feedback destination: {error}"))?;
        info!(port = port_name, "MIDI_FEEDBACK_READY");
        Ok(Self {
            _connection: connection,
            receiver,
            overflow,
        })
    }

    pub fn drain(&self, state: &mut FeedbackState, now: Duration) -> anyhow::Result<usize> {
        ensure!(
            !self.overflow.load(Ordering::Relaxed),
            "MIDI feedback queue overflow; restart to synchronize software state"
        );
        let mut count = 0;
        for message in self.receiver.try_iter().take(QUEUE_CAPACITY) {
            state.apply(message, now);
            count += 1;
        }
        ensure!(
            !self.overflow.load(Ordering::Relaxed),
            "MIDI feedback queue overflow; restart to synchronize software state"
        );
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_values_survive_running_status_and_realtime_interleaving() {
        // Given a packet split during one pad update.
        let mut decoder = Decoder::default();
        let mut output = Vec::new();
        decoder.feed(&[0x92, 40], |message| output.push(message));
        // When the packet finishes and running status changes another pad.
        decoder.feed(&[3, 41, 0xf8, 8], |message| output.push(message));
        // Then color tokens and the hidden deck are retained.
        assert_eq!(
            output,
            [
                Message {
                    status: 0x92,
                    number: 40,
                    value: 3
                },
                Message {
                    status: 0x92,
                    number: 41,
                    value: 8
                },
            ]
        );
    }

    #[test]
    fn sysex_and_system_common_do_not_leak_into_note_feedback() {
        // Given running status before system messages.
        let mut decoder = Decoder::default();
        let mut output = Vec::new();
        decoder.feed(&[0x90, 40, 1], |message| output.push(message));
        output.clear();
        // When unrelated SysEx and system-common payloads arrive.
        decoder.feed(&[0xf0, 40, 9, 0xf7, 41, 2, 0xf1, 3, 42, 4], |message| {
            output.push(message);
        });
        // Then none are mistaken for pad feedback.
        assert!(output.is_empty());
    }

    #[test]
    fn note_off_with_nonzero_velocity_clears_software_state() {
        // Given active Play.
        let mut state = FeedbackState::default();
        state.apply(
            Message {
                status: 0x90,
                number: 0,
                value: 127,
            },
            Duration::ZERO,
        );
        // When Djay emits a release velocity.
        state.apply(
            Message {
                status: 0x80,
                number: 0,
                value: 64,
            },
            Duration::ZERO,
        );
        // Then inactive state is zero, not the release velocity.
        assert_eq!(state.notes[0][0], Some(0));
    }

    #[test]
    fn sample_feedback_updates_both_decks_of_its_bank_only() {
        // Given unknown bank states.
        let mut state = FeedbackState::default();
        // When hidden deck C receives sample-playing feedback.
        state.apply(
            Message {
                status: 0x92,
                number: 50,
                value: 127,
            },
            Duration::ZERO,
        );
        // Then A/C show the shared state, B/D remain unknown.
        assert_eq!(
            state.notes.map(|notes| notes[50]),
            [Some(127), None, Some(127), None, None]
        );
    }

    #[test]
    fn shifted_action_feedback_cannot_overwrite_normal_state() {
        // Given unknown normal-deck feedback.
        let mut state = FeedbackState::default();
        // When an unsupported shifted/action address arrives.
        let changed = state.apply(
            Message {
                status: 0x98,
                number: 40,
                value: 127,
            },
            Duration::ZERO,
        );
        // Then normal pad state is untouched.
        assert!(!changed);
        assert_eq!(state.notes[0][40], None);
    }

    #[test]
    fn continuous_level_feedback_is_deck_addressed_and_refreshes_its_age() {
        // Given unknown meter levels.
        let mut state = FeedbackState::default();
        let now = Duration::from_millis(350);
        // When channel seven CC two arrives.
        state.apply(
            Message {
                status: 0xb6,
                number: 2,
                value: 93,
            },
            now,
        );
        // Then only deck C's meter and freshness are updated.
        assert_eq!(state.meters, [None, None, Some(93), None]);
        assert_eq!(state.meter_updated, [None, None, Some(now), None]);
    }

    #[test]
    fn raw_playback_feedback_is_independent_of_blinking_play_lamp() {
        let mut state = FeedbackState::default();
        state.apply(
            Message {
                status: 0xb6,
                number: 6,
                value: 127,
            },
            Duration::ZERO,
        );
        assert_eq!(state.playing, [None, None, Some(true), None]);
        for value in [0, 127, 0] {
            state.apply(
                Message {
                    status: 0x92,
                    number: 0,
                    value,
                },
                Duration::ZERO,
            );
            assert_eq!(state.playing[2], Some(true));
        }
        state.apply(
            Message {
                status: 0xb6,
                number: 6,
                value: 0,
            },
            Duration::ZERO,
        );
        assert_eq!(state.playing[2], Some(false));
    }

    #[test]
    fn loading_false_invalidates_old_cue_colors_without_clearing_sampler_bank() {
        // Given a deck with cached cue and sample colors.
        let mut state = FeedbackState::default();
        state.notes[2][40] = Some(4);
        state.notes[2][48] = Some(127);
        state.playing[2] = Some(true);
        // When Djay signals that deck C is no longer loaded.
        state.apply(
            Message {
                status: 0x96,
                number: 2,
                value: 0,
            },
            Duration::ZERO,
        );
        // Then stale cues disappear, independent sampler state stays known.
        assert_eq!(state.notes[2][40], None);
        assert_eq!(state.notes[2][48], Some(127));
        assert_eq!(state.loaded[2], Some(false));
        assert_eq!(state.playing[2], None);
    }

    #[test]
    fn virtual_destination_receives_real_midi_color_without_creating_an_echo_source()
    -> anyhow::Result<()> {
        // Given a native virtual destination already subscribed before sending.
        let name = format!("S4 feedback test {}", std::process::id());
        let input = FeedbackInput::new(&name)?;
        let output = midir::MidiOutput::new("S4 feedback test sender")?;
        let mut selected = None;
        for port in output.ports() {
            match output.port_name(&port) {
                Ok(port_name) if port_name == name => {
                    selected = Some(port);
                    break;
                }
                Ok(_) => {}
                Err(error) if !output.ports().contains(&port) => {
                    debug!(%error, "foreign MIDI destination departed during enumeration");
                }
                Err(error) => return Err(error).context("read active MIDI destination name"),
            }
        }
        let port = selected.context("test feedback destination missing")?;
        let mut connection = output
            .connect(&port, "feedback test")
            .map_err(|error| anyhow::anyhow!("connect test destination: {error}"))?;
        // When a hidden deck's color arrives through CoreMIDI.
        connection.send(&[0x92, 40, 8])?;
        let message = input.receiver.recv_timeout(Duration::from_secs(2))?;
        // Then the token is preserved and no MIDI source echoes it.
        assert_eq!(
            message,
            Message {
                status: 0x92,
                number: 40,
                value: 8
            }
        );
        let sources = MidiInput::new("S4 echo check")?;
        for port in sources.ports() {
            match sources.port_name(&port) {
                Ok(port_name) => assert_ne!(port_name, name),
                Err(error) if !sources.ports().contains(&port) => {
                    debug!(%error, "foreign MIDI source departed during echo check");
                }
                Err(error) => return Err(error).context("read active MIDI source name"),
            }
        }
        Ok(())
    }
}
