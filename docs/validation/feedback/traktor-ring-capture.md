# Traktor ring capture findings

Source: user-provided `traktor_usbcap.pcapng`, 502,529,200 bytes, captured on
Windows with USBPcap. Analysis streamed all 318,073 pcapng blocks, verified
block lengths, and isolated host-to-device packets on bus 1, device 3,
interrupt endpoint 0x02. Audio and screen packets were not used as LED reports.

## Observed reports

Lengths include the report ID.

| ID | Length | Count |
| --- | --- | --- |
| 0x30 ring initialization | 28 | 2 |
| 0x32 ring control | 41 | 10,975 |
| 0x31 motors | 11 | 57,834 |
| 0x80 buttons | 95 | 482 |
| 0x81 meters | 79 | 969 |

Motor reports are evidence only; the bridge must not replay them.

Initialization is `[0x30, side, 1, 3, ...zeros]`, **28 bytes**, for sides 0/1.
The existing bridge used 27 bytes, matching the Mixxx reference rather than
this capture. Ring reports use physical side at byte 1, mode at byte 2,
little-endian position at bytes 3-4, and packed color at byte 5. All bytes
6-40 are zero throughout this capture.

A physical test using the captured 28-byte initialization and mode-5 packed
pixel colors still left both rings dark. Correcting initialization length
alone does not establish individual ring control.

## Ring behavior

Traktor sent only modes 0, 2, and 3. Mode 2 supplied the moving needle/chase,
with position values spanning a revolution. No mode-4 or mode-5 packet appears.
Thus this capture does not establish an individually addressable or full
steady-ring encoding.

Times below are seconds after the first ring output, not capture start.
They align with the user's A, B, C, D playback/loop action order.

| Time | Side | Mode | Color | Interpretation |
| --- | --- | --- | --- | --- |
| 0.386 | Left/right | 2 | 46 | Blue A/B |
| 49.966 | Left | 2 | 30 | A loop green |
| 51.961 | Left | 2 | 46 | A loop exit |
| 57.134 | Right | 2 | 30 | B loop green |
| 59.034 | Right | 2 | 46 | B loop exit |
| 71.900 | Left | 2 | 14 | C orange |
| 75.708 | Left | 2 | 30 | C loop green |
| 77.667 | Left | 2 | 14 | C loop exit |
| 78.666 | Right | 2 | 14 | D orange |
| 82.464 | Right | 2 | 30 | D loop green |
| 84.296 | Right | 2 | 14 | D loop exit |

Brief mode-3 blue intervals occurred at 33.418-33.929 seconds on the left
and 35.894-36.652 seconds on the right; their trigger is not established by
the action notes. Do not interpret mode 3 as Traktor's normal loop mode.

The observed chase follows software position, including the user's shifted
C jog seek. It is not evidence that Djay supplies an equivalent position
output; no playback-position estimate should be substituted.
