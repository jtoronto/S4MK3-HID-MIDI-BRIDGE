# Tension timeout recovery

## Practice-session report

On October 20, 2026 at 23:55:04 UTC, the full-profile bridge exited during
practice with:

```text
INFO TENSION_FINAL_NEUTRAL: native write accepted
Error: no fresh input within 50ms
EXIT: 1
```

This was an error exit from the motor freshness watchdog, not a recorded Rust
panic. The normal bridge propagated that error and stopped MIDI/LED service
along with tension. The preceding button/fader log lines are control changes,
not a record of every hardware snapshot. They do not identify what delayed
the reader or consumer.

## Recovery policy

The beta.3 fix keeps the 50 ms cutoff for motor input. When its deadline
is reached:

1. Request neutral for both wheels. If that write fails, exit with the error.
2. Log `TENSION_SUSPENDED` with the current time and input deadline.
3. Discard the old sensor baseline, movement qualification, and smoothing state.
   Keep force neutral while waiting for a new, fresh snapshot.
4. Continue processing control snapshots and MIDI feedback/LED updates.
   Physical controls can respond again when fresh snapshots arrive.
5. Use the first new snapshot only as a baseline and log
   `TENSION_INPUT_RECOVERED`. Each wheel must then qualify new movement before
   its assistance or braking returns, using the same touch rules and saved
   force limit as before the gap.

No Stop/Start is required. Fresh readings alone, stationary touch, or old
movement from before the gap cannot restore force. Assistance still requires
that wheel's top touch; braking can resume with fresh movement without touch.
The existing minimum qualification is 6 ms of same-direction movement and
6 counts, followed by the existing magnitude smoothing.

The saved slider value is not changed. Another input gap repeats the same
neutral, reset, baseline, and qualification sequence.
Native input/output errors, reader shutdown, and disconnection still stop the
bridge. The standalone diagnostics retain their original fail-stop behavior.

The watchdog checks before accepting a queued snapshot as well as before
force output. Snapshots timestamped at or before suspension cannot serve as
the new force baseline. Recovery candidates must also still be within the
50 ms freshness bound when processed; ones that have aged out while waiting
are skipped for tension. A new snapshot cannot hide an expired input history
or resume its old force.

This automatic recovery replaces the earlier working-tree policy that left
tension off until manual Stop/Start. The user requested the change on
October 21, 2026 to avoid restarting control translation during a set.

## Software evidence and limits

An injected-clock regression using the production scheduler, real tension
engine, and existing reader channels reproduced the exact `no fresh input
within 50ms` error before the fix. After the fix, the same scenario clears
force and continues dispatching fresh snapshots and feedback.

Additional regressions cover assistance, neutral, and braking; a fresh
snapshot queued at the deadline; movement requalification before force
returns; repeated gaps; per-wheel touch gates; stale recovery candidates;
and a failed neutral write that must remain fatal.
Existing Stop priority and native input/output, MIDI, and LED failure checks
remain in place.

The automatic-recovery regression first failed against the manual-restart
version because force never returned. A second regression reproduced a stale
candidate aborting recovery after a delayed neutral write; recovery now skips
that candidate for tension and waits for a fresh one.

The automatic-recovery source passed all 254 Rust and 18 Swift tests,
formatting, strict all-target/all-feature Clippy, the locked all-target build,
shell syntax, plist lint, and diff checks. Language-server diagnostics were
clean. Existing SwiftPM unhandled icon/diagnostic-script warnings remain.

These are software checks with recorded output actions, not physical force or
USB scheduling tests. The source of the original input gap remains unknown.
The recovery behavior still needs a focused check on the controller. No HID
or MIDI ports were opened and no motor commands were sent during these tests.

This change is included in `v0.1.1-beta.3`. Older `v0.1.1-beta.2` packages
still have the original fatal-timeout behavior. See
[release validation](release-0.1.1-beta.3.md).
