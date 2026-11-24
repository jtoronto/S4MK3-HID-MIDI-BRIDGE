# Djay control and genuine feedback over LoopBe1

## Result and scope

The three scoped behaviors were demonstrated on Kensuke:

1. A native MIDI command visibly moved Djay's crossfader.
2. A separate mouse-driven Djay change produced genuine MIDI feedback while
   the diagnostic sent no MIDI.
3. Outgoing control echoes did not update the diagnostic's feedback cache.

The working feedback declaration was an **embedded continuous output** with
its own channel. This is a software-only proof for one control, not full S4
mapping acceptance or a completed Windows bridge.

**Limit:** the observed MIDI-driven changes did not return a value to the
probe. The combined command/feedback modes timed out and remain failed cases.
Only independent mouse-driven state feedback was established. Production
integration must not infer software state from the echoed commands.

## Address ownership

Current production mapping code was read without changing it:

- `src/full/catalog.rs:354`: the crossfader targets `mixer.crossfade` using CC 20.
- `src/full/mapping.rs:461`: embedded outputs are generated for numerous note
  controls and normally inherit their input MIDI address.
- `src/full/feedback.rs`: deck-note feedback updates cached state. On a shared
  bus, a sent note could therefore be confused with actual software feedback.
- Dedicated channel-7 outputs and separately assigned root CC outputs already
  demonstrate distinct address domains, but do not make every input/output
  address in the existing mapping disjoint.

The proof used one-based channels:

| Role | Endpoint | Direction | Channel | Type/data | Packed value 127 |
| --- | --- | --- | --- | --- | --- |
| Control | LoopBe Internal MIDI | Probe to Djay | 5 | CC 20 | `0x007F14B4` |
| Feedback | LoopBe Internal MIDI | Djay to probe | 8 | CC 20 | `0x007F14B7` |

Plist channel values are 4 and 7. CC message type is 3.
`CrossfaderFeedback.Apply` accepts only channel 8 CC 20 with a valid 7-bit value.
It records unknown until genuine feedback arrives, then updates only on that
address. There is no timing-based echo suppression or "recently sent" heuristic.

## Baseline and preservation

- Windows MIDI service PID 8940; NIHardwareService PID 4772.
- Djay 5.6.10 PID 15608; LoopBe monitor PID 10076.
- LoopBe's selected configuration was the empty default, `LoopBe Internal MIDI`.
  Its configuration list contained no saved LoopBe mappings.
- The existing user mapping was
  `Hercules DJ Control Inpulse 500 Edit.djayMidiMapping`, 62,010 bytes, SHA-256
  `F4BD84D5E400CC5A7D07A69DED1661C27B0B6476B9C5503C452FD0F46792DFFB`.
- The user loaded two tracks during setup. Both stayed paused at 00:00.0.
  Their loaded state was preserved.
- The crossfader started at the left endpoint. The mixer was visible.
- No S4 HID, audio playback, motor or controller-screen traffic, purchases,
  service restarts or reboot occurred. Production Rust/macOS code was unchanged.

Mac desktop input was unavailable because Accessibility permission was denied.
No alternate Mac input mechanism was used. The user connected a Windows MCP
server, which supplied native Windows screenshots, UI inspection and input.
SSH remained the build-artifact transfer and diagnostic execution path.

## Prepared artifacts and checks

The existing isolated `windows/loopbe-probe` diagnostic gained:

- A packet observation event at the real WinMM callback boundary.
- The shared-bus crossfader decoder and `--route-check`.
- Bounded sending and non-sending Djay proof modes.
- A direct legacy WinRT MIDI output comparison.
- Two separately named mapping fixtures; neither replaced a user mapping.

The Mac publish passed with warnings treated as errors:

```sh
diagnostics/windows-tools/dotnet-10.0.403/dotnet publish \
  windows/loopbe-probe/S4LoopBeProbe.csproj \
  -c Release -r win-x64 --self-contained true \
  -o dist/windows-loopbe-probe --tl:off
```

Both mapping files passed `plutil -lint`. The five generated probe files were
staged beside the previous portable runtime, with no Windows development
installation.

An attempted direct Mac execution of the Windows self-contained output failed
with missing `libhostpolicy.dylib`. That was an execution-environment mismatch,
not a successful decoder check. The deployed Windows `--route-check` then
exited 0: it rejected control echoes, a different controller number and a Note
message, accepted dedicated values 127/64/0, and rejected an echo after a known
value without changing that value.

## Experiments, including failures

### Root-output candidate

`S4 LoopBe Djay Proof 20261104` was imported through Djay's registered file
association. Djay selected it and displayed the expected Mixer/Crossfader row.
The input used channel 5 CC 20; a root output assigned `mixer.crossfade` to
channel 8 CC 20.

Parent PID 4348 sent `0x007F14B4`. A screenshot showed the crossfader at the right
endpoint. The exact input returned through LoopBe as an echo:

```json
{"packed":"0x007F14B4","isFeedback":false,"before":null,"after":null,"accepted":0}
```

No dedicated feedback arrived within the bound; `DJAY_CONTROL_PROOF_EXIT=1`.
This case proves control and echo rejection, not a passing feedback declaration.

### Native MIDI Out settings and API comparison

In the isolated mapping's Advanced options, MIDI Out was enabled and assigned
channel 8 CC 20. The default output had Blend off. A non-sending mouse-change
capture PID 11672 timed out without a packet. A separate raw capture PID 2028
received zero messages around the output Test-button action; its capture exit 0
is not an output pass.

LoopBe's tray menu showed Mute unchecked. No mute state was changed.

