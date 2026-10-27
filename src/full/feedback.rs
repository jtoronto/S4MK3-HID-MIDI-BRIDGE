//! Bounded MIDI feedback ingestion; HID output stays on the bridge thread.

pub mod info;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context as _, ensure};
use crossbeam_channel::{Receiver, TrySendError};
use midir::os::unix::VirtualInput as _;
use midir::{MidiInput, MidiInputConnection};
use tracing::{debug, info, warn};

use super::catalog::{EQ_MODE_NOTE, EXT_THRU_CC};

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
    pub info: info::State,
    pub notes: [[Option<u8>; 128]; 5],
    pub meters: [Option<u8>; 4],
    pub meter_updated: [Option<Duration>; 4],
    pub loaded: [Option<bool>; 4],
    pub playing: [Option<bool>; 4],
    /// Whether Djay's Neural Mix EQ mode is enabled, independently per deck.
    pub eq_mode: [Option<bool>; 4],
    /// Confirmed per-deck THRU mode, not the global DVS-enable setting.
    pub thru: [Option<bool>; 4],
    /// Canonical FX parameters 1-3, then independent slot wet/dry values.
    pub fx_values: [[Option<u8>; 6]; 4],
    instant_fx_order: [[Option<usize>; 4]; 4],
    /// A playback output arrived from the mapping during this bridge session.
    pub playback_received: bool,
}

impl Default for FeedbackState {
    fn default() -> Self {
        Self {
            info: info::State::default(),
            notes: [[None; 128]; 5],
            meters: [None; 4],
            meter_updated: [None; 4],
            loaded: [None; 4],
            playing: [None; 4],
            eq_mode: [None; 4],
            thru: [None; 4],
            fx_values: [[None; 6]; 4],
            instant_fx_order: [[None; 4]; 4],
            playback_received: false,
        }
    }
}

impl FeedbackState {
    /// Return the most recently activated still-active Instant FX slot (0-3).
    pub fn active_instant_fx(&self, deck: usize) -> Option<usize> {
        self.instant_fx_order.get(deck)?.first().copied().flatten()
    }

    fn apply_control(&mut self, message: Message) -> Option<bool> {
        if message.status & 0xf0 != 0xb0 {
            return None;
        }
        let deck = usize::from(message.status & 0x0f);
        if deck < 4 && message.number == EXT_THRU_CC {
            let thru = message.value != 0;
            let changed = self.thru[deck] != Some(thru);
            self.thru[deck] = Some(thru);
            return Some(changed);
        }
        if deck < 4 && (32..38).contains(&message.number) {
            let value = &mut self.fx_values[deck][usize::from(message.number - 32)];
            let changed = *value != Some(message.value);
            *value = Some(message.value);
            return Some(changed);
        }
        let changed = self.info.apply(message);
        (deck < 4 && (80..=112).contains(&message.number) || deck == 6 && message.number == 8)
            .then_some(changed)
    }

