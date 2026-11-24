# Wheel-tension diagnostic

October 19, 2026. **Cold recovery PASSED. Current phase: RIGHT ONLY FIRST.**
Use the RIGHT wheel with **channel 1, SECOND FROM LEFT**, in physical order
**3 | 1 | 2 | 4**, as the shared tension selector. Don't switch to channel 2.
The new build extends only `fader-play` to `--side left`, `right`, or `both`;
the default remains `left`. Other fader profiles remain left only, and fixed
modes remain left/right only. E braking and F assistance stay unchanged.
Right-wheel polarity and both-wheel feel still need physical confirmation.

## Setup and launch

Stop any old diagnostic and the bridge; quit Traktor and Mixxx and keep the
bridge app closed. The same powered, initialized controller is fine.
**No further cold test or power cycle is required by default.**
Explicitly **cd into the newly built
suite's extracted folder** containing `s4mk3-jog-diagnostic`: type `cd `,
drag that new folder from Finder into Terminal, then press Return. An old
Terminal window may still point at an older folder. Preserve existing logs.

Put channel 1 near CENTER and paste this one launch block. The lines after
the diagnostic print its exit code, end record, final-neutral record, and
filename once it stops:

```sh
check="wheel-right-$(date -u +%Y%m%dT%H%M%SZ).jsonl"
./s4mk3-jog-diagnostic live --out "$check" \
  --mode fader-play --side right --max-force 30000 --assist-force 7000 \
  --enable-motor --initialize-rings --start-after-neutral --until-stopped
check_status=$?
printf 'Right-wheel exit code: %s\n' "$check_status"
grep '"kind":"initialization"' "$check"
grep '"kind":"end"' "$check"
grep '"kind":"motor","reason":"final_neutral"' "$check"
printf 'Right-wheel log: %s\n' "$check"
```

During `DIAGNOSTIC_FIND_NEUTRAL`, output stays zero. CENTER is raw
**2016-2080 inclusive**. Keep it there for **one continuous second** until
**`DIAGNOSTIC_TRIAL_READY`** appears; leaving the band resets preparation.
`armed=true` alone isn't READY. There is no preparation deadline or run
deadline, and no `--seconds` option in this launch.

After READY, **play freely on the RIGHT wheel**: channel 1 DOWN selects
assistance, CENTER selects neutral, and UP selects braking. The LEFT wheel
must remain unforced even if moved or touched. There are no prescribed
motions, holds, timings, or trial checklist. Don't raise either cap.
Press **Ctrl-C** to finish, or immediately for wrong direction, unexpected
drive, or chatter. A fault can also stop the run. Return the filename,
how right assistance and braking felt, whether there was any left-wheel
force, and the complete Terminal error if one appeared.

The launch retains **`--initialize-rings`**: two existing 28-byte
`0x30` startup reports, also sent by the normal bridge. Expect two
`initialization` records with `written:28` and `error:null`. Startup motor
neutral precedes them; all motor output remains gated until READY.
They may change ring LEDs. Cold-start testing supports this startup sequence;
the individual firmware-enable fields have not been decoded. The positive
physical contrast is recorded below. Without the flag, initialization remains
omitted. No new packet format is introduced.

This comparison retains the passed cold run's **7000 assistance / 30000 braking**
ceilings. The ordinary free-play default is still 2000 assistance; the table
below distinguishes those defaults from this launch.

The input queue remains bounded to 32 snapshots; overflow, stale input and
the 50 ms watchdog remain errors. Snapshots are not discarded to make room,
and no failed publication is silently retried. The previous capture cannot
prove whether its terminal overflow was a producer burst or a consumer/I/O
stall. The existing paced reader removes the known unpaced-burst path; it doesn't promise
to tolerate a stalled writer. If an error recurs, return its complete text,
including `rejected_snapshot_us`, along with the log.

## Simple controls

