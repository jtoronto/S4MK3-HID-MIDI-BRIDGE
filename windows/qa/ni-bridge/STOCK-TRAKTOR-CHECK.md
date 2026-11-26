# Stock Traktor JOG reference capture

This is a future, user-attended Windows capture, not a bridge trial. It observes
stock Traktor's requests and NIHardwareService's matching output submissions;
it does not send bridge requests, change a service, or install anything. Frida
does attach hooks to running processes, so "read-only" describes the observed
controller state, not a non-invasive capture. This procedure has not been run.

## Question and limits

Compare the left deck in paused JOG and playing JOG, first hands off and then
with the same light manual rim nudge. Record the actual ring and detent feel,
not just requests. The retained `33b3acf2` and `d89a4875` IPC traces contain
mode 1 and `MotorOn(true)` with stale phase markers, so they cannot establish
which action caused those requests. Native renderer analysis indicates this
state can change JOG detent force as well as advance the needle; it is not a
force-neutral animation switch.

The offline stock macOS Traktor 4.4.1 build 22 QML inspection is
independent evidence of intent: `S4MK3Deck.qml:120` connects `.play` to
`jogwheel.motor_on`, with a comment that it tells HWS about playback in JOG;
`S4MK3.qml:26` separately maps `haptic.tension`. This does not establish zero
force, physical output, or a TT setting's effect.

Native Instruments' *Traktor Kontrol S4 MK3 Manual*, English 0719,
“Preferences” (p. 83), lists Jogwheel Tension under Haptic Drive. It
lists Base (platter RPM) as the only TT Mode preference. That does not establish
whether Haptic Drive tension reaches the TT renderer. Do not change
bridge force settings or use the bridge to test this question.

## Prepare, then wait for the observer

1. Use the existing complete `S4 Traktor IPC Trace-20261123-178b94.zip` and
   `S4 Output Writer Trace-20261122-9fbe057e-r2.zip` from `dist`. Extract each
   into its own new Windows folder. Keep their original files and record the
   package names. Do not use `Start-WriterTrace.ps1`, `08-writer-trace.cmd`,
   or the left-jog diagnostic launcher: they prepare or launch a bridge
   diagnostic rather than a stock-only Traktor run.
2. Close Traktor, Djay, every bridge and screen helper normally. Leave the
   already-running NIHardwareService alone; do not restart it. Connect only
   the target S4. Stock Traktor will be the only S4 client; the NI service is
   its output owner. The observer attaches to the existing service and does
   not acquire a second S4 session. If ownership, service identity, or a
   stopped competing client is uncertain, stop rather than proceeding.
3. Lower audio, clear both platters, keep the power switch within reach, and
   keep the right deck paused in JOG throughout. The operator must be present
   and watching before either observer starts. If a platter moves unexpectedly
   or a stop cannot be confirmed, switch off the S4. Record stock Traktor
   version, track, deck, original preferences and any deviations.

## Observe the service, then start stock Traktor

The two observers each have a 180-second collection limit. Prepare the
consoles first; perform the short comparison promptly. Neither timeout stops
Traktor or proves the platter stopped.

1. Open **Windows PowerShell as administrator** in the extracted output
   writer folder. The following uses the existing packaged `WriterSession`,
   not its bridge-launching script. It requires exactly one service and saves
   the observer's stdout and stderr in a new unique local capture folder:

   ```powershell
   $p = @(Get-Process -Name NIHardwareService -ErrorAction Stop)
   if ($p.Count -ne 1) { throw 'Expected exactly one NIHardwareService' }
   $src = @('WriterSession.cs','WriterSession.Protocol.cs','WriterSession.Process.cs') | ForEach-Object { Join-Path $PWD $_ }
   Add-Type -Path $src -ReferencedAssemblies @('System.dll','System.Core.dll','System.Xml.dll','System.Runtime.Serialization.dll')
   $run = Join-Path $PWD ('stock-captures\' + [guid]::NewGuid().ToString('N'))
   $p[0] | Select-Object Id,Path
   Get-FileHash -LiteralPath $p[0].Path -Algorithm SHA256
   $s = [WriterSession]::new((Join-Path $PWD 'frida-inject.exe'), $p[0].Id, (Join-Path $PWD 'writer-agent.js'), $run)
   $s.Start([TimeSpan]::FromSeconds(10))
   "NI OBSERVER READY: $run"
   ```

   Record the displayed PID, service hash and folder. If this fails, do not
   start Traktor; preserve the error and any created folder. The ready message
   means hooks attached, not that output is correct.
