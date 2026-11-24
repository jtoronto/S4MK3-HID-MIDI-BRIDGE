# Jog free-play protocol and E/F baselines

October 19, 2026. **Cold recovery PASSED. Current phase: RIGHT ONLY FIRST.**
Use the compact setup and single right-only launch in
`docs/jog-tension-diagnostic.md`, packaged as the suite's README. Stop the
bridge and old diagnostic, quit Traktor and Mixxx, and explicitly `cd` into
the newly extracted suite folder; an old Terminal working directory may be
stale. The same powered, initialized controller is fine. No further cold
test is required by default.

The launch selects `fader-play`, side `right`, assistance **7000**, braking
**30000**, and `--initialize-rings --start-after-neutral --until-stopped
--enable-motor`, with a `wheel-right-<date>.jsonl` log and no `--seconds`.
Put **channel 1, SECOND FROM LEFT**, near CENTER and wait for READY.
Then play freely on the RIGHT wheel: DOWN assists, UP brakes. The LEFT must
remain unforced even if moved or touched. There are no prescribed gestures,
holds, or timed trial sequence. Stop with Ctrl-C, immediately if direction is
wrong or there is unexpected drive or chatter. Return the filename, right
assist/brake feel, any left-wheel force, and any Terminal error.

E and F remain separate baselines with unchanged modes. B/C and a D repeat
remain paused. The extension changes only `fader-play` side selection, not
the force law, cadence, watchdog, 32-snapshot FIFO, paced reader, or native
initialization packets.

## Historical cold-start comparisons and passed gate

`cold-start-20261019T224211Z.jsonl` reached READY and logged 3092 accepted
nonzero motor writes, with assist7000/brake30000 peaks, but the user felt
neither effect. Its final neutral-tick I/O error coincided with confirmed
Ctrl-C; final neutral was subsequently accepted. This doesn't establish
spontaneous device loss or identify a motor-enable packet.

The normal bridge sends two existing 28-byte reports starting
`[0x30,side,1,3]` with the remaining bytes zero. The diagnostic already
implements that pair under `--initialize-rings`; it is now also permitted
explicitly for `fader-play`, not added to its default path. Its controller/
timing/force law is unchanged. Other E/F startup-option exclusions remain.
The initial comparison on that cold-powered state restored the expected feel;
the ring was blue and followed jog movement. Traktor and the bridge stayed
closed. This supports a startup dependency without decoding firmware bits.
Preparation still gates all motor force until the stable-neutral READY.
Two `initialization` records document accepted 28-byte writes; they alone do
not prove physical motor readiness. The first fresh-power-off repeat,
`cold-restart-20261019T230507Z.jsonl`, aborted spontaneously on snapshot queue
overflow after 3912 processed samples. Both initialization writes succeeded,
but the fader remained neutral and no nonzero force was emitted. That is no
verdict on cold-start force recovery; final neutral was accepted.

The diagnostic reader now spaces acquisition starts by at least 2 ms using
an independent cancelable clock, rather than treating a maximum interrupt
read timeout as a minimum period. Late acquisitions do not catch up in
bursts. Native polling/force packets, the 32-frame FIFO, stale-input checks,
50 ms watchdog and fatal overflow remain unchanged. Overflow now reports
`rejected_snapshot_us`; disconnection is identified separately.
The terminal 32 queued frames were not logged, so the previous trace cannot
distinguish a burst from a consumer/write/logging stall. Explicit pacing
removes a proven pacing gap, not an established root cause for that overflow.
The paced retry, `cold-paced-20261019T232541Z.jsonl`, **passed cold recovery**.
It logged **15079 samples**, with the last snapshot at **35.639203 s**,
two accepted initialization writes, and READY at **7.200647 s**. All
**17813 motor writes were accepted**, with no errors or overflow. Assistance
peaked at **7000** and braking at **30000**, with both signs represented;
right-wheel requests stayed zero. It ended with `ctrl_c` and accepted final
neutral.

