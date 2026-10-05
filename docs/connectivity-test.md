# Stage 1: Encdr connectivity

This stage tests the real S4 MK3 control connection on macOS. It does not implement
MIDI, a Djay mapping, audio forwarding, screen rendering, or powered jog behavior.

## Dependency and access boundaries

Encdr 0.7.0 is pinned to commit
`5d7a689ca5789161b3efd1b2049d7e54504fe881`. Cargo.lock records the resolved
dependencies. Use `--locked` to reproduce them.

The probe uses Encdr's descriptor registry, packet parser, and LED builder.
It selects only the S4 MK3 (`17cc:1720`) and opens its HID collection by path using
`hidapi` with shared macOS access, then decodes reports using Encdr:

- Only the `control` interface, USB interface 3, is retained in the descriptor.
  Native HID access does not directly claim USB interfaces.
- Only the `buttons` LED output report is retained.
- Screen, meter, jog-ring, and motor output groups are excluded.
- Audio interfaces are not claimed.
- No GPU context or WebView renderer is created. Encdr still depends on `wgpu`
  at compile time.

This is not an Encdr fork. The descriptor-boundary test protects the permitted
output reports.

The original Encdr USB-worker attempt failed live with `0xe00002c5` (exclusive
access). IORegistry showed interface 3 owned by `AppleUserUSBHostHIDDevice`, while
audio interfaces 0-2 were owned by `usbaudiod`. Encdr's `nusb` backend can detach
kernel drivers only on Linux, not macOS. Native HID access avoids that direct
USB claim.

The connected S4's HID report descriptor defines payload sizes of 22, 78, and 59
bytes for reports 1, 2, and 3, respectively. The probe strips the leading report
ID and corrects the pinned Encdr descriptor's button/jog packet sizes (25 and 48)
to 22 and 59. Other input reports are ignored. Known reports with unexpected
lengths fail visibly rather than silently losing controls.

LED reports contain the whole button/pad LED buffer, so this probe may clear
other button/pad lights. Run it without another application controlling S4 LEDs.
Reopen the usual controller software afterward to restore its lighting.

## Preparation

1. Connect the S4's own power adapter and power it on.
2. Connect its USB cable to the Mac.
3. Close Traktor, Mixxx, or other applications that may own its control interface.
4. Install Rust with Cargo, rustfmt, and Clippy if they are not already available.
   The project selects the stable toolchain; dependencies may require a newer
   compiler than an old installed stable toolchain.
5. Build from the project directory:

   ```sh
   cargo build --locked
   ```

The first build downloads Encdr and its dependencies. No Djay configuration is
needed for this stage.

## Enumeration without claiming the controller

```sh
cargo run --locked -- --list
```

`S4_USB_FOUND: 17cc:1720` means the USB device was enumerated. No interfaces are
opened in this mode. A zero-device listing succeeds but does not prove
connectivity; use the input probe next.

## Input and LED test

```sh
cargo run --locked -- --seconds 60
```

The probe requires exactly one connected S4 MK3. It stops after the specified
positive number of seconds or Ctrl-C.

During the run:

1. Press and hold the left deck's Play button. Find a `BUTTON` event for
   `left_play` with `pressed=true`.
2. Confirm that its LED lights while held.
3. Release Play. Find `pressed=false` and confirm that the LED goes off.
4. Move a channel or tempo fader. Find changing `SLIDER` values.
5. Turn a browse/loop encoder and then touch/rotate a jog. Check for encoder and
   touch events with values changing in response to the physical controls.
6. Release Play before ending the probe.

`S4_HID_OPEN` means the native HID collection was opened. Actual decoded controls
provide evidence of the input path.

`LED_WRITE_OK` means the native HID API accepted the complete 95-byte button LED
report. It does not prove that the physical LED changed; check the actual light.

The final counts distinguish received input from completed HID writes. A normal
exit requires at least one decoded input event. Initial slider state alone can
satisfy that count, so exit code zero does not replace the manual checks above.

## Raw HID diagnostics

```sh
cargo run --locked -- --raw --seconds 600
```

`HID_RAW` logs every received input report as a complete hex byte array,
including the leading report ID, before filtering or decoding. This includes
report types the probe does not map. Normal decoded events and LED behavior
continue alongside the raw logs. Logging uses the default `info` level; an
explicit `RUST_LOG` filter can suppress it.

For the unmapped EXT buttons and front crossfader assignment switches, operate
one control at a time and compare reports of the same ID before and after.
Byte index 0 is the report ID; Encdr's payload offsets start at index 1.
Raw logging does not itself add those controls to the mapping. The existing
requirement for at least one decoded event before a successful exit still applies.

