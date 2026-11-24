# Stage 2: minimal Djay MIDI test

This stage runs on the Mac, not a standalone bridge board. Native HID reads feed
Encdr's parser, then a small fixed profile emits MIDI through a CoreMIDI virtual
source named `S4 MK3 MIDI`. Audio continues directly through the S4 audio driver.

## Run

Connect and power the S4, close other applications controlling its HID interface,
then run:

```sh
cargo run --locked -- --midi --seconds 600
```

Keep this process running while configuring and using Djay. The port exists only
while the bridge runs. Stop with Ctrl-C or let the duration expire; the bridge
sends Note Off for its four transport notes before closing the source.
`--raw` can be combined with `--midi`. `--list` cannot.

Without `--midi`, the original connectivity probe remains available and no
virtual MIDI source is created. The default duration is still 30 seconds, so
specify a longer duration for mapping.

## MIDI profile

Channels below use MIDI's human-facing numbering (1-16). Note and CC numbers are
decimal; use these numbers rather than assuming a particular note-name octave.

| Physical control | MIDI channel | Message | Number | Djay target |
| --- | --- | --- | --- | --- |
| Left Play | 1 | Note | 0 | Deck 1 Play/Pause |
| Left Cue | 1 | Note | 1 | Deck 1 Cue |
| Right Play | 2 | Note | 0 | Deck 2 Play/Pause |
| Right Cue | 2 | Note | 1 | Deck 2 Cue |
| Channel 1 volume | 1 | CC | 16 | Deck 1 volume |
| Channel 2 volume | 1 | CC | 17 | Deck 2 volume |
| Channel 3 volume | 1 | CC | 18 | Deck 3 volume |
| Channel 4 volume | 1 | CC | 19 | Deck 4 volume |
| Crossfader | 1 | CC | 20 | Mixer crossfader |

Press sends Note On with velocity 127; release sends Note Off with velocity 0.
Faders emit absolute values 0-127, rounding the normalized S4 position to the
nearest value. The physical mixer order is 3, 1, 2, 4.

These five faders carry 12-bit positions (0-4095) inside 16-bit HID fields.
The probe corrects the pinned Encdr descriptor's 65535 normalization maximum
to 4095 for these controls, following Mixxx's S4 mapping. Full travel still
requires a hardware endpoint check; partial fader movement is not that check.

Left and right transport always address decks 1 and 2. A/C and B/D selection,
shift, jogs, tempo, other controls, and MIDI feedback are not implemented here.
The probe's left Play LED still mirrors the held button; it is not Djay playback
feedback. EXT and front-panel assignment/curve switches remain deferred as
recorded in [connectivity-test.md](connectivity-test.md#deferred-controls-verified-hid-fields).

## Configure Djay

Use Djay Pro for Mac's built-in MIDI Learn with an active PRO subscription.
No hand-authored Djay mapping-file format is assumed.

1. Start the bridge and find `MIDI_READY` in its log.
2. Open Djay's MIDI configuration and select `S4 MK3 MIDI`. If it prompts to
   configure an unsupported controller, create a new configuration.
3. Press each Play/Cue button and assign its deck and action from the table.
   Treat these as buttons: Play/Pause should act on press, not toggle again on
   release. Cue must receive release so it does not remain held.
4. Move each fader and assign its target as an absolute fader/knob, not a
   relative encoder. Use the four-deck layout to configure/test deck 3/4 volume.
5. Save the configuration with Done.
6. Select the S4 for Djay audio. Check master output and, where configured,
   headphone output separately.

If the source is not listed, first check macOS Audio MIDI Setup -> Window ->
Show MIDI Studio with the bridge running. A visible source and successful
learning in Djay are separate checks.

Official references:

- [Djay Pro Mac MIDI mapping](https://help.algoriddim.com/user-manual/djay-pro-mac/midi/mapping)
- [MIDI Learn for unsupported controllers](https://help.algoriddim.com/topic/hardware/how-to-map-a-controller)
- [Hardware recognition troubleshooting](https://help.algoriddim.com/topic/troubleshooting/hardware-not-recognized)

## Observe real MIDI delivery

In another terminal, while the bridge is running:

```sh
cargo run --locked --example midi-monitor -- --seconds 60
```

Wait for `MIDI_MONITOR_READY`, then operate the mapped controls. `MIDI_RECEIVED`
comes from a separate CoreMIDI input connection, not the bridge's send log.
For example, left Play press/release should yield `[90, 00, 7f]` and
`[80, 00, 00]`; crossfader values should span `[b0, 14, 00]` to
`[b0, 14, 7f]`. These arrays are hexadecimal.

## Pass criteria and checks

- Play/Pause toggles each correct deck once per press; release does not toggle it.
- Cue holds/releases correctly on both decks.
- Each volume fader controls only its assigned deck and reaches both endpoints.
- Crossfader reaches left, center, and right with the expected direction.
- Audio continues through the S4 during the test.
- Releasing buttons and stopping the bridge does not leave Cue held.

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked --all-targets
```

Tests cover profile messages, scaling, excluded controls, invalid fader values,
and actual CoreMIDI source-to-receiver delivery without sleeps. They do not
establish Djay action behavior or audio routing by themselves.

During implementation, macOS initially denied UI automation with
`osascript is not allowed assistive access. (-1728)`. The user subsequently
granted permissions and performed MIDI Learn and the physical Djay test.
The saved configuration was inspected directly; action verification is
user-observed rather than an automated UI test.

## Observed results: October 5, 2026

- The user confirmed that Djay lists `S4 MK3 MIDI`.
- An independent CoreMIDI receiver observed real S4 Play press/release and all
  five mapped fader CCs before the normalization correction.
- After the correction, the user's full-travel sweep emitted values 0-127 for
  each of CC 16, 17, 18, 19, and 20, with no bridge errors.
- The receiver was attached late in that sweep; post-correction reception was
  observed, but full-range reception of every CC was not independently captured.
- Formatting, strict Clippy, ten tests (including real CoreMIDI delivery), the
  locked all-target build, and Rust LSP diagnostics passed.
- The user confirmed all functions in this guide were learned and working in
  Djay, with channel 3 and channel 4 volumes operating independently.
- The user confirmed the working mapping was saved and Djay audio plays through
  the S4. The final saved file contains all nine assignments, with CC 18 mapped
  to channel 3 volume and no relative Rotary override on CC 18/19.

The working configuration is saved at
`~/Music/djay/MIDI Mappings/S4 MK3 MIDI.djayMidiMapping`, recorded by Djay
Pro 5.6.9. It is a user-created application setting, not a file edited by the
bridge. Keep the virtual source name `S4 MK3 MIDI` when running the bridge.

This establishes the minimal S4 -> MIDI -> Djay control path and direct S4 audio
coexistence. Separate master/headphone routing, latency, long-duration use,
and behavior beyond the nine-control profile are not established by this test.
