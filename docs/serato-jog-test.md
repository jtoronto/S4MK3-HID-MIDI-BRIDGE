# Minimal Serato jog feasibility test

**Status: deferred, November 3, 2026.** Transport verification passed, but
installed-Serato jog acceptance was blocked at the MIDI setup gate, not
rejected by a demonstrated jog test. See the
[investigation results](serato-investigation-results-20261103.md).
The sequence below is retained for a future attempt, not active work.

This is a disposable, synthetic MIDI experiment, not the proposed Serato mode.
It does not open the S4, change bridge preferences, route audio or send motor/
screen reports. Leave the normal bridge stopped and the S4 disconnected.

Prepared against the locally installed **Serato DJ Pro 5.0.0,
build 5.0.0.1278**. Acceptance in that version is not yet established.
The goal is to find out whether Serato still accepts legacy `platter_move`
and `platter_touch`, before spending the trial on a complete implementation.

## What is provided

- `examples/serato-jog-test.rs`: interactive virtual controller.
- `examples/serato-jog-test.xml`: one-deck experimental preset.
- Virtual source and return destination: **S4 Serato Jog Test**.
- MIDI channel 1: Note 0 Play/Pause, Note 7 touch, CC 16 movement.
- Movement is binary-offset relative: default forward value 72, backward 56.
  Each command sends 128 packets, nominally 8 ms apart (about one second).
  This is synthetic motion, not S4 resolution or an audio-latency measurement.

The XML deliberately preserves the original experiment's legacy movement
`deck_id="1"` and modern touch/Play `deck_id="0"`. The author used different
indexing conventions in those elements. Check both decks when testing; a
movement/touch mismatch is a finding, not proof that all jog XML is rejected.
The range is the example code's 1546, not a calibrated S4 value.