    pub fn apply(&mut self, message: Message, now: Duration) -> bool {
        if let Some(changed) = self.apply_control(message) {
            return changed;
        }
        let channel = usize::from(message.status & 0x0f);
        let kind = message.status & 0xf0;
        let number = usize::from(message.number);
        if channel == 6 && kind == 0xb0 && (4..8).contains(&number) {
            self.playback_received = true;
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
                        self.info.clear_deck(number);
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
            number == usize::from(EQ_MODE_NOTE)
                || matches!(number, 0..=4 | 6 | 8 | 15..=19 | 32..=35 | 40..=63)
        };
        if !supported {
            return false;
        }
        let value = if kind == 0x80 { 0 } else { message.value };
        let old = self.notes[channel][number];
        self.notes[channel][number] = Some(value);
        if channel < 4 && number == usize::from(EQ_MODE_NOTE) {
            let enabled = value != 0;
            let changed = self.eq_mode[channel] != Some(enabled);
            self.eq_mode[channel] = Some(enabled);
            return changed;
        }
        if channel < 4 && (16..=19).contains(&number) {
            let slot = number - 16;
            if value != 0 && old.is_none_or(|old| old == 0) {
                let mut order = [None; 4];
                order[0] = Some(slot);
                let mut next = 1;
                for active in self.instant_fx_order[channel].iter().flatten().copied() {
                    if active != slot {
                        order[next] = Some(active);
                        next += 1;
                    }
                }
                self.instant_fx_order[channel] = order;
            } else if value == 0 && old.is_some_and(|old| old != 0) {
                let mut order = [None; 4];
                let mut next = 0;
                for active in self.instant_fx_order[channel].iter().flatten().copied() {
                    if active != slot {
                        order[next] = Some(active);
                        next += 1;
                    }
                }
                self.instant_fx_order[channel] = order;
            }
        }
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
    fn thru_feedback_is_per_deck_and_independent_of_track_loading() {
        // Given unknown mode state and the real MIDI stream decoder.
        let mut state = FeedbackState::default();
        let mut decoder = Decoder::default();
        assert_eq!(state.thru, [None; 4]);
        // When every deck reports a flag, with running status and realtime.
        for deck in 0_u8..4 {
            decoder.feed(&[0xb0 | deck, EXT_THRU_CC, 0], |message| {
                state.apply(message, Duration::ZERO);
            });
            assert_eq!(state.thru[usize::from(deck)], Some(false));
            decoder.feed(&[EXT_THRU_CC, 0xf8, 127], |message| {
                state.apply(message, Duration::ZERO);
            });
            assert_eq!(state.thru[usize::from(deck)], Some(true));
        }
        // Then input notes, shifted channels, mixer CCs and unload do not
        // masquerade as deck-mode feedback or invalidate an independent mode.
        decoder.feed(
            &[
                0x90,
                27,
                0,
                0xb8,
                EXT_THRU_CC,
                0,
                0xb4,
                EXT_THRU_CC,
                0,
                0x86,
                3,
                0,
            ],
            |message| {
                state.apply(message, Duration::ZERO);
            },
        );
        assert_eq!(state.thru, [Some(true); 4]);
        decoder.feed(&[0xb2, EXT_THRU_CC, 0], |message| {
            state.apply(message, Duration::ZERO);
        });
        assert_eq!(
            state.thru,
            [Some(true), Some(true), Some(false), Some(true)]
        );
    }

    #[test]
    fn fx_value_feedback_is_independent_per_deck_and_excludes_input_aliases() {
        // Given a real MIDI decoder and unknown values for every deck.
        let mut state = FeedbackState::default();
        let mut decoder = Decoder::default();
        // When canonical zero/maximum CCs and unrelated input aliases arrive.
        for channel in 0_u8..4 {
            for number in 32_u8..38 {
                decoder.feed(
                    &[
                        0xb0 | channel,
                        number,
                        if channel.is_multiple_of(2) { 0 } else { 127 },
                    ],
                    |message| {
                        state.apply(message, Duration::ZERO);
                    },
                );
            }
            decoder.feed(
                &[0xb0 | channel, 40, 42, 0xb8 | channel, 32, 42],
                |message| {
                    state.apply(message, Duration::ZERO);
                },
            );
        }
        // Then neither the other strip's input address nor Shift contaminates feedback.
        assert_eq!(
            state.fx_values,
            [[Some(0); 6], [Some(127); 6], [Some(0); 6], [Some(127); 6]]
        );
    }

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
    fn eq_mode_feedback_is_unknown_until_each_deck_reports_state() {
        let mut state = FeedbackState::default();
        assert_eq!(state.eq_mode, [None; 4]);
        let mut expected = [None; 4];

        for deck in 0_u8..4 {
            assert!(state.apply(
                Message {
                    status: 0x90 | deck,
                    number: EQ_MODE_NOTE,
                    value: 127,
                },
                Duration::ZERO,
            ));
            expected[usize::from(deck)] = Some(true);
            assert_eq!(state.eq_mode, expected);
        }

        assert!(state.apply(
            Message {
                status: 0x83,
                number: EQ_MODE_NOTE,
                value: 64,
            },
            Duration::ZERO,
        ));
        assert_eq!(
            state.eq_mode,
            [Some(true), Some(true), Some(true), Some(false)]
        );
    }

    #[test]
    fn instant_fx_order_ignores_duplicate_edges_and_falls_back_on_release() {
        let mut state = FeedbackState::default();
        let apply = |state: &mut FeedbackState, status, number, value| {
            state.apply(
                Message {
                    status,
                    number,
                    value,
                },
                Duration::ZERO,
            )
        };

        for slot in 0_u8..4 {
            assert!(apply(&mut state, 0x90, 16 + slot, 127));
        }
        assert!(!apply(&mut state, 0x90, 16, 127));
        assert_eq!(state.active_instant_fx(0), Some(3));

        assert!(apply(&mut state, 0x80, 19, 32));
        assert_eq!(state.active_instant_fx(0), Some(2));
        assert!(apply(&mut state, 0x80, 18, 32));
        assert_eq!(state.active_instant_fx(0), Some(1));
        assert!(apply(&mut state, 0x80, 17, 32));
        assert_eq!(state.active_instant_fx(0), Some(0));
        assert!(apply(&mut state, 0x91, 19, 127));
        assert_eq!(state.active_instant_fx(1), Some(3));
        assert_eq!(state.active_instant_fx(4), None);

        assert!(apply(&mut state, 0x80, 16, 0));
        assert_eq!(state.active_instant_fx(0), None);
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
    fn numeric_outputs_decode_from_the_serialized_generated_mapping() -> anyhow::Result<()> {
        let mut bytes = Vec::new();
        super::super::mapping::generate()?.to_writer_xml(&mut bytes)?;
        let mapping = plist::Value::from_reader(std::io::Cursor::new(bytes))?;
        let outputs = mapping
            .as_dictionary()
            .and_then(|root| root.get("outputs"))
            .and_then(plist::Value::as_array)
            .context("serialized mapping outputs missing")?;
        let mut decoder = Decoder::default();
        let mut state = FeedbackState::default();
        for output in outputs {
            let entry = output.as_dictionary().context("output dictionary")?;
            let channel = entry
                .get("midiChannel")
                .and_then(plist::Value::as_unsigned_integer)
                .context("output channel")?;
            let data = entry
                .get("midiData")
                .and_then(plist::Value::as_unsigned_integer)
                .context("output data")?;
            if channel < 4 && (80..112).contains(&data) {
                let data = u8::try_from(data)?;
                let status = 0xb0 | u8::try_from(channel)?;
                let (first, end, expected) = if data < 90 {
                    (
                        80_u8,
                        90_u8,
                        super::info::JUMP_INTERVALS
                            .iter()
                            .find(|(interval, _)| {
                                entry.get("keyPath").and_then(plist::Value::as_string)
                                    == Some(
                                        format!(
                                            "turntable{}.skipping{interval}BeatInterval",
                                            channel + 1
                                        )
                                        .as_str(),
                                    )
                            })
                            .map(|(_, label)| *label),
                    )
                } else {
                    (
                        90_u8,
                        112_u8,
                        super::info::LOOP_INTERVALS
                            .iter()
                            .find(|(interval, _)| {
                                entry.get("keyPath").and_then(plist::Value::as_string)
                                    == Some(
                                        format!(
                                            "turntable{}.autoLoop{interval}BeatIntervalActive",
                                            channel + 1
                                        )
                                        .as_str(),
                                    )
                            })
                            .map(|(_, label)| *label),
                    )
                };
                for number in first..end {
                    decoder.feed(&[status, number, 0], |message| {
                        state.apply(message, Duration::ZERO);
                    });
                }
                decoder.feed(&[status, data, 127], |message| {
                    state.apply(message, Duration::ZERO);
                });
                let deck = usize::try_from(channel)?;
                if data < 90 {
                    assert_eq!(state.info.jump_size(deck), expected);
                } else {
                    assert_eq!(state.info.active_loop_size(deck), expected);
                }
            }
        }
        assert_eq!(state.info.jump_size(4), None);
        Ok(())
    }

    #[test]
    fn numeric_info_preserves_unknown_conflicts_and_clears_deck_ranges() {
        // Given unknown sizes, then mutually exclusive states in transition.
        let mut state = FeedbackState::default();
        assert_eq!(state.info.jump_size(0), None);
        let apply = |state: &mut FeedbackState, channel, number, value| {
            state.apply(
                Message {
                    status: 0xb0_u8 | channel,
                    number,
                    value,
                },
                Duration::ZERO,
            )
        };
        assert!(apply(&mut state, 0, 83, 127));
        assert_eq!(state.info.jump_size(0), Some("1"));
        assert!(!apply(&mut state, 0, 83, 127));
        assert!(apply(&mut state, 0, 84, 127));
        assert_eq!(state.info.jump_size(0), None);
        assert!(apply(&mut state, 0, 83, 0));
        assert_eq!(state.info.jump_size(0), Some("2"));
        apply(&mut state, 0, 90, 127);
        state.info.key_raw[0] = Some(81);
        apply(&mut state, 6, 8, 32);
        apply(&mut state, 6, 0, 0);
        assert_eq!(state.info.active_loop_size(0), Some("1/32"));
        // When loading-false arrives as a Note, not the meter CC at the same number.
        state.apply(
            Message {
                status: 0x96,
                number: 0,
                value: 0,
            },
            Duration::ZERO,
        );
        // Then song state clears while the deck key, selection and global range survive.
        assert_eq!(state.info.active_loop_size(0), None);
        assert_eq!(state.info.jump_size(0), Some("2"));
        assert_eq!(state.info.key_raw[0], Some(81));
        assert_eq!(state.info.tempo_range, Some(25));
    }

    #[test]
    fn tempo_range_decodes_only_verified_values() {
        let mut state = FeedbackState::default();
        for (raw, percent) in [
            (5, 4),
            (8, 6),
            (10, 8),
            (13, 10),
            (20, 16),
            (32, 25),
            (64, 50),
            (95, 75),
        ] {
            state.apply(
                Message {
                    status: 0xb6,
                    number: 8,
                    value: raw,
                },
                Duration::ZERO,
            );
            assert_eq!(state.info.tempo_range, Some(percent));
        }
        state.apply(
            Message {
                status: 0xb6,
                number: 8,
                value: 7,
            },
            Duration::ZERO,
        );
        assert_eq!(state.info.tempo_range, None);
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
