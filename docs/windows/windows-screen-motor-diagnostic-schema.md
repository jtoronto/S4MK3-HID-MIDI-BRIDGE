# Screen/motor capture format

The opt-in JSONL capture uses version 1. Header and footer are machine records;
rows between them have `us`, `role`, `seq`, `kind`, `values[8]`, and `payload`.
`us` uses one engine-wide monotonic epoch. Sequences and begin/end references
are local to each role (`input`, `output`, `screen`). Export groups rows by
role; sort by timestamp for a combined timeline, retaining role/sequence as
the tie breaker. Do not interpret file order as cross-thread execution order.

`utc_unix_us` is an approximate wall-clock anchor sampled beside the
monotonic epoch. Operator UTC markers are annotations; clock jumps or
sampling offset limit cross-clock precision.

## Numeric fields

Unused `values` entries are zero. Operation success means that boundary's
software call returned successfully; it does not establish physical response.

| Kind | Used values, in order |
| --- | --- |
| `input_begin` | operation (0 interrupt, 1/2/3 numbered GET), requested buffer length |
| `input_end` | begin sequence, operation, native success, native returned length, returned report ID; payload contains available returned bytes |
| `batch_published` | batch host timestamp, queue admission success, queue depth after admission, normalized report-1/3/2 lengths |
| `batch_consumed` | batch host timestamp, consumption host timestamp, queue age |
| `sensor` | batch host timestamp, side, counter, sensor timer, signed delta bits, status, last nonduplicate host timestamp, shared freshness deadline |
| `motor_state` | batch host timestamp, side, touch, enabled, MIDI-owned, continuous, regulator mode, signed selected force bits |
| `playback` | host timestamp, left/right enabled, left/right owned, left/right continuous |
| `feedback` | host timestamp, side, logical deck, loaded, playing, Internal, THRU, local mode bit mask |
| `decision` | output tick host timestamp, suspended/recovery flag; payload is the selected native motor report |
| `input_timeout` | host timestamp, expired deadline |
| `input_recovered` | new batch host timestamp, consumption host timestamp |
| `motion_reset` | previous input host timestamp, previous left/right sensor-freshness timestamps |
| `native_write_begin` | report ID, logical length, Windows flag; payload is the actual logical output report |
| `native_write_end` | begin sequence, native API success, raw native returned byte count |
| `screen_connect_begin` | generation |
| `screen_ready` | begin sequence, generation, open/READY success |
| `screen_render_begin` | generation, zero-based pair index |
| `screen_render_end` | begin sequence, generation, pair index, left/right pixel hashes |
| `screen_publish_begin` | generation, pair index, left/right pixel hashes |
| `screen_publish_end` | begin sequence, generation, success, pair index, left/right pixel hashes |
| `screen_release_begin` | generation |
| `screen_release_end` | begin sequence, generation, close success |
| `screen_case_ready` | generation; payload is the UTF-8 case name |
| `lifecycle` | code, followed by code-specific fields below |

Sensor status: 0 baseline, 1 moving, 2 rest, 3 duplicate. Regulator mode:
0 absent, 1 disabled, 2 qualifying, 3 regulating, 4 assisting.
Side is physical left=0/right=1; logical deck is A=0 through D=3.
Feedback flags are 0 unknown / 1 false / 2 true. Local mode mask:
bit 0 Vinyl, bit 1 motor enabled, bit 2 Shift, bit 3 Grid.
Delta and force bits are signed i32 values stored through their u32 bit pattern.

Lifecycle code 1 starts the diagnostic session; 2 closes it with execution-success in
`values[1]`; 3 records the bridge host-clock epoch in `values[1]`, in capture
microseconds; 4 marks completed capture preparation before endpoint publication;
5 records the host's permission to continue after endpoint creation;
10 records controller usage page/usage and serial bytes;
100 records a failed screen generation in `values[1]`.
Bridge host timestamps approximately map to capture time by adding the code-3
epoch; use directly timestamped native spans for precise operation timing.

Current diagnostic hosts launch the worker with
`--prepare-screen-motor-diagnostics`. After all capture allocation, configuration
and metadata work, the worker flushes `S4_SCREEN_DIAG_PREPARED 1` and waits.
The host creates the BLOOP endpoint only after receiving that line, then
flushes `S4_SCREEN_DIAG_BEGIN 1`. The worker does not open HID, MIDI or screens
before this grant. STOP or stdin closure while waiting aborts preparation
without entering the device runtime. These preparation receipts are distinct
from normal bridge and screen readiness.

## Completeness and interpretation

The footer records runtime success, incomplete state, first Stop timestamp
and each role's count/capacity/losses. Loss, missing producer, missing footer,
failed runtime, failed screen generation or unconfirmed release prevents a
complete causal timeline claim. A trace can still contain useful positive
observations. An empty file after a killed process is not a completed capture.

Input payloads preserve native framing before normalization. On Windows,
output calls may return 95 bytes for a shorter logical report. The native end
row records that raw count before normalization and short-write checks; a
successful API return with an invalid count is not an accepted application
write. See the following runtime error and cleanup records.

READY/FRAME_OK are supervisor/worker software observations. They do not
acknowledge device pixels. Motor writes do not measure torque. Correct
submissions with poor physical response leave NI/device state or competing
outputs unresolved; these records alone cannot attribute a different writer.