| Control or state | What it does |
| --- | --- |
| Channel 1, SECOND FROM LEFT, order 3 \| 1 \| 2 \| 4 | Shared selector for the selected side; RIGHT only in this launch, never channel 2 |
| Lower half, raw 0-2015 | Assistance, rising from zero at CENTER to 7000 fully DOWN in this launch; default 2000 |
| CENTER, raw 2016-2080 inclusive | Zero force and cleared movement/filter history for both wheels |
| Upper half, raw 2081-4095 | Braking, rising from zero at CENTER to a 30000 ceiling fully UP |
| Touch plus fresh qualified movement in assistance | Force follows movement |
| Release in assistance | Immediate zero; fresh touched movement must qualify again |
| Qualified movement in braking | Force opposes movement; may continue after release while moving, as in E |
| Still wheel or stationary retouch | No force |
| Any zone change, even a jump that skips CENTER | Immediately zero both wheels, reset both histories, then freshly qualify movement |
| Reversal or inactivity cutoff on one wheel | Immediately zero only that wheel, then require fresh qualification |
| Unselected wheel | Always zero request, even if moved or touched; LEFT in this launch |
| Ctrl-C | End the run and send final neutral |

The halves have **different ceilings**, not matched strengths: assistance
defaults to **2000** and braking to **30000**. This launch explicitly uses
**7000 assistance / 30000 braking**.
These are uncalibrated diagnostic units, not torque settings. Selection is a
ceiling, not constant output or a claim of definite assistance strength.

## Fader-play movement and output rules

Both halves use the proven E/F smooth qualified law. A recent **20 ms
same-direction displacement window** has a **4-count deadband**, then tapers
linearly to full target at **20 counts**. Direction must accumulate **6 counts
over at least 6 ms**. Magnitude smoothing uses **Q16** fractions on independent
best-effort **2 ms output slots**, with actual elapsed microseconds:

```text
tau = 20000 us
y = (previous * tau + target * elapsed) / (tau + elapsed)
```

A hard **8 ms without real movement** cutoff zeros output and invalidates
prior movement. Rest or duplicate samples don't refresh it. Assistance needs
metal-top touch and fresh qualified movement; untouched motion can't prime
it, and release immediately zeros and clears its history. Braking opposes
qualified movement and may continue after release while moving, as in E.

Each physical wheel has independent touch, direction, movement qualification,
filter, and inactivity state. Moving or touching one can't prime the other.
Assistance release, reversal, or the inactivity cutoff immediately zeros only
the affected wheel before output ticks; the other selected wheel continues
under its own qualified state. Braking retains E's moving-after-release rule.

Every zone change zeros both wheels and clears both movement and filter histories,
including direct assistance-to-braking or braking-to-assistance changes that
skip CENTER. Fresh movement must qualify in the new zone. CENTER, reversal,
unqualified direction, Stop, fault, and the **50 ms watchdog** also bypass
smoothing and zero immediately.

Repeated zero Sample proposals do not add writes for a wheel already at
zero. Any nonzero-to-zero transition writes immediately, including a partial
neutral report while the other wheel remains active. Periodic clock reports
continue best-effort. Startup and final neutral are unconditional. Ring
initialization is omitted unless explicitly requested, as in the launch above.
The force law, cadence, 50 ms watchdog, 32-snapshot FIFO, paced reader, and
native initialization packets are unchanged. No TT or free-spinning mode is requested.

The preparation record has `seconds:null` and `neutral_hold_us:1000000`.
READY starts free play until Ctrl-C, unless a fault stops it. `host_us` stays
relative to process start. A clean log ends with exit code 0, `end` reason
`ctrl_c`, and an accepted `final_neutral` report
`[49,1,32,1,0,0,1,32,1,0,0]`, with `written:11` and `error:null`.
Accepted writes don't prove physical removal of drive. An error ending isn't
a clean exit even if final neutral was accepted.

## Passed cold gate and historical startup comparisons

