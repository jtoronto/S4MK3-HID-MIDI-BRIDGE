# Attended NI two-platter bridge trial

This is a separate, unaccepted development trial, not the installed desktop
app. Its separately packaged desktop host owns a BLOOP connection and starts
the full Djay bridge. The NI session runs in the bridge process; there is no
separate screen helper. Both platters can move. Screens, rings,
motor response and cleanup require physical observation, not just log messages.
The ordinary desktop app is unchanged.

The staged host must be built from source containing the experimental opt-in,
and its bundled engine must contain the NI-decks implementation. An older
desktop build does not become an NI trial merely by using this launcher.

NIHardwareService drives the platter force in this trial. The bridge's usual
wheel tension and TT braking settings do not change NI force. If the wheels
feel unexpectedly resistant, stop the trial rather than trying to tune that
resistance with the bridge controls.

This revision restores the stock NI playback signal in JOG and corrects
independent screen routing. In JOG, the ring should advance during
hands-off playback without the platter spinning. NI can change its haptic
response during manual movement; the bridge does not tune it.

## Before starting

1. Copy the whole package into a new folder on the Windows laptop. Do not
   replace an installed app, settings or mapping. Keep the package's
   `SHA256SUMS` and note the host and engine hashes with the observations.
2. Use an already working Windows MIDI Services Basic Loopback (BLOOP) setup
   and the existing matching Djay mapping. This trial does not install, repair,
   replace or edit either. The trial host creates **S4 MK3 Bridge BLOOP** when
   started and removes its own connection after the bridge exits. Do not run a
   second bridge or remove an existing endpoint to bypass an ownership error.
3. Close Traktor and all desktop and standalone bridge instances normally.
   Leave NIHardwareService running. Connect and power the S4. Open Djay and
   select the existing matching BLOOP mapping after starting the bridge and
   the endpoint becomes available.
   If the endpoint or real feedback is missing, stop; do not switch or edit
   mappings as an improvised recovery.
4. Pause both decks, turn both TT modes off, lower the audio output, clear both
   platters and keep the S4 power switch within reach. The person observing
   must be present and watching before launch. If either platter moves
   unexpectedly, movement persists after a stop, or loss of control or stop
   cannot be confirmed, switch off the S4 immediately.

The generated mapping bundled with the host is a separate artifact for a
clean installation; this trial does not require importing it. If needed
for a clean install outside this run, import under a new name and preserve all
existing user mappings.

## Run while watching

Only when ready and watching, open a Windows Command Prompt in the package
folder and run:

```cmd
Start-NI-Bridge.cmd
```

The launcher opts this host process into the experimental path. In the
host, click **Start bridge**. The host creates the BLOOP connection and
launches the engine with the full profile, `--midi-backend bloop`,
`--ni-decks` and `--until-stopped`, plus its managed-stop arguments.
Check **Diagnostics > Command preview** for the actual arguments. The
ordinary host remains unchanged. Do not use screen toggle or retry controls
for this trial.

Verify actual Djay feedback and that both screens and
rings respond; a connection or acquisition message is not proof of physical
output. If the bridge reports failure, feedback is absent, or the displays
or motors behave unexpectedly, stop the trial and preserve Diagnostics output.

For a startup comparison, leave the physical crossfader untouched and note
its position and the Djay fader's starting and ending positions. Record
whether Djay was already open, which mapping was selected, and whether
playback, loaded-track and Internal feedback appeared before touching any
control. Keep both TT modes off for this comparison. This revision starts
listening for MIDI before controller setup, but the Windows MIDI connection
still has to appear before the bridge can listen. Compare more than one
Start with Djay already open; do not treat a single success or a change in
launch order as proof that startup is reliable.

Before the TT checks, keep both decks paused in JOG. Select left HOTCUE and
right STEMS, then left STEMS and right HOTCUE. Verify that changing either
side leaves the other screen unchanged, without a momentary overwrite.

1. With both TT modes off, play the left deck while the right stays paused.
   Check that only the left needle advances on its own and that both wheels
   stay still. Pause and verify that left needle stops advancing by itself.
   Repeat for the right deck, then play and pause both together. If either
   wheel rotates under power in JOG, switch off the S4 and end the test.
2. With the right deck paused and right TT off, load the agreed track on the
   left deck, confirm Djay's Internal mode and feedback, then play and enable
   left TT. Observe only the left platter, its ring and both screens. Once
   running normally, perform one ordinary backspin and release completely.
   Verify that the platter returns to forward drive without pressing Play
   again. Pause and turn left TT off; confirm it stops before continuing.
3. Repeat for the right deck alone, keeping left paused and left TT off.
   Confirm the right platter stops before continuing.
4. Play both prepared Internal-mode decks and enable both TT modes. Observe
   both platters, rings and screens. Pause each deck and turn TT off, checking
   each platter stops. Note any unexpected movement, ring change, frozen or
   stale screen, missing feedback or audio/control issue.
5. With both platters confirmed stopped, click **Stop bridge** for the host's
   orderly managed cleanup. Wait for stopped status and inspect Diagnostics;
   verify physically that both wheels remain stopped. If stop or loss of
   control is unconfirmed, or movement persists, power off the S4. Quit the
   trial host normally after cleanup. Do not simulate a disconnect while
   wheels are moving.

Record the engine hash, mapping name, deck and mode, observations for each
phase, Diagnostics errors and whether both physical stops and process exit were
confirmed. A successful exit or NI request does not prove physical motor stop,
ring correctness or screen ownership release. Keep the output and observations
for review before repeating. Do not force-kill the bridge or restart services
as a substitute for orderly cleanup.
