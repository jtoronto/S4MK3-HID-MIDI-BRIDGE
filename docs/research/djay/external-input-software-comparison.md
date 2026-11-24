# External inputs and normal vinyl: software comparison

Research completed October 27, 2026. No controller, audio, MIDI, application
setting or NI service was operated during this investigation. This is research,
not acceptance of a new bridge feature.

**Subsequent user confirmation:** channel 4 in DVS THRU works with the bridge
active, including channel fader, FX and mix/cue. The earlier mixing-blocker
concern is superseded. Its cause remains unknown; this report's old support
draft and proposed diagnostic are retained below as history, not current work.

**Later implementation:** current full-profile source adds per-channel EXT
INT/THRU switching and confirmed-THRU indicators, with
[attended acceptance on all four channels](ext-input-validation.md).
Global DVS enablement and input assignments remain manual setup.

## Conclusion

**The absence of software-mixed external inputs is not a universal DJ-software
limitation.** Traktor has Live Input channels, VirtualDJ can assign an input
to a software deck, and Mixxx has deck passthrough and auxiliary input channels.
These are different from simply forwarding an input to an output. [T1, V1, M1]

**Bypass-style routes and reliance on onboard hardware mixers are also common.**
Serato explicitly describes software THRU as bypassing most processing.
rekordbox's reviewed DVS guide describes THROUGH for a standalone interface as
passing the input to an output as-is. Neither description is the same contract
as a fully processed software input channel. [S1, R1]

For this S4/Djay project, the user has now confirmed that the tested channel 4
THRU path supports fader, FX and mix/cue with the bridge active. We no longer
treat input-channel mixing as blocked. The earlier headphone-only result does
not establish a Djay defect; its cause is unconfirmed. The user has also
confirmed the USB input map A 1-2, B 3-4, C 5-6, D 7-8; complete mixing-control
coverage on the remaining channels and four-source operation still need their
own checks.

## What the attended test established

The [MIC D test record](external-input-routing-test.md) contains the exact
settings and user observations:

- S4 MIC D was assigned to Djay Deck 4 through USB input pair 7-8.
- The bridge remained stopped; no ordinary EXT press was requested.
- Global Microphone was set to None. The user explicitly confirmed the top-bar
  microphone toggle was off throughout, enabling it only afterward.
- The user heard silence in INT and microphone audio in THRU, repeatedly.
- During the instructed headphone MIX check, the mic remained audible at the
  on-screen channel fader's zero position; INT silenced it.
- The user subsequently restored the original settings.

**The fader observation is real, but its interpretation is limited.** We did
not record the physical main outputs or independently measure the CUE/MIX knob.
Deck 4 cue was enabled, and the actual listening instruction did not explicitly
turn it off. A cue contribution is a hypothesis to rule out, not an explanation
we have demonstrated. Djay documents cue/main blending separately from channel
volume. [D5]

The older MIDI-action reference describes a 5.6.9 audit, not a version captured
during this live test. Read-only inspection now finds an installed Djay bundle
at **5.6.10, build 67091**, matching the version listed in the October 19 Mac
release notes. The running application's version during the test was not
independently recorded. Confirm About before filing a version-specific bug. [D4]

## Comparison

This compares documented capabilities, not S4 compatibility with every product.
Hardware eligibility, audio-interface routing and licensing still apply.