2. Open **Command Prompt as administrator** in the extracted IPC trace folder.
   Use the actual stock executable path from its shortcut; the documented
   default example is:

   ```cmd
   Start-Traktor-IPC-Capture.cmd "C:\Program Files\Native Instruments\Traktor Pro 4\Traktor.exe"
   ```

   Wait for `CAPTURE READY` and the Traktor window. If either fails, quit
   Traktor normally if open, stop the observer as below, and preserve errors.
   Load a track on Deck A; confirm left JOG and paused in the stock UI.
3. In the IPC console type each marker and wait for `MARKED` before the
   corresponding action. Only `READY_IDLE`, `TT`, `PLAY`, `PAUSE`, `JOG`
   and `QUIT` are accepted. Reuse markers as shown; write down the order and
   wall-clock time of each nudge separately. Do not invent marker names.

   | Marker | Action and observation |
   | --- | --- |
   | `READY_IDLE` | Paused JOG: hands off; note ring movement and wheel stillness. |
   | `READY_IDLE` | Still paused: one light rim nudge; note direction, approximate distance and felt detents, then hands off. |
   | `PLAY` | Press ordinary Play. Hands off: note whether the ring advances and whether the wheel itself moves. |
   | `PLAY` | While still playing, repeat a comparably light rim nudge in the same direction and distance; note detents and ring response. |
   | `PAUSE` | Pause normally. Confirm the wheel is stopped; note the ring. |
   | `QUIT` | Quit Traktor normally and wait for the IPC launcher to return. |

   Keep both hands off except for the two nudges. Do not infer force from
   ring motion or use a stronger push to obtain a result. If a marker is
   missing, late, or the collector times out, label that portion uncorrelated;
   do not treat the previous marker as the current action.
4. In the writer PowerShell console, stop and detach the observer:

   ```powershell
   $s.Stop([TimeSpan]::FromSeconds(5))
   $s.Dispose()
   ```

   If it has already timed out or stop fails, record the error and dispose
   the session; do not restart a service or kill another process. Confirm
   both wheels are physically stopped. Keep the IPC `captures\<run-id>`
   directory, the printed `stock-captures` folder, hashes, console errors,
   and the action/physical observation notes together.

Repeat with Deck B in a fresh run to check the right wheel. Keep the left
deck paused in JOG for that run. A result on one side does not establish
matching behavior on the other.

## Optional separate TT setting check

Do not mix this with the JOG run. Only if the installed stock UI or official
documentation identifies a specific candidate setting with a relevant scope,
record its exact label, source, original value and two available values.
Otherwise stop: there is no established TT braking setting to test. RPM is a
speed setting, not a braking comparison; JOG tension is not established as a
TT control. A hypothetical setting should not be guessed from the blog.

For one candidate only, perform a separate attended stock-only A/B: use the
same deck, track, TT base RPM and other settings. Repeat the observer setup in
a fresh run for each value. While paused, set A; mark `TT` and select TT, then
mark `PLAY` and play, and mark `PAUSE` and pause. Observe stopping behavior
without manually restraining a spinning platter; confirm it stops, mark `JOG`
and return to JOG, then mark `QUIT` and quit normally. Stop the observer as
above. Repeat with B, preserving separate captures and notes. Restore the
original value while paused, then quit and confirm physical stop. Record
marker times and deviations. Do not claim subjective stopping behavior
establishes a measured braking force.

## What the evidence can establish

The IPC trace records selected Traktor pipe submissions, immediate return
status, limited display headers and fixed markers, not completion, applied
force, physical wheel movement or full screen images. The NI writer observer
records matching `0x31`/`0x32` native write candidates in the attached service,
their returns and sampled stacks, not necessarily completed USB transfers.
Nested hooks can duplicate a submission; embedded IOCTL matches can be false
positives. Its absence of a match does not rule out another writer or path.
Check ready/stopped records, drops, errors and timestamps in both captures.
Correlated requests and observed ring/detent changes support a coupling
hypothesis; neither alone proves a causal mechanism or measures force.
