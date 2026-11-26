# First attended A attempt with diagnostic kit 3

This attempt ran on November 21 local time / November 22, 2026 UTC.
It is an incomplete recording, not a passed USB shutdown or controlled
20-second observation.

Run directory:
`C:\Users\toron\Desktop\S4 Screen Motor Diagnostics 3\usb-captures\20261121-190751-3ef33a2c`.

## Physical report and feedback

The operator recorded motor response as "Normal movement" and rings as
"Normal behavior". The operator subsequently explained that the launcher asked
for missing-feedback/input-mode changes that were unavailable without selecting
ASIO. The audio driver remained unchanged.

The Thru/Internal instruction was inappropriate for this audio setup. The delay
was spent trying to interpret that instruction, not evidence that the operator
failed to follow a usable procedure. No driver change should be a prerequisite
for this diagnostic.

The actual worker eventually confirmed Deck B, right TT on, Loaded, Playing,
Internal, Thru off, and an open motor gate. Stop revoked readiness. This
establishes authorization in this attempt without an audio-driver change;
it does not establish a reliable initial-state handshake.

## Why the recording is incomplete

- USB started at 00:12:53.071 UTC and hit its 180-second duration guard at
  00:15:53.103. Forced termination is recorded.
- The manual Start prompt was 00:13:26.675. CHECK confirmed authorization at
  00:15:55.284, after 148.614 seconds, exceeding the 90-second setup limit.
- The script therefore did not create an observation-start/end interval.
- Stop confirmation was entered at 00:16:28.256, after USB had already expired.
  This does not exercise the ordinary q shutdown path and cannot confirm or
  refute that path's physical repair.
- The bridge footer reports runtime_ok=true but incomplete=true. Its output
  buffer filled at 136,770 entries and lost 11,660 entries. Input and screen
  losses were zero. The timeline cannot be treated as complete.

The PCAP is structurally complete: 216,970 target packets, including 9,197 motor
and 281 ring requests. Its SHA-256 is
`54247e19e14046c1e8281f8795ce214a5da11fb5b4a4f3ebda45da78b86b184f`.
Structural validity does not establish a complete tail or zero USB packet loss.

## Preservation and disposition

Only A exists in this pair; B did not start. No bridge, worker or USBPcap process
remained in the post-run inspection. The 13 Djay mapping hashes match the
preserved baseline. Of the 24 pre-run preference/log files, only
`Logs\last-session.log` changed through normal app use; its original is preserved
in the run's `preferences-before` snapshot.

The entire recording remains in its original desktop run directory. Small
results, observations, readiness, footer and preservation records are retained
under `.omo/evidence/s4-usb-a-20261122-01a1dbd8/`. No temporary build workspace
was created for this attempt, and no source, service, driver or registry change
was made while operating it.

After reporting the unusable instruction, the user offered a fresh A attempt
with that instruction ignored. The agreed procedure keeps the original driver,
uses post-connection track reload/Play/right TT, and retains the actual CHECK
authorization gate. This is an explicitly understood workaround for the
instruction defect, not a claim that the packaged prompt has been repaired.