Original experiment:
[Teo Tormo, April 11, 2018](https://djtechtools.com/2018/04/11/hacking-serato-djs-midi-mapping-jogwheels-touchstrips-and-modifiers/).

## Build and transport check

The agent-prepared binary is at `target/debug/examples/serato-jog-test` within
the worktree. To rebuild it:

```sh
cd /Users/joey/Development/S4MK3-HID-MIDI-BRIDGE/.slim/worktrees/serato-mode
cargo build --locked --example serato-jog-test
```

Optional transport check, without loading a Serato mapping:

```sh
./target/debug/examples/serato-jog-test --self-test
```

Expected: `SERATO_JOG_LOOPBACK_PASS`, `messages=260`, exit status 0.
The command subscribes to a uniquely named CoreMIDI test source before sending
the same Play/touch/forward/backward/release messages as the interactive probe.
It compares the actual received bytes and then closes the port. This establishes
MIDI transport only, not Serato command acceptance or audible scratch behavior.

## Test in execution order

Do not start this sequence until you are ready to watch Serato and listen.
Start with low listening volume. No action here controls the physical S4.

1. Open Serato and finish its normal trial/sign-in setup if required. Do not
   activate or buy another license for this test. Select the two-deck Practice
   view if that is the available layout.
2. Load a local/demo audio track onto the **left deck**. Leave it paused.
   Load a different track onto the right deck if available, also paused;
   this makes a deck-index mismatch visible. Confirm the left track plays
   audibly using Serato's GUI, then pause it again. If audio itself is
   unavailable, position/touch observations are still useful; note that limit.
3. In Terminal, start the prepared probe:

   ```sh
   cd /Users/joey/Development/S4MK3-HID-MIDI-BRIDGE/.slim/worktrees/serato-mode
   ./target/debug/examples/serato-jog-test
   ```

   Expected: `SERATO_JOG_READY`, the virtual-port name and a command list.
   Starting it sends no MIDI; commands require Return.
4. Open Serato **Setup > MIDI**. Look for **S4 Serato Jog Test** and enable it
   with **Enable MIDI Device** if that option is available. Do not change
   hardware remapping or restore defaults. If MIDI assignment/device enabling
   is unavailable in this trial/hardware configuration, record exactly what
   is missing or disabled. That is a setup gate, not a failed jog command.
5. Preserve existing mappings. In a **second Terminal tab**, copy the supplied
   preset to the directory this Serato installation created, under a new name:

   ```sh
   cd /Users/joey/Development/S4MK3-HID-MIDI-BRIDGE/.slim/worktrees/serato-mode
   mkdir -p "$HOME/Music/_Serato_/MIDI/xml"
   cp -n examples/serato-jog-test.xml "$HOME/Music/_Serato_/MIDI/xml/S4 Serato Jog Test.xml"
   cmp examples/serato-jog-test.xml "$HOME/Music/_Serato_/MIDI/xml/S4 Serato Jog Test.xml"
   ```

   Expected: `cmp` prints nothing and exits 0. `cp -n` preserves an existing
   file; if comparison fails, use a fresh filename instead of overwriting it.
   This installation's actual directory is lowercase `xml`.
6. Return to Setup > MIDI, select **S4 Serato Jog Test** in the preset list and
   choose **Load**. Close/reopen Setup if necessary. If it does not appear,
   leave the probe running, quit/relaunch Serato and look again. If still
   missing, record that result before any jog commands; XML discovery/loading
   is not established.
7. In the **first Terminal tab**, type `p` and Return. Expected: left deck
   toggles once. Type `p` again to pause. If nothing happens, stop the jog
   sequence here: first prove that this port and preset can operate Play.
   MIDI Learn can map the left Play control to `p` in a separate preset; retain
   any XML it generates. Do not save over the experimental jog preset.
8. With the left deck paused, type `f`, Return, then `b`, Return. Watch both
   track positions. Record which deck responds, whether directions differ,
   and whether movement works without touch. An unmoving paused deck alone is
   not decisive; some movement paths depend on playback or touch.
9. Click Play on the left deck in Serato. Type `t`, Return. This sends touch ON
   and holds it. Record whether the deck stops/enters a scratch hold.
10. While touch is still held, type `f`, Return; after its completion message,
    type `b`, Return. Expected if scratching works: opposite position movement
    and audible forward/backward scratching rather than just a brief tempo
    change. Record which deck responds even if behavior is unexpected.
11. Type `r`, Return. Expected if touch works: scratch hold releases and normal
    playback resumes. Use the GUI to pause immediately if release fails.
12. While playing with touch released, repeat `f` and `b`. Record bend versus
    seek versus scratch behavior. This distinguishes a usable jog path from a
    command that only jumps the playhead. Pause using the GUI when finished.
13. Type `r`, Return, then `q`, Return. Expected: `SERATO_JOG_STOPPED`.
    Ctrl-C or terminal EOF also requests quit; a running burst finishes within
    about one second, then touch is released before ports close.
14. Preserve the loaded file and observations before saving anything in Serato's
    mapping editor. If the initial experiment works, test Save As/reload and
    restart using a separately named copy; compare rewritten XML. The historical
    source warns that saving can rewrite legacy jog syntax.

## Sensitivity and interpreting a failure

If basic movement works but is too small/large, release touch, pause the deck,
quit the probe, and restart with `--step 1` through `--step 63`:

```sh
./target/debug/examples/serato-jog-test --step 16
```

Enable/reload its port/preset if Serato needs that after the restart. This
changes only synthetic movement magnitude; it does not change the XML.
Do not judge S4 jog feel from this artificial stream.

Return these observations in order: port visible/enabled; preset loaded;
`p` works; `t` effect; `f`/`b` effect and deck; `r` effect; audible versus only
visual movement; any `SERATO_RETURN` messages or Serato error. The probe logs
the actual return bytes when Serato sends them; silence is not a diagnosis.

If Play works but jogging does not, the next isolated experiment is the XML
jog action/index/range contract, not the rest of the bridge. If the synthetic
jog works, connect actual S4 input only in the next test. Motors, TT feedback,
screens and complete mapping coverage remain separate work.

## Preparation evidence, November 3, 2026

The compiled probe passed real CoreMIDI loopback with all 260 expected
messages in order. The XML passed `xmllint`; parsed attributes confirm three
controls, binary-offset movement and the deliberately different movement/
touch deck indices. CLI help works; `--step 0` fails with exit status 2.

Rust formatting, all-target/all-feature Clippy, all 510 Rust tests, all-target
build, all 80 Swift tests, build-script syntax and Info.plist validation passed.
SwiftPM reported its existing warning about two unhandled files; no macOS
package sources or configuration were changed.

The diagnostic is isolated to one example, has no new dependency and uses
typed terminal commands with exhaustive dispatch. Its 233 nonblank/noncomment
source lines are near the local 250-line guideline; if this experiment grows,
split transport verification out rather than expanding it into a backend.
The real MIDI loopback is the verification boundary, not a mock of Serato.

No mapping was installed into the user's Serato directory, no Serato playback
test was triggered, and no S4 interface was opened. Jog acceptance, actual
scratch/nudge semantics, audio and mapping persistence remain unverified.