The user said "It worked. I forgot to return to neutral." and then explicitly
confirmed "Both worked correctly". This confirms left-wheel assistance and
braking, not both physical wheels. Manual return to CENTER wasn't tested in
this run; omitting it doesn't invalidate the passed cold gate.
**No more cold tests are required by default.**

## Fader-play controls and force contract

Only `fader-play` extends side selection to `left`, `right`, and `both`; its
default remains `left`. Other fader profiles remain left only, while fixed
modes remain left/right only. All `fader-play` sides use **channel 1, SECOND
FROM LEFT**, in physical order **3 | 1 | 2 | 4**, as the **shared tension
selector**, including right and both. It is never channel 2.
An unselected wheel always requests zero, even if moved or touched.

| Control or state | Result |
| --- | --- |
| Lower half, raw 0-2015 | Assistance ceiling rises from zero at CENTER to `--assist-force` fully DOWN, default 2000 |
| CENTER, raw 2016-2080 inclusive | Zero both wheels; clear both movement and filter histories |
| Upper half, raw 2081-4095 | Braking ceiling rises from zero at CENTER to `--max-force` fully UP, default 30000 |
| Each selected, touched wheel with its own fresh qualified movement in assistance | Force follows that wheel's movement |
| Release in assistance | Immediately zero only that wheel and clear its history; retouch needs fresh qualified movement |
| Qualified movement in braking | Force opposes movement, and may continue after release while moving, as in E |
| Still wheel or stationary retouch | No force |
| Any zone change, including a jump that skips CENTER | Immediately zero both wheels; reset both histories, then require fresh qualification |
| Observed reversal or inactivity cutoff on one wheel | Immediately zero only that wheel, then require fresh qualification |
| Unselected wheel | Always zero request; LEFT in the current right-only launch |
| Ctrl-C | Finish the run and send final neutral |

The two halves have **different, uncalibrated default ceilings**, 2000 assistance
and 30000 braking. The active launch uses **7000 assistance / 30000 braking**.
They aren't strength-matched, physical torque values, or a
claim about definite assistance strength. Fader selection is a ceiling, not
constant force. Don't raise either cap for this comparison.

Both halves use the proven E/F smooth qualified law: a **20 ms recent
same-direction displacement window**, **4-count deadband**, linear taper to
full target at **20 counts**, and direction qualification at **6 counts over
at least 6 ms**. Independent best-effort **2 ms output slots** smooth magnitude
with **Q16** fractions and actual elapsed microseconds:

```text
tau = 20000 us
y = (previous * tau + target * elapsed) / (tau + elapsed)
```

The hard **8 ms without real movement** cutoff zeros output and invalidates
movement history. Rest and duplicate samples don't refresh it. Assistance
requires metal-top touch plus fresh qualified movement; untouched movement
can't prime it. Braking follows E's movement qualification without requiring
touch. Release is an immediate zero gate for assistance, not for moving
braking.

Each physical wheel owns independent touch, direction, movement qualification,
filter, and inactivity state. Moving or touching one can't prime the other.
Assistance release, reversal, or stopping at the inactivity cutoff zeros only
that wheel immediately, before output ticks, while the other selected wheel
continues under its own qualified state. A single quantized rest sample isn't
the inactivity cutoff.

CENTER, any zone change, reversal, unqualified direction, Stop, fault, and the
**50 ms watchdog** bypass smoothing and zero output. No old history carries
across a zone change, even if no CENTER sample was observed. CENTER and zone
changes clear both wheels together. Startup and final
neutral are unconditional. Repeated zero Sample proposals for a wheel already
at zero do not add writes. A nonzero-to-zero transition writes immediately,
even if the other wheel remains active. Periodic clock reports continue
best-effort.

