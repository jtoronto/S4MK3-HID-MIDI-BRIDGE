# Windows wheel-tension timing investigation

On November 9, 2026, an attended diagnostic reproduced chatter while the
operator gently moved the right wheel at 100% wheel tension, with Djay open
and playback paused. This was the older loopMIDI comparison build, not the
current BLOOP app or NI Bridge. On Stop, the operator confirmed the wheel was
unforced. The log reported a final neutral write and exit code 0, and the
five original preference files matched their pre-run hashes.

The 52.36-second trace retained 55,204 events with zero reported drops. All
18,703 HID writes returned the Windows backend's raw success length of 95.
In the nonzero command interval, write-start spacing was 2.696 ms median,
3.935 ms at the 95th percentile, and 4.723 ms maximum, against a nominal 2 ms
target. Six input gates issued zero-force writes followed by nonzero commands
9.3–46.6 ms later. The trace did not record stale input, a sensor gap, reset,
recovery, playback gate, or backspin gate.

Command timing and zero/resume transitions were candidates, not established
causes. There was no synchronized audio marker. Snapshot times reflect host
read-batch completion, HID write success does not measure bus timing or
torque, and capture overhead was not measured. Earlier smooth standalone
runs also used a lower force limit, so they were not a controlled comparison.
The original decompressed trace SHA-256 was
`8b02ca5a0d31ea98cfb05287f0d6348f7dcb1e289201a554804a5584937c4acc`.
