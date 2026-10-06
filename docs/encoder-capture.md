# Verified MOVE/LOOP physical fields

October 6, 2026. The user operated each physical encoder independently:
five clockwise clicks, five counterclockwise clicks, and a press/release.
Both sequences were repeated with the corresponding Shift button held.

A temporary native IOHID diagnostic read Report 1 without creating MIDI ports
or writing LEDs, motors, or audio. The capture is stopped. Local raw evidence:
`.omo/evidence/encoder-capture.log` and `encoder-identification.md`.

Offsets below are zero-based **payload** offsets, excluding the report ID.
The native report is 23 bytes including ID, with a 22-byte payload.

| Physical control | Rotation field | Clockwise / counterclockwise | Press field | Pinned descriptor names |
| --- | --- | --- | --- | --- |
| Left MOVE | Byte 19, low nibble | +1 / -1 modulo 16 | Byte 6, mask `0x04` | `left_loop_encoder` / `left_loop_encoder_press` |
| Left LOOP | Byte 19, high nibble | +1 / -1 modulo 16 | Byte 6, mask `0x20` | `left_move_encoder` / `left_move_encoder_press` |
| Right MOVE | Byte 20, high nibble | +1 / -1 modulo 16 | Byte 15, mask `0x20` | `right_loop_encoder` / `right_move_encoder_press` |
| Right LOOP | Byte 21, low nibble | +1 / -1 modulo 16 | Byte 15, mask `0x04` | `right_move_encoder` / `right_loop_encoder_press` |

All four rotation names are reversed. Left press names are also reversed;
right press names are already correct. A blanket swap of every MOVE/LOOP
identifier would therefore break the right press pairing.

## Shift verification

Left Shift is payload byte 5 mask `0x02`; right Shift is byte 14 mask `0x02`.
Both were observed held through their respective MOVE and LOOP rotations and
presses, then released. Encoder fields, masks, and polarity did not change:
Shift is a separate modifier for the bridge to interpret, not different
hardware encoder addresses.

| Sequence | Event interval, Unix seconds |
| --- | --- |
| Left MOVE | 1791304305.076181-1791304316.479588 |
| Left LOOP | 1791304985.018019-1791304992.399852 |
| Right MOVE | 1791305196.196612-1791305203.397161 |
| Right LOOP | 1791305723.3170419-1791305735.797607 |
| Left Shift with MOVE then LOOP | 1791305842.715894-1791305857.771163 |
| Right Shift with MOVE then LOOP | 1791306028.064518-1791306044.864417 |

Additional byte 6/15 bits `0x08` and `0x10` changed during handling. Their
meaning is not established; they are not the confirmed press masks above.

## Implementation implications

Correct physical naming at the descriptor-normalization boundary: swap the
four rotation names and the two left press names, leaving right presses alone.
Preserve the existing decoded clockwise polarity for these knobs.

The working-tree full-profile descriptor applies that correction.
The captured-field regression exercises Encdr's wrap16 decoder and the real
translator for both sides, with and without Shift. It failed before correction
and passed afterward; formatter, strict Clippy, and all 65 Rust tests passed
at that historical correction revision.
The minimal profile is unchanged. At that revision, the portable app hadn't
yet been rebuilt with the correction; this record doesn't establish the
contents of a currently installed bundle.

The capture identifies physical controls, not the musical behavior of the
subsequent encoder layout. Mapping/decoder code and packaged binaries were
not changed during the capture itself.

## Current routing overlay

The implemented full-profile layout keeps native loop and jump sizes
independent. LOOP turn uses `autoLoopDurationRotary`; press uses
`autoLoopOnOff`. Shifted LOOP turn uses `autoLoopMoveRotary`; shifted press
uses `reloop` to reactivate the stored loop range. Manual loop in/out is
unmapped. The user observed Shift+LOOP moving an inactive loop region too,
apparently by that region's loop length.

Normal MOVE uses CC 2 on channels 1-4 (`skipRotary`) with or without an
active loop. MOVE press emits no MIDI and toggles the logical deck's local
size-select mode. There is no timeout. In selection mode, unshifted MOVE
uses CC 7 on channels 1-4 (`skipDurationRotary`) to change the native jump
value immediately; pressing again exits without a deferred save.
Shift+MOVE always sends fixed one-beat jumps, note 5 forward or note 14
backward on channels 9-12, even while selecting, without changing jump size.
Base-channel aliases exist in the native catalog but aren't runtime Shift
destinations.

A/B/C/D keep independent Djay values and independent local mode flags;
the bridge doesn't count or reset their sizes. All mode flags start off on
bridge restart. The currently selected deck selector pulses through a
1.2-second palette-brightness cycle with a dark trough. A hidden deck keeps
its mode without flashing its inactive selector. No screen or motor traffic
is added.

The user mouse-verified beat jump moving an active loop. Fresh MIDI/native
and physical LED checks for this layout remain pending. Historical decoder
test results above don't establish those passes. Reinstall a new mapping
under a new filename, keeping custom mappings intact; follow the
[encoder live check](full-midi-test.md#encoder-layout-live-check).

Browse polarity, jog sensitivity/backspin, software loop-state routing, and
other MIDI actions were not tested by this encoder-only diagnostic.