With `--start-after-neutral --until-stopped`, output stays zero until channel
1 has remained at CENTER for **one continuous second** and
`DIAGNOSTIC_TRIAL_READY` appears. Leaving CENTER during preparation resets that
hold. `armed=true` alone isn't READY. Preparation and free play have **no
deadline**; after READY the run lasts until Ctrl-C unless a fault stops it.
The preparation record has `seconds:null` and `neutral_hold_us:1000000`;
`host_us` remains relative to process start.

## Both-wheel live stage, gated on right-result review

Don't advance to both-wheel live testing until the right-only result has been
reviewed. Both selection uses the same channel 1 tension selector, per-wheel
touch, and independent movements and qualification. That later stage needs
the user's physical confirmation. Right-wheel polarity and both-wheel feel
aren't validated by the left cold pass, accepted writes, or offline replay.
There is no active both-wheel launch in this protocol.

## A and D: qualitative baselines

Preserve `continuity-A-left-20261019T191447Z.jsonl` as the qualitative A
assistance baseline and `continuity-A-brake-20261019T200443Z.jsonl` as the
qualitative A braking baseline. The assistance log includes neutral movement,
both directions, release zeros, and accepted final neutral, but no braking
phase or precisely matched gesture timing. A's initial assistance evidence is
partial and qualitative; its stationary-retouch check is incomplete.

During A braking, the user felt definite tension with chatter even after
release; it stopped at process exit. Analysis found **399 full-30000 responses
to alternating +2/-2 counts in 7.35 seconds of released motion**. The manual
return to CENTER was missed, so that check remains incomplete, although final
neutral was accepted. These observations motivate the hypothesis that small
alternating motion should not trigger full-cap braking. They don't establish
the cause of chatter or prove that D will feel smooth.

Keep `continuity-A-brake-20261019T193747Z.jsonl` separately as a failed timed
attempt, not a braking baseline. It produced no nonzero requests and ended
while neutral was being found. D removes that time pressure with manual stop.

Preserve `damped-brake-20261019T202900Z.jsonl` as the qualitative D braking
baseline. The user reported chatter **only while attempting movement**.
No nonzero requests followed release at **23.0762 s**; CENTER at
**35.921195 s** was quiet for **7.57 s**. Requested force changed from
**14968 to 8308 in about 2 ms** within the same clockwise direction.

D issued **42650 writes in 43.49 s**, about **981/s**, because zero Sample
writes duplicated periodic clock reports. At **43.492 s**, the run ended with
`IOHIDDeviceSetReport` general error **`0xE00002BC`**; final neutral was
accepted. The user **confirmed pressing Ctrl-C**. This doesn't establish
spontaneous controller loss or overload, and the accepted final write doesn't
make the error ending a clean pass.

## E: successful braking baseline

Preserve `smooth-brake-20261019T210513Z.jsonl`. The user reported
**"Definitely much smoother"**. The clean run lasted **41.592 s**, with
**21287 samples** and **20766 writes, all accepted**. Release at
**17.717071 s** was followed by quiet requested output. Final CENTER at
**36.093143 s** was quiet for **5.499 s**. Right-wheel requests stayed zero,
and the run ended with **`ctrl_c`**.

E tested smoother magnitude while moving and fewer redundant zero writes.
Its unchanged 30000 ceiling wasn't strength-matched to D: removing decay
can raise average force. This successful braking baseline doesn't establish
F assistance behavior.

## F: qualitative assistance baseline and unchanged contract

The user reported F felt **smooth** and believed it was assisting. They
confirmed the wheel was still during the held stop. This is qualitative
evidence, not a definite measurement of assistance strength.
The clean log lasted **61.060 seconds**, with **30973 samples**
and **30511 writes, all accepted**, ending with **`ctrl_c`** and accepted final
neutral. Untouched force and right-wheel requests stayed **zero**.
Stationary retouch lasted **about 2 seconds, not 5**; don't describe it as a
completed five-second hold.