| Application | Regular-audio input path | Evidence for channel mixing | Important boundary |
| --- | --- | --- | --- |
| Traktor Pro | Live Input deck flavor/channel | NI explicitly routes external sources into the mix and documents channel faders, EQ, FX assignment, cue and crossfader. | Internal mixing supplies software mixer controls; External Mixing deactivates software faders/crossfader/cue in favor of a hardware mixer. [T1, T2] |
| VirtualDJ | `linein` activates a configured line, mic or aux input on a selected deck | The scripting reference explicitly targets a deck; hardware guides document software-routed inputs and effects. | The generic Line-In page alone is not a complete processing contract. Some controllers also offer a separate hardware bypass path. [V1, V2, V3] |
| Mixxx | Deck passthrough from Vinyl Control inputs; separate auxiliary channels | The released control manual documents both. Inspected source places deck passthrough before pregain/pre-fader effects, then applies channel/crossfader gains to main buses separately from PFL. | This is documentation and source-level evidence, not an executed audio test. The pinned source is development code, not a claimed released-binary test. [M1, M2, M3] |
| Serato DJ Pro | Hardware-dependent software THRU or a hardware analog-input path | Official THRU documentation says most processing is bypassed, with DJ-FX and Sampler retained. | A pass-through mode does not promise the complete normal deck strip; hardware eligibility and downstream mixer behavior matter. [S1] |
| rekordbox | Hardware analog-input mixing; DVS THROUGH in the reviewed interface setup | The versioned guide describes THROUGH as direct input-to-output, with virtual-deck playback paused. | That section concerns a standalone audio interface. It is not evidence that every controller or every input facility behaves identically. [R1, R2] |
| Djay Pro for Mac | Per-deck DVS input assignments and THRU; separate global microphone path | The user subsequently confirmed channel 4 fader, FX and mix/cue working with the bridge active; see the [test record](external-input-routing-test.md). | Input pairs A-D are user-confirmed, but not every gain/EQ control or four-source operation. The detailed vendor FX/gain discussions are iOS-scoped. [D1, D2, D3, D6] |

Traktor's four deck positions and Mixxx's Vinyl Control 1-4 input slots support
multi-input configurations in principle. This research did not exercise four
simultaneous sources, establish S4 support in every application, or verify
every control on every listed path. [T1, M1]

### Why similar mode names are misleading

The relevant distinction is where the incoming audio joins the signal path:

```text
Software input channel:
physical input -> USB capture -> channel processing -> fader -> main mix
                                      \-> cue/PFL path

Hardware-mixer input:
physical input -> onboard channel processing -> onboard output mix
software decks -> USB playback --------------------^

Bypass/THRU:
physical input -> input/output forwarding -> downstream mixing, if provided
```

These are conceptual architectures, not identical diagrams for all products.
Mixxx even uses the name passthrough for a deck input that subsequently enters
its software mixing chain. Serato's THRU description explicitly skips most
processing. Names alone do not establish equivalence. [M2, M3, S1]

## Why an FLX10 is not the same problem

The FLX10 is a controller without standalone track playback, but its
manufacturer-authored manual documents onboard mixing of external inputs.
The signal-flow diagram shows analog channel processing and separate computer
master/headphone streams. Its analog channels therefore need not become Djay
software decks to be mixed. [F1]

Algoriddim nevertheless requires **Internal Mixer Mode** for the FLX10's
Djay integration. That setting describes the software/computer audio route;
it does not mean the controller lacks an onboard analog mixer. [F2]

VirtualDJ's DDJ-RR guide supplies another useful example: its PCMIX-off route
uses hardware mixing without EQ/filter on that path, while PCMIX-on routes
the analog inputs into USB/software decks. Even within one application,
hardware and selected routing can change which controls process the input.
This is not a recommendation to buy or change controllers. [V3]

## What remains specific to the S4 and Djay

NI documents two supported S4 workflows: Traktor Live Input while connected,
and standalone hardware mixing after USB disconnection and a power cycle.
That is not proof that every other USB-connected hardware route is impossible.
The examined Mixxx mapping did not reveal a confirmed command to enable a
four-channel hardware mixer while retaining this project's USB workflow.
No protocol command was executed. [T2, M4]

Djay's general Internal/External mixer distinction also does not settle THRU
processing. Internal mode mixes software decks into Main Output; External
mode sends separate deck outputs to a hardware mixer. [D1]

Further evidence prevents a simple "THRU bypasses all processing" conclusion:

- Algoriddim staff recommends THRU for applying FX to analog vinyl, with an
  FX BPM-synchronization limitation in that discussion. The thread is
  iOS-scoped, so this is not a complete Mac processing specification. [D2]
- A separate iOS/Numark Scratch discussion describes deliberately separate
  THRU gain behavior. Gain is not the channel fader, and that answer must not
  be promoted into a Mac fader contract. [D3]
- Djay release notes add external-input recording for certain controllers
  while internally mixing. Recording support is not evidence that arbitrary
  capture inputs become fully processed software decks. [D4]
- Installed presets contain `mixer.externalLineVolume1..4`; their existence
  does not establish whether they attenuate THRU, indicate an external
  hardware fader, or serve another mapping role. No such action was tested.

The user's later working result removes the proposed mixing blocker. The
unverified details above are not evidence of an outstanding fault, and there
is no basis to allege that every vinyl user encounters the earlier observation.

## Historical proposed check, only if the problem recurs

The subsequent working result means this diagnostic is not an outstanding
acceptance requirement.

Do not run this unattended. Prepare the chosen observation path first and
obtain fresh watching confirmation before changing routing or output levels.

1. Re-establish the same input with global Mic None and its top-bar toggle off.
   Confirm the actual running Djay version.
2. Disable Deck 4 cue and all other cue selections. Explicitly verify both
   Djay and S4 cue/main monitoring settings rather than relying on a single
   headphone MIX instruction.
3. Observe an isolated physical main output using an existing appropriate
   listening or recording setup. Keep speakers/listening levels controlled;
   do not create an output-to-input feedback loop.
4. Compare THRU with the on-screen channel fader raised and at zero, then INT,
   holding every other control fixed. Compare headphone cue separately.
5. If main mutes but the earlier headphone test did not, investigate monitoring.
   If isolated main remains audible at zero, ask Algoriddim which THRU controls
   are intended to attenuate it before implementing a bridge workaround.
6. Restore the recorded routes and control positions.

This is a proposed follow-up, not a missing result disguised as a pass.

## Historical unsent support-forum draft (superseded)

Do not post this as a current fault report: the user has since confirmed the
channel fader, FX and mix/cue working on channel 4 with the bridge active.
The text below records the question considered before that result.

**Title: macOS DVS THRU with a software-mixed controller: expected fader/EQ/cue behavior?**

I am trying to mix ordinary vinyl or another analog source alongside Djay's
digital decks using a Traktor Kontrol S4 MK3 as a CoreAudio interface. My
installed Djay bundle is currently 5.6.10 (67091); I will confirm the running
version before supplying a version-specific reproduction.

For an initial input test I used MIC D, assigned S4 capture channels 7-8 to
DVS Deck 4, and kept Internal Mixer Mode, Main 1-2 and Pre-cue 3-4. The global
microphone device was None and the top-bar microphone toggle was off.
My separate HID-to-MIDI bridge was stopped throughout.

Switching Deck 4 from INT to THRU made the mic audible; INT silenced it,
repeatedly. During a headphone MIX check it remained audible with the
on-screen channel fader at zero. I have not yet independently isolated the
physical main output or excluded every cue-monitoring contribution, so I am
not claiming a confirmed main-fader defect.

What is the intended Mac THRU signal path in Internal Mixer Mode? Should its
audio follow the software channel fader, EQ/filter, FX and headphone cue?
Is there a supported equivalent to a Live Input deck for an interface such
as the S4, or does normal-vinyl channel mixing require onboard hardware mixing?

I am looking for the supported setup and a precise test, rather than assuming
that THRU and a fully processed input deck mean the same thing.

Nothing has been posted or sent.

## Sources and verification

Accessed October 27, 2026. Vendor manuals/staff replies are primary evidence
for their documented scope. A manufacturer-authored manual on a mirror is
identified below; multiple pages from one vendor are not independent runtime
verification. No software was installed or updated, and no hardware test was
performed during this research.