## Deferred controls: verified HID fields

The October 5 `raw-test.log` capture contains 99 complete reports: 21 button
reports (`0x01`) and 78 jog/sensor reports (`0x03`). EXT and front-panel switches
are present in report `0x01`; their fields are absent from the pinned Encdr
descriptor, not discarded by the probe's report filtering.

All byte indices below are zero-based. A raw index includes the leading report
ID; a payload index excludes it, so payload byte 0 is raw byte 1.

| Channel | EXT raw byte | EXT payload byte | EXT mask | Crossfader assignment bits |
| --- | --- | --- | --- | --- |
| 3 | 3 | 2 | `0x02` | 6-7 |
| 1 | 12 | 11 | `0x01` | 4-5 |
| 2 | 12 | 11 | `0x02` | 2-3 |
| 4 | 12 | 11 | `0x04` | 0-1 |

EXT press/release changes at `raw-test.log:15-33` match the user's channel order
3, 1, 2, 4. A set EXT bit represents the held button, not a latched software mode.

All four crossfader assignment fields share raw byte 18 (payload byte 17).
Each is a two-bit selector, decoded as `(payload[17] >> shift) & 0x03`, with
shifts 6, 4, 2, and 0 for channels 3, 1, 2, and 4. Changes at
`raw-test.log:62-100` show selector values 0, 1, and 2 across the capture.
Physical position labels were not recorded; verify them before defining a MIDI
assignment rather than treating these fields as individual button presses.

The crossfader curve selector uses raw byte 19 (payload byte 18), bits 0-1.
Its state changes from 2 to 1 to 0 at `raw-test.log:47-52`.
The [Mixxx S4 MK3 mapping](https://github.com/mixxxdj/mixxx/blob/main/res/controllers/Traktor-Kontrol-S4-MK3.js)
corroborates the channel assignment fields and identifies this curve selector.

The constant report `0x03` traffic arrives approximately every 256 ms. Relative
jog values stay unchanged in this capture while other telemetry bytes vary;
these periodic reports are separate from the button and switch changes.

These locations are retained for later implementation. EXT, crossfader
assignment, and curve decoding and their Djay/MIDI behavior remain deferred.
No new input mappings or software assignments are implemented by this record.

## Audio coexistence check

This probe leaves the S4 audio interfaces alone, but coexistence still needs
verification:

1. Select the S4 for audio in Djay or another audio application.
2. Confirm audio works before starting the probe.
3. Keep audio playing during the input/LED test.
4. Check that audio continues and the S4 remains available as an audio interface.
5. Where the application permits it, check separate master and headphone streams.

This does not test audio forwarding through bridge hardware or establish a
latency budget.

## Troubleshooting

- **No S4 found:** check power and USB, then rerun `--list`. On macOS,
  `ioreg -p IOUSB -w 0` can independently show whether the controller enumerates.
- **Interface claim/access error:** close controller applications. If macOS
  presents an accessory-access prompt, handle it and rerun. Preserve the actual
  error instead of assuming that running as root is the fix.
- **HID open/read failure or no input:** inspect the error. USB discovery alone
  is not a successful control connection.
- **LED stays dark:** verify `left_play` press/release events and HID writes.
  Treat the visible output as unverified until the light
  physically follows the button.
- **Disconnect during the probe:** the probe reports a HID read/write failure.
  Reconnect and start a new run; reconnect automation is not part of
  this stage.

For more detailed dependency logs:

```sh
RUST_LOG=debug cargo run --locked -- --seconds 60
```

## Development checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

The automated tests check the actual pinned descriptor's access/output boundary,
numbered-report framing, and rejection of malformed known reports. They do not
substitute for USB input, visual LED confirmation, or audio listening.

## Observed results: October 5, 2026

The live native-HID run completed with exit code 0:

```text
Probe ended ... input_events=15 led_writes=8
```

- The S4 enumerated as `17cc:1720` and its native HID collection opened.
- Left and right Play press/release events were decoded.
- Right jog movement produced positive and negative fine-encoder deltas.
- Eight complete 95-byte button LED reports were written.
- The user confirmed that the left Play LED followed the held/released button.
- The user confirmed audio continued through the S4 while the probe ran.
- Ctrl-C stopped the probe cleanly.
- Formatting, Clippy with warnings denied, three tests, and the final locked
  build passed. CLI help and rejection of `--seconds 0` were checked.

Faders, all other controls, scratch feel, separate master/headphone routing,
latency, and long-duration operation were not established by this run. No MIDI
or audio-forwarding behavior is claimed.

LSP diagnostic requests timed out during this session. Compiler and Clippy
checks supplied the code diagnostics; no clean LSP result is claimed.