The F baseline used **`--mode fader-assist --side left`**, not fixed `assist`.
`--assist-force` defaults to **2000** and permits **0-8000 inclusive**;
the baseline used **2000**. Zero disables assistance.
**`--max-force` is forbidden for F**,
and resolved braking force is **0**. The upper half is disabled.

Physical fader order is **3 | 1 | 2 | 4**. **Channel 1, second from the
left**, controls lower-half assistance from **0 at CENTER, raw 2016-2080
inclusive**, to the selected **2000 fully DOWN**. Selection isn't constant
output: lowering the fader with a still wheel must produce no immediate
force. The right wheel always requests zero. No TT or free spinning is added.

Assistance requires **real metal-top touch AND newly qualified same-direction
movement**, accumulating **6 counts over at least 6 ms**. Use a **20 ms recent
same-direction displacement window**, with **4 counts or less** as the
deadband, linear taper between **4 and 20 counts**, and full target at
**20 counts**. Smooth magnitude on independent best-effort **2 ms output
ticks**, retaining **Q16** fractions:

```text
tau = 20000 us
y = (previous * tau + target * elapsed) / (tau + elapsed)
```

Use actual elapsed microseconds. A separate **8 ms no-real-movement cutoff**
zeros output and invalidates prior movement. Stopping means that cutoff, not zeroing on
every quantized rest sample. Rest and duplicate samples don't refresh it.
**Release, CENTER, disabled upper half, reversal, unqualified direction,
Stop, fault, and the existing 50 ms watchdog immediately zero output**,
bypassing smoothing. Release, CENTER and the upper half clear motion and filter
history; reversal starts qualification for the new direction.
No untouched motion can prime the filter or qualification. Stationary retouch
never revives force; fresh touched movement must qualify again.

Retain zero-Sample dedup only when the last successful motor report is
already neutral. Periodic clock reports continue best-effort; immediate zero
transitions write. **Startup and final neutral are unconditional**, even when
dedup applies. No initialization is requested for F. Combined left-wheel
`fader-play` subsequently passed the cold gate above; F's separate result
alone doesn't establish that result or validate right/both behavior.

## Shared D/E movement and force contract

`fader-damped` (D) and `fader-smooth` (E) are opt-in, left only, with a default
braking cap of **30000** and a maximum of **60000**. Don't raise the cap for E.
Assistance is **disabled**: `--assist-force` defaults to **0**, and any nonzero
value is rejected. Below center provides no assistance; CENTER raw
**2016-2080 inclusive** selects zero and arms the session. Above center selects
the braking cap, reaching 30000 fully UP in this comparison. The right-wheel
request stays zero. Selected force is a cap, not constant applied force.

The controller uses a **20 ms recent same-direction displacement window**:

| Condition | Movement target or immediate zero gate |
| --- | --- |
| Displacement at or below 4 counts/20 ms | Zero |
| Displacement between 4 and 20 counts/20 ms | Linear taper from zero to the selected cap |
| Displacement at 20 counts/20 ms or above | Full selected cap, subject to qualification and zero gates |
| Direction has not accumulated at least 6 counts over at least 6 ms | Zero until qualified |
| Opposite motion observed | Immediate neutral, then requalify direction |
| No real movement for 8 ms | Zero; rest and duplicates don't refresh the cutoff |

The movement target precedes D's decay or E's smoothing. Reaching the target
cap doesn't mean E's filtered output immediately reaches that cap.

Output uses independent best-effort **2 ms slots**, rather than waiting for
each input poll, with a **50 ms watchdog** and no initialization.
Scheduling and accepted writes don't prove actual
physical smoothness or guarantee that a blocked/disconnected device receives
neutral. These raw force units are not calibrated physical torque.

## E smoothing and zero-write contract

D retains its per-count linear **8 ms decay**, including while moving. E
removes that decay while moving and applies a time-based first-order filter
to magnitude on **actual output ticks**:

```text
tau = 20000 us
y = (previous * tau + target * elapsed) / (tau + elapsed)
```

`elapsed` is actual elapsed time in microseconds, not an assumed 2 ms interval.
The calculation retains fixed-point fractions. The target comes from the
shared 20 ms movement window, 4-count deadband, 20-count full-scale threshold,
and 6-count/6-ms direction qualification.

The separate hard **8 ms no-real-movement cutoff** zeros immediately.
CENTER, the disabled lower half, observed reversal, unqualified direction,
Stop, fault, and watchdog zeros also bypass the filter. Smoothing must not
carry force through any of these zero gates.

E suppresses a redundant zero write on **Sample only**, and only when the
last **successful motor report** is already neutral. Periodic 2 ms clock
reports continue best-effort. Immediate zero transitions and final neutral
always write. D's write behavior is unchanged.

The `smoothing` metadata record contains `time_constant_us:20000`,
`rest_cutoff_us:8000`, and `zero_sample_dedup:true`. These describe the software
policy, not measured physical smoothness.

## Historical F readiness and scripted trial reference

F supports `--start-after-neutral --until-stopped`, with **no
`--seconds` option**, just as D and E do. This pair is permitted
in `fader-assist`, `fader-damped` and `fader-smooth`;
`fader-tension` and `fader-continuity` still reject it. Their existing
behavior is preserved.

During `DIAGNOSTIC_FIND_NEUTRAL`, all motor output is gated to zero. Hold
channel 1, second fader from the left, at raw **2016-2080 inclusive for one
continuous second**. Leaving the band resets preparation. Sticky
`armed=true` alone isn't readiness. The preparation JSON record has
**`seconds:null`** and `neutral_hold_us:1000000`.

**`DIAGNOSTIC_TRIAL_READY` means active until Ctrl-C, with no duration limit**,
unless a fault stops the session first. Preparation has no deadline either.
The `ready` record contains `host_us`; timestamps remain relative to process
start. Ctrl-C can also stop preparation, but a run without READY is incomplete.

The former F script requested neutral turns, assistance turns, a stopped-held
pause, release, stationary retouch, fresh movement, and a return to CENTER.
Its requested pauses were about five seconds. That script is historical,
not a next action or a checklist for free play. The actual stationary retouch
was about two seconds, and the user confirmed the held stop was still.

A clean logged exit has status 0, a `ready` record, an accepted
`final_neutral` motor record with exact bytes
`[49,1,32,1,0,0,1,32,1,0,0]`, `written:11`, and `error:null`, plus an
`end` reason of `ctrl_c`. Accepted bytes alone don't establish physical removal
of drive. An I/O error remains an error ending even if final neutral succeeded.

## Optional replay reference

The new bundled stage fixture is `jog-diagnostic-wheels-replay.jsonl`.
Pass it to `replay --input` with `--mode fader-play`, side `right` or `both`,
and `--max-force 30000 --assist-force 7000`. This offline reference is optional,
not a human prerequisite or a request for another capture. Offline `both`
replay doesn't open the gated both-wheel live stage. Existing bundled assist
and damped fixtures remain left-wheel references. Replay opens no HID device
and sends no reports; it doesn't verify physical coupling, right polarity,
both-wheel feel, or real output timing.

## Archived A/B/C reference, not next actions

The former sequence compared A (`fader-tension`) with B
(`fader-continuity`), then B with C (continuity plus
`--initialize-rings`). **B/C and a D repeat remain paused.** Don't perform
assistance or initialization as part of E. Old timed split commands and
fresh-start capture instructions are superseded by the free-play setup.

The old modes retain their behavior. Continuity starts a full-selected-force
envelope on real movement and decays it linearly to zero after 8000
microseconds without movement; rest and duplicates don't refresh it. CENTER,
zone transitions, and assist release clear it. Retouch without new movement
doesn't revive it, and raising the fader alone can't amplify retained force.
Continuity uses independent best-effort 2 ms output slots, skipping missed
slots without catch-up bursts. Its old stable-neutral timed preparation stays
separate from manual stop.