- **T1:** [NI Traktor Pro manual: Live Input, decks and mixer](https://docs.native-instruments.com/online-guides/traktor-pro-manual/en/traktor-overview.html).
- **T2:** [NI S4 MK3 manual](https://www.native-instruments.com/fileadmin/ni_media/downloads/manuals/traktor/TRAKTOR_KONTROL_S4_MK3_Manual_English_0719.pdf), external-input and standalone sections.
- **V1:** [VirtualDJ scripting reference](https://virtualdj.com/manuals/virtualdj/appendix/vdjscriptverbs.html), `linein`.
- **V2:** [VirtualDJ DDJ-1000 advanced audio](https://virtualdj.com/manuals/hardware/pioneer/ddj1000/advancedaudio.html), software line-input routing/effects.
- **V3:** [VirtualDJ DDJ-RR input routing](https://virtualdj.com/manuals/hardware/pioneer/ddjrr/inputs.html), PCMIX hardware/software distinction.
- **M1:** [Mixxx released 2.5 controls manual](https://manual.mixxx.org/2.5/en/chapters/appendix/mixxx_controls.html), passthrough, auxiliary controls and input slots.
- **M2:** [Mixxx EngineDeck source, pinned revision](https://github.com/mixxxdj/mixxx/blob/ee17a272664ee9ad0aa978fd499c49802af90a99/src/engine/channels/enginedeck.cpp#L211-L254), development source inspected, not executed.
- **M3:** [Mixxx EngineMixer source, same revision](https://github.com/mixxxdj/mixxx/blob/ee17a272664ee9ad0aa978fd499c49802af90a99/src/engine/enginemixer.cpp#L337-L468), main gains and separate PFL.
- **M4:** [Mixxx S4 MK3 mapping source](https://github.com/mixxxdj/mixxx/blob/ee17a272664ee9ad0aa978fd499c49802af90a99/res/controllers/Traktor-Kontrol-S4-MK3.js), inspected mapping, not proof of firmware capabilities.
- **S1:** [Serato Through Mode](https://support.serato.com/hc/en-us/articles/234760848-Through-Mode), bypass and hardware restrictions.
- **R1:** [rekordbox 7.0.5 DVS setup guide](https://cdn.rekordbox.com/files/20241203185031/rekordbox7.0.5_dvs_setup_guide_EN.pdf), standalone-interface THROUGH.
- **R2:** [rekordbox internal/external mixer FAQ](https://rekordbox.com/en/support/faq/operation-hint/#faq-q500212).
- **F1:** [FLX10 manufacturer manual, hosted copy](https://device.report/m/93018ee7a10eb62e8e00c14f2880860c65a893bbfee60e30a87e3c8954b7e7b3_optim.pdf), external inputs and printed signal-flow p. 174; mirror provenance, not an independent reviewer.
- **F2:** [Algoriddim staff: FLX10 Internal Mixer Mode](https://community.algoriddim.com/t/flx10-internal-mixer-or-external/38961).
- **D1:** [Djay Mac mixer modes](https://help.algoriddim.com/user-manual/djay-pro-mac/hardware/mixer-mode).
- **D2:** [Algoriddim staff: FX on analog vinyl via THRU](https://community.algoriddim.com/t/analog-vinyl-effects-djay-pro/38038), iOS-scoped.
- **D3:** [Algoriddim staff: THRU gain behavior](https://community.algoriddim.com/t/trim-gain-not-working-on-thru-mode-on-ipados/42207), iOS/Numark Scratch scope; gain is not the channel fader.
- **D4:** [Djay Mac release notes](https://www.algoriddim.com/djay-pro-mac/releasenotes), regular-vinyl THRU introduction, controller-specific input recording and current version listing.
- **D5:** [Djay Mac volume/output controls](https://help.algoriddim.com/user-manual/djay-pro-mac/mixing-basics/audio-settings), cue/main blending.
- **D6:** [Djay Mac DVS manual](https://help.algoriddim.com/user-manual/djay-pro-mac/hardware/dvs), enablement, input assignment and deck control modes.