`cold-paced-20261019T232541Z.jsonl` **passed cold recovery**: 15079 samples,
last snapshot at **35.639203 s**, two accepted initialization writes, READY at
**7.200647 s**, and all **17813 motor writes accepted**. There were no errors
or overflow. Assistance peaked at **7000** and braking at **30000**, with both
signs represented; right-wheel requests stayed zero. The run ended with
`ctrl_c` and accepted final neutral.

The user first reported "It worked. I forgot to return to neutral." and then
explicitly confirmed "Both worked correctly", meaning assistance and braking
on the left wheel, not both physical wheels. Manual return to CENTER wasn't
tested in this run; its omission doesn't invalidate the cold gate.
**No more cold tests are required by default.**

Earlier, `cold-start-20261019T224211Z.jsonl` produced 3092 accepted nonzero
writes without either effect being felt after a power cycle. Explicit native
initialization restored the expected feel, with a blue ring tracking movement.
The first fresh-power repeat, `cold-restart-20261019T230507Z.jsonl`, overflowed
the snapshot queue after 3912 samples while still neutral; both initialization
writes and final neutral were accepted. It didn't test either force zone.
The passed paced run supersedes that retry, without proving the cause of the
earlier overflow or decoding a firmware motor-enable specification.

## Both-wheel stage, gated on right-result review

Both-wheel live testing is **not the current action**. Review the right-only
log and physical feel report before advancing. The `both` selection uses the
same channel 1 selector, per-wheel touch, and independently qualified
movements. It needs separate user physical confirmation; neither right-wheel
polarity nor both-wheel feel is established by the left cold pass or replay.

## Historical A/D/E/F baselines

Keep these logs as **qualitative baselines**, not precisely matched trials:

| Log | What it establishes and what it doesn't |
| --- | --- |
| `continuity-A-left-20261019T191447Z.jsonl` | A assistance: neutral movement, both assistance directions, release zeros, and accepted final neutral. No braking phase; gesture timing and a full stationary-retouch check weren't established. |
| `continuity-A-brake-20261019T200443Z.jsonl` | A braking: the user felt definite tension but chatter even after release, which stopped at process exit. Manual return to CENTER was missed; final neutral was accepted. |
| `damped-brake-20261019T202900Z.jsonl` | D braking: the user reported chatter only while attempting movement. No nonzero requests followed release at 23.0762 s. CENTER at 35.921195 s was quiet for 7.57 s. The run ended with an I/O error coinciding with user-confirmed Ctrl-C; final neutral was accepted. |
| `smooth-brake-20261019T210513Z.jsonl` | E braking: the user said "Definitely much smoother". Clean 41.592 s run, 21287 samples, 20766 writes, all accepted. Release at 17.717071 s was followed by quiet requested output. Final CENTER at 36.093143 s was quiet for 5.499 s. Right-wheel requests stayed zero; the run ended with `ctrl_c`. E is the successful braking baseline, not evidence of F assistance. |
| `smooth-assist-20261019T221653Z.jsonl` | The user reported smooth feel and believed it was assisting; definite assistance strength isn't established. The held stop was confirmed still. Clean **61.060 s**, with **30973 samples** and **30511 writes, all accepted**, ending with `ctrl_c` and accepted final neutral. Untouched force and right-wheel requests stayed zero. Stationary retouch was **about 2 s, not 5 s**. |

The A braking analysis found **399 full-30000 responses to alternating +2/-2
counts during 7.35 seconds of released motion**. The working hypothesis is
that full-cap responses to small direction changes contributed to chatter.
D adds a movement threshold, taper, direction qualification, and a short
decaying tail to test that hypothesis. It doesn't establish the cause or prove
smooth physical braking. D remains unchanged, including its linear 8 ms decay.

In D, requested force jumped from **14968 to 8308 in about 2 ms** while motion
remained clockwise. The log contains **42650 writes in 43.49 s**, about
**981/s**, because zero Sample reports duplicated periodic clock reports.
At **43.492 s**, `IOHIDDeviceSetReport` returned general error **`0xE00002BC`**.
The user confirmed pressing Ctrl-C, so this isn't proof of spontaneous
controller loss or overload. Accepted final neutral doesn't turn the error
ending into a clean pass.