The archived initialization option sends two native 28-byte numbered 0x30
reports once under guarded neutral. It may affect LEDs and isn't a proven
motor enable. Neither D, E, nor F requests it. The older continuity fixture remains
software-only evidence.

## Archived bipolar and legacy modes

`fader-tension` uses physical channel 1, second from the left. Raw 2016-2080
inclusive selects zero and arms the session. Starting below or above that band
can't arm it. Below center, selection rises to 8000 at raw 0 by default, but
assistance applies only while the left wheel is touched and moving. Release
gates assistance to zero. Above center, braking rises to 30000 at raw 4095 by
default and can continue after release while moving. Rest, duplicate snapshots,
unarmed motion, and the right wheel always propose zero.

`--assist-force` accepts 0-8000 in `fader-tension` and `fader-continuity`;
0 disables assistance.
`--max-force` accepts 1-60000 here, with 30000 as the default. A higher cap is a
separate user-chosen experiment, never an automatic progression to 60000.
The three archived fader modes reject `--gain` and `--side right`, and support
`--until-stopped`. The optional legacy `fader-brake` mode is unchanged:
minimum raw 0-32 arms it, its default is 5000, and its ceiling remains 25000.

Held-touch assistance is deliberately restricted compared with native free
rotation. The piecewise fader policy is not a Traktor curve clone.

## Physical channel-1 input

The channel faders are ordered `3 | 1 | 2 | 4`; channel 1 is second from the left.
It is not a deck-layer-dependent MIDI control in this diagnostic.