To distinguish the port/API path from Djay, PID 12956 opened the exact legacy
Windows output ID and used `Windows.Devices.Midi.MidiOutPort` to send
`MidiControlChangeMessage(7,20,127)`. The native WinMM input received exact
`0x007F14B7`; `LEGACY_WINRT_CHECK_EXIT=0`, followed by normal closure.
This proves that API path can deliver the packet independently of Djay. It
does not identify a Windows defect or prove why a Djay declaration was silent.

### Embedded continuous output

`S4 LoopBe Djay Embedded Proof 20261104` used:

```xml
<key>output</key>
<dict>
    <key>controlType</key><string>control</string>
    <key>midiChannel</key><integer>7</integer>
    <key>midiMinValue</key><integer>0</integer>
    <key>midiMaxValue</key><integer>127</integer>
</dict>
```

The message type and data number inherit CC 20 from the input control; the
channel is overridden. Djay's editor showed MIDI Out on, Channel 8, CC 20,
and **Blend on**. The source fixture was recognized natively.

The installed app saved real-valued bounds and its editor version. During
diagnostics it also displayed/saved an unassigned learned channel-8 CC-20 row
with key path `mixer`; that was not part of the authored control fixture and
was removed with the temporary configuration at cleanup. The successful
control's input/output fields matched the retained Embedded fixture.

PID 17296's MIDI-driven command visibly moved the crossfader and received its
ignored echo, but still timed out waiting for feedback:
`EMBEDDED_CONTROL_PROOF_EXIT=1`. This failure is not relabelled a pass.

For the independent GUI proof, PID 10496 opened the receiver and reported
`sendControl=false` before any mouse action. The crossfader was visibly right.
Windows MCP moved the pointer onto its thumb at approximately `(1048,447)` and
dragged to `(872,447)`, the left endpoint. Both track positions remained 00:00.0.

The probe received actual channel-8 feedback:

```json
{"stage":"crossfader-route","pid":10496,"data":{"packed":"0x007F14B7","isFeedback":true,"before":null,"after":127,"accepted":1}}
{"stage":"crossfader-route","pid":10496,"data":{"packed":"0x000014B7","isFeedback":true,"before":127,"after":0,"accepted":2}}
{"stage":"crossfader-feedback-confirmed","pid":10496,"data":{"expected":0,"sendControl":false,"accepted":2}}
{"stage":"connection-closed","pid":10496,"data":{}}
```

`EMBEDDED_GUI_FEEDBACK_EXIT=0`. The probe sent no MIDI in this process, so its
feedback cannot be its own outgoing control echo.

## Reproduction order

1. Install LoopBe1 separately and build/stage the portable probe as above.
2. Keep playback paused; record the original mapping and crossfader position.
3. Run `S4LoopBeProbe.exe --route-check` and require exit 0.
4. Double-click the **Embedded Proof** mapping, accept only its named import,
   select it in MIDI Devices > Configure, and verify the advanced output
   channel and Blend setting. Close the editor before live control actions.
5. With the fader left, `--crossfade-right` demonstrates MIDI control and logs
   echo rejection, but its combined feedback requirement failed in this test.
6. With the fader visibly right, start `--watch-crossfade-left`. Wait for
   `crossfader-ready`, then drag the onscreen fader left. Require actual feedback,
   `crossfader-feedback-confirmed`, closure and exit 0.
7. Restore the original position and configuration; remove only the named
   temporary proof configurations and binaries.

`--legacy-winrt-check` is an optional API-path comparison. Its generated packet
is not evidence of Djay feedback.

## Restoration and final checks

Both proof configurations and the five-file probe overlay were removed from
Windows. Djay caches configurations: deleting their files alone did not clear
the active mapping, and deleting controls did not remove an earlier root output.
Cleanup therefore used the native configuration deletion flow and the retained
empty fixture at `windows/loopbe-probe/restore/LoopBe Internal MIDI.djayMidiMapping`.
That fixture explicitly empties both `controls` and `outputs`. It was selected
before the remaining owned configurations were deleted through Djay.

The final editor showed the original default name, `LoopBe Internal MIDI`,
an empty control table and Advanced options off. Done closed the editor.
Final filesystem inventory contained only the original Hercules mapping, with
its original length and SHA-256 above. Neither proof configuration nor the
temporary empty restoration configuration remained installed. The staged
restoration fixture was also removed.

Final inventory exited 0: no diagnostic processes or `S4LoopBeProbe` overlay
files remained. Djay PID 15608, LoopBe monitor PID 10076, MIDI service PID 8940
Running/Manual and NIHardwareService PID 4772 Running/Auto were unchanged.
LoopBe1 remained installed. Both loaded tracks stayed paused at 00:00.0 and
the crossfader was left. No service restart, reboot, commit or push occurred.

Final source review covered packet subscription lifetime, exact address
matching, bounded waits and normal disposal. `git diff --check` and the empty
restoration fixture's `plutil -lint` passed with exit 0. A separate whitespace
check covering the untracked diagnostic and report files found no trailing
whitespace.

## Evidence limits

The successful case identifies a usable isolated feedback declaration, not
general support for arbitrary root-output keys or every control type. The
initial root assignment and default non-continuous output did not demonstrate
feedback. Differences in declaration and app state mean the exact cause of
each earlier silence was not independently proven.

No feedback after a MIDI-driven move was established. No full-profile generator
change, production Rust Windows backend, latency/throughput test, long-session
stability check or physical S4 integration was performed. Existing macOS source
checks were not repeated because that source was untouched.

## Primary references

- [Djay Windows mapping import](https://help.algoriddim.com/user-manual/djay-pro-windows/midi/sharing).
- [Microsoft legacy MidiOutPort](https://learn.microsoft.com/en-us/uwp/api/windows.devices.midi.midioutport).
- [Earlier LoopBe1 transport proof](windows-loopbe1-probe-validation-20261104.md).