E tested smoothing while moving and suppression of redundant zero Sample
writes, with the successful physical result above. The same
30000 ceiling is **not strength-matched**: removing D's decay can raise average
force and change the feel. Preserve this baseline without changing E to enable
assistance. A's initial assistance evidence is partial and qualitative, with
the stationary-retouch check incomplete. F's qualitative result is recorded
above; combined left-wheel E/F physical coupling in `fader-play` subsequently
passed the cold gate recorded above. Right/both physical behavior remains unvalidated.

Also preserve `continuity-A-brake-20261019T193747Z.jsonl` as a failed timed
attempt, not a braking baseline: it produced no nonzero requests and ended
while neutral was still being found. Repeated time pressure is why D uses
manual stop rather than `--seconds`.

## Historical F assistance control and readiness contract

The unchanged `--mode fader-assist` is left only, separate from fixed `assist`.
`--assist-force` defaults to **2000**, permits **0-8000 inclusive**, and
was explicitly **2000** for the baseline. Zero disables assistance. The range
is a validation bound, not a procedure for increasing force.
**`--max-force` is forbidden for F; resolved braking force is 0.**
The upper half is disabled, never braking. Channel 1 is second from the left
in physical order **3 | 1 | 2 | 4**. Its lower half selects assistance from
**0 at CENTER, raw 2016-2080 inclusive**, to the selected **2000 fully DOWN**.
The right-wheel request is always zero. There is no TT or free-spinning mode.

Real metal-top touch **and newly qualified same-direction movement** are
required. Direction qualifies at **6 counts over at least 6 ms**. A recent
**20 ms same-direction displacement window** supplies the movement target:
**4 counts or less** is the deadband; between **4 and 20 counts** it tapers
linearly to the selected cap; **20 counts** reaches full target.

On independent best-effort **2 ms output ticks**, smooth magnitude using
**Q16** fixed-point fractions and actual elapsed time:

```text
tau = 20000 us
y = (previous * tau + target * elapsed) / (tau + elapsed)
```

A separate **8 ms without real movement** cutoff zeros output and invalidates
prior movement. Stopping means this cutoff; a single quantized rest sample doesn't
zero otherwise-qualified output. Rest or duplicate samples don't refresh
the cutoff. **Release, CENTER, disabled upper half, reversal, unqualified
direction, Stop, fault, and the existing 50 ms watchdog immediately zero
output.** These gates bypass smoothing. Release, CENTER and the upper half
clear motion and filter history; reversal starts qualification for the new
direction. Untouched motion cannot prime the filter or qualification;
stationary retouch never revives assistance. Fresh touched movement must
qualify again.

Retain zero-Sample dedup only when the last successful motor report is already
neutral. Periodic clock reports continue best-effort; immediate zero
transitions write. **Startup and final neutral are unconditional**, regardless
of dedup. No initialization is requested.

F uses the same no-deadline, one-second stable-neutral preparation and
`--start-after-neutral --until-stopped`, with **no `--seconds`**.
The preparation record has `seconds:null` and `neutral_hold_us:1000000`.
Output stays zero until `DIAGNOSTIC_TRIAL_READY`; `armed=true` alone isn't
READY. Leaving CENTER resets the continuous hold. After READY there is no
duration limit, unless a fault stops the run. `host_us` stays relative to
process start. Ctrl-C can stop preparation, but that isn't a completed trial.

These are approved software rules in uncalibrated diagnostic units.
F's qualitative physical result is recorded above; these rules don't establish
definite assistance strength or guarantee delivery to a blocked or disconnected
device. The former scripted F trial is historical reference in
`docs/jog-fader-braking-protocol.md`, not a next action.

## Historical E control reference, braking only