The [pinned Encdr descriptor, line 158](https://github.com/RufusIbiza/Encdr/blob/5d7a689ca5789161b3efd1b2049d7e54504fe881/encdr/descriptors/ni_kontrol_s4_mk3.json#L158)
puts `ch1_fader` at payload bytes 2-3 in the 78-byte sliders report. Its
`max_value: 65535` is not the correct normalization range.

[Pinned Mixxx, line 1457](https://github.com/mixxxdj/mixxx/blob/f12661e29b36cfdd261939674f1ab94c52f1b2d8/res/controllers/Traktor-Kontrol-S4-MK3.js#L1457)
assigns channel-1 volume to `inByte: 2`. Its original `HIDInputReport` decodes
little-endian fields, and `incomingData` removes the numbered report-ID byte.
`Pot.prototype.inBitLength = 16` and
[`Pot.prototype.max = 2 ** 12 - 1`, line 1808](https://github.com/mixxxdj/mixxx/blob/f12661e29b36cfdd261939674f1ab94c52f1b2d8/res/controllers/Traktor-Kontrol-S4-MK3.js#L1808)
independently establish the 4095 range. The existing bridge already corrects
this descriptor range in `src/main.rs`.

Our capture stores numbered report 2, exactly 79 bytes including ID. It decodes
numbered bytes 3-4 and rejects a short/wrong-ID report or a value above 4095.
Historical captures without analog remain valid for fixed diagnostic modes;
the archived fader modes reject missing analog rather than inferring a level.

## Native capture evidence and limits

The native evidence has inferred phases, not calibrated torque settings:

| Inferred native phase | Observed magnitude |
| --- | --- |
| 0, assistance | Median 5159, peak 7829 |
| 127, braking | Median 22170, peak 30312 |
| 255, braking | Median 41927, peak 60000 |

These captures use the same 11-byte numbered `0x31` report structure described
below. They motivate separate assistance and braking bounds, but don't prove
the meaning of every native phase or reproduce its motion-dependent curve.
Those captures alone do not establish the macOS wire effect or physical feel. The existing mode
doesn't add TT-mode, ring initialization, screen traffic, or other new
initialization traffic by default. Ring reports are a separate explicit option.

Existing captures also show roughly 2 ms native output intervals versus
2.989 ms diagnostic intervals, and native nonzero output after zero-count
input. These observations motivated the now-paused continuity comparison. They
don't establish USB endpoint equivalence, a firmware watchdog requirement,
the native motion-dependent curve, or a physical benefit from either change.

## Motor payload and HIDAPI boundary

The original [Mixxx motor manager, lines 3303-3384](https://github.com/mixxxdj/mixxx/blob/f12661e29b36cfdd261939674f1ab94c52f1b2d8/res/controllers/Traktor-Kontrol-S4-MK3.js#L3303)
starts each wheel block as `[1,32,1,0,0]`. Negative signed requests replace the
middle word with bytes `[224,254]`; the last two bytes hold the bounded
unsigned magnitude, little-endian. The left block precedes the right block.

Its [original `motorCallback`](https://github.com/mixxxdj/mixxx/blob/f12661e29b36cfdd261939674f1ab94c52f1b2d8/res/controllers/Traktor-Kontrol-S4-MK3.js#L3746)
sends a ten-byte payload with report ID 49 (`0x31`) and strict FIFO enabled.
The [native FIFO, lines 26-28 and 70-72](https://github.com/mixxxdj/mixxx/blob/f12661e29b36cfdd261939674f1ab94c52f1b2d8/src/controllers/hid/hidioglobaloutputreportfifo.cpp#L26)
prepends that report ID before `hid_write`. Rust HIDAPI 2.6.7 likewise requires
the first byte to contain the ID. Both paths therefore submit **11 bytes**;
comparing Mixxx's ten-byte wrapper payload with our eleven-byte HIDAPI buffer
does not establish a transport discrepancy.

Examples including the mandatory report ID:

| Left request | Complete HIDAPI bytes; right wheel zero |
| --- | --- |
| Zero | `49,1,32,1,0,0,1,32,1,0,0` |
| -5000 | `49,1,224,254,136,19,1,32,1,0,0` |
| +5000 | `49,1,32,1,136,19,1,32,1,0,0` |
| -25000 | `49,1,224,254,168,97,1,32,1,0,0` |

## Original-source offline oracle

The complete upstream script at revision
`f12661e29b36cfdd261939674f1ab94c52f1b2d8` was fetched and archived locally.
Its SHA-256 is
`c058a168d537ccaf0374701004755af344c62a395d55e7d9f5aa668e9a8597c8`.
The executed regions were unchanged original `HIDInputReport`, `Component`,
`Pot`, `S4Mk3MotorManager`, `incomingData`, and `motorCallback` code.
Only Mixxx's engine/output boundary was captured, not its parser or encoder.

For packet comparison, synthetic jog speeds were chosen inside each desired
integer-magnitude floor bucket. This tests encoding and callback assembly,
not whether Mixxx's tension algorithm equals our direct fader policy.
For the earlier legacy `fader-brake` implementation, all 33 packets from actual
diagnostic replays at caps 3666, 5000, and 25000
matched the original manager/callback output exactly, including neutral,
negative and positive requests, and the right-wheel zero block.
The original input parser and potentiometer also matched all 11 fader positions.

Local evidence is in `.omo/evidence/jog-fader/protocol-oracle.json`,
`replay.jsonl`, `cli-errors.json`, and archived upstream sources.
These are local validation artifacts, not hardware captures. This earlier
oracle doesn't establish bipolar-policy correctness or acceptance at 30000
or 60000. The new synthetic `jog-diagnostic-tension-replay.jsonl` fixture is
packaged separately; its archived checks covered zones, endpoints, zero gates, and
completion without claiming a full expected trace.

Protocol agreement does not establish firmware interpretation, physical torque,
feel, or successful removal of drive. The diagnostic's raw values are not
factory tension settings or physical safety limits. Those conclusions require
physical evidence, not replay or accepted writes alone.