`fader-smooth` is opt-in and left only. Its default braking cap is **30000**,
with a maximum of **60000**, not a requested next step.
Assistance is **disabled**: `--assist-force` defaults to **0**, and any nonzero
value is rejected. CENTER remains raw **2016-2080 inclusive** for arming;
below center provides no assistance. The right-wheel request remains zero.

E reuses D's recent **same-direction displacement over 20 ms**:

- At **4 counts/20 ms or less**, the movement target is zero.
- Between **4 and 20 counts/20 ms**, the target magnitude tapers linearly from
  zero to the selected cap; **20 counts/20 ms** reaches that target cap.
  E's filtered output need not reach the target immediately.
- Direction must accumulate **at least 6 counts over at least 6 ms** before
  braking qualifies. Observed opposite motion immediately neutralizes output
  and requires direction to qualify again.
- E removes D's per-count linear **8 ms decay while moving**. On actual output
  ticks, it smooths magnitude with a time-based first-order filter:
  `y = (previous * tau + target * elapsed) / (tau + elapsed)`, where
  `tau = 20000 us` and `elapsed` is actual elapsed time in microseconds.
  Fixed-point fractions are retained.
- A separate hard **8 ms without real movement** cutoff immediately zeros
  output. Rest and duplicate snapshots don't refresh it.
- CENTER, the disabled lower half, observed reversal or unqualified direction,
  Stop, fault, and the **50 ms watchdog** bypass the filter and zero immediately.
- Output uses independent best-effort **2 ms slots**, without initialization.
  Only a zero **Sample** write is suppressed when the last successful motor
  report is already neutral. Periodic clock reports continue every 2 ms
  best-effort; immediate zero transitions and final neutral always write.

The `smoothing` metadata record contains `time_constant_us:20000`,
`rest_cutoff_us:8000`, and `zero_sample_dedup:true`.

These are software control rules and uncalibrated diagnostic units, not
physical torque settings or a hardware failsafe. E's physical result is
recorded above. A blocked or disconnected device may not receive neutral.

`fader-play`, `fader-damped`, `fader-smooth`, and `fader-assist` permit
**`--start-after-neutral --until-stopped` together**.
The old `fader-tension` and `fader-continuity` modes still reject that pair.
For D and E, the `preparation` JSON record has **`seconds:null`** and
`neutral_hold_us:1000000`. Output stays zero throughout preparation.
The `ready` record marks activation, not the start of a duration limit.
`host_us` remains relative to process start. Ctrl-C can also stop preparation,
but a run without READY isn't a completed comparison.

## Optional offline free-play replay, no HID

This is optional, not a human prerequisite or a request for another capture.
The new bundled fixture for this stage is `jog-diagnostic-wheels-replay.jsonl`.
It can be passed to `replay --input` with `--mode fader-play`, side `right`
or `both`, and `--max-force 30000 --assist-force 7000`. Offline `both` replay
doesn't open the gated both-wheel live stage. The existing assist and damped
fixtures remain left-wheel references.

Expect exit status 0 and an `end` record. Replay opens no HID device and sends
no output reports. Its proposals can help inspect software decisions, but
don't verify physical feel, USB delivery, or real output timing.

## Archived comparisons, not next actions

Existing `fader-tension`, `fader-continuity`, and `fader-damped` behavior is
preserved.
The earlier plan compared A (`fader-tension`), B (`fader-continuity`), and C
(continuity plus `--initialize-rings`), with separate assistance and braking
runs. **B/C and a D repeat are paused.** Their old combined or split
walkthroughs are not the current procedure. Don't repeat A/D or request another
input capture. The active right-only launch explicitly retains ring
initialization. E remains braking only and F
remains assistance only; neither baseline mode is changed by `fader-play`.

Protocol history, packet layout, and the earlier offline oracle remain in
`docs/jog-fader-braking-protocol.md`. The suite packages this guide as its
`README.md`; use the new executable for `fader-play`, not an older extracted
copy. The only active procedure is the compact free-play setup above.
