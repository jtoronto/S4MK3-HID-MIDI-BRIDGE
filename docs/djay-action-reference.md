# Djay Pro 5.6.9 MIDI action and output reference

This is the installed-version reference for the S4 MK3 bridge audit. It separates Djay key paths (software actions and model-state values) from the MIDI channel, message type, and data number assigned by a controller mapping. A Djay key path does not have a universal MIDI note or CC. The address examples below are assignments from the generated S4 MK3 mapping only.

## Sources and completeness

- Installed application: **djay Pro 5.6.9**, from `/Applications/djay Pro.app/Contents/Info.plist`.
- `Contents/Resources/MidiModelMetadata.plist`: **944** action/control/rotary metadata entries; this is the source of the exhaustive declared input-key table.
- Metadata type counts: **494** `button-action`, **211** `button-toggle`, **145** `control`, **71** `button-hold`, **5** `button-trigger`, and **18** entries with no `type` field.
- `Contents/Resources/MidiModelKeyPath.strings`: **1085** localized labels. It provides an exact label for **745 / 944** metadata paths; every other label in the tables is explicitly marked inferred.
- `Contents/Resources/MIDI Mappings/*.djayMidiMapping`: **210** parseable preset files scanned. Across those files there are **2427** distinct concrete `controls[].keyPath` values, **169** distinct top-level `outputs[].keyPath` values, and **1849** distinct control paths carrying an embedded `output` dictionary. The tables below list the complete metadata catalog, preset-observed control paths not represented by it, and the complete top-level-output key-path union.
- The metadata-declared `modelState` / `modelValue` links resolve to **634** unique state/value paths (**503** modelState, **132** modelValue; one appears in both fields). These are associated model references, not automatically MIDI output assignments.

This is complete relative to the installed metadata and the scanned shipped presets, not a claim about undocumented/private APIs or other Djay versions. Preset-only rows are identifiers actually present in installed presets but absent from the metadata dictionary; do not promote those rows into a cross-controller contract. A blank S4 usage cell means this bridge mapping does not assign the key.

## How to read keys and MIDI addresses

A row in the action catalog names a Djay mapping input key. The metadata `type` describes the kind of operation (for example `button-hold`, `button-toggle`, or `control`); `modelState` and `modelValue` name state/value references. `modelEnabled` is a gate, not a separate output key. Preset `controls[].output` is an embedded feedback declaration on that input control and reuses that control's MIDI address. A root-level `outputs[]` entry is a separate output assignment with its own key path and MIDI address.

All channel labels in this document are **MIDI channels 1-based**, as requested. In the plist they are 0-based. MIDI message type `1` is Note and `3` is Control Change (CC). The generated S4 mapping uses `rotary-64` for relative controls.

## Generated S4 mapping shape

The counts and S4 usage cells below are the historical audit snapshot, not
the rebuilt encoder mapping. Current encoder assignments supersede that
snapshot: MOVE uses CC 2 for `skipRotary`, selection uses CC 7 for
`skipDurationRotary`, and Shift+MOVE uses note 5/14 for fixed one-beat jumps.
REV now uses `turntableN.reverseHold` on note 3, channels 1-4. Browse CC 3
has `flipped=true` on normal channels 1-4 and shifted channels 9-12.
Shift+LOOP press uses `turntableN.reloop` on note 6, channels 9-12.
`loopIn`, `loopOut`, and `loopInOut` are now unmapped. Normal LOOP remains
`autoLoopDurationRotary` / `autoLoopOnOff`; Shift+LOOP turn remains
`autoLoopMoveRotary`. Reloop's declared enable gate is
`turntable.song.masterLoopRegion.hasRange`, so it needs an existing loop range.

The audited generated mapping is `dist/S4 MK3 Bridge.app/Contents/Resources/S4 MK3 Bridge.djayMidiMapping`, generated from `src/full/catalog.rs` and `src/full/mapping.rs`. Its root reports `schemeVersion=1`, `version=0`; it contains **474** controls, **361** distinct concrete input key paths, **164** embedded output dictionaries, and **12** top-level outputs.

The generic `turntable{deck}` catalog expands to decks A-D as `turntable1`–`turntable4` on MIDI channels 1-4. A shifted target uses internal channels 8-11, shown to hardware tools as MIDI channels 9-12. The sampler's `{bank}` expansion is bank 1 for odd-numbered physical decks (A/C) and bank 2 for even decks (B/D). These are bridge/controller choices, not Djay-wide MIDI defaults.

The 474 input controls expand to 74 per channel on MIDI CH1-CH4 (55 Note, 19 CC each), 42 on CH5 (16 Note, 26 CC), 4 Notes on CH6, and 33 per channel on shifted MIDI CH9-CH12 (29 Note, 4 CC each). These sum to 474; top-level CH7 outputs are counted separately.

### Top-level outputs in the generated S4 mapping

| Djay output key path | Purpose | MIDI address (1-based channel) |
|---|---|---|
| `turntable1.monoMeter` | Deck mono meter | CC CH7 data 0 (control) |
| `turntable1.playing` | Raw deck playing state | CC CH7 data 4 |
| `turntable1.song.loadingSuccess` | Track load-success state | Note CH7 data 0 |
| `turntable2.monoMeter` | Deck mono meter | CC CH7 data 1 (control) |
| `turntable2.playing` | Raw deck playing state | CC CH7 data 5 |
| `turntable2.song.loadingSuccess` | Track load-success state | Note CH7 data 1 |
| `turntable3.monoMeter` | Deck mono meter | CC CH7 data 2 (control) |
| `turntable3.playing` | Raw deck playing state | CC CH7 data 6 |
| `turntable3.song.loadingSuccess` | Track load-success state | Note CH7 data 2 |
| `turntable4.monoMeter` | Deck mono meter | CC CH7 data 3 (control) |
| `turntable4.playing` | Raw deck playing state | CC CH7 data 7 |
| `turntable4.song.loadingSuccess` | Track load-success state | Note CH7 data 3 |

The four `monoMeter` outputs are CC on MIDI channel 7, data 0-3 (decks A-D). The four `playing` outputs are CC on channel 7, data 4-7. The four `song.loadingSuccess` outputs are Note on channel 7, data 0-3. The generator sets output ranges to 0-127; meter entries are `controlType=control`. Embedded outputs are separate: they sit inside `controls[]`, do not add another MIDI address, and may have an empty dictionary for special pad-feedback handling.

### Representative Deck C (turntable3) addresses

| Djay key path | Layer / control | Exact S4 MIDI address |
|---|---|---|
| `turntable3.playPause` | Deck C Play | Note CH3 data 0 |
| `turntable3.reverse` | Deck C Reverse (toggle action) | Note CH3 data 3 |
| `turntable3.autoLoopOnOff` | Loop encoder press, unshifted | Note CH3 data 6 |
| `turntable3.loopInOut` | Loop encoder press, shifted | Note CH11 data 6 |
| `turntable3.recordSample` | Record Sample | Note CH3 data 22 |
| `turntable3.fxActive` | FX enable (left strip bank) | Note CH3 data 32 |
| `turntable3.fxActive` | FX enable (right strip bank) | Note CH3 data 36 |
| `turntable3.speed` | Tempo fader (unshifted; pickup + flipped) | CC CH3 data 0 |
| `turntable3.speedRelative` | Tempo fader (shifted) | CC CH11 data 0 |
| `turntable3.pitchBendMove` | Jog bend (rotary-64; sensitivity 7) | CC CH3 data 4 |
| `musicLibrary.togglePreview` | Preview selected track | Note CH3 data 13 |
| `mixer.lineVolume3` | Channel 3 fader | CC CH5 data 18 |
| `turntable3.cueOrJumpIfAlreadySet1` | Deck C hotcue pad 1 | Note CH3 data 40 |
| `sampler.turntable1.player1.playingConsideringHoldSetting` | Deck C sampler bank 1, player 1 | Note CH3 data 48 |
| `turntable3.unmixerFourTrackChannel1Muted` | Deck C stem pad 1 | Note CH3 data 56 |

Deck C here means `turntable3` (MIDI CH3 in this bridge). If “C2” means the MIDI pitch name, the common convention maps it to data number 36: **Note CH3 data 36** is `turntable3.fxActive` on the right FX strip. Octave names vary between MIDI applications; the channel, message type, and data number are the unambiguous address. If “C2” means Deck C hotcue pad 2 instead, it is `turntable3.cueOrJumpIfAlreadySet2` at Note CH3 data 41; shifted clear-cue is `turntable3.clearCuePoint2` at Note CH11 data 41. Numbers are decimal MIDI data numbers. The active sampler assignment on deck C resolves to sampler bank 1, shared with deck A.

## Audit-specific findings

| Question | Installed-version evidence | Bridge finding |
|---|---|---|
| Momentary reverse | `turntable.reverseHold` is declared `button-hold` and references `turntable.reverse`; `turntable.reverse` itself is `button-toggle`. | Current S4 control uses `turntable1..4.reverseHold` on note 3, channels 1-4. Press/release and Stop preserve ownership; no automatic Slip is added. Native physical behavior still needs live confirmation. |
| Tempo-range cycle | Presets and localized strings include `application.tempoSliderRangeMinus`, `application.tempoSliderRangePlus`, and `application.tempoSliderRangeNext`; labels are decrease, increase, and switch tempo slider range. None is an entry in `MidiModelMetadata.plist`. | No S4 binding in the generated mapping. Preset presence confirms use by installed controller mappings, but metadata does not declare their action type/state link. |
| Preview state and scrubbing | `musicLibrary.togglePreview` is labeled “Preview Selected Track” and occurs in shipped presets and the S4 mapping. The metadata declares `turntable.scrubbing` as a `control` whose value is `turntable.display.songProgress` (deck track progress). No distinct preview-player state or preview-position key is declared in the inspected metadata. | Preview toggle is Note CH3 data 13. Do not treat `turntable.scrubbing` as proof of preview-track scrubbing. No dedicated preview scrub assignment is present in the generated S4 mapping. |
| Sample recording | `turntable.recordSample` is localized as “Record Sample” and appears in shipped presets; it is preset-observed but absent from the metadata dictionary. | Bound for deck C as Note CH3 data 22. |
| FX active | `turntable.fxActive` is `button-toggle` with state `turntable.fxActive`. | Left FX strip: Note CH3 data 32 with embedded feedback. Right FX strip: Note CH3 data 36, without embedded feedback in the generated mapping. The separately named `fx1Enabled` style controls are individual slot controls, not the overall `fxActive` state. |
| Jog and tempo-fader handling | Catalog maps jog bend/scratch/seek to relative CC controls; the tempo fader is pickup-enabled and flipped, with a shifted `speedRelative` target. | `relative()` translates motion into signed chunks up to 63 rather than discarding larger deltas. No maximum speed cap is established by the inspected bridge behavior. |
| Front-panel startup | Startup state uses default `switches=[None; 5]`; the front-panel translator emits the first snapshot once, then emits only changes and advances its cache even if Djay has not attached. | Physical switch positions are not entirely ignored. Whether a listener-readiness replay is needed remains unproven at runtime. |

## Complete metadata input/action catalog

One row per dictionary entry in `MidiModelMetadata.plist` (944 total). The key path is the Djay-side mapping identifier. “S4 MIDI usage” is blank for keys not assigned in the generated bundle. If the metadata entry has no `type`, the raw metadata column preserves the fields as shipped; do not infer a button behavior from the name alone.

### application (38)

| Djay input key path | Type | Human description | Metadata fields | S4 MIDI usage |
|---|---|---|---|---|
| `application.automix` | button-toggle | Automix on-off | `{"type":"button-toggle","modelState":"automix"}` |  |
| `application.jogPitchBendMode` | button-hold | Jog Pitch Bend Mode | `{"type":"button-hold","modelState":"turntable1.jogPitchBendMode"}` |  |
| `application.jogPitchBendModeToggle` | button-toggle | Toggle Jog Pitch Bend Mode | `{"type":"button-toggle","modelState":"turntable1.jogPitchBendMode"}` |  |
| `application.jogSeekMode` | button-hold | Jog Seek Mode | `{"type":"button-hold","modelState":"turntable1.jogSeekMode"}` |  |
| `application.jogSeekModeToggle` | button-toggle | Toggle Jog Seek Mode | `{"type":"button-toggle","modelState":"turntable1.jogSeekMode"}` |  |
| `application.openDashboardSettings` | button-action | Open Dashboard Settings | `{"type":"button-action"}` |  |
| `application.permanentGlobalSyncMode` | button-toggle | [Inferred] application / permanent Global Sync Mode | `{"type":"button-toggle","modelState":"permanentGlobalSyncMode"}` |  |
| `application.playPause` | button-toggle | Play / Pause | `{"type":"button-toggle","modelState":"isAnyTurntablePlaying"}` |  |
| `application.quantize` | button-hold | Quantize | `{"type":"button-hold","modelState":"turntable1.deckQuantize"}` |  |
| `application.quantizeToggle` | button-toggle | Toggle Quantize | `{"type":"button-toggle","modelState":"turntable1.deckQuantize"}` |  |
| `application.recording` | button-toggle | Recording on-off | `{"type":"button-toggle","modelState":"recorder.isRecording"}` |  |
| `application.syncAll` | button-toggle | [Inferred] application / sync All | `{"type":"button-toggle","modelState":"isInSyncMode"}` |  |
| `application.toggleShowCueBar` | button-toggle | Toggle Tools Bar | `{"type":"button-toggle","modelState":"view.state.showCueBar"}` |  |
| `application.toggleShowEffects` | button-toggle | Toggle FX Bar | `{"type":"button-toggle","modelState":"view.state.showFXBar"}` |  |
| `application.toggleShowTools` | button-toggle | Toggle Tools | `{"type":"button-toggle","modelState":"view.state.showTools"}` |  |
| `application.toggleShowUnmixerBar` | button-toggle | Toggle Neural Mix Bar | `{"type":"button-toggle","modelState":"view.state.showUnmixerBar"}` |  |
| `application.toggleShowUnmixerPopup` | button-toggle | [Inferred] application / toggle Show Unmixer Popup | `{"type":"button-toggle","modelState":"view.state.showUnmixerCrossfader"}` |  |
| `application.toggleShowWaveforms` | button-toggle | Toggle Waveforms | `{"type":"button-toggle","modelState":"view.state.showWaveform"}` |  |
| `application.toggleWaveformZoomed` | button-toggle | Toggle Waveform Zoomed | `{"type":"button-toggle","modelState":"view.state.waveformZoomFactor"}` |  |
| `application.toggleWaveSlice` | button-toggle | [Inferred] application / toggle Wave Slice | `{"type":"button-toggle","modelState":"view.state.waveSlice"}` |  |
| `application.turntable1Selected` | button-action | Select Deck 1 | `{"type":"button-action","modelState":"turntable1Selected"}` | Note CH6 data 0 |
| `application.turntable2Selected` | button-action | Select Deck 2 | `{"type":"button-action","modelState":"turntable2Selected"}` | Note CH6 data 1 |
| `application.turntable3Selected` | button-action | Select Deck 3 | `{"type":"button-action","modelState":"turntable3Selected"}` | Note CH6 data 2 |
| `application.turntable4Selected` | button-action | Select Deck 4 | `{"type":"button-action","modelState":"turntable4Selected"}` | Note CH6 data 3 |
| `application.turntableToggleSelected` | button-toggle | Toggle Selected Deck | `{"type":"button-toggle","modelState":"turntable2Selected"}` |  |
| `application.viewModeFourDeckWaveform` | button-action | Select Four Decks | `{"type":"button-action","modelState":"view.isFourDeckMode"}` |  |
| `application.viewModeOneDeckAutomix` | button-action | Select Automix | `{"type":"button-action","modelState":"view.isAutomixViewMode"}` |  |
| `application.viewModeOneDeckTurntable` | button-action | Select One Deck | `{"type":"button-action","modelState":"view.isViewModeOneDeckTurntable"}` |  |
| `application.viewModeSelect` | button-action | View Mode Select | `{"actionParameterMinValue":0,"modelState":"view.state.modeIndex","type":"button-action","hasActionParameter":true,"actionParameterMaxValue":7}` |  |
| `application.viewModeTwoDeckPro` | button-action | Select Two Decks / Pro | `{"type":"button-action","modelState":"view.isViewModeTwoDeckPro"}` |  |
| `application.viewModeTwoDeckSequencer` | button-action | Select Looper | `{"type":"button-action","modelState":"view.isViewModeTwoDeckSequencer"}` |  |
| `application.viewModeTwoDeckSimple` | button-action | Select Starter | `{"type":"button-action","modelState":"view.isViewModeTwoDeckSimple"}` |  |
| `application.viewModeTwoDeckTurntable` | button-action | Select Two Decks / Classic | `{"type":"button-action","modelState":"view.isViewModeTwoDeckTurntable"}` |  |
| `application.viewModeVideo` | button-action | Select Video | `{"type":"button-action","modelState":"view.isVideoMode"}` |  |
| `application.waveformZoomed` | button-hold | [Inferred] application / waveform Zoomed | `{"type":"button-hold","modelState":"view.state.waveformZoomFactor"}` |  |
| `application.waveformZoomFactorMinus` | button-action | Decrease Waveform Zoom Factor | `{"type":"button-action","modelEnabled":"view.state.canDecreaseWaveformZoomFactor"}` |  |
| `application.waveformZoomFactorPlus` | button-action | Increase Waveform Zoom Factor | `{"type":"button-action","modelEnabled":"view.state.canIncreaseWaveformZoomFactor"}` |  |
| `application.waveformZoomFactorReset` | button-action | [Inferred] application / waveform Zoom Factor Reset | `{"type":"button-action","modelState":"view.state.canResetWaveformZoomFactor","modelEnabled":"view.state.canResetWaveformZoomFactor"}` |  |

### looper (60)

| Djay input key path | Type | Human description | Metadata fields | S4 MIDI usage |
|---|---|---|---|---|
| `looper.playPause` | button-toggle | Play / Pause | `{"type":"button-toggle","modelState":"looper.playing"}` |  |
| `looper.showLooper` | button-action | [Inferred] looper / show Looper | `{"type":"button-action","modelState":"view.state.showLooper"}` |  |
| `looper.toggleLooperShown` | button-toggle | Toggle Looper | `{"type":"button-toggle","modelState":"view.state.showLooper"}` |  |
| `looper.track1.sample1.trigger` | button-action | Track 1: Trigger Sample 1 | `{"type":"button-action","modelState":"looper.track1.sample1.statePlaying"}` |  |
| `looper.track1.sample2.trigger` | button-action | Track 1: Trigger Sample 2 | `{"type":"button-action","modelState":"looper.track1.sample2.statePlaying"}` |  |
| `looper.track1.sample3.trigger` | button-action | Track 1: Trigger Sample 3 | `{"type":"button-action","modelState":"looper.track1.sample3.statePlaying"}` |  |
| `looper.track1.sample4.trigger` | button-action | Track 1: Trigger Sample 4 | `{"type":"button-action","modelState":"looper.track1.sample4.statePlaying"}` |  |
| `looper.track1.sample5.trigger` | button-action | Track 1: Trigger Sample 5 | `{"type":"button-action","modelState":"looper.track1.sample5.statePlaying"}` |  |
| `looper.track1.sample6.trigger` | button-action | Track 1: Trigger Sample 6 | `{"type":"button-action","modelState":"looper.track1.sample6.statePlaying"}` |  |
| `looper.track1.volume` | control | Track 1 Volume | `{"type":"control","modelValue":"looper.track1.volume"}` |  |
| `looper.track2.sample1.trigger` | button-action | Track 2: Trigger Sample 1 | `{"type":"button-action","modelState":"looper.track2.sample1.statePlaying"}` |  |
| `looper.track2.sample2.trigger` | button-action | Track 2: Trigger Sample 2 | `{"type":"button-action","modelState":"looper.track2.sample2.statePlaying"}` |  |
| `looper.track2.sample3.trigger` | button-action | Track 2: Trigger Sample 3 | `{"type":"button-action","modelState":"looper.track2.sample3.statePlaying"}` |  |
| `looper.track2.sample4.trigger` | button-action | Track 2: Trigger Sample 4 | `{"type":"button-action","modelState":"looper.track2.sample4.statePlaying"}` |  |
| `looper.track2.sample5.trigger` | button-action | Track 2: Trigger Sample 5 | `{"type":"button-action","modelState":"looper.track2.sample5.statePlaying"}` |  |
| `looper.track2.sample6.trigger` | button-action | Track 2: Trigger Sample 6 | `{"type":"button-action","modelState":"looper.track2.sample6.statePlaying"}` |  |
| `looper.track2.volume` | control | Track 2 Volume | `{"type":"control","modelValue":"looper.track2.volume"}` |  |
| `looper.track3.sample1.trigger` | button-action | Track 3: Trigger Sample 1 | `{"type":"button-action","modelState":"looper.track3.sample1.statePlaying"}` |  |
| `looper.track3.sample2.trigger` | button-action | Track 3: Trigger Sample 2 | `{"type":"button-action","modelState":"looper.track3.sample2.statePlaying"}` |  |
| `looper.track3.sample3.trigger` | button-action | Track 3: Trigger Sample 3 | `{"type":"button-action","modelState":"looper.track3.sample3.statePlaying"}` |  |
| `looper.track3.sample4.trigger` | button-action | Track 3: Trigger Sample 4 | `{"type":"button-action","modelState":"looper.track3.sample4.statePlaying"}` |  |
| `looper.track3.sample5.trigger` | button-action | Track 3: Trigger Sample 5 | `{"type":"button-action","modelState":"looper.track3.sample5.statePlaying"}` |  |
| `looper.track3.sample6.trigger` | button-action | Track 3: Trigger Sample 6 | `{"type":"button-action","modelState":"looper.track3.sample6.statePlaying"}` |  |
| `looper.track3.volume` | control | Track 3 Volume | `{"type":"control","modelValue":"looper.track3.volume"}` |  |
| `looper.track4.sample1.trigger` | button-action | Track 4: Trigger Sample 1 | `{"type":"button-action","modelState":"looper.track4.sample1.statePlaying"}` |  |
| `looper.track4.sample2.trigger` | button-action | Track 4: Trigger Sample 2 | `{"type":"button-action","modelState":"looper.track4.sample2.statePlaying"}` |  |
| `looper.track4.sample3.trigger` | button-action | Track 4: Trigger Sample 3 | `{"type":"button-action","modelState":"looper.track4.sample3.statePlaying"}` |  |
| `looper.track4.sample4.trigger` | button-action | Track 4: Trigger Sample 4 | `{"type":"button-action","modelState":"looper.track4.sample4.statePlaying"}` |  |
| `looper.track4.sample5.trigger` | button-action | Track 4: Trigger Sample 5 | `{"type":"button-action","modelState":"looper.track4.sample5.statePlaying"}` |  |
| `looper.track4.sample6.trigger` | button-action | Track 4: Trigger Sample 6 | `{"type":"button-action","modelState":"looper.track4.sample6.statePlaying"}` |  |
| `looper.track4.volume` | control | Track 4 Volume | `{"type":"control","modelValue":"looper.track4.volume"}` |  |
| `looper.track5.sample1.trigger` | button-action | Track 5: Trigger Sample 1 | `{"type":"button-action","modelState":"looper.track5.sample1.statePlaying"}` |  |
| `looper.track5.sample2.trigger` | button-action | Track 5: Trigger Sample 2 | `{"type":"button-action","modelState":"looper.track5.sample2.statePlaying"}` |  |
| `looper.track5.sample3.trigger` | button-action | Track 5: Trigger Sample 3 | `{"type":"button-action","modelState":"looper.track5.sample3.statePlaying"}` |  |
| `looper.track5.sample4.trigger` | button-action | Track 5: Trigger Sample 4 | `{"type":"button-action","modelState":"looper.track5.sample4.statePlaying"}` |  |
| `looper.track5.sample5.trigger` | button-action | Track 5: Trigger Sample 5 | `{"type":"button-action","modelState":"looper.track5.sample5.statePlaying"}` |  |
| `looper.track5.sample6.trigger` | button-action | Track 5: Trigger Sample 6 | `{"type":"button-action","modelState":"looper.track5.sample6.statePlaying"}` |  |
| `looper.track5.volume` | control | Track 5 Volume | `{"type":"control","modelValue":"looper.track5.volume"}` |  |
| `looper.track6.sample1.trigger` | button-action | Track 6: Trigger Sample 1 | `{"type":"button-action","modelState":"looper.track6.sample1.statePlaying"}` |  |
| `looper.track6.sample2.trigger` | button-action | Track 6: Trigger Sample 2 | `{"type":"button-action","modelState":"looper.track6.sample2.statePlaying"}` |  |
| `looper.track6.sample3.trigger` | button-action | Track 6: Trigger Sample 3 | `{"type":"button-action","modelState":"looper.track6.sample3.statePlaying"}` |  |
| `looper.track6.sample4.trigger` | button-action | Track 6: Trigger Sample 4 | `{"type":"button-action","modelState":"looper.track6.sample4.statePlaying"}` |  |
| `looper.track6.sample5.trigger` | button-action | Track 6: Trigger Sample 5 | `{"type":"button-action","modelState":"looper.track6.sample5.statePlaying"}` |  |
| `looper.track6.sample6.trigger` | button-action | Track 6: Trigger Sample 6 | `{"type":"button-action","modelState":"looper.track6.sample6.statePlaying"}` |  |
| `looper.track6.volume` | control | Track 6 Volume | `{"type":"control","modelValue":"looper.track6.volume"}` |  |
| `looper.track7.sample1.trigger` | button-action | Track 7: Trigger Sample 1 | `{"type":"button-action","modelState":"looper.track7.sample1.statePlaying"}` |  |
| `looper.track7.sample2.trigger` | button-action | Track 7: Trigger Sample 2 | `{"type":"button-action","modelState":"looper.track7.sample2.statePlaying"}` |  |
| `looper.track7.sample3.trigger` | button-action | Track 7: Trigger Sample 3 | `{"type":"button-action","modelState":"looper.track7.sample3.statePlaying"}` |  |
| `looper.track7.sample4.trigger` | button-action | Track 7: Trigger Sample 4 | `{"type":"button-action","modelState":"looper.track7.sample4.statePlaying"}` |  |
| `looper.track7.sample5.trigger` | button-action | Track 7: Trigger Sample 5 | `{"type":"button-action","modelState":"looper.track7.sample5.statePlaying"}` |  |
| `looper.track7.sample6.trigger` | button-action | Track 7: Trigger Sample 6 | `{"type":"button-action","modelState":"looper.track7.sample6.statePlaying"}` |  |
| `looper.track7.volume` | control | Track 7 Volume | `{"type":"control","modelValue":"looper.track7.volume"}` |  |
| `looper.track8.sample1.trigger` | button-action | Track 8: Trigger Sample 1 | `{"type":"button-action","modelState":"looper.track8.sample1.statePlaying"}` |  |
| `looper.track8.sample2.trigger` | button-action | Track 8: Trigger Sample 2 | `{"type":"button-action","modelState":"looper.track8.sample2.statePlaying"}` |  |
| `looper.track8.sample3.trigger` | button-action | Track 8: Trigger Sample 3 | `{"type":"button-action","modelState":"looper.track8.sample3.statePlaying"}` |  |
| `looper.track8.sample4.trigger` | button-action | Track 8: Trigger Sample 4 | `{"type":"button-action","modelState":"looper.track8.sample4.statePlaying"}` |  |
| `looper.track8.sample5.trigger` | button-action | Track 8: Trigger Sample 5 | `{"type":"button-action","modelState":"looper.track8.sample5.statePlaying"}` |  |
| `looper.track8.sample6.trigger` | button-action | Track 8: Trigger Sample 6 | `{"type":"button-action","modelState":"looper.track8.sample6.statePlaying"}` |  |
| `looper.track8.volume` | control | Track 8 Volume | `{"type":"control","modelValue":"looper.track8.volume"}` |  |
| `looper.volume` | control | Volume | `{"type":"control","modelValue":"looper.volume"}` |  |

### microphone (1)

| Djay input key path | Type | Human description | Metadata fields | S4 MIDI usage |
|---|---|---|---|---|
| `microphone.active` | button-toggle | Microphone on-off | `{"type":"button-toggle","modelState":"microphone.active"}` |  |

### mixer (64)

| Djay input key path | Type | Human description | Metadata fields | S4 MIDI usage |
|---|---|---|---|---|
| `mixer.boothLevel` | control | Booth Volume | `{"type":"control","modelValue":"mixer.boothVolume"}` |  |
| `mixer.crossfade` | control | Crossfader | `{"type":"control","modelValue":"mixer.masterCrossfade","modelEnabled":"mixer.isModeInternal"}` | CC CH5 data 20 |
| `mixer.crossfadeAcapella` | control | Neural Mix Vocals Crossfader | `{"type":"control","modelValue":"mixer.masterAcapellaCrossfade"}` |  |
| `mixer.crossfadeAssignment1Toggle` | button-toggle | [Inferred] mixer / crossfade Assignment 1 Toggle | `{"type":"button-toggle","modelState":"mixer.crossfadeAssignment1"}` |  |
| `mixer.crossfadeAssignment2Toggle` | button-toggle | [Inferred] mixer / crossfade Assignment 2 Toggle | `{"type":"button-toggle","modelState":"mixer.crossfadeAssignment2"}` |  |
| `mixer.crossfadeAssignment3Toggle` | button-toggle | [Inferred] mixer / crossfade Assignment 3 Toggle | `{"type":"button-toggle","modelState":"mixer.crossfadeAssignment3"}` |  |
| `mixer.crossfadeAssignment4Toggle` | button-toggle | [Inferred] mixer / crossfade Assignment 4 Toggle | `{"type":"button-toggle","modelState":"mixer.crossfadeAssignment4"}` |  |
| `mixer.crossfadeFourTrackChannel1` | control | Neural Mix Crossfader (4ch: Drums) | `{"type":"control","modelValue":"mixer.masterInstrumentalCrossfade","modelDescription":"turntable.unmixer.fourTrackChannel1Name"}` |  |
| `mixer.crossfadeFourTrackChannel2` | control | Neural Mix Crossfader (4ch: Bass) | `{"type":"control","modelValue":"mixer.masterBassFourCrossfade","modelDescription":"turntable.unmixer.fourTrackChannel2Name"}` |  |
| `mixer.crossfadeFourTrackChannel3` | control | Neural Mix Crossfader (4ch: Harmonic) | `{"type":"control","modelValue":"mixer.masterHarmonicCrossfade","modelDescription":"turntable.unmixer.fourTrackChannel3Name"}` |  |
| `mixer.crossfadeFourTrackChannel4` | control | Neural Mix Crossfader (4ch: Vocals) | `{"type":"control","modelValue":"mixer.masterAcapellaCrossfade","modelDescription":"turntable.unmixer.fourTrackChannel4Name"}` |  |
| `mixer.crossfadeFXAutoPlayToggle` | button-toggle | Toggle Auto-Play | `{"type":"button-toggle","modelState":"mixer.crossfadeFXAutoPlay"}` |  |
| `mixer.crossfadeFXRotary` | not declared | Crossfader FX Select (Rotary) | `{"steppedRotary":true}` |  |
| `mixer.crossfadeFXSelect` | button-action | Crossfader FX Select | `{"hasActionParameter":true,"actionParameterMinValue":0,"actionParameterMaxValue":15,"type":"button-action","modelState":"mixer.selectedCrossfadeFXIndex","modelBlinkingState":"mixer.selectedRunningCrossfadeFXIndex"}` |  |
| `mixer.crossfadeFXSelectAndToggle` | button-action | Crossfader FX Select and Toggle | `{"hasActionParameter":true,"actionParameterMinValue":0,"actionParameterMaxValue":15,"type":"button-action","modelState":"mixer.selectedActiveCrossfadeFXIndex","modelBlinkingState":"mixer.selectedRunningCrossfadeFXIndex"}` |  |
| `mixer.crossfadeFXTempoBlendToggle` | button-toggle | Toggle Tempo Blend | `{"type":"button-toggle","modelState":"mixer.crossfadeFXTempoAdjustModeIsMorph"}` |  |
| `mixer.crossfadeFXToggle` | button-toggle | Crossfader FX Toggle | `{"type":"button-toggle","modelState":"mixer.enableCrossfadeFX","modelBlinkingState":"mixer.crossfadeFXIsRunning"}` |  |
| `mixer.crossfadeFXTransition` | button-action | Transition Now | `{"type":"button-action","modelState":"mixer.manualTransition.isRunning"}` |  |
| `mixer.crossfadeFXTransitionDurationRotary` | not declared | Transition Duration (Rotary) | `{"steppedRotary":true}` |  |
| `mixer.crossfadeHarmonic` | control | Neural Mix Harmonic Crossfader | `{"type":"control","modelValue":"mixer.masterHarmonicCrossfade"}` |  |
| `mixer.crossfadeInstrumental` | control | Neural Mix Drums Crossfader | `{"type":"control","modelValue":"mixer.masterInstrumentalCrossfade"}` |  |
| `mixer.crossfadeLeft` | button-action | Crossfade to Left | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.crossfadeLeftStepwise` | button-action | Crossfade to Left (Stepwise) | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.crossfadeMiddle` | button-action | Crossfade to Middle | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.crossfadeRight` | button-action | Crossfade to Right | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.crossfadeRightStepwise` | button-action | Crossfade to Right (Stepwise) | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.crossfadeThreeTrackChannel1` | control | Neural Mix Crossfader (3ch: Drums) | `{"type":"control","modelValue":"mixer.masterInstrumentalCrossfade","modelDescription":"turntable.unmixer.threeTrackChannel1Name"}` |  |
| `mixer.crossfadeThreeTrackChannel2` | control | Neural Mix Crossfader (3ch: Harmonic) | `{"type":"control","modelValue":"mixer.masterHarmonicCrossfade","modelDescription":"turntable.unmixer.threeTrackChannel2Name"}` |  |
| `mixer.crossfadeThreeTrackChannel3` | control | Neural Mix Crossfader (3ch: Vocals) | `{"type":"control","modelValue":"mixer.masterAcapellaCrossfade","modelDescription":"turntable.unmixer.threeTrackChannel3Name"}` |  |
| `mixer.crossfadeTwoTrackChannel1` | control | Neural Mix Crossfader (2ch: Instrumental) | `{"type":"control","modelValue":"mixer.masterInstrumentalCrossfade","modelDescription":"turntable.unmixer.twoTrackChannel1Name"}` |  |
| `mixer.crossfadeTwoTrackChannel2` | control | Neural Mix Crossfader (2ch: Acappella) | `{"type":"control","modelValue":"mixer.masterAcapellaCrossfade","modelDescription":"turntable.unmixer.twoTrackChannel2Name"}` |  |
| `mixer.cutCrossfade` | button-action | Cut Crossfader | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.cutCrossfadeAutoFast` | button-action | Auto Cut (Fast) | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.cutCrossfadeAutoSlow` | button-action | Auto Cut | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.cutCrossfadeAutoSlower` | button-action | [Inferred] mixer / cut Crossfade Auto Slower | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.cutCrossfadeAutoTripletFast` | button-action | [Inferred] mixer / cut Crossfade Auto Triplet Fast | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.cutCrossfadeAutoTripletSlow` | button-action | [Inferred] mixer / cut Crossfade Auto Triplet Slow | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.lineVolume` | control | [Inferred] mixer / line Volume | `{"type":"control","modelValue":"mixer.volumeEffectChannel.currentValue"}` |  |
| `mixer.lineVolume1` | control | Deck 1: Line Volume | `{"type":"control","modelValue":"mixer.volumeEffectChannel1.currentValue"}` | CC CH5 data 16 |
| `mixer.lineVolume2` | control | Deck 2: Line Volume | `{"type":"control","modelValue":"mixer.volumeEffectChannel2.currentValue"}` | CC CH5 data 17 |
| `mixer.lineVolume3` | control | Deck 3: Line Volume | `{"type":"control","modelValue":"mixer.volumeEffectChannel3.currentValue"}` | CC CH5 data 18 |
| `mixer.lineVolume4` | control | Deck 4: Line Volume | `{"type":"control","modelValue":"mixer.volumeEffectChannel4.currentValue"}` | CC CH5 data 19 |
| `mixer.masterLevel` | control | Main Volume | `{"type":"control","modelValue":"mixer.masterVolume"}` |  |
| `mixer.midiCrossfadeCuttingModeToggle` | button-toggle | Toggle Crossfader Cutting Mode | `{"type":"button-toggle","modelState":"userDefaults.DJMidiCrossfadeCuttingMode"}` |  |
| `mixer.midiHamsterSwitchToggle` | button-toggle | Toggle Hamster Switch | `{"type":"button-toggle","modelState":"userDefaults.DJMidiHamsterSwitch"}` |  |
| `mixer.monitorActive` | button-toggle | [Inferred] mixer / monitor Active | `{"type":"button-toggle","modelState":"mixer.isPrecueing"}` |  |
| `mixer.monitorLevel` | control | Monitor Volume | `{"type":"control","modelValue":"mixer.preCueVolume"}` |  |
| `mixer.monitorMix` | control | Monitor Cue/Mix | `{"type":"control","modelValue":"mixer.preCueMix"}` |  |
| `mixer.monitorMixToCue` | button-toggle | Monitor Cue | `{"type":"button-toggle","modelState":"mixer.preCueMixIsCue"}` |  |
| `mixer.monitorMixToggle` | button-toggle | Toggle Monitor Cue/Mix | `{"type":"button-toggle","modelState":"mixer.preCueMixIsMix"}` |  |
| `mixer.monitorMixToMiddle` | button-toggle | Monitor Cue/Mix | `{"type":"button-toggle","modelState":"mixer.preCueMixIsMiddle"}` |  |
| `mixer.monitorMixToMix` | button-toggle | Monitor Mix | `{"type":"button-toggle","modelState":"mixer.preCueMixIsMix"}` |  |
| `mixer.monitorSelect` | control | Monitor Select | `{"type":"control","modelValue":"mixer.preCueCrossfade"}` |  |
| `mixer.monitorSplitCueMode` | button-hold | Monitor Split Cue | `{"type":"button-hold","modelState":"mixer.splitCueMode"}` |  |
| `mixer.monitorSplitOutputToggle` | button-toggle | [Inferred] mixer / monitor Split Output Toggle | `{"type":"button-toggle","modelState":"mixer.audioConfigSplitOutput"}` |  |
| `mixer.toggleUnmixerMuteFxEnabled` | button-toggle | Toggle Neural Mix - Mute/Solo Echo Out | `{"type":"button-toggle","modelState":"turntable1.unmixer.muteWithFx"}` |  |
| `mixer.transitionActiveToInactive` | button-action | Transition to Inactive Deck | `{"type":"button-action"}` |  |
| `mixer.transitionLeft` | button-action | Transition to Left | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.transitionMiddle` | button-action | Transition to Middle | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.transitionRight` | button-action | Transition to Right | `{"type":"button-action","modelEnabled":"mixer.isModeInternal"}` |  |
| `mixer.unmixerCrossfadeModeSeparate` | button-toggle | [Inferred] mixer / unmixer Crossfade Mode Separate | `{"type":"button-toggle","modelState":"mixer.unmixerCrossfadeModeSeparate"}` |  |
| `mixer.videoCrossfade` | control | Video Crossfader | `{"type":"control","modelValue":"mixer.masterVideoCrossfade"}` |  |
| `mixer.videoCrossfadeModeSeparate` | button-toggle | Toggle Split Audio/Video Crossfader | `{"type":"button-toggle","modelValue":"userDefaults.VJVideoCrossfadeModeSeparate"}` |  |
| `mixer.videoTransitionTypeRotary` | not declared | Select Video Transition (Rotary) | `{"steppedRotary":true}` |  |

### musicLibrary (4)

| Djay input key path | Type | Human description | Metadata fields | S4 MIDI usage |
|---|---|---|---|---|
| `musicLibrary.libraryRotary` | not declared | Move Library Selection (Rotary) | `{"steppedRotary":true}` | CC CH1 data 3 (rotary-64); CC CH2 data 3 (rotary-64); CC CH3 data 3 (rotary-64); CC CH4 data 3 (rotary-64) |
| `musicLibrary.sectionRotary` | not declared | Move Library Section (Rotary) | `{"steppedRotary":true}` | CC CH10 data 3 (rotary-64); CC CH11 data 3 (rotary-64); CC CH12 data 3 (rotary-64); CC CH9 data 3 (rotary-64) |
| `musicLibrary.showSongInLibrary` | button-action | Locate Track | `{"type":"button-action","modelEnabled":"isAnyTurntableLoaded"}` |  |
| `musicLibrary.sourceRotary` | not declared | Select Library Source (Rotary) | `{"steppedRotary":true}` |  |

### sampler (151)

| Djay input key path | Type | Human description | Metadata fields | S4 MIDI usage |
|---|---|---|---|---|
| `sampler.clearSequenceRecording` | button-action | Clear Sequence Recording | `{"type":"button-action","modelState":"sampler.sequencePlayerPlaying"}` |  |
| `sampler.player1.gain` | control | [Inferred] sampler / player 1 / gain | `{"type":"control","modelValue":"sampler.player1.gain","modelDescription":"sampler.player1.name"}` |  |
| `sampler.player1.paused` | button-action | [Inferred] sampler / player 1 / paused | `{"modelEnabled":"sampler.player1.loadingSuccess","modelDescription":"sampler.player1.name","type":"button-action","modelState":"sampler.player1.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player1.playing` | button-toggle | [Inferred] sampler / player 1 / playing | `{"modelEnabled":"sampler.player1.loadingSuccess","type":"button-toggle","modelState":"sampler.player1.statePlaying","modelDescription":"sampler.player1.name"}` |  |
| `sampler.player1.playingConsideringHoldSetting` | button-action | Play Sample 1 | `{"modelEnabled":"sampler.player1.loadingSuccess","type":"button-action","modelState":"sampler.player1.statePlaying","modelDescription":"sampler.player1.name"}` |  |
| `sampler.player1.playingHold` | button-hold | [Inferred] sampler / player 1 / playing Hold | `{"modelEnabled":"sampler.player1.loadingSuccess","type":"button-hold","modelState":"sampler.player1.statePlaying","modelDescription":"sampler.player1.name"}` |  |
| `sampler.player1.volume` | control | Sampler 1 Volume | `{"type":"control","modelValue":"sampler.player1.volume","modelDescription":"sampler.player1.name"}` |  |
| `sampler.player10.gain` | control | [Inferred] sampler / player 10 / gain | `{"type":"control","modelValue":"sampler.player10.gain","modelDescription":"sampler.player10.name"}` |  |
| `sampler.player10.paused` | button-action | [Inferred] sampler / player 10 / paused | `{"type":"button-action","modelState":"sampler.player10.playing","modelStateIsFlipped":true}` |  |
| `sampler.player10.playing` | button-toggle | [Inferred] sampler / player 10 / playing | `{"type":"button-toggle","modelState":"sampler.player10.playing"}` |  |
| `sampler.player10.playingConsideringHoldSetting` | button-action | Play Sample 10 | `{"type":"button-action","modelState":"sampler.player10.playing"}` |  |
| `sampler.player10.playingHold` | button-hold | [Inferred] sampler / player 10 / playing Hold | `{"type":"button-hold","modelState":"sampler.player10.playing"}` |  |
| `sampler.player10.volume` | control | Sampler 10 Volume | `{"type":"control","modelValue":"sampler.player10.volume","modelDescription":"sampler.player10.name"}` |  |
| `sampler.player11.gain` | control | [Inferred] sampler / player 11 / gain | `{"type":"control","modelValue":"sampler.player11.gain","modelDescription":"sampler.player11.name"}` |  |
| `sampler.player11.paused` | button-action | [Inferred] sampler / player 11 / paused | `{"type":"button-action","modelState":"sampler.player11.playing","modelStateIsFlipped":true}` |  |
| `sampler.player11.playing` | button-toggle | [Inferred] sampler / player 11 / playing | `{"type":"button-toggle","modelState":"sampler.player11.playing"}` |  |
| `sampler.player11.playingConsideringHoldSetting` | button-action | Play Sample 11 | `{"type":"button-action","modelState":"sampler.player11.playing"}` |  |
| `sampler.player11.playingHold` | button-hold | [Inferred] sampler / player 11 / playing Hold | `{"type":"button-hold","modelState":"sampler.player11.playing"}` |  |
| `sampler.player11.volume` | control | Sampler 11 Volume | `{"type":"control","modelValue":"sampler.player11.volume","modelDescription":"sampler.player11.name"}` |  |
| `sampler.player12.gain` | control | [Inferred] sampler / player 12 / gain | `{"type":"control","modelValue":"sampler.player12.gain","modelDescription":"sampler.player12.name"}` |  |
| `sampler.player12.paused` | button-action | [Inferred] sampler / player 12 / paused | `{"type":"button-action","modelState":"sampler.player12.playing","modelStateIsFlipped":true}` |  |
| `sampler.player12.playing` | button-toggle | [Inferred] sampler / player 12 / playing | `{"type":"button-toggle","modelState":"sampler.player12.playing"}` |  |
| `sampler.player12.playingConsideringHoldSetting` | button-action | Play Sample 12 | `{"type":"button-action","modelState":"sampler.player12.playing"}` |  |
| `sampler.player12.playingHold` | button-hold | [Inferred] sampler / player 12 / playing Hold | `{"type":"button-hold","modelState":"sampler.player12.playing"}` |  |
| `sampler.player12.volume` | control | Sampler 12 Volume | `{"type":"control","modelValue":"sampler.player12.volume","modelDescription":"sampler.player12.name"}` |  |
| `sampler.player13.gain` | control | [Inferred] sampler / player 13 / gain | `{"type":"control","modelValue":"sampler.player13.gain","modelDescription":"sampler.player13.name"}` |  |
| `sampler.player13.paused` | button-action | [Inferred] sampler / player 13 / paused | `{"type":"button-action","modelState":"sampler.player13.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player13.playing` | button-toggle | [Inferred] sampler / player 13 / playing | `{"type":"button-toggle","modelState":"sampler.player13.statePlaying"}` |  |
| `sampler.player13.playingConsideringHoldSetting` | button-action | Play Sample 13 | `{"type":"button-action","modelState":"sampler.player13.statePlaying"}` |  |
| `sampler.player13.playingHold` | button-hold | [Inferred] sampler / player 13 / playing Hold | `{"type":"button-hold","modelState":"sampler.player13.statePlaying"}` |  |
| `sampler.player13.volume` | control | Sampler 13 Volume | `{"type":"control","modelValue":"sampler.player13.volume","modelDescription":"sampler.player13.name"}` |  |
| `sampler.player14.gain` | control | [Inferred] sampler / player 14 / gain | `{"type":"control","modelValue":"sampler.player14.gain","modelDescription":"sampler.player14.name"}` |  |
| `sampler.player14.paused` | button-action | [Inferred] sampler / player 14 / paused | `{"type":"button-action","modelState":"sampler.player14.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player14.playing` | button-toggle | [Inferred] sampler / player 14 / playing | `{"type":"button-toggle","modelState":"sampler.player14.statePlaying"}` |  |
| `sampler.player14.playingConsideringHoldSetting` | button-action | Play Sample 14 | `{"type":"button-action","modelState":"sampler.player14.statePlaying"}` |  |
| `sampler.player14.playingHold` | button-hold | [Inferred] sampler / player 14 / playing Hold | `{"type":"button-hold","modelState":"sampler.player14.statePlaying"}` |  |
| `sampler.player14.volume` | control | Sampler 14 Volume | `{"type":"control","modelValue":"sampler.player14.volume","modelDescription":"sampler.player14.name"}` |  |
| `sampler.player15.gain` | control | [Inferred] sampler / player 15 / gain | `{"type":"control","modelValue":"sampler.player15.gain","modelDescription":"sampler.player15.name"}` |  |
| `sampler.player15.paused` | button-action | [Inferred] sampler / player 15 / paused | `{"type":"button-action","modelState":"sampler.player15.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player15.playing` | button-toggle | [Inferred] sampler / player 15 / playing | `{"type":"button-toggle","modelState":"sampler.player15.statePlaying"}` |  |
| `sampler.player15.playingConsideringHoldSetting` | button-action | Play Sample 15 | `{"type":"button-action","modelState":"sampler.player15.statePlaying"}` |  |
| `sampler.player15.playingHold` | button-hold | [Inferred] sampler / player 15 / playing Hold | `{"type":"button-hold","modelState":"sampler.player15.statePlaying"}` |  |
| `sampler.player15.volume` | control | Sampler 15 Volume | `{"type":"control","modelValue":"sampler.player15.volume","modelDescription":"sampler.player15.name"}` |  |
| `sampler.player16.gain` | control | [Inferred] sampler / player 16 / gain | `{"type":"control","modelValue":"sampler.player16.gain","modelDescription":"sampler.player16.name"}` |  |
| `sampler.player16.paused` | button-action | [Inferred] sampler / player 16 / paused | `{"type":"button-action","modelState":"sampler.player16.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player16.playing` | button-toggle | [Inferred] sampler / player 16 / playing | `{"type":"button-toggle","modelState":"sampler.player16.statePlaying"}` |  |
| `sampler.player16.playingConsideringHoldSetting` | button-action | Play Sample 16 | `{"type":"button-action","modelState":"sampler.player16.statePlaying"}` |  |
| `sampler.player16.playingHold` | button-hold | [Inferred] sampler / player 16 / playing Hold | `{"type":"button-hold","modelState":"sampler.player16.statePlaying"}` |  |
| `sampler.player16.volume` | control | Sampler 16 Volume | `{"type":"control","modelValue":"sampler.player16.volume","modelDescription":"sampler.player16.name"}` |  |
| `sampler.player2.gain` | control | [Inferred] sampler / player 2 / gain | `{"type":"control","modelValue":"sampler.player2.gain","modelDescription":"sampler.player2.name"}` |  |
| `sampler.player2.paused` | button-action | [Inferred] sampler / player 2 / paused | `{"modelEnabled":"sampler.player2.loadingSuccess","modelDescription":"sampler.player2.name","type":"button-action","modelState":"sampler.player2.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player2.playing` | button-toggle | [Inferred] sampler / player 2 / playing | `{"modelEnabled":"sampler.player2.loadingSuccess","type":"button-toggle","modelState":"sampler.player2.statePlaying","modelDescription":"sampler.player2.name"}` |  |
| `sampler.player2.playingConsideringHoldSetting` | button-action | Play Sample 2 | `{"modelEnabled":"sampler.player2.loadingSuccess","type":"button-action","modelState":"sampler.player2.statePlaying","modelDescription":"sampler.player2.name"}` |  |
| `sampler.player2.playingHold` | button-hold | [Inferred] sampler / player 2 / playing Hold | `{"modelEnabled":"sampler.player2.loadingSuccess","type":"button-hold","modelState":"sampler.player2.statePlaying","modelDescription":"sampler.player2.name"}` |  |
| `sampler.player2.volume` | control | Sampler 2 Volume | `{"type":"control","modelValue":"sampler.player2.volume","modelDescription":"sampler.player2.name"}` |  |
| `sampler.player3.gain` | control | [Inferred] sampler / player 3 / gain | `{"type":"control","modelValue":"sampler.player3.gain","modelDescription":"sampler.player3.name"}` |  |
| `sampler.player3.paused` | button-action | [Inferred] sampler / player 3 / paused | `{"modelEnabled":"sampler.player3.loadingSuccess","modelDescription":"sampler.player3.name","type":"button-action","modelState":"sampler.player3.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player3.playing` | button-toggle | [Inferred] sampler / player 3 / playing | `{"modelEnabled":"sampler.player3.loadingSuccess","type":"button-toggle","modelState":"sampler.player3.statePlaying","modelDescription":"sampler.player3.name"}` |  |
| `sampler.player3.playingConsideringHoldSetting` | button-action | Play Sample 3 | `{"modelEnabled":"sampler.player3.loadingSuccess","type":"button-action","modelState":"sampler.player3.statePlaying","modelDescription":"sampler.player3.name"}` |  |
| `sampler.player3.playingHold` | button-hold | [Inferred] sampler / player 3 / playing Hold | `{"modelEnabled":"sampler.player3.loadingSuccess","type":"button-hold","modelState":"sampler.player3.statePlaying","modelDescription":"sampler.player3.name"}` |  |
| `sampler.player3.volume` | control | Sampler 3 Volume | `{"type":"control","modelValue":"sampler.player3.volume","modelDescription":"sampler.player3.name"}` |  |
| `sampler.player4.gain` | control | [Inferred] sampler / player 4 / gain | `{"type":"control","modelValue":"sampler.player4.gain","modelDescription":"sampler.player4.name"}` |  |
| `sampler.player4.paused` | button-action | [Inferred] sampler / player 4 / paused | `{"modelEnabled":"sampler.player4.loadingSuccess","modelDescription":"sampler.player4.name","type":"button-action","modelState":"sampler.player4.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player4.playing` | button-toggle | [Inferred] sampler / player 4 / playing | `{"modelEnabled":"sampler.player4.loadingSuccess","type":"button-toggle","modelState":"sampler.player4.statePlaying","modelDescription":"sampler.player4.name"}` |  |
| `sampler.player4.playingConsideringHoldSetting` | button-action | Play Sample 4 | `{"modelEnabled":"sampler.player4.loadingSuccess","type":"button-action","modelState":"sampler.player4.statePlaying","modelDescription":"sampler.player4.name"}` |  |
| `sampler.player4.playingHold` | button-hold | [Inferred] sampler / player 4 / playing Hold | `{"modelEnabled":"sampler.player4.loadingSuccess","type":"button-hold","modelState":"sampler.player4.statePlaying","modelDescription":"sampler.player4.name"}` |  |
| `sampler.player4.volume` | control | Sampler 4 Volume | `{"type":"control","modelValue":"sampler.player4.volume","modelDescription":"sampler.player4.name"}` |  |
| `sampler.player5.gain` | control | [Inferred] sampler / player 5 / gain | `{"type":"control","modelValue":"sampler.player5.gain","modelDescription":"sampler.player5.name"}` |  |
| `sampler.player5.paused` | button-action | [Inferred] sampler / player 5 / paused | `{"modelEnabled":"sampler.player5.loadingSuccess","modelDescription":"sampler.player5.name","type":"button-action","modelState":"sampler.player5.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player5.playing` | button-toggle | [Inferred] sampler / player 5 / playing | `{"modelEnabled":"sampler.player5.loadingSuccess","type":"button-toggle","modelState":"sampler.player5.statePlaying","modelDescription":"sampler.player5.name"}` |  |
| `sampler.player5.playingConsideringHoldSetting` | button-action | Play Sample 5 | `{"modelEnabled":"sampler.player5.loadingSuccess","type":"button-action","modelState":"sampler.player5.statePlaying","modelDescription":"sampler.player5.name"}` |  |
| `sampler.player5.playingHold` | button-hold | [Inferred] sampler / player 5 / playing Hold | `{"modelEnabled":"sampler.player5.loadingSuccess","type":"button-hold","modelState":"sampler.player5.statePlaying","modelDescription":"sampler.player5.name"}` |  |
| `sampler.player5.volume` | control | Sampler 5 Volume | `{"type":"control","modelValue":"sampler.player5.volume","modelDescription":"sampler.player5.name"}` |  |
| `sampler.player6.gain` | control | [Inferred] sampler / player 6 / gain | `{"type":"control","modelValue":"sampler.player6.gain","modelDescription":"sampler.player6.name"}` |  |
| `sampler.player6.paused` | button-action | [Inferred] sampler / player 6 / paused | `{"modelEnabled":"sampler.player6.loadingSuccess","modelDescription":"sampler.player6.name","type":"button-action","modelState":"sampler.player6.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player6.playing` | button-toggle | [Inferred] sampler / player 6 / playing | `{"modelEnabled":"sampler.player6.loadingSuccess","type":"button-toggle","modelState":"sampler.player6.statePlaying","modelDescription":"sampler.player6.name"}` |  |
| `sampler.player6.playingConsideringHoldSetting` | button-action | Play Sample 6 | `{"modelEnabled":"sampler.player6.loadingSuccess","type":"button-action","modelState":"sampler.player6.statePlaying","modelDescription":"sampler.player6.name"}` |  |
| `sampler.player6.playingHold` | button-hold | [Inferred] sampler / player 6 / playing Hold | `{"modelEnabled":"sampler.player6.loadingSuccess","type":"button-hold","modelState":"sampler.player6.statePlaying","modelDescription":"sampler.player6.name"}` |  |
| `sampler.player6.volume` | control | Sampler 6 Volume | `{"type":"control","modelValue":"sampler.player6.volume","modelDescription":"sampler.player6.name"}` |  |
| `sampler.player7.gain` | control | [Inferred] sampler / player 7 / gain | `{"type":"control","modelValue":"sampler.player7.gain","modelDescription":"sampler.player7.name"}` |  |
| `sampler.player7.paused` | button-action | [Inferred] sampler / player 7 / paused | `{"modelEnabled":"sampler.player7.loadingSuccess","modelDescription":"sampler.player7.name","type":"button-action","modelState":"sampler.player7.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player7.playing` | button-toggle | [Inferred] sampler / player 7 / playing | `{"modelEnabled":"sampler.player7.loadingSuccess","type":"button-toggle","modelState":"sampler.player7.statePlaying","modelDescription":"sampler.player7.name"}` |  |
| `sampler.player7.playingConsideringHoldSetting` | button-action | Play Sample 7 | `{"modelEnabled":"sampler.player7.loadingSuccess","type":"button-action","modelState":"sampler.player7.statePlaying","modelDescription":"sampler.player7.name"}` |  |
| `sampler.player7.playingHold` | button-hold | [Inferred] sampler / player 7 / playing Hold | `{"modelEnabled":"sampler.player7.loadingSuccess","type":"button-hold","modelState":"sampler.player7.statePlaying","modelDescription":"sampler.player7.name"}` |  |
| `sampler.player7.volume` | control | Sampler 7 Volume | `{"type":"control","modelValue":"sampler.player7.volume","modelDescription":"sampler.player7.name"}` |  |
| `sampler.player8.gain` | control | [Inferred] sampler / player 8 / gain | `{"type":"control","modelValue":"sampler.player8.gain","modelDescription":"sampler.player8.name"}` |  |
| `sampler.player8.paused` | button-action | [Inferred] sampler / player 8 / paused | `{"modelEnabled":"sampler.player8.loadingSuccess","modelDescription":"sampler.player8.name","type":"button-action","modelState":"sampler.player8.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player8.playing` | button-toggle | [Inferred] sampler / player 8 / playing | `{"modelEnabled":"sampler.player8.loadingSuccess","type":"button-toggle","modelState":"sampler.player8.statePlaying","modelDescription":"sampler.player8.name"}` |  |
| `sampler.player8.playingConsideringHoldSetting` | button-action | Play Sample 8 | `{"modelEnabled":"sampler.player8.loadingSuccess","type":"button-action","modelState":"sampler.player8.statePlaying","modelDescription":"sampler.player8.name"}` |  |
| `sampler.player8.playingHold` | button-hold | [Inferred] sampler / player 8 / playing Hold | `{"modelEnabled":"sampler.player8.loadingSuccess","type":"button-hold","modelState":"sampler.player8.statePlaying","modelDescription":"sampler.player8.name"}` |  |
| `sampler.player8.volume` | control | Sampler 8 Volume | `{"type":"control","modelValue":"sampler.player8.volume","modelDescription":"sampler.player8.name"}` |  |
| `sampler.player9.gain` | control | [Inferred] sampler / player 9 / gain | `{"type":"control","modelValue":"sampler.player9.gain","modelDescription":"sampler.player9.name"}` |  |
| `sampler.player9.paused` | button-action | [Inferred] sampler / player 9 / paused | `{"modelEnabled":"sampler.player9.loadingSuccess","modelDescription":"sampler.player9.name","type":"button-action","modelState":"sampler.player9.statePlaying","modelStateIsFlipped":true}` |  |
| `sampler.player9.playing` | button-toggle | [Inferred] sampler / player 9 / playing | `{"modelEnabled":"sampler.player9.loadingSuccess","type":"button-toggle","modelState":"sampler.player9.statePlaying","modelDescription":"sampler.player9.name"}` |  |
| `sampler.player9.playingConsideringHoldSetting` | button-action | Play Sample 9 | `{"modelEnabled":"sampler.player9.loadingSuccess","type":"button-action","modelState":"sampler.player9.statePlaying","modelDescription":"sampler.player9.name"}` |  |
| `sampler.player9.playingHold` | button-hold | [Inferred] sampler / player 9 / playing Hold | `{"modelEnabled":"sampler.player9.loadingSuccess","type":"button-hold","modelState":"sampler.player9.statePlaying","modelDescription":"sampler.player9.name"}` |  |
| `sampler.player9.volume` | control | Sampler 9 Volume | `{"type":"control","modelValue":"sampler.player9.volume","modelDescription":"sampler.player9.name"}` |  |
| `sampler.showSampler` | button-action | [Inferred] sampler / show Sampler | `{"type":"button-action","modelState":"view.state.showSampler"}` |  |
| `sampler.toggleSamplerShown` | button-toggle | Toggle Sampler | `{"type":"button-toggle","modelState":"view.state.showSampler"}` |  |
| `sampler.toggleSequencePlayAndRecord` | button-toggle | Toggle Sequence Recording | `{"type":"button-toggle","modelState":"sampler.sequenceRecording"}` |  |
| `sampler.toggleSequencePlaying` | button-toggle | [Inferred] sampler / toggle Sequence Playing | `{"type":"button-toggle","modelState":"sampler.sequencePlaying"}` |  |
| `sampler.toggleSequenceRecording` | button-toggle | [Inferred] sampler / toggle Sequence Recording | `{"type":"button-toggle","modelState":"sampler.sequenceRecording"}` |  |
| `sampler.turntable.player1.gain` | control | [Inferred] sampler / turntable / player 1 / gain | `{"type":"control","modelValue":"sampler.turntable.player1.gain","modelDescription":"sampler.turntable.player1.name"}` |  |
| `sampler.turntable.player1.paused` | button-action | [Inferred] sampler / turntable / player 1 / paused | `{"modelEnabled":"sampler.turntable.player1.loadingSuccess","modelDescription":"sampler.turntable.player1.name","type":"button-action","modelState":"sampler.turntable.player1.statePlaying","modelStateIsFlipped":true}` | Note CH10 data 48; Note CH11 data 48; Note CH12 data 48; Note CH9 data 48 |
| `sampler.turntable.player1.playing` | button-toggle | [Inferred] sampler / turntable / player 1 / playing | `{"modelEnabled":"sampler.turntable.player1.loadingSuccess","type":"button-toggle","modelState":"sampler.turntable.player1.statePlaying","modelDescription":"sampler.turntable.player1.name"}` |  |
| `sampler.turntable.player1.playingConsideringHoldSetting` | button-action | [Inferred] sampler / turntable / player 1 / playing Considering Hold Setting | `{"modelEnabled":"sampler.turntable.player1.loadingSuccess","type":"button-action","modelState":"sampler.turntable.player1.statePlaying","modelDescription":"sampler.turntable.player1.name"}` | Note CH1 data 48; Note CH2 data 48; Note CH3 data 48; Note CH4 data 48 |
| `sampler.turntable.player1.playingHold` | button-hold | [Inferred] sampler / turntable / player 1 / playing Hold | `{"modelEnabled":"sampler.turntable.player1.loadingSuccess","type":"button-hold","modelState":"sampler.turntable.player1.statePlaying","modelDescription":"sampler.turntable.player1.name"}` |  |
| `sampler.turntable.player1.volume` | control | [Inferred] sampler / turntable / player 1 / volume | `{"type":"control","modelValue":"sampler.turntable.player1.volume","modelDescription":"sampler.turntable.player1.name"}` |  |
| `sampler.turntable.player2.gain` | control | [Inferred] sampler / turntable / player 2 / gain | `{"type":"control","modelValue":"sampler.turntable.player2.gain","modelDescription":"sampler.turntable.player2.name"}` |  |
| `sampler.turntable.player2.paused` | button-action | [Inferred] sampler / turntable / player 2 / paused | `{"modelEnabled":"sampler.turntable.player2.loadingSuccess","modelDescription":"sampler.turntable.player2.name","type":"button-action","modelState":"sampler.turntable.player2.statePlaying","modelStateIsFlipped":true}` | Note CH10 data 49; Note CH11 data 49; Note CH12 data 49; Note CH9 data 49 |
| `sampler.turntable.player2.playing` | button-toggle | [Inferred] sampler / turntable / player 2 / playing | `{"modelEnabled":"sampler.turntable.player2.loadingSuccess","type":"button-toggle","modelState":"sampler.turntable.player2.statePlaying","modelDescription":"sampler.turntable.player2.name"}` |  |
| `sampler.turntable.player2.playingConsideringHoldSetting` | button-action | [Inferred] sampler / turntable / player 2 / playing Considering Hold Setting | `{"modelEnabled":"sampler.turntable.player2.loadingSuccess","type":"button-action","modelState":"sampler.turntable.player2.statePlaying","modelDescription":"sampler.turntable.player2.name"}` | Note CH1 data 49; Note CH2 data 49; Note CH3 data 49; Note CH4 data 49 |
| `sampler.turntable.player2.playingHold` | button-hold | [Inferred] sampler / turntable / player 2 / playing Hold | `{"modelEnabled":"sampler.turntable.player2.loadingSuccess","type":"button-hold","modelState":"sampler.turntable.player2.statePlaying","modelDescription":"sampler.turntable.player2.name"}` |  |
| `sampler.turntable.player2.volume` | control | [Inferred] sampler / turntable / player 2 / volume | `{"type":"control","modelValue":"sampler.turntable.player2.volume","modelDescription":"sampler.turntable.player2.name"}` |  |
| `sampler.turntable.player3.gain` | control | [Inferred] sampler / turntable / player 3 / gain | `{"type":"control","modelValue":"sampler.turntable.player3.gain","modelDescription":"sampler.turntable.player3.name"}` |  |
| `sampler.turntable.player3.paused` | button-action | [Inferred] sampler / turntable / player 3 / paused | `{"modelEnabled":"sampler.turntable.player3.loadingSuccess","modelDescription":"sampler.turntable.player3.name","type":"button-action","modelState":"sampler.turntable.player3.statePlaying","modelStateIsFlipped":true}` | Note CH10 data 50; Note CH11 data 50; Note CH12 data 50; Note CH9 data 50 |
| `sampler.turntable.player3.playing` | button-toggle | [Inferred] sampler / turntable / player 3 / playing | `{"modelEnabled":"sampler.turntable.player3.loadingSuccess","type":"button-toggle","modelState":"sampler.turntable.player3.statePlaying","modelDescription":"sampler.turntable.player3.name"}` |  |
| `sampler.turntable.player3.playingConsideringHoldSetting` | button-action | [Inferred] sampler / turntable / player 3 / playing Considering Hold Setting | `{"modelEnabled":"sampler.turntable.player3.loadingSuccess","type":"button-action","modelState":"sampler.turntable.player3.statePlaying","modelDescription":"sampler.turntable.player3.name"}` | Note CH1 data 50; Note CH2 data 50; Note CH3 data 50; Note CH4 data 50 |
| `sampler.turntable.player3.playingHold` | button-hold | [Inferred] sampler / turntable / player 3 / playing Hold | `{"modelEnabled":"sampler.turntable.player3.loadingSuccess","type":"button-hold","modelState":"sampler.turntable.player3.statePlaying","modelDescription":"sampler.turntable.player3.name"}` |  |
| `sampler.turntable.player3.volume` | control | [Inferred] sampler / turntable / player 3 / volume | `{"type":"control","modelValue":"sampler.turntable.player3.volume","modelDescription":"sampler.turntable.player3.name"}` |  |
| `sampler.turntable.player4.gain` | control | [Inferred] sampler / turntable / player 4 / gain | `{"type":"control","modelValue":"sampler.turntable.player4.gain","modelDescription":"sampler.turntable.player4.name"}` |  |
| `sampler.turntable.player4.paused` | button-action | [Inferred] sampler / turntable / player 4 / paused | `{"modelEnabled":"sampler.turntable.player4.loadingSuccess","modelDescription":"sampler.turntable.player4.name","type":"button-action","modelState":"sampler.turntable.player4.statePlaying","modelStateIsFlipped":true}` | Note CH10 data 51; Note CH11 data 51; Note CH12 data 51; Note CH9 data 51 |
| `sampler.turntable.player4.playing` | button-toggle | [Inferred] sampler / turntable / player 4 / playing | `{"modelEnabled":"sampler.turntable.player4.loadingSuccess","type":"button-toggle","modelState":"sampler.turntable.player4.statePlaying","modelDescription":"sampler.turntable.player4.name"}` |  |
| `sampler.turntable.player4.playingConsideringHoldSetting` | button-action | [Inferred] sampler / turntable / player 4 / playing Considering Hold Setting | `{"modelEnabled":"sampler.turntable.player4.loadingSuccess","type":"button-action","modelState":"sampler.turntable.player4.statePlaying","modelDescription":"sampler.turntable.player4.name"}` | Note CH1 data 51; Note CH2 data 51; Note CH3 data 51; Note CH4 data 51 |
| `sampler.turntable.player4.playingHold` | button-hold | [Inferred] sampler / turntable / player 4 / playing Hold | `{"modelEnabled":"sampler.turntable.player4.loadingSuccess","type":"button-hold","modelState":"sampler.turntable.player4.statePlaying","modelDescription":"sampler.turntable.player4.name"}` |  |
| `sampler.turntable.player4.volume` | control | [Inferred] sampler / turntable / player 4 / volume | `{"type":"control","modelValue":"sampler.turntable.player4.volume","modelDescription":"sampler.turntable.player4.name"}` |  |
| `sampler.turntable.player5.gain` | control | [Inferred] sampler / turntable / player 5 / gain | `{"type":"control","modelValue":"sampler.turntable.player5.gain","modelDescription":"sampler.turntable.player5.name"}` |  |
| `sampler.turntable.player5.paused` | button-action | [Inferred] sampler / turntable / player 5 / paused | `{"modelEnabled":"sampler.turntable.player5.loadingSuccess","modelDescription":"sampler.turntable.player5.name","type":"button-action","modelState":"sampler.turntable.player5.statePlaying","modelStateIsFlipped":true}` | Note CH10 data 52; Note CH11 data 52; Note CH12 data 52; Note CH9 data 52 |
| `sampler.turntable.player5.playing` | button-toggle | [Inferred] sampler / turntable / player 5 / playing | `{"modelEnabled":"sampler.turntable.player5.loadingSuccess","type":"button-toggle","modelState":"sampler.turntable.player5.statePlaying","modelDescription":"sampler.turntable.player5.name"}` |  |
| `sampler.turntable.player5.playingConsideringHoldSetting` | button-action | [Inferred] sampler / turntable / player 5 / playing Considering Hold Setting | `{"modelEnabled":"sampler.turntable.player5.loadingSuccess","type":"button-action","modelState":"sampler.turntable.player5.statePlaying","modelDescription":"sampler.turntable.player5.name"}` | Note CH1 data 52; Note CH2 data 52; Note CH3 data 52; Note CH4 data 52 |
| `sampler.turntable.player5.playingHold` | button-hold | [Inferred] sampler / turntable / player 5 / playing Hold | `{"modelEnabled":"sampler.turntable.player5.loadingSuccess","type":"button-hold","modelState":"sampler.turntable.player5.statePlaying","modelDescription":"sampler.turntable.player5.name"}` |  |
| `sampler.turntable.player5.volume` | control | [Inferred] sampler / turntable / player 5 / volume | `{"type":"control","modelValue":"sampler.turntable.player5.volume","modelDescription":"sampler.turntable.player5.name"}` |  |
| `sampler.turntable.player6.gain` | control | [Inferred] sampler / turntable / player 6 / gain | `{"type":"control","modelValue":"sampler.turntable.player6.gain","modelDescription":"sampler.turntable.player6.name"}` |  |
| `sampler.turntable.player6.paused` | button-action | [Inferred] sampler / turntable / player 6 / paused | `{"modelEnabled":"sampler.turntable.player6.loadingSuccess","modelDescription":"sampler.turntable.player6.name","type":"button-action","modelState":"sampler.turntable.player6.statePlaying","modelStateIsFlipped":true}` | Note CH10 data 53; Note CH11 data 53; Note CH12 data 53; Note CH9 data 53 |
| `sampler.turntable.player6.playing` | button-toggle | [Inferred] sampler / turntable / player 6 / playing | `{"modelEnabled":"sampler.turntable.player6.loadingSuccess","type":"button-toggle","modelState":"sampler.turntable.player6.statePlaying","modelDescription":"sampler.turntable.player6.name"}` |  |
| `sampler.turntable.player6.playingConsideringHoldSetting` | button-action | [Inferred] sampler / turntable / player 6 / playing Considering Hold Setting | `{"modelEnabled":"sampler.turntable.player6.loadingSuccess","type":"button-action","modelState":"sampler.turntable.player6.statePlaying","modelDescription":"sampler.turntable.player6.name"}` | Note CH1 data 53; Note CH2 data 53; Note CH3 data 53; Note CH4 data 53 |
| `sampler.turntable.player6.playingHold` | button-hold | [Inferred] sampler / turntable / player 6 / playing Hold | `{"modelEnabled":"sampler.turntable.player6.loadingSuccess","type":"button-hold","modelState":"sampler.turntable.player6.statePlaying","modelDescription":"sampler.turntable.player6.name"}` |  |
| `sampler.turntable.player6.volume` | control | [Inferred] sampler / turntable / player 6 / volume | `{"type":"control","modelValue":"sampler.turntable.player6.volume","modelDescription":"sampler.turntable.player6.name"}` |  |
| `sampler.turntable.player7.gain` | control | [Inferred] sampler / turntable / player 7 / gain | `{"type":"control","modelValue":"sampler.turntable.player7.gain","modelDescription":"sampler.turntable.player7.name"}` |  |
| `sampler.turntable.player7.paused` | button-action | [Inferred] sampler / turntable / player 7 / paused | `{"modelEnabled":"sampler.turntable.player7.loadingSuccess","modelDescription":"sampler.turntable.player7.name","type":"button-action","modelState":"sampler.turntable.player7.statePlaying","modelStateIsFlipped":true}` | Note CH10 data 54; Note CH11 data 54; Note CH12 data 54; Note CH9 data 54 |
| `sampler.turntable.player7.playing` | button-toggle | [Inferred] sampler / turntable / player 7 / playing | `{"modelEnabled":"sampler.turntable.player7.loadingSuccess","type":"button-toggle","modelState":"sampler.turntable.player7.statePlaying","modelDescription":"sampler.turntable.player7.name"}` |  |
| `sampler.turntable.player7.playingConsideringHoldSetting` | button-action | [Inferred] sampler / turntable / player 7 / playing Considering Hold Setting | `{"modelEnabled":"sampler.turntable.player7.loadingSuccess","type":"button-action","modelState":"sampler.turntable.player7.statePlaying","modelDescription":"sampler.turntable.player7.name"}` | Note CH1 data 54; Note CH2 data 54; Note CH3 data 54; Note CH4 data 54 |
| `sampler.turntable.player7.playingHold` | button-hold | [Inferred] sampler / turntable / player 7 / playing Hold | `{"modelEnabled":"sampler.turntable.player7.loadingSuccess","type":"button-hold","modelState":"sampler.turntable.player7.statePlaying","modelDescription":"sampler.turntable.player7.name"}` |  |
| `sampler.turntable.player7.volume` | control | [Inferred] sampler / turntable / player 7 / volume | `{"type":"control","modelValue":"sampler.turntable.player7.volume","modelDescription":"sampler.turntable.player7.name"}` |  |
| `sampler.turntable.player8.gain` | control | [Inferred] sampler / turntable / player 8 / gain | `{"type":"control","modelValue":"sampler.turntable.player8.gain","modelDescription":"sampler.turntable.player8.name"}` |  |
| `sampler.turntable.player8.paused` | button-action | [Inferred] sampler / turntable / player 8 / paused | `{"modelEnabled":"sampler.turntable.player8.loadingSuccess","modelDescription":"sampler.turntable.player8.name","type":"button-action","modelState":"sampler.turntable.player8.statePlaying","modelStateIsFlipped":true}` | Note CH10 data 55; Note CH11 data 55; Note CH12 data 55; Note CH9 data 55 |
| `sampler.turntable.player8.playing` | button-toggle | [Inferred] sampler / turntable / player 8 / playing | `{"modelEnabled":"sampler.turntable.player8.loadingSuccess","type":"button-toggle","modelState":"sampler.turntable.player8.statePlaying","modelDescription":"sampler.turntable.player8.name"}` |  |
| `sampler.turntable.player8.playingConsideringHoldSetting` | button-action | [Inferred] sampler / turntable / player 8 / playing Considering Hold Setting | `{"modelEnabled":"sampler.turntable.player8.loadingSuccess","type":"button-action","modelState":"sampler.turntable.player8.statePlaying","modelDescription":"sampler.turntable.player8.name"}` | Note CH1 data 55; Note CH2 data 55; Note CH3 data 55; Note CH4 data 55 |
| `sampler.turntable.player8.playingHold` | button-hold | [Inferred] sampler / turntable / player 8 / playing Hold | `{"modelEnabled":"sampler.turntable.player8.loadingSuccess","type":"button-hold","modelState":"sampler.turntable.player8.statePlaying","modelDescription":"sampler.turntable.player8.name"}` |  |
| `sampler.turntable.player8.volume` | control | [Inferred] sampler / turntable / player 8 / volume | `{"type":"control","modelValue":"sampler.turntable.player8.volume","modelDescription":"sampler.turntable.player8.name"}` |  |
| `sampler.volume` | control | Volume | `{"type":"control","modelValue":"sampler.volume"}` |  |

### sequencer (1)

| Djay input key path | Type | Human description | Metadata fields | S4 MIDI usage |
|---|---|---|---|---|
| `sequencer.toggleMetronomeEnabled` | button-toggle | [Inferred] sequencer / toggle Metronome Enabled | `{"type":"button-toggle","modelState":"sequencer.metronomeEnabled"}` |  |

### turntable (625)

| Djay input key path | Type | Human description | Metadata fields | S4 MIDI usage |
|---|---|---|---|---|
| `turntable.autoLoop003125BeatInterval` | button-toggle | Auto Loop 1/32 | `{"type":"button-toggle","modelState":"turntable.autoLoop003125BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop00625BeatInterval` | button-toggle | Auto Loop 1/16 | `{"type":"button-toggle","modelState":"turntable.autoLoop00625BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop0125BeatInterval` | button-toggle | Auto Loop 1/8 | `{"type":"button-toggle","modelState":"turntable.autoLoop0125BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop025BeatInterval` | button-toggle | Auto Loop 1/4 | `{"type":"button-toggle","modelState":"turntable.autoLoop025BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop033BeatInterval` | button-toggle | Auto Loop 1/3 | `{"type":"button-toggle","modelState":"turntable.autoLoop033BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop05BeatInterval` | button-toggle | Auto Loop 1/2 | `{"type":"button-toggle","modelState":"turntable.autoLoop05BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop075BeatInterval` | button-toggle | Auto Loop 3/4 | `{"type":"button-toggle","modelState":"turntable.autoLoop075BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop1_5BeatInterval` | button-toggle | Auto Loop 3/2 | `{"type":"button-toggle","modelState":"turntable.autoLoop1_5BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop128BeatInterval` | button-toggle | Auto Loop 128 | `{"type":"button-toggle","modelState":"turntable.autoLoop128BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop12BeatInterval` | button-toggle | Auto Loop 12 | `{"type":"button-toggle","modelState":"turntable.autoLoop12BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop16BeatInterval` | button-toggle | Auto Loop 16 | `{"type":"button-toggle","modelState":"turntable.autoLoop16BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop1BeatInterval` | button-toggle | Auto Loop 1 | `{"type":"button-toggle","modelState":"turntable.autoLoop1BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop2BeatInterval` | button-toggle | Auto Loop 2 | `{"type":"button-toggle","modelState":"turntable.autoLoop2BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop32BeatInterval` | button-toggle | Auto Loop 32 | `{"type":"button-toggle","modelState":"turntable.autoLoop32BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop3BeatInterval` | button-toggle | Auto Loop 3 | `{"type":"button-toggle","modelState":"turntable.autoLoop3BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop4BeatInterval` | button-toggle | Auto Loop 4 | `{"type":"button-toggle","modelState":"turntable.autoLoop4BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop5BeatInterval` | button-toggle | Auto Loop 5 | `{"type":"button-toggle","modelState":"turntable.autoLoop5BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop64BeatInterval` | button-toggle | Auto Loop 64 | `{"type":"button-toggle","modelState":"turntable.autoLoop64BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop6BeatInterval` | button-toggle | Auto Loop 6 | `{"type":"button-toggle","modelState":"turntable.autoLoop6BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop7BeatInterval` | button-toggle | Auto Loop 7 | `{"type":"button-toggle","modelState":"turntable.autoLoop7BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop8BeatInterval` | button-toggle | Auto Loop 8 | `{"type":"button-toggle","modelState":"turntable.autoLoop8BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoop9BeatInterval` | button-toggle | Auto Loop 9 | `{"type":"button-toggle","modelState":"turntable.autoLoop9BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.autoLoopDurationRotary` | not declared | Loop Duration (Rotary) | `{"steppedRotary":true}` | CC CH1 data 1 (rotary-64); CC CH2 data 1 (rotary-64); CC CH3 data 1 (rotary-64); CC CH4 data 1 (rotary-64) |
| `turntable.autoLoopMoveRotary` | not declared | Loop Move (Rotary) | `{"steppedRotary":true}` | CC CH1 data 2 (rotary-64); CC CH10 data 1 (rotary-64); CC CH11 data 1 (rotary-64); CC CH12 data 1 (rotary-64); CC CH2 data 2 (rotary-64); CC CH3 data 2 (rotary-64); CC CH4 data 2 (rotary-64); CC CH9 data 1 (rotary-64) |
| `turntable.autoLoopOnOff` | button-toggle | Auto Loop | `{"type":"button-toggle","modelState":"turntable.loopingEnabledInView","modelEnabled":"turntable.transportControlsAvailable"}` | Note CH1 data 6; Note CH2 data 6; Note CH3 data 6; Note CH4 data 6 |
| `turntable.autoRepeat` | button-toggle | [Inferred] turntable / auto Repeat | `{"type":"button-toggle","modelState":"turntable.autoRepeat","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.backspin` | button-action | Backspin | `{"type":"button-action","modelEnabled":"turntable.playbackControlsAvailable"}` |  |
| `turntable.bounceLoop003125BeatInterval` | button-hold | Bounce Loop 1/32 | `{"type":"button-hold","modelState":"turntable.bounceLoop003125BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoop00625BeatInterval` | button-hold | Bounce Loop 1/16 | `{"type":"button-hold","modelState":"turntable.bounceLoop00625BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoop0125BeatInterval` | button-hold | Bounce Loop 1/8 | `{"type":"button-hold","modelState":"turntable.bounceLoop0125BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoop025BeatInterval` | button-hold | Bounce Loop 1/4 | `{"type":"button-hold","modelState":"turntable.bounceLoop025BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoop033BeatInterval` | button-hold | Bounce Loop 1/3 | `{"type":"button-hold","modelState":"turntable.bounceLoop033BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoop05BeatInterval` | button-hold | Bounce Loop 1/2 | `{"type":"button-hold","modelState":"turntable.bounceLoop05BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoop075BeatInterval` | button-hold | Bounce Loop 3/4 | `{"type":"button-hold","modelState":"turntable.bounceLoop075BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoop16BeatInterval` | button-hold | Bounce Loop 16 | `{"type":"button-hold","modelState":"turntable.bounceLoop16BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoop1BeatInterval` | button-hold | Bounce Loop 1 | `{"type":"button-hold","modelState":"turntable.bounceLoop1BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoop2BeatInterval` | button-hold | Bounce Loop 2 | `{"type":"button-hold","modelState":"turntable.bounceLoop2BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoop32BeatInterval` | button-hold | Bounce Loop 32 | `{"type":"button-hold","modelState":"turntable.bounceLoop32BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoop4BeatInterval` | button-hold | Bounce Loop 4 | `{"type":"button-hold","modelState":"turntable.bounceLoop4BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoop8BeatInterval` | button-hold | Bounce Loop 8 | `{"type":"button-hold","modelState":"turntable.bounceLoop8BeatIntervalActive","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoopDrums003125BeatInterval` | button-hold | Bounce Loop Drums 1/32 | `{"type":"button-hold","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoopDrums00625BeatInterval` | button-hold | Bounce Loop Drums 1/16 | `{"type":"button-hold","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoopDrums0125BeatInterval` | button-hold | Bounce Loop Drums 1/8 | `{"type":"button-hold","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoopDrums025BeatInterval` | button-hold | Bounce Loop Drums 1/4 | `{"type":"button-hold","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoopDrums05BeatInterval` | button-hold | Bounce Loop Drums 1/2 | `{"type":"button-hold","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoopDrums1BeatInterval` | button-hold | Bounce Loop Drums 1 | `{"type":"button-hold","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoopDrums2BeatInterval` | button-hold | Bounce Loop Drums 2 | `{"type":"button-hold","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.bounceLoopRouting.routingDeck` | button-action | Bounce Loop Routing Deck | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.isDeck"}` |  |
| `turntable.bounceLoopRouting.routingFourTrackChannel1` | button-action | Bounce Loop Routing (4ch: Drums) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.fourTrackChannel1"}` |  |
| `turntable.bounceLoopRouting.routingFourTrackChannel2` | button-action | Bounce Loop Routing (4ch: Bass) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.fourTrackChannel2"}` |  |
| `turntable.bounceLoopRouting.routingFourTrackChannel3` | button-action | Bounce Loop Routing (4ch: Harmonic) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.fourTrackChannel3"}` |  |
| `turntable.bounceLoopRouting.routingFourTrackChannel4` | button-action | Bounce Loop Routing (4ch: Vocals) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.fourTrackChannel4"}` |  |
| `turntable.bounceLoopRouting.routingGenericChannel1` | button-action | Bounce Loop Routing (1) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.genericChannel1","modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeOne"}` |  |
| `turntable.bounceLoopRouting.routingGenericChannel2` | button-action | Bounce Loop Routing (2) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.genericChannel2","modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeTwo"}` |  |
| `turntable.bounceLoopRouting.routingGenericChannel3` | button-action | Bounce Loop Routing (3) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.genericChannel3","modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeThree"}` |  |
| `turntable.bounceLoopRouting.routingGenericChannel4` | button-action | Bounce Loop Routing (4) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.genericChannel4","modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeFour"}` |  |
| `turntable.bounceLoopRouting.routingThreeTrackChannel1` | button-action | Bounce Loop Routing (3ch: Drums) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.threeTrackChannel1"}` |  |
| `turntable.bounceLoopRouting.routingThreeTrackChannel2` | button-action | Bounce Loop Routing (3ch: Harmonic / Bass) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.threeTrackChannel2"}` |  |
| `turntable.bounceLoopRouting.routingThreeTrackChannel3` | button-action | Bounce Loop Routing (3ch: Vocals / Melodic) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.threeTrackChannel3"}` |  |
| `turntable.bounceLoopRouting.routingTwoTrackChannel1` | button-action | Bounce Loop Routing (2ch: Instrumental / Drums) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.twoTrackChannel1"}` |  |
| `turntable.bounceLoopRouting.routingTwoTrackChannel1Instrumental` | button-action | Bounce Loop Routing Instrumental | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.twoTrackChannel1"}` |  |
| `turntable.bounceLoopRouting.routingTwoTrackChannel2` | button-action | Bounce Loop Routing (2ch: Acappella / Tonal) | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.twoTrackChannel2"}` |  |
| `turntable.bounceLoopRouting.routingTwoTrackChannel2Acapella` | button-action | Bounce Loop Routing Acappella | `{"type":"button-action","modelState":"turntable.bounceLoopRouting.twoTrackChannel2"}` |  |
| `turntable.bounceLoopRoutingActiveToggle` | button-toggle | Toggle Bounce Loop Routing (Neural Mix) | `{"type":"button-toggle","modelState":"turntable.bounceLoopRouting.isDeck","modelStateIsFlipped":true}` |  |
| `turntable.bpmDouble` | button-action | [Inferred] turntable / bpm Double | `{"type":"button-action","modelEnabled":"turntable.song.canDoubleBPM"}` |  |
| `turntable.bpmHalf` | button-action | [Inferred] turntable / bpm Half | `{"type":"button-action","modelEnabled":"turntable.song.canHalfBPM"}` |  |
| `turntable.bpmSync` | button-toggle | Sync | `{"type":"button-toggle","modelState":"turntable.syncButtonState","modelEnabled":"turntable.canSync"}` | Note CH1 data 2; Note CH2 data 2; Note CH3 data 2; Note CH4 data 2 |
| `turntable.brakeTransitionEffect` | button-action | Brake Transition FX | `{"type":"button-action","modelEnabled":"turntable.playbackControlsAvailable"}` |  |
| `turntable.censor` | button-hold | Censor | `{"type":"button-hold","modelState":"turntable.reverse"}` |  |
| `turntable.clearAutomixStartPoint` | button-action | Clear Automix Start Point | `{"type":"button-action","modelState":"turntable.song.automixStartPoint.hasStart"}` |  |
| `turntable.clearCuePoint1` | button-action | Clear Cue 1 / 9 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex1","type":"button-action","modelState":"turntable.cuePointPadIsSet1","modelDescription":"turntable.cuePointPadName1"}` | Note CH10 data 40; Note CH11 data 40; Note CH12 data 40; Note CH9 data 40 |
| `turntable.clearCuePoint2` | button-action | Clear Cue 2 / 10 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex2","type":"button-action","modelState":"turntable.cuePointPadIsSet2","modelDescription":"turntable.cuePointPadName2"}` | Note CH10 data 41; Note CH11 data 41; Note CH12 data 41; Note CH9 data 41 |
| `turntable.clearCuePoint3` | button-action | Clear Cue 3 / 11 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex3","type":"button-action","modelState":"turntable.cuePointPadIsSet3","modelDescription":"turntable.cuePointPadName3"}` | Note CH10 data 42; Note CH11 data 42; Note CH12 data 42; Note CH9 data 42 |
| `turntable.clearCuePoint4` | button-action | Clear Cue 4 / 12 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex4","type":"button-action","modelState":"turntable.cuePointPadIsSet4","modelDescription":"turntable.cuePointPadName4"}` | Note CH10 data 43; Note CH11 data 43; Note CH12 data 43; Note CH9 data 43 |
| `turntable.clearCuePoint5` | button-action | Clear Cue 5 / 13 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex5","type":"button-action","modelState":"turntable.cuePointPadIsSet5","modelDescription":"turntable.cuePointPadName5"}` | Note CH10 data 44; Note CH11 data 44; Note CH12 data 44; Note CH9 data 44 |
| `turntable.clearCuePoint6` | button-action | Clear Cue 6 / 14 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex6","type":"button-action","modelState":"turntable.cuePointPadIsSet6","modelDescription":"turntable.cuePointPadName6"}` | Note CH10 data 45; Note CH11 data 45; Note CH12 data 45; Note CH9 data 45 |
| `turntable.clearCuePoint7` | button-action | Clear Cue 7 / 15 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex7","type":"button-action","modelState":"turntable.cuePointPadIsSet7","modelDescription":"turntable.cuePointPadName7"}` | Note CH10 data 46; Note CH11 data 46; Note CH12 data 46; Note CH9 data 46 |
| `turntable.clearCuePoint8` | button-action | Clear Cue 8 / 16 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex8","type":"button-action","modelState":"turntable.cuePointPadIsSet8","modelDescription":"turntable.cuePointPadName8"}` | Note CH10 data 47; Note CH11 data 47; Note CH12 data 47; Note CH9 data 47 |
| `turntable.clearCuePointCuePoint1` | button-action | Clear Cue 1 | `{"modelPadColorIndex":"turntable.song.cuePoint1.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint1.hasStart","modelDescription":"turntable.song.cuePoint1.name"}` |  |
| `turntable.clearCuePointCuePoint10` | button-action | Clear Cue 10 | `{"modelPadColorIndex":"turntable.song.cuePoint10.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint10.hasStart","modelDescription":"turntable.song.cuePoint10.name"}` |  |
| `turntable.clearCuePointCuePoint11` | button-action | Clear Cue 11 | `{"modelPadColorIndex":"turntable.song.cuePoint11.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint11.hasStart","modelDescription":"turntable.song.cuePoint11.name"}` |  |
| `turntable.clearCuePointCuePoint12` | button-action | Clear Cue 12 | `{"modelPadColorIndex":"turntable.song.cuePoint12.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint12.hasStart","modelDescription":"turntable.song.cuePoint12.name"}` |  |
| `turntable.clearCuePointCuePoint13` | button-action | Clear Cue 13 | `{"modelPadColorIndex":"turntable.song.cuePoint13.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint13.hasStart","modelDescription":"turntable.song.cuePoint13.name"}` |  |
| `turntable.clearCuePointCuePoint14` | button-action | Clear Cue 14 | `{"modelPadColorIndex":"turntable.song.cuePoint14.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint14.hasStart","modelDescription":"turntable.song.cuePoint14.name"}` |  |
| `turntable.clearCuePointCuePoint15` | button-action | Clear Cue 15 | `{"modelPadColorIndex":"turntable.song.cuePoint15.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint15.hasStart","modelDescription":"turntable.song.cuePoint15.name"}` |  |
| `turntable.clearCuePointCuePoint16` | button-action | Clear Cue 16 | `{"modelPadColorIndex":"turntable.song.cuePoint16.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint16.hasStart","modelDescription":"turntable.song.cuePoint16.name"}` |  |
| `turntable.clearCuePointCuePoint2` | button-action | Clear Cue 2 | `{"modelPadColorIndex":"turntable.song.cuePoint2.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint2.hasStart","modelDescription":"turntable.song.cuePoint2.name"}` |  |
| `turntable.clearCuePointCuePoint3` | button-action | Clear Cue 3 | `{"modelPadColorIndex":"turntable.song.cuePoint3.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint3.hasStart","modelDescription":"turntable.song.cuePoint3.name"}` |  |
| `turntable.clearCuePointCuePoint4` | button-action | Clear Cue 4 | `{"modelPadColorIndex":"turntable.song.cuePoint4.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint4.hasStart","modelDescription":"turntable.song.cuePoint4.name"}` |  |
| `turntable.clearCuePointCuePoint5` | button-action | Clear Cue 5 | `{"modelPadColorIndex":"turntable.song.cuePoint5.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint5.hasStart","modelDescription":"turntable.song.cuePoint5.name"}` |  |
| `turntable.clearCuePointCuePoint6` | button-action | Clear Cue 6 | `{"modelPadColorIndex":"turntable.song.cuePoint6.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint6.hasStart","modelDescription":"turntable.song.cuePoint6.name"}` |  |
| `turntable.clearCuePointCuePoint7` | button-action | Clear Cue 7 | `{"modelPadColorIndex":"turntable.song.cuePoint7.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint7.hasStart","modelDescription":"turntable.song.cuePoint7.name"}` |  |
| `turntable.clearCuePointCuePoint8` | button-action | Clear Cue 8 | `{"modelPadColorIndex":"turntable.song.cuePoint8.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint8.hasStart","modelDescription":"turntable.song.cuePoint8.name"}` |  |
| `turntable.clearCuePointCuePoint9` | button-action | Clear Cue 9 | `{"modelPadColorIndex":"turntable.song.cuePoint9.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint9.hasStart","modelDescription":"turntable.song.cuePoint9.name"}` |  |
| `turntable.clearEndPoint` | button-action | Clear Automix End Point | `{"type":"button-action","modelState":"turntable.song.automixEndPoint.hasStart"}` |  |
| `turntable.clearStartPoint` | button-action | Clear Start CUE | `{"type":"button-action","modelState":"turntable.song.cuePointStart.hasStart"}` |  |
| `turntable.controlModeAbsolute` | button-trigger | Control Mode Absolute | `{"type":"button-trigger","modelState":"turntable.controlModeIsAbsolute"}` |  |
| `turntable.controlModeInternal` | button-trigger | Control Mode Internal | `{"type":"button-trigger","modelState":"turntable.controlModeIsInternal"}` |  |
| `turntable.controlModeRelative` | button-trigger | Control Mode Relative | `{"type":"button-trigger","modelState":"turntable.controlModeIsRelative"}` |  |
| `turntable.controlModeThru` | button-trigger | Control Mode Thru | `{"type":"button-trigger","modelState":"turntable.controlModeIsThru"}` |  |
| `turntable.controlModeWireless` | button-trigger | Control Mode Wireless | `{"type":"button-trigger","modelState":"turntable.controlModeIsWireless"}` |  |
| `turntable.cueOrJumpIfAlreadySet1` | button-action | Cue 1 / 9 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex1","type":"button-action","modelState":"turntable.cuePointPadIsSet1","modelDescription":"turntable.cuePointPadName1"}` | Note CH1 data 40; Note CH2 data 40; Note CH3 data 40; Note CH4 data 40 |
| `turntable.cueOrJumpIfAlreadySet2` | button-action | Cue 2 / 10 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex2","type":"button-action","modelState":"turntable.cuePointPadIsSet2","modelDescription":"turntable.cuePointPadName2"}` | Note CH1 data 41; Note CH2 data 41; Note CH3 data 41; Note CH4 data 41 |
| `turntable.cueOrJumpIfAlreadySet3` | button-action | Cue 3 / 11 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex3","type":"button-action","modelState":"turntable.cuePointPadIsSet3","modelDescription":"turntable.cuePointPadName3"}` | Note CH1 data 42; Note CH2 data 42; Note CH3 data 42; Note CH4 data 42 |
| `turntable.cueOrJumpIfAlreadySet4` | button-action | Cue 4 / 12 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex4","type":"button-action","modelState":"turntable.cuePointPadIsSet4","modelDescription":"turntable.cuePointPadName4"}` | Note CH1 data 43; Note CH2 data 43; Note CH3 data 43; Note CH4 data 43 |
| `turntable.cueOrJumpIfAlreadySet5` | button-action | Cue 5 / 13 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex5","type":"button-action","modelState":"turntable.cuePointPadIsSet5","modelDescription":"turntable.cuePointPadName5"}` | Note CH1 data 44; Note CH2 data 44; Note CH3 data 44; Note CH4 data 44 |
| `turntable.cueOrJumpIfAlreadySet6` | button-action | Cue 6 / 14 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex6","type":"button-action","modelState":"turntable.cuePointPadIsSet6","modelDescription":"turntable.cuePointPadName6"}` | Note CH1 data 45; Note CH2 data 45; Note CH3 data 45; Note CH4 data 45 |
| `turntable.cueOrJumpIfAlreadySet7` | button-action | Cue 7 / 15 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex7","type":"button-action","modelState":"turntable.cuePointPadIsSet7","modelDescription":"turntable.cuePointPadName7"}` | Note CH1 data 46; Note CH2 data 46; Note CH3 data 46; Note CH4 data 46 |
| `turntable.cueOrJumpIfAlreadySet8` | button-action | Cue 8 / 16 | `{"modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex8","type":"button-action","modelState":"turntable.cuePointPadIsSet8","modelDescription":"turntable.cuePointPadName8"}` | Note CH1 data 47; Note CH2 data 47; Note CH3 data 47; Note CH4 data 47 |
| `turntable.cueOrJumpIfAlreadySetAndAutoloop1` | button-action | Cue Loop 1 / 9 | `{"modelDescription":"turntable.cuePointPadName1","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime1","modelState":"turntable.cuePointPadIsSet1"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloop2` | button-action | Cue Loop 2 / 10 | `{"modelDescription":"turntable.cuePointPadName2","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime2","modelState":"turntable.cuePointPadIsSet2"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloop3` | button-action | Cue Loop 3 / 11 | `{"modelDescription":"turntable.cuePointPadName3","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime3","modelState":"turntable.cuePointPadIsSet3"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloop4` | button-action | Cue Loop 4 / 12 | `{"modelDescription":"turntable.cuePointPadName4","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime4","modelState":"turntable.cuePointPadIsSet4"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloop5` | button-action | Cue Loop 5 / 13 | `{"modelDescription":"turntable.cuePointPadName5","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime5","modelState":"turntable.cuePointPadIsSet5"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloop6` | button-action | Cue Loop 6 / 14 | `{"modelDescription":"turntable.cuePointPadName6","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime6","modelState":"turntable.cuePointPadIsSet6"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloop7` | button-action | Cue Loop 7 / 15 | `{"modelDescription":"turntable.cuePointPadName7","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime7","modelState":"turntable.cuePointPadIsSet7"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloop8` | button-action | Cue Loop 8 / 16 | `{"modelDescription":"turntable.cuePointPadName8","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime8","modelState":"turntable.cuePointPadIsSet8"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint1` | button-action | Cue Loop 1 | `{"modelDescription":"turntable.song.cuePoint1.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint1.loopingFromStartTime","modelState":"turntable.song.cuePoint1.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint10` | button-action | Cue Loop 10 | `{"modelDescription":"turntable.song.cuePoint10.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint10.loopingFromStartTime","modelState":"turntable.song.cuePoint10.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint11` | button-action | Cue Loop 11 | `{"modelDescription":"turntable.song.cuePoint11.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint11.loopingFromStartTime","modelState":"turntable.song.cuePoint11.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint12` | button-action | Cue Loop 12 | `{"modelDescription":"turntable.song.cuePoint12.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint12.loopingFromStartTime","modelState":"turntable.song.cuePoint12.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint13` | button-action | Cue Loop 13 | `{"modelDescription":"turntable.song.cuePoint13.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint13.loopingFromStartTime","modelState":"turntable.song.cuePoint13.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint14` | button-action | Cue Loop 14 | `{"modelDescription":"turntable.song.cuePoint14.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint14.loopingFromStartTime","modelState":"turntable.song.cuePoint14.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint15` | button-action | Cue Loop 15 | `{"modelDescription":"turntable.song.cuePoint15.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint15.loopingFromStartTime","modelState":"turntable.song.cuePoint15.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint16` | button-action | Cue Loop 16 | `{"modelDescription":"turntable.song.cuePoint16.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint16.loopingFromStartTime","modelState":"turntable.song.cuePoint16.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint2` | button-action | Cue Loop 2 | `{"modelDescription":"turntable.song.cuePoint2.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint2.loopingFromStartTime","modelState":"turntable.song.cuePoint2.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint3` | button-action | Cue Loop 3 | `{"modelDescription":"turntable.song.cuePoint3.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint3.loopingFromStartTime","modelState":"turntable.song.cuePoint3.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint4` | button-action | Cue Loop 4 | `{"modelDescription":"turntable.song.cuePoint4.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint4.loopingFromStartTime","modelState":"turntable.song.cuePoint4.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint5` | button-action | Cue Loop 5 | `{"modelDescription":"turntable.song.cuePoint5.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint5.loopingFromStartTime","modelState":"turntable.song.cuePoint5.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint6` | button-action | Cue Loop 6 | `{"modelDescription":"turntable.song.cuePoint6.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint6.loopingFromStartTime","modelState":"turntable.song.cuePoint6.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint7` | button-action | Cue Loop 7 | `{"modelDescription":"turntable.song.cuePoint7.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint7.loopingFromStartTime","modelState":"turntable.song.cuePoint7.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint8` | button-action | Cue Loop 8 | `{"modelDescription":"turntable.song.cuePoint8.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint8.loopingFromStartTime","modelState":"turntable.song.cuePoint8.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint9` | button-action | Cue Loop 9 | `{"modelDescription":"turntable.song.cuePoint9.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint9.loopingFromStartTime","modelState":"turntable.song.cuePoint9.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloop1` | button-action | Cue Reloop 1 / 9 | `{"modelDescription":"turntable.cuePointPadName1","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime1","modelState":"turntable.cuePointPadIsSet1"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloop2` | button-action | Cue Reloop 2 / 10 | `{"modelDescription":"turntable.cuePointPadName2","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime2","modelState":"turntable.cuePointPadIsSet2"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloop3` | button-action | Cue Reloop 3 / 11 | `{"modelDescription":"turntable.cuePointPadName3","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime3","modelState":"turntable.cuePointPadIsSet3"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloop4` | button-action | Cue Reloop 4 / 12 | `{"modelDescription":"turntable.cuePointPadName4","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime4","modelState":"turntable.cuePointPadIsSet4"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloop5` | button-action | Cue Reloop 5 / 13 | `{"modelDescription":"turntable.cuePointPadName5","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime5","modelState":"turntable.cuePointPadIsSet5"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloop6` | button-action | Cue Reloop 6 / 14 | `{"modelDescription":"turntable.cuePointPadName6","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime6","modelState":"turntable.cuePointPadIsSet6"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloop7` | button-action | Cue Reloop 7 / 15 | `{"modelDescription":"turntable.cuePointPadName7","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime7","modelState":"turntable.cuePointPadIsSet7"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloop8` | button-action | Cue Reloop 8 / 16 | `{"modelDescription":"turntable.cuePointPadName8","type":"button-action","modelBlinkingState":"turntable.cuePointPadLoopingFromStartTime8","modelState":"turntable.cuePointPadIsSet8"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint1` | button-action | Cue Reloop 1 | `{"modelDescription":"turntable.song.cuePoint1.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint1.loopingFromStartTime","modelState":"turntable.song.cuePoint1.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint10` | button-action | Cue Reloop 10 | `{"modelDescription":"turntable.song.cuePoint10.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint10.loopingFromStartTime","modelState":"turntable.song.cuePoint10.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint11` | button-action | Cue Reloop 11 | `{"modelDescription":"turntable.song.cuePoint11.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint11.loopingFromStartTime","modelState":"turntable.song.cuePoint11.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint12` | button-action | Cue Reloop 12 | `{"modelDescription":"turntable.song.cuePoint12.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint12.loopingFromStartTime","modelState":"turntable.song.cuePoint12.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint13` | button-action | Cue Reloop 13 | `{"modelDescription":"turntable.song.cuePoint13.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint13.loopingFromStartTime","modelState":"turntable.song.cuePoint13.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint14` | button-action | Cue Reloop 14 | `{"modelDescription":"turntable.song.cuePoint14.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint14.loopingFromStartTime","modelState":"turntable.song.cuePoint14.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint15` | button-action | Cue Reloop 15 | `{"modelDescription":"turntable.song.cuePoint15.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint15.loopingFromStartTime","modelState":"turntable.song.cuePoint15.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint16` | button-action | Cue Reloop 16 | `{"modelDescription":"turntable.song.cuePoint16.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint16.loopingFromStartTime","modelState":"turntable.song.cuePoint16.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint2` | button-action | Cue Reloop 2 | `{"modelDescription":"turntable.song.cuePoint2.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint2.loopingFromStartTime","modelState":"turntable.song.cuePoint2.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint3` | button-action | Cue Reloop 3 | `{"modelDescription":"turntable.song.cuePoint3.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint3.loopingFromStartTime","modelState":"turntable.song.cuePoint3.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint4` | button-action | Cue Reloop 4 | `{"modelDescription":"turntable.song.cuePoint4.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint4.loopingFromStartTime","modelState":"turntable.song.cuePoint4.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint5` | button-action | Cue Reloop 5 | `{"modelDescription":"turntable.song.cuePoint5.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint5.loopingFromStartTime","modelState":"turntable.song.cuePoint5.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint6` | button-action | Cue Reloop 6 | `{"modelDescription":"turntable.song.cuePoint6.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint6.loopingFromStartTime","modelState":"turntable.song.cuePoint6.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint7` | button-action | Cue Reloop 7 | `{"modelDescription":"turntable.song.cuePoint7.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint7.loopingFromStartTime","modelState":"turntable.song.cuePoint7.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint8` | button-action | Cue Reloop 8 | `{"modelDescription":"turntable.song.cuePoint8.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint8.loopingFromStartTime","modelState":"turntable.song.cuePoint8.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint9` | button-action | Cue Reloop 9 | `{"modelDescription":"turntable.song.cuePoint9.name","type":"button-action","modelBlinkingState":"turntable.song.cuePoint9.loopingFromStartTime","modelState":"turntable.song.cuePoint9.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetAutomixStart` | button-action | Cue Automix Start Point | `{"type":"button-action","modelState":"turntable.song.automixStartPoint.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint1` | button-action | Cue 1 | `{"modelPadColorIndex":"turntable.song.cuePoint1.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint1.hasStart","modelDescription":"turntable.song.cuePoint1.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint10` | button-action | Cue 10 | `{"modelPadColorIndex":"turntable.song.cuePoint10.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint10.hasStart","modelDescription":"turntable.song.cuePoint10.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint11` | button-action | Cue 11 | `{"modelPadColorIndex":"turntable.song.cuePoint11.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint11.hasStart","modelDescription":"turntable.song.cuePoint11.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint12` | button-action | Cue 12 | `{"modelPadColorIndex":"turntable.song.cuePoint12.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint12.hasStart","modelDescription":"turntable.song.cuePoint12.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint13` | button-action | Cue 13 | `{"modelPadColorIndex":"turntable.song.cuePoint13.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint13.hasStart","modelDescription":"turntable.song.cuePoint13.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint14` | button-action | Cue 14 | `{"modelPadColorIndex":"turntable.song.cuePoint14.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint14.hasStart","modelDescription":"turntable.song.cuePoint14.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint15` | button-action | Cue 15 | `{"modelPadColorIndex":"turntable.song.cuePoint15.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint15.hasStart","modelDescription":"turntable.song.cuePoint15.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint16` | button-action | Cue 16 | `{"modelPadColorIndex":"turntable.song.cuePoint16.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint16.hasStart","modelDescription":"turntable.song.cuePoint16.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint2` | button-action | Cue 2 | `{"modelPadColorIndex":"turntable.song.cuePoint2.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint2.hasStart","modelDescription":"turntable.song.cuePoint2.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint3` | button-action | Cue 3 | `{"modelPadColorIndex":"turntable.song.cuePoint3.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint3.hasStart","modelDescription":"turntable.song.cuePoint3.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint4` | button-action | Cue 4 | `{"modelPadColorIndex":"turntable.song.cuePoint4.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint4.hasStart","modelDescription":"turntable.song.cuePoint4.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint5` | button-action | Cue 5 | `{"modelPadColorIndex":"turntable.song.cuePoint5.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint5.hasStart","modelDescription":"turntable.song.cuePoint5.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint6` | button-action | Cue 6 | `{"modelPadColorIndex":"turntable.song.cuePoint6.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint6.hasStart","modelDescription":"turntable.song.cuePoint6.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint7` | button-action | Cue 7 | `{"modelPadColorIndex":"turntable.song.cuePoint7.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint7.hasStart","modelDescription":"turntable.song.cuePoint7.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint8` | button-action | Cue 8 | `{"modelPadColorIndex":"turntable.song.cuePoint8.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint8.hasStart","modelDescription":"turntable.song.cuePoint8.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetCuePoint9` | button-action | Cue 9 | `{"modelPadColorIndex":"turntable.song.cuePoint9.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint9.hasStart","modelDescription":"turntable.song.cuePoint9.name"}` |  |
| `turntable.cueOrJumpIfAlreadySetEnd` | button-action | Cue Automix End Point | `{"type":"button-action","modelState":"turntable.song.automixEndPoint.hasStart"}` |  |
| `turntable.cueOrJumpIfAlreadySetStart` | button-action | Start CUE Set/Jump | `{"type":"button-action","modelState":"turntable.song.cuePointStart.hasStart"}` |  |
| `turntable.cuePointPageToggle` | button-toggle | Toggle Cue Point Range | `{"type":"button-toggle"}` |  |
| `turntable.cuePositionOrJumpConsideringPlayState1` | button-action | CUE | `{"type":"button-action","modelState":"turntable.isTraditionalCueSettableButtonState"}` | Note CH1 data 1; Note CH2 data 1; Note CH3 data 1; Note CH4 data 1 |
| `turntable.deckSlip` | button-hold | [Inferred] turntable / deck Slip | `{"type":"button-hold","modelState":"turntable.deckSlipButtonState"}` |  |
| `turntable.deckSlipToggle` | button-toggle | Toggle Slip Mode | `{"type":"button-toggle","modelState":"turntable.deckSlipButtonState"}` | Note CH1 data 4; Note CH2 data 4; Note CH3 data 4; Note CH4 data 4 |
| `turntable.downBeat` | not declared | Set Downbeat | `{"modelEnabled":"turntable.song.canEditGrid"}` |  |
| `turntable.echo` | button-action | Echo on-off | `{"type":"button-action","modelState":"turntable.fx1.enabled"}` |  |
| `turntable.equalizerEQHigh` | control | [Inferred] turntable / equalizer EQ High | `{"type":"control","modelState":"turntable.effects.highEQ.isReset","modelValue":"turntable.effects.highEQ.currentValue"}` |  |
| `turntable.equalizerEQLow` | control | [Inferred] turntable / equalizer EQ Low | `{"type":"control","modelState":"turntable.effects.lowEQ.isReset","modelValue":"turntable.effects.lowEQ.currentValue"}` |  |
| `turntable.equalizerEQMid` | control | [Inferred] turntable / equalizer EQ Mid | `{"type":"control","modelState":"turntable.effects.midEQ.isReset","modelValue":"turntable.effects.midEQ.currentValue"}` |  |
| `turntable.filter` | control | Filter | `{"type":"control","modelState":"turntable.effects.filter.isReset","modelValue":"turntable.effects.filter.currentValue"}` | CC CH5 data 21; CC CH5 data 22; CC CH5 data 23; CC CH5 data 24 |
| `turntable.filterKill` | button-toggle | Kill Filter | `{"type":"button-toggle","modelState":"turntable.effects.filter.isReset","modelStateIsFlipped":true}` |  |
| `turntable.forwardspin` | button-action | Forwardspin | `{"type":"button-action","modelEnabled":"turntable.playbackControlsAvailable"}` |  |
| `turntable.fx1Enabled` | button-toggle | FX 1 Enabled | `{"type":"button-toggle","modelState":"turntable.fx1.enabled"}` | Note CH1 data 33; Note CH1 data 37; Note CH2 data 33; Note CH2 data 37; Note CH3 data 33; Note CH3 data 37; Note CH4 data 33; Note CH4 data 37 |
| `turntable.fx1EnabledHold` | button-hold | [Inferred] turntable / fx 1 Enabled Hold | `{"type":"button-hold","modelState":"turntable.fx1.enabled"}` |  |
| `turntable.fx1ParameterDefaultValue` | button-toggle | FX 1 Set Default Parameter | `{"type":"button-toggle","modelState":"turntable.fx1.enabled","modelStateIsFlipped":true}` |  |
| `turntable.fx1ParameterValue` | control | FX 1 Parameter | `{"type":"control","modelValue":"turntable.fx1.parameterValue"}` | CC CH1 data 32; pickup; CC CH1 data 40; pickup; CC CH2 data 32; pickup; CC CH2 data 40; pickup; CC CH3 data 32; pickup; CC CH3 data 40; pickup; CC CH4 data 32; pickup; CC CH4 data 40; pickup |
| `turntable.fx1Routing.routingDeck` | button-action | FX 1 Routing Deck | `{"type":"button-action","modelState":"turntable.fx1.routing.isDeck"}` |  |
| `turntable.fx1Routing.routingFourTrackChannel1` | button-action | FX 1 Routing (4ch: Drums) | `{"type":"button-action","modelState":"turntable.fx1.routing.fourTrackChannel1","modelDescription":"turntable.unmixer.fourTrackChannel1Name"}` |  |
| `turntable.fx1Routing.routingFourTrackChannel2` | button-action | FX 1 Routing (4ch: Bass) | `{"type":"button-action","modelState":"turntable.fx1.routing.fourTrackChannel2","modelDescription":"turntable.unmixer.fourTrackChannel2Name"}` |  |
| `turntable.fx1Routing.routingFourTrackChannel3` | button-action | FX 1 Routing (4ch: Harmonic) | `{"type":"button-action","modelState":"turntable.fx1.routing.fourTrackChannel3","modelDescription":"turntable.unmixer.fourTrackChannel3Name"}` |  |
| `turntable.fx1Routing.routingFourTrackChannel4` | button-action | FX 1 Routing (4ch: Vocals) | `{"type":"button-action","modelState":"turntable.fx1.routing.fourTrackChannel4","modelDescription":"turntable.unmixer.fourTrackChannel4Name"}` |  |
| `turntable.fx1Routing.routingGenericChannel1` | button-action | FX 1 Routing (1) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeOne","type":"button-action","modelState":"turntable.fx1.routing.genericChannel1","modelDescription":"turntable.unmixer.genericChannel1Name"}` |  |
| `turntable.fx1Routing.routingGenericChannel2` | button-action | FX 1 Routing (2) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeTwo","type":"button-action","modelState":"turntable.fx1.routing.genericChannel2","modelDescription":"turntable.unmixer.genericChannel2Name"}` |  |
| `turntable.fx1Routing.routingGenericChannel3` | button-action | FX 1 Routing (3) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeThree","type":"button-action","modelState":"turntable.fx1.routing.genericChannel3","modelDescription":"turntable.unmixer.genericChannel3Name"}` |  |
| `turntable.fx1Routing.routingGenericChannel4` | button-action | FX 1 Routing (4) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeFour","type":"button-action","modelState":"turntable.fx1.routing.genericChannel4","modelDescription":"turntable.unmixer.genericChannel4Name"}` |  |
| `turntable.fx1Routing.routingThreeTrackChannel1` | button-action | FX 1 Routing (3ch: Drums) | `{"type":"button-action","modelState":"turntable.fx1.routing.threeTrackChannel1","modelDescription":"turntable.unmixer.threeTrackChannel1Name"}` |  |
| `turntable.fx1Routing.routingThreeTrackChannel2` | button-action | FX 1 Routing (3ch: Harmonic / Bass) | `{"type":"button-action","modelState":"turntable.fx1.routing.threeTrackChannel2","modelDescription":"turntable.unmixer.threeTrackChannel2Name"}` |  |
| `turntable.fx1Routing.routingThreeTrackChannel3` | button-action | FX 1 Routing (3ch: Vocals / Melodic) | `{"type":"button-action","modelState":"turntable.fx1.routing.threeTrackChannel3","modelDescription":"turntable.unmixer.threeTrackChannel3Name"}` |  |
| `turntable.fx1Routing.routingTwoTrackChannel1` | button-action | FX 1 Routing (2ch: Instrumental / Drums) | `{"type":"button-action","modelState":"turntable.fx1.routing.twoTrackChannel1","modelDescription":"turntable.unmixer.twoTrackChannel1Name"}` |  |
| `turntable.fx1Routing.routingTwoTrackChannel1Instrumental` | button-action | FX 1 Routing Instrumental | `{"type":"button-action","modelState":"turntable.fx1.routing.twoTrackChannel1"}` |  |
| `turntable.fx1Routing.routingTwoTrackChannel2` | button-action | FX 1 Routing (2ch: Acappella / Tonal) | `{"type":"button-action","modelState":"turntable.fx1.routing.twoTrackChannel2","modelDescription":"turntable.unmixer.twoTrackChannel2Name"}` |  |
| `turntable.fx1Routing.routingTwoTrackChannel2Acapella` | button-action | FX 1 Routing Acappella | `{"type":"button-action","modelState":"turntable.fx1.routing.twoTrackChannel2"}` |  |
| `turntable.fx1RoutingAcapella` | button-action | FX 1 Assignment Acapella | `{"type":"button-action","modelState":"turntable.fx1.routing.threeTrackChannel3"}` |  |
| `turntable.fx1RoutingDeck` | button-action | FX 1 Assignment Deck | `{"type":"button-action","modelState":"turntable.fx1.routing.isDeck"}` |  |
| `turntable.fx1RoutingHarmonic` | button-action | FX 1 Assignment Harmonic | `{"type":"button-action","modelState":"turntable.fx1.routing.threeTrackChannel2"}` |  |
| `turntable.fx1RoutingInstrumental` | button-action | FX 1 Assignment Drums | `{"type":"button-action","modelState":"turntable.fx1.routing.threeTrackChannel1"}` |  |
| `turntable.fx1Select` | button-action | FX 1 Select | `{"actionParameterMinValue":0,"actionParameterMaxValue":120,"type":"button-action","modelState":"turntable.fx1.typeIndexInSourcePack","hasActionParameter":true}` |  |
| `turntable.fx1SelectFavorite` | button-action | FX 1 Select Favorite | `{"actionParameterMinValue":0,"actionParameterMaxValue":120,"type":"button-action","modelState":"turntable.fx1.typeIndexInSourceFavorites","hasActionParameter":true}` |  |
| `turntable.fx1SelectRotary` | not declared | FX 1 Select | `{"steppedRotary":true}` |  |
| `turntable.fx1WetDryValue` | control | FX 1 Wet/Dry | `{"type":"control","modelValue":"turntable.fx1.wetDryValue"}` | CC CH1 data 35; pickup; CC CH1 data 43; pickup; CC CH2 data 35; pickup; CC CH2 data 43; pickup; CC CH3 data 35; pickup; CC CH3 data 43; pickup; CC CH4 data 35; pickup; CC CH4 data 43; pickup |
| `turntable.fx2Enabled` | button-toggle | FX 2 Enabled | `{"type":"button-toggle","modelState":"turntable.fx2.enabled"}` | Note CH1 data 34; Note CH1 data 38; Note CH2 data 34; Note CH2 data 38; Note CH3 data 34; Note CH3 data 38; Note CH4 data 34; Note CH4 data 38 |
| `turntable.fx2EnabledHold` | button-hold | [Inferred] turntable / fx 2 Enabled Hold | `{"type":"button-hold","modelState":"turntable.fx2.enabled"}` |  |
| `turntable.fx2ParameterDefaultValue` | button-toggle | FX 2 Set Default Parameter | `{"type":"button-toggle","modelState":"turntable.fx2.enabled","modelStateIsFlipped":true}` |  |
| `turntable.fx2ParameterValue` | control | FX 2 Parameter | `{"type":"control","modelValue":"turntable.fx2.parameterValue"}` | CC CH1 data 33; pickup; CC CH1 data 41; pickup; CC CH2 data 33; pickup; CC CH2 data 41; pickup; CC CH3 data 33; pickup; CC CH3 data 41; pickup; CC CH4 data 33; pickup; CC CH4 data 41; pickup |
| `turntable.fx2Routing.routingDeck` | button-action | FX 2 Routing Deck | `{"type":"button-action","modelState":"turntable.fx2.routing.isDeck"}` |  |
| `turntable.fx2Routing.routingFourTrackChannel1` | button-action | FX 2 Routing (4ch: Drums) | `{"type":"button-action","modelState":"turntable.fx2.routing.fourTrackChannel1","modelDescription":"turntable.unmixer.fourTrackChannel1Name"}` |  |
| `turntable.fx2Routing.routingFourTrackChannel2` | button-action | FX 2 Routing (4ch: Bass) | `{"type":"button-action","modelState":"turntable.fx2.routing.fourTrackChannel2","modelDescription":"turntable.unmixer.fourTrackChannel2Name"}` |  |
| `turntable.fx2Routing.routingFourTrackChannel3` | button-action | FX 2 Routing (4ch: Harmonic) | `{"type":"button-action","modelState":"turntable.fx2.routing.fourTrackChannel3","modelDescription":"turntable.unmixer.fourTrackChannel3Name"}` |  |
| `turntable.fx2Routing.routingFourTrackChannel4` | button-action | FX 2 Routing (4ch: Vocals) | `{"type":"button-action","modelState":"turntable.fx2.routing.fourTrackChannel4","modelDescription":"turntable.unmixer.fourTrackChannel4Name"}` |  |
| `turntable.fx2Routing.routingGenericChannel1` | button-action | FX 2 Routing (1) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeOne","type":"button-action","modelState":"turntable.fx2.routing.genericChannel1","modelDescription":"turntable.unmixer.genericChannel1Name"}` |  |
| `turntable.fx2Routing.routingGenericChannel2` | button-action | FX 2 Routing (2) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeTwo","type":"button-action","modelState":"turntable.fx2.routing.genericChannel2","modelDescription":"turntable.unmixer.genericChannel2Name"}` |  |
| `turntable.fx2Routing.routingGenericChannel3` | button-action | FX 2 Routing (3) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeThree","type":"button-action","modelState":"turntable.fx2.routing.genericChannel3","modelDescription":"turntable.unmixer.genericChannel3Name"}` |  |
| `turntable.fx2Routing.routingGenericChannel4` | button-action | FX 2 Routing (4) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeFour","type":"button-action","modelState":"turntable.fx2.routing.genericChannel4","modelDescription":"turntable.unmixer.genericChannel4Name"}` |  |
| `turntable.fx2Routing.routingThreeTrackChannel1` | button-action | FX 2 Routing (3ch: Drums) | `{"type":"button-action","modelState":"turntable.fx2.routing.threeTrackChannel1","modelDescription":"turntable.unmixer.threeTrackChannel1Name"}` |  |
| `turntable.fx2Routing.routingThreeTrackChannel2` | button-action | FX 2 Routing (3ch: Harmonic / Bass) | `{"type":"button-action","modelState":"turntable.fx2.routing.threeTrackChannel2","modelDescription":"turntable.unmixer.threeTrackChannel2Name"}` |  |
| `turntable.fx2Routing.routingThreeTrackChannel3` | button-action | FX 2 Routing (3ch: Vocals / Melodic) | `{"type":"button-action","modelState":"turntable.fx2.routing.threeTrackChannel3","modelDescription":"turntable.unmixer.threeTrackChannel3Name"}` |  |
| `turntable.fx2Routing.routingTwoTrackChannel1` | button-action | FX 2 Routing (2ch: Instrumental / Drums) | `{"type":"button-action","modelState":"turntable.fx2.routing.twoTrackChannel1","modelDescription":"turntable.unmixer.twoTrackChannel1Name"}` |  |
| `turntable.fx2Routing.routingTwoTrackChannel1Instrumental` | button-action | FX 2 Routing Instrumental | `{"type":"button-action","modelState":"turntable.fx2.routing.twoTrackChannel1"}` |  |
| `turntable.fx2Routing.routingTwoTrackChannel2` | button-action | FX 2 Routing (2ch: Acappella / Tonal) | `{"type":"button-action","modelState":"turntable.fx2.routing.twoTrackChannel2","modelDescription":"turntable.unmixer.twoTrackChannel2Name"}` |  |
| `turntable.fx2Routing.routingTwoTrackChannel2Acapella` | button-action | FX 2 Routing Acappella | `{"type":"button-action","modelState":"turntable.fx2.routing.twoTrackChannel2"}` |  |
| `turntable.fx2RoutingAcapella` | button-action | FX 2 Assignment Acapella | `{"type":"button-action","modelState":"turntable.fx2.routing.threeTrackChannel3"}` |  |
| `turntable.fx2RoutingDeck` | button-action | FX 2 Assignment Deck | `{"type":"button-action","modelState":"turntable.fx2.routing.isDeck"}` |  |
| `turntable.fx2RoutingHarmonic` | button-action | FX 2 Assignment Harmonic | `{"type":"button-action","modelState":"turntable.fx2.routing.threeTrackChannel2"}` |  |
| `turntable.fx2RoutingInstrumental` | button-action | FX 2 Assignment Drums | `{"type":"button-action","modelState":"turntable.fx2.routing.threeTrackChannel1"}` |  |
| `turntable.fx2Select` | button-action | FX 2 Select | `{"actionParameterMinValue":0,"actionParameterMaxValue":120,"type":"button-action","modelState":"turntable.fx2.typeIndexInSourcePack","hasActionParameter":true}` |  |
| `turntable.fx2SelectFavorite` | button-action | FX 2 Select Favorite | `{"actionParameterMinValue":0,"actionParameterMaxValue":120,"type":"button-action","modelState":"turntable.fx2.typeIndexInSourceFavorites","hasActionParameter":true}` |  |
| `turntable.fx2SelectRotary` | not declared | FX 2 Select | `{"steppedRotary":true}` |  |
| `turntable.fx2WetDryValue` | control | FX 2 Wet/Dry | `{"type":"control","modelValue":"turntable.fx2.wetDryValue"}` | CC CH1 data 36; pickup; CC CH1 data 44; pickup; CC CH2 data 36; pickup; CC CH2 data 44; pickup; CC CH3 data 36; pickup; CC CH3 data 44; pickup; CC CH4 data 36; pickup; CC CH4 data 44; pickup |
| `turntable.fx3Enabled` | button-toggle | FX 3 Enabled | `{"type":"button-toggle","modelState":"turntable.fx3.enabled"}` | Note CH1 data 35; Note CH1 data 39; Note CH2 data 35; Note CH2 data 39; Note CH3 data 35; Note CH3 data 39; Note CH4 data 35; Note CH4 data 39 |
| `turntable.fx3EnabledHold` | button-hold | [Inferred] turntable / fx 3 Enabled Hold | `{"type":"button-hold","modelState":"turntable.fx3.enabled"}` |  |
| `turntable.fx3ParameterDefaultValue` | button-toggle | FX 3 Set Default Parameter | `{"type":"button-toggle","modelState":"turntable.fx3.enabled","modelStateIsFlipped":true}` |  |
| `turntable.fx3ParameterValue` | control | FX 3 Parameter | `{"type":"control","modelValue":"turntable.fx3.parameterValue"}` | CC CH1 data 34; pickup; CC CH1 data 42; pickup; CC CH2 data 34; pickup; CC CH2 data 42; pickup; CC CH3 data 34; pickup; CC CH3 data 42; pickup; CC CH4 data 34; pickup; CC CH4 data 42; pickup |
| `turntable.fx3Routing.routingDeck` | button-action | FX 3 Routing Deck | `{"type":"button-action","modelState":"turntable.fx3.routing.isDeck"}` |  |
| `turntable.fx3Routing.routingFourTrackChannel1` | button-action | FX 3 Routing (4ch: Drums) | `{"type":"button-action","modelState":"turntable.fx3.routing.fourTrackChannel1","modelDescription":"turntable.unmixer.fourTrackChannel1Name"}` |  |
| `turntable.fx3Routing.routingFourTrackChannel2` | button-action | FX 3 Routing (4ch: Bass) | `{"type":"button-action","modelState":"turntable.fx3.routing.fourTrackChannel2","modelDescription":"turntable.unmixer.fourTrackChannel2Name"}` |  |
| `turntable.fx3Routing.routingFourTrackChannel3` | button-action | FX 3 Routing (4ch: Harmonic) | `{"type":"button-action","modelState":"turntable.fx3.routing.fourTrackChannel3","modelDescription":"turntable.unmixer.fourTrackChannel3Name"}` |  |
| `turntable.fx3Routing.routingFourTrackChannel4` | button-action | FX 3 Routing (4ch: Vocals) | `{"type":"button-action","modelState":"turntable.fx3.routing.fourTrackChannel4","modelDescription":"turntable.unmixer.fourTrackChannel4Name"}` |  |
| `turntable.fx3Routing.routingGenericChannel1` | button-action | FX 3 Routing (1) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeOne","type":"button-action","modelState":"turntable.fx3.routing.genericChannel1","modelDescription":"turntable.unmixer.genericChannel1Name"}` |  |
| `turntable.fx3Routing.routingGenericChannel2` | button-action | FX 3 Routing (2) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeTwo","type":"button-action","modelState":"turntable.fx3.routing.genericChannel2","modelDescription":"turntable.unmixer.genericChannel2Name"}` |  |
| `turntable.fx3Routing.routingGenericChannel3` | button-action | FX 3 Routing (3) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeThree","type":"button-action","modelState":"turntable.fx3.routing.genericChannel3","modelDescription":"turntable.unmixer.genericChannel3Name"}` |  |
| `turntable.fx3Routing.routingGenericChannel4` | button-action | FX 3 Routing (4) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeFour","type":"button-action","modelState":"turntable.fx3.routing.genericChannel4","modelDescription":"turntable.unmixer.genericChannel4Name"}` |  |
| `turntable.fx3Routing.routingThreeTrackChannel1` | button-action | FX 3 Routing (3ch: Drums) | `{"type":"button-action","modelState":"turntable.fx3.routing.threeTrackChannel1","modelDescription":"turntable.unmixer.threeTrackChannel1Name"}` |  |
| `turntable.fx3Routing.routingThreeTrackChannel2` | button-action | FX 3 Routing (3ch: Harmonic / Bass) | `{"type":"button-action","modelState":"turntable.fx3.routing.threeTrackChannel2","modelDescription":"turntable.unmixer.threeTrackChannel2Name"}` |  |
| `turntable.fx3Routing.routingThreeTrackChannel3` | button-action | FX 3 Routing (3ch: Vocals / Melodic) | `{"type":"button-action","modelState":"turntable.fx3.routing.threeTrackChannel3","modelDescription":"turntable.unmixer.threeTrackChannel3Name"}` |  |
| `turntable.fx3Routing.routingTwoTrackChannel1` | button-action | FX 3 Routing (2ch: Instrumental / Drums) | `{"type":"button-action","modelState":"turntable.fx3.routing.twoTrackChannel1","modelDescription":"turntable.unmixer.twoTrackChannel1Name"}` |  |
| `turntable.fx3Routing.routingTwoTrackChannel1Instrumental` | button-action | FX 3 Routing Instrumental | `{"type":"button-action","modelState":"turntable.fx3.routing.twoTrackChannel1"}` |  |
| `turntable.fx3Routing.routingTwoTrackChannel2` | button-action | FX 3 Routing (2ch: Acappella / Tonal) | `{"type":"button-action","modelState":"turntable.fx3.routing.twoTrackChannel2","modelDescription":"turntable.unmixer.twoTrackChannel2Name"}` |  |
| `turntable.fx3Routing.routingTwoTrackChannel2Acapella` | button-action | FX 3 Routing Acappella | `{"type":"button-action","modelState":"turntable.fx3.routing.twoTrackChannel2"}` |  |
| `turntable.fx3RoutingAcapella` | button-action | FX 3 Assignment Acapella | `{"type":"button-action","modelState":"turntable.fx3.routing.threeTrackChannel3"}` |  |
| `turntable.fx3RoutingDeck` | button-action | FX 3 Assignment Deck | `{"type":"button-action","modelState":"turntable.fx3.routing.isDeck"}` |  |
| `turntable.fx3RoutingHarmonic` | button-action | FX 3 Assignment Harmonic | `{"type":"button-action","modelState":"turntable.fx3.routing.threeTrackChannel2"}` |  |
| `turntable.fx3RoutingInstrumental` | button-action | FX 3 Assignment Drums | `{"type":"button-action","modelState":"turntable.fx3.routing.threeTrackChannel1"}` |  |
| `turntable.fx3Select` | button-action | FX 3 Select | `{"actionParameterMinValue":0,"actionParameterMaxValue":120,"type":"button-action","modelState":"turntable.fx3.typeIndexInSourcePack","hasActionParameter":true}` |  |
| `turntable.fx3SelectFavorite` | button-action | FX 3 Select Favorite | `{"actionParameterMinValue":0,"actionParameterMaxValue":120,"type":"button-action","modelState":"turntable.fx3.typeIndexInSourceFavorites","hasActionParameter":true}` |  |
| `turntable.fx3SelectRotary` | not declared | FX 3 Select | `{"steppedRotary":true}` |  |
| `turntable.fx3WetDryValue` | control | FX 3 Wet/Dry | `{"type":"control","modelValue":"turntable.fx3.wetDryValue"}` | CC CH1 data 37; pickup; CC CH1 data 45; pickup; CC CH2 data 37; pickup; CC CH2 data 45; pickup; CC CH3 data 37; pickup; CC CH3 data 45; pickup; CC CH4 data 37; pickup; CC CH4 data 45; pickup |
| `turntable.fxActive` | button-toggle | FX Active | `{"type":"button-toggle","modelState":"turntable.fxActive"}` | Note CH1 data 32; Note CH1 data 36; Note CH2 data 32; Note CH2 data 36; Note CH3 data 32; Note CH3 data 36; Note CH4 data 32; Note CH4 data 36 |
| `turntable.fxBpmTap` | not declared | Tap FX BPM | `{"modelState":"turntable.useFxBpmValue"}` |  |
| `turntable.fxWetDryValue` | control | [Inferred] turntable / fx Wet Dry Value | `{"type":"control","modelValue":"turntable.fx1.wetDryValue"}` |  |
| `turntable.gain` | control | Gain | `{"type":"control","modelValue":"turntable.effects.gain.currentValue"}` | CC CH5 data 0; CC CH5 data 1; CC CH5 data 2; CC CH5 data 3 |
| `turntable.gatedCue` | button-toggle | Gated Cue | `{"type":"button-toggle","modelState":"turntable.isGatedCueEnabled"}` |  |
| `turntable.gotoAutomixStartConsideringPlayState` | button-action | Jump to Automix Start Point | `{"type":"button-action","modelState":"turntable.song.automixStartPoint.hasStart","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.gotoBeginning` | button-action | Jump to Start CUE | `{"type":"button-action","modelState":"turntable.song.cuePointStart.hasStart","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.gotoBeginningAndPause` | button-action | Jump to Start CUE and Pause | `{"type":"button-action","modelState":"turntable.song.cuePointStart.hasStart","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.gotoBeginningAndPlay` | button-action | Jump to Start CUE and Play | `{"type":"button-action","modelState":"turntable.song.cuePointStart.hasStart","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.gotoBeginningConsideringPlayState` | button-action | Jump to Start CUE | `{"type":"button-action","modelState":"turntable.song.cuePointStart.hasStart","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.gotoEnd` | button-action | Jump to End Point | `{"type":"button-action","modelState":"turntable.song.automixEndPoint.hasStart","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.gotoEndConsideringPlayState` | button-action | Jump to Automix End Point | `{"type":"button-action","modelState":"turntable.song.automixEndPoint.hasStart","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.gotoZero` | button-action | Jump to Beginning of Song | `{"type":"button-action","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.highEQ` | control | High EQ | `{"type":"control","modelState":"turntable.highEQEffect.isReset","modelValue":"turntable.highEQEffect.currentValue"}` | CC CH5 data 4; CC CH5 data 5; CC CH5 data 6; CC CH5 data 7 |
| `turntable.highEQKill` | button-toggle | Kill High EQ | `{"type":"button-toggle","modelState":"turntable.highEQEffect.isReset","modelStateIsFlipped":true}` |  |
| `turntable.instantFx1` | button-hold | Instant FX 1 | `{"modelParameterDescription":"turntable.instantFx1.parameterName","type":"button-hold","modelState":"turntable.instantFx1.enabled","modelDescription":"turntable.instantFx1.name"}` | Note CH1 data 16; Note CH2 data 16; Note CH3 data 16; Note CH4 data 16 |
| `turntable.instantFx2` | button-hold | Instant FX 2 | `{"modelParameterDescription":"turntable.instantFx2.parameterName","type":"button-hold","modelState":"turntable.instantFx2.enabled","modelDescription":"turntable.instantFx2.name"}` | Note CH1 data 17; Note CH2 data 17; Note CH3 data 17; Note CH4 data 17 |
| `turntable.instantFx3` | button-hold | Instant FX 3 | `{"modelParameterDescription":"turntable.instantFx3.parameterName","type":"button-hold","modelState":"turntable.instantFx3.enabled","modelDescription":"turntable.instantFx3.name"}` | Note CH1 data 18; Note CH2 data 18; Note CH3 data 18; Note CH4 data 18 |
| `turntable.instantFx4` | button-hold | Instant FX 4 | `{"modelParameterDescription":"turntable.instantFx4.parameterName","type":"button-hold","modelState":"turntable.instantFx4.enabled","modelDescription":"turntable.instantFx4.name"}` | Note CH1 data 19; Note CH2 data 19; Note CH3 data 19; Note CH4 data 19 |
| `turntable.instantFx5` | button-hold | Instant FX 5 | `{"modelParameterDescription":"turntable.instantFx5.parameterName","type":"button-hold","modelState":"turntable.instantFx5.enabled","modelDescription":"turntable.instantFx5.name"}` |  |
| `turntable.instantFx6` | button-hold | Instant FX 6 | `{"modelParameterDescription":"turntable.instantFx6.parameterName","type":"button-hold","modelState":"turntable.instantFx6.enabled","modelDescription":"turntable.instantFx6.name"}` |  |
| `turntable.instantFx7` | button-hold | Instant FX 7 | `{"modelParameterDescription":"turntable.instantFx7.parameterName","type":"button-hold","modelState":"turntable.instantFx7.enabled","modelDescription":"turntable.instantFx7.name"}` |  |
| `turntable.instantFx8` | button-hold | Instant FX 8 | `{"modelParameterDescription":"turntable.instantFx8.parameterName","type":"button-hold","modelState":"turntable.instantFx8.enabled","modelDescription":"turntable.instantFx8.name"}` |  |
| `turntable.instantFxMain1` | button-hold | Toggle Main Instant FX 1 | `{"modelParameterDescription":"turntable.instantFxMain1.parameterName","type":"button-hold","modelState":"turntable.instantFxMain1.enabled","modelDescription":"turntable.instantFxMain1.name"}` |  |
| `turntable.instantFxMain2` | button-hold | Toggle Main Instant FX 2 | `{"modelParameterDescription":"turntable.instantFxMain2.parameterName","type":"button-hold","modelState":"turntable.instantFxMain2.enabled","modelDescription":"turntable.instantFxMain2.name"}` |  |
| `turntable.instantFxToggleMode` | button-toggle | Instant FX Toggle Mode | `{"type":"button-toggle","modelState":"turntable.instantFxToggleMode"}` |  |
| `turntable.jogPitchBendMode` | button-hold | [Inferred] turntable / jog Pitch Bend Mode | `{"type":"button-hold","modelState":"turntable.jogPitchBendMode"}` |  |
| `turntable.jogPitchBendModeToggle` | button-toggle | Toggle Jog Pitch Bend Mode | `{"type":"button-toggle","modelState":"turntable.jogPitchBendMode"}` |  |
| `turntable.jogSeekMode` | button-hold | [Inferred] turntable / jog Seek Mode | `{"type":"button-hold","modelState":"turntable.jogSeekMode"}` | Note CH10 data 7; Note CH11 data 7; Note CH12 data 7; Note CH9 data 7 |
| `turntable.jogSeekModeToggle` | button-toggle | Toggle Jog Seek Mode | `{"type":"button-toggle","modelState":"turntable.jogSeekMode"}` |  |
| `turntable.jogwheelSlice` | button-hold | [Inferred] turntable / jogwheel Slice | `{"type":"button-hold","modelState":"turntable.view.state.jogwheelSlice"}` |  |
| `turntable.jumpToCueConsideringPlayState1` | button-action | Jump to Cue Point 1 | `{"modelEnabled":"turntable.transportControlsAvailable","modelPadColorIndex":"turntable.song.cuePoint1.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint1.hasStart"}` |  |
| `turntable.jumpToCueConsideringPlayState2` | button-action | Jump to Cue Point 2 | `{"modelEnabled":"turntable.transportControlsAvailable","modelPadColorIndex":"turntable.song.cuePoint2.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint2.hasStart"}` |  |
| `turntable.jumpToCueConsideringPlayState3` | button-action | Jump to Cue Point 3 | `{"modelEnabled":"turntable.transportControlsAvailable","modelPadColorIndex":"turntable.song.cuePoint3.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint3.hasStart"}` |  |
| `turntable.jumpToCueConsideringPlayState4` | button-action | Jump to Cue Point 4 | `{"modelEnabled":"turntable.transportControlsAvailable","modelPadColorIndex":"turntable.song.cuePoint4.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint4.hasStart"}` |  |
| `turntable.jumpToCueConsideringPlayState5` | button-action | Jump to Cue Point 5 | `{"modelEnabled":"turntable.transportControlsAvailable","modelPadColorIndex":"turntable.song.cuePoint5.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint5.hasStart"}` |  |
| `turntable.jumpToCueConsideringPlayState6` | button-action | Jump to Cue Point 6 | `{"modelEnabled":"turntable.transportControlsAvailable","modelPadColorIndex":"turntable.song.cuePoint6.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint6.hasStart"}` |  |
| `turntable.jumpToCueConsideringPlayState7` | button-action | Jump to Cue Point 7 | `{"modelEnabled":"turntable.transportControlsAvailable","modelPadColorIndex":"turntable.song.cuePoint7.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint7.hasStart"}` |  |
| `turntable.jumpToCueConsideringPlayState8` | button-action | Jump to Cue Point 8 | `{"modelEnabled":"turntable.transportControlsAvailable","modelPadColorIndex":"turntable.song.cuePoint8.presentationColorIndex","type":"button-action","modelState":"turntable.song.cuePoint8.hasStart"}` |  |
| `turntable.key` | button-toggle | Key Lock on-off | `{"type":"button-toggle","modelState":"turntable.preservesPitch"}` |  |
| `turntable.libraryRotary` | not declared | Move Library Selection (Rotary) | `{"steppedRotary":true}` |  |
| `turntable.loopIn` | button-action | Loop In / Half | `{"modelEnabled":"turntable.transportControlsAvailable","type":"button-action","modelBlinkingState":"turntable.song.showLoopRegionLiveUpdateInView","modelState":"turntable.song.showLoopRegionLiveUpdateOrLoopingInView"}` |  |
| `turntable.loopingActive` | button-toggle | Loop On/Off | `{"type":"button-toggle","modelState":"turntable.loopingEnabledInView","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.loopingActiveAndReloopWhenOff` | button-action | Reloop / Loop Off | `{"type":"button-action","modelState":"turntable.loopingEnabledInView","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.loopInOrMoveInPoint` | button-action | Loop In / Move | `{"type":"button-action","modelState":"turntable.loopInOrMoveInPointMidiButtonState","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.loopInOut` | button-action | Loop In/Out | `{"modelEnabled":"turntable.transportControlsAvailable","type":"button-action","modelBlinkingState":"turntable.song.showLoopRegionLiveUpdateInView","modelState":"turntable.song.showLoopRegionLiveUpdateOrLoopingInView"}` | Note CH1 data 5; Note CH10 data 6; Note CH11 data 6; Note CH12 data 6; Note CH2 data 5; Note CH3 data 5; Note CH4 data 5; Note CH9 data 6 |
| `turntable.loopOut` | button-action | Loop Out / Half | `{"type":"button-action","modelState":"turntable.loopingEnabledInView","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.loopOutAndReloopOrUnloop` | button-action | Loop Out / Off | `{"type":"button-action","modelState":"turntable.loopingEnabledInView","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.loopOutOrDouble` | button-action | Loop Out / Double | `{"type":"button-action","modelState":"turntable.loopingEnabledInView","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.loopOutOrMoveOutPoint` | button-action | Loop Out / Move | `{"type":"button-action","modelState":"turntable.loopOutOrMoveOutPointMidiButtonState","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.lowEQ` | control | Low EQ | `{"type":"control","modelState":"turntable.lowEQEffect.isReset","modelValue":"turntable.lowEQEffect.currentValue"}` | CC CH5 data 12; CC CH5 data 13; CC CH5 data 14; CC CH5 data 15 |
| `turntable.lowEQKill` | button-toggle | Kill Low EQ | `{"type":"button-toggle","modelState":"turntable.lowEQEffect.isReset","modelStateIsFlipped":true}` |  |
| `turntable.matchKey` | button-toggle | Match Key | `{"type":"button-toggle","modelState":"turntable.isKeyMatched","modelEnabled":"turntable.canMatchKey"}` |  |
| `turntable.midEQ` | control | Mid EQ | `{"type":"control","modelState":"turntable.midEQEffect.isReset","modelValue":"turntable.midEQEffect.currentValue"}` | CC CH5 data 10; CC CH5 data 11; CC CH5 data 8; CC CH5 data 9 |
| `turntable.midEQKill` | button-toggle | Kill Mid EQ | `{"type":"button-toggle","modelState":"turntable.midEQEffect.isReset","modelStateIsFlipped":true}` |  |
| `turntable.muteUntilCue` | button-toggle | Mute until Cue | `{"type":"button-toggle","modelState":"turntable.isMutedUntilCue"}` |  |
| `turntable.padFxEnabled` | button-hold | [Inferred] turntable / pad Fx Enabled | `{"type":"button-hold","modelState":"turntable.padFx.enabled"}` |  |
| `turntable.padFxParameterDefaultValue` | button-toggle | [Inferred] turntable / pad Fx Parameter Default Value | `{"type":"button-toggle","modelState":"turntable.padFx.enabled","modelStateIsFlipped":true}` |  |
| `turntable.padFxParameterValue` | control | [Inferred] turntable / pad Fx Parameter Value | `{"type":"control","modelValue":"turntable.padFx.parameterValue"}` |  |
| `turntable.padFxRouting.routingDeck` | button-action | Pad FX Routing Deck | `{"type":"button-action","modelState":"turntable.padFx.routing.isDeck"}` |  |
| `turntable.padFxRouting.routingFourTrackChannel1` | button-action | Pad FX Routing (4ch: Drums) | `{"type":"button-action","modelState":"turntable.padFx.routing.fourTrackChannel1","modelDescription":"turntable.unmixer.fourTrackChannel1Name"}` |  |
| `turntable.padFxRouting.routingFourTrackChannel2` | button-action | Pad FX Routing (4ch: Bass) | `{"type":"button-action","modelState":"turntable.padFx.routing.fourTrackChannel2","modelDescription":"turntable.unmixer.fourTrackChannel2Name"}` |  |
| `turntable.padFxRouting.routingFourTrackChannel3` | button-action | Pad FX Routing (4ch: Harmonic) | `{"type":"button-action","modelState":"turntable.padFx.routing.fourTrackChannel3","modelDescription":"turntable.unmixer.fourTrackChannel3Name"}` |  |
| `turntable.padFxRouting.routingFourTrackChannel4` | button-action | Pad FX Routing (4ch: Vocals) | `{"type":"button-action","modelState":"turntable.padFx.routing.fourTrackChannel4","modelDescription":"turntable.unmixer.fourTrackChannel4Name"}` |  |
| `turntable.padFxRouting.routingGenericChannel1` | button-action | Pad FX Routing (1) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeOne","type":"button-action","modelState":"turntable.padFx.routing.genericChannel1","modelDescription":"turntable.unmixer.genericChannel1Name"}` |  |
| `turntable.padFxRouting.routingGenericChannel2` | button-action | Pad FX Routing (2) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeTwo","type":"button-action","modelState":"turntable.padFx.routing.genericChannel2","modelDescription":"turntable.unmixer.genericChannel2Name"}` |  |
| `turntable.padFxRouting.routingGenericChannel3` | button-action | Pad FX Routing (3) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeThree","type":"button-action","modelState":"turntable.padFx.routing.genericChannel3","modelDescription":"turntable.unmixer.genericChannel3Name"}` |  |
| `turntable.padFxRouting.routingGenericChannel4` | button-action | Pad FX Routing (4) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeFour","type":"button-action","modelState":"turntable.padFx.routing.genericChannel4","modelDescription":"turntable.unmixer.genericChannel4Name"}` |  |
| `turntable.padFxRouting.routingThreeTrackChannel1` | button-action | Pad FX Routing (3ch: Drums) | `{"type":"button-action","modelState":"turntable.padFx.routing.threeTrackChannel1","modelDescription":"turntable.unmixer.threeTrackChannel1Name"}` |  |
| `turntable.padFxRouting.routingThreeTrackChannel2` | button-action | Pad FX Routing (3ch: Harmonic / Bass) | `{"type":"button-action","modelState":"turntable.padFx.routing.threeTrackChannel2","modelDescription":"turntable.unmixer.threeTrackChannel2Name"}` |  |
| `turntable.padFxRouting.routingThreeTrackChannel3` | button-action | Pad FX Routing (3ch: Vocals / Melodic) | `{"type":"button-action","modelState":"turntable.padFx.routing.threeTrackChannel3","modelDescription":"turntable.unmixer.threeTrackChannel3Name"}` |  |
| `turntable.padFxRouting.routingTwoTrackChannel1` | button-action | Pad FX Routing (2ch: Instrumental / Drums) | `{"type":"button-action","modelState":"turntable.padFx.routing.twoTrackChannel1","modelDescription":"turntable.unmixer.twoTrackChannel1Name"}` |  |
| `turntable.padFxRouting.routingTwoTrackChannel2` | button-action | Pad FX Routing (2ch: Acappella / Tonal) | `{"type":"button-action","modelState":"turntable.padFx.routing.twoTrackChannel2","modelDescription":"turntable.unmixer.twoTrackChannel2Name"}` |  |
| `turntable.padFxRoutingAcapella` | button-action | [Inferred] turntable / pad Fx Routing Acapella | `{"type":"button-action","modelState":"turntable.padFx.routing.threeTrackChannel3"}` |  |
| `turntable.padFxRoutingDeck` | button-action | [Inferred] turntable / pad Fx Routing Deck | `{"type":"button-action","modelState":"turntable.padFx.routing.isDeck"}` |  |
| `turntable.padFxRoutingHarmonic` | button-action | [Inferred] turntable / pad Fx Routing Harmonic | `{"type":"button-action","modelState":"turntable.padFx.routing.threeTrackChannel2"}` |  |
| `turntable.padFxRoutingInstrumental` | button-action | [Inferred] turntable / pad Fx Routing Instrumental | `{"type":"button-action","modelState":"turntable.padFx.routing.threeTrackChannel1"}` |  |
| `turntable.padFxWetDryValue` | control | [Inferred] turntable / pad Fx Wet Dry Value | `{"type":"control","modelValue":"turntable.padFx.wetDryValue"}` |  |
| `turntable.padModeAutoLoop` | button-action | Pad Mode Auto Loop | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsAutoLoop"}` |  |
| `turntable.padModeBounceLoop` | button-action | Pad Mode Bounce Loop | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsBounceLoop"}` |  |
| `turntable.padModeCueLoop` | button-action | Pad Mode Cue Loop | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsCueLoop"}` |  |
| `turntable.padModeEmpty` | button-action | Pad Mode Empty | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsEmpty"}` |  |
| `turntable.padModeGatedCue` | button-action | Pad Mode Gated Cue | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsGatedCue"}` |  |
| `turntable.padModeHotCue` | button-action | Pad Mode Hot Cue | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsHotCue"}` |  |
| `turntable.padModeInstantFX` | button-action | Pad Mode Instant FX | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsInstantFX"}` |  |
| `turntable.padModeLooper` | button-action | Pad Mode Looper | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsLooper"}` |  |
| `turntable.padModeManualLoop` | button-action | Pad Mode Manual Loop | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsManualLoop"}` |  |
| `turntable.padModePitchPlay` | button-action | Pad Mode Pitch Play | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsPitchPlay"}` |  |
| `turntable.padModePitchShift` | button-action | Pad Mode Pitch Shift | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsPitchShift"}` |  |
| `turntable.padModeSampler` | button-action | Pad Mode Sampler | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsSampler"}` |  |
| `turntable.padModeSavedLoop` | button-action | Pad Mode Saved Loop | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsSavedLoop"}` |  |
| `turntable.padModeSkipping` | button-action | Pad Mode Skipping | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsSkipping"}` |  |
| `turntable.padModeSlicer` | button-action | Pad Mode Slicer | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsSlicer"}` |  |
| `turntable.padModeSlicerLoop` | button-action | Pad Mode Slicer Loop | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsSlicerLoop"}` |  |
| `turntable.padModeUnmixer` | button-action | Pad Mode Neural Mix | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsUnmixer"}` |  |
| `turntable.padModeUser1` | button-action | Pad Mode User 1 | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsUser1"}` |  |
| `turntable.padModeUser2` | button-action | Pad Mode User 2 | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsUser2"}` |  |
| `turntable.padModeUser3` | button-action | Pad Mode User 3 | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsUser3"}` |  |
| `turntable.padModeUser4` | button-action | Pad Mode User 4 | `{"type":"button-action","modelState":"midiModel.turntable.padModeIsUser4"}` |  |
| `turntable.pan` | control | Pan | `{"type":"control","modelState":"turntable.effects.pan.isReset","modelValue":"turntable.effects.pan.currentValue"}` |  |
| `turntable.pitch` | control | Key | `{"type":"control","modelValue":"turntable.effects.pitch.currentValue"}` |  |
| `turntable.pitchBendMinus` | button-action | Pitch Bend - | `{"type":"button-action","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.pitchBendPlus` | button-action | Pitch Bend + | `{"type":"button-action","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.pitchOnOff` | button-toggle | Key on-off | `{"type":"button-toggle","modelState":"turntable.effects.pitch.isEnabled"}` |  |
| `turntable.pitchPlayAction1` | button-action | Pitch Play 1 | `{"modelPadColorIndex":"turntable.pitchPlayButtonColorIndex1","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayButtonState1","modelDescription":"turntable.pitchPlayButtonSemitone1"}` |  |
| `turntable.pitchPlayAction2` | button-action | Pitch Play 2 | `{"modelPadColorIndex":"turntable.pitchPlayButtonColorIndex2","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayButtonState2","modelDescription":"turntable.pitchPlayButtonSemitone2"}` |  |
| `turntable.pitchPlayAction3` | button-action | Pitch Play 3 | `{"modelPadColorIndex":"turntable.pitchPlayButtonColorIndex3","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayButtonState3","modelDescription":"turntable.pitchPlayButtonSemitone3"}` |  |
| `turntable.pitchPlayAction4` | button-action | Pitch Play 4 | `{"modelPadColorIndex":"turntable.pitchPlayButtonColorIndex4","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayButtonState4","modelDescription":"turntable.pitchPlayButtonSemitone4"}` |  |
| `turntable.pitchPlayAction5` | button-action | Pitch Play 5 | `{"modelPadColorIndex":"turntable.pitchPlayButtonColorIndex5","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayButtonState5","modelDescription":"turntable.pitchPlayButtonSemitone5"}` |  |
| `turntable.pitchPlayAction6` | button-action | Pitch Play 6 | `{"modelPadColorIndex":"turntable.pitchPlayButtonColorIndex6","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayButtonState6","modelDescription":"turntable.pitchPlayButtonSemitone6"}` |  |
| `turntable.pitchPlayAction7` | button-action | Pitch Play 7 | `{"modelPadColorIndex":"turntable.pitchPlayButtonColorIndex7","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayButtonState7","modelDescription":"turntable.pitchPlayButtonSemitone7"}` |  |
| `turntable.pitchPlayAction8` | button-action | Pitch Play 8 | `{"modelPadColorIndex":"turntable.pitchPlayButtonColorIndex8","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayButtonState8","modelDescription":"turntable.pitchPlayButtonSemitone8"}` |  |
| `turntable.pitchPlayCuePoint1` | button-action | Select Pitch Play Cue Point 1 / 9 | `{"modelEnabled":"turntable.cuePointPadIsSet1","modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex1","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointState1","modelDescription":"turntable.cuePointComment1"}` |  |
| `turntable.pitchPlayCuePoint2` | button-action | Select Pitch Play Cue Point 2 / 10 | `{"modelEnabled":"turntable.cuePointPadIsSet2","modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex2","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointState2","modelDescription":"turntable.cuePointComment2"}` |  |
| `turntable.pitchPlayCuePoint3` | button-action | Select Pitch Play Cue Point 3 / 11 | `{"modelEnabled":"turntable.cuePointPadIsSet3","modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex3","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointState3","modelDescription":"turntable.cuePointComment3"}` |  |
| `turntable.pitchPlayCuePoint4` | button-action | Select Pitch Play Cue Point 4 / 12 | `{"modelEnabled":"turntable.cuePointPadIsSet4","modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex4","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointState4","modelDescription":"turntable.cuePointComment4"}` |  |
| `turntable.pitchPlayCuePoint5` | button-action | Select Pitch Play Cue Point 5 / 13 | `{"modelEnabled":"turntable.cuePointPadIsSet5","modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex5","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointState5","modelDescription":"turntable.cuePointComment5"}` |  |
| `turntable.pitchPlayCuePoint6` | button-action | Select Pitch Play Cue Point 6 / 14 | `{"modelEnabled":"turntable.cuePointPadIsSet6","modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex6","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointState6","modelDescription":"turntable.cuePointComment6"}` |  |
| `turntable.pitchPlayCuePoint7` | button-action | Select Pitch Play Cue Point 7 / 15 | `{"modelEnabled":"turntable.cuePointPadIsSet7","modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex7","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointState7","modelDescription":"turntable.cuePointComment7"}` |  |
| `turntable.pitchPlayCuePoint8` | button-action | Select Pitch Play Cue Point 8 / 16 | `{"modelEnabled":"turntable.cuePointPadIsSet8","modelPadColorIndex":"turntable.cuePointPadPresentationColorIndex8","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointState8","modelDescription":"turntable.cuePointComment8"}` |  |
| `turntable.pitchPlayCuePointCuePoint1` | button-action | Select Pitch Play Cue Point 1 | `{"modelEnabled":"turntable.song.cuePoint1.hasStart","modelPadColorIndex":"turntable.song.cuePoint1.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint1","modelDescription":"turntable.song.cuePoint1.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint10` | button-action | Select Pitch Play Cue Point 10 | `{"modelEnabled":"turntable.song.cuePoint10.hasStart","modelPadColorIndex":"turntable.song.cuePoint10.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint10","modelDescription":"turntable.song.cuePoint10.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint11` | button-action | Select Pitch Play Cue Point 11 | `{"modelEnabled":"turntable.song.cuePoint11.hasStart","modelPadColorIndex":"turntable.song.cuePoint11.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint11","modelDescription":"turntable.song.cuePoint11.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint12` | button-action | Select Pitch Play Cue Point 12 | `{"modelEnabled":"turntable.song.cuePoint12.hasStart","modelPadColorIndex":"turntable.song.cuePoint12.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint12","modelDescription":"turntable.song.cuePoint12.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint13` | button-action | Select Pitch Play Cue Point 13 | `{"modelEnabled":"turntable.song.cuePoint13.hasStart","modelPadColorIndex":"turntable.song.cuePoint13.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint13","modelDescription":"turntable.song.cuePoint13.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint14` | button-action | Select Pitch Play Cue Point 14 | `{"modelEnabled":"turntable.song.cuePoint14.hasStart","modelPadColorIndex":"turntable.song.cuePoint14.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint14","modelDescription":"turntable.song.cuePoint14.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint15` | button-action | Select Pitch Play Cue Point 15 | `{"modelEnabled":"turntable.song.cuePoint15.hasStart","modelPadColorIndex":"turntable.song.cuePoint15.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint15","modelDescription":"turntable.song.cuePoint15.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint16` | button-action | Select Pitch Play Cue Point 16 | `{"modelEnabled":"turntable.song.cuePoint16.hasStart","modelPadColorIndex":"turntable.song.cuePoint16.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint16","modelDescription":"turntable.song.cuePoint16.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint2` | button-action | Select Pitch Play Cue Point 2 | `{"modelEnabled":"turntable.song.cuePoint2.hasStart","modelPadColorIndex":"turntable.song.cuePoint2.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint2","modelDescription":"turntable.song.cuePoint2.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint3` | button-action | Select Pitch Play Cue Point 3 | `{"modelEnabled":"turntable.song.cuePoint3.hasStart","modelPadColorIndex":"turntable.song.cuePoint3.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint3","modelDescription":"turntable.song.cuePoint3.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint4` | button-action | Select Pitch Play Cue Point 4 | `{"modelEnabled":"turntable.song.cuePoint4.hasStart","modelPadColorIndex":"turntable.song.cuePoint4.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint4","modelDescription":"turntable.song.cuePoint4.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint5` | button-action | Select Pitch Play Cue Point 5 | `{"modelEnabled":"turntable.song.cuePoint5.hasStart","modelPadColorIndex":"turntable.song.cuePoint5.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint5","modelDescription":"turntable.song.cuePoint5.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint6` | button-action | Select Pitch Play Cue Point 6 | `{"modelEnabled":"turntable.song.cuePoint6.hasStart","modelPadColorIndex":"turntable.song.cuePoint6.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint6","modelDescription":"turntable.song.cuePoint6.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint7` | button-action | Select Pitch Play Cue Point 7 | `{"modelEnabled":"turntable.song.cuePoint7.hasStart","modelPadColorIndex":"turntable.song.cuePoint7.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint7","modelDescription":"turntable.song.cuePoint7.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint8` | button-action | Select Pitch Play Cue Point 8 | `{"modelEnabled":"turntable.song.cuePoint8.hasStart","modelPadColorIndex":"turntable.song.cuePoint8.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint8","modelDescription":"turntable.song.cuePoint8.name"}` |  |
| `turntable.pitchPlayCuePointCuePoint9` | button-action | Select Pitch Play Cue Point 9 | `{"modelEnabled":"turntable.song.cuePoint9.hasStart","modelPadColorIndex":"turntable.song.cuePoint9.presentationColorIndex","usePadColorsDimmed":true,"type":"button-action","modelState":"turntable.pitchPlayCuePointStateCuePoint9","modelDescription":"turntable.song.cuePoint9.name"}` |  |
| `turntable.pitchPlayRangeDown` | button-action | Pitch Play Range Down | `{"type":"button-action","modelState":"turntable.pitchPlayRangeIsDown"}` |  |
| `turntable.pitchPlayRangeUp` | button-action | Pitch Play Range Up | `{"type":"button-action","modelState":"turntable.pitchPlayRangeIsUp"}` |  |
| `turntable.pitchSelect` | button-action | Key Select | `{"actionParameterMinValue":-12,"actionParameterMaxValue":12,"type":"button-action","modelState":"turntable.effects.pitch.currentValue","hasActionParameter":true}` |  |
| `turntable.playPause` | button-toggle | Play / Pause | `{"type":"button-toggle","modelState":"turntable.isPlayingButtonState","modelEnabled":"turntable.playbackControlsAvailable"}` | Note CH1 data 0; Note CH2 data 0; Note CH3 data 0; Note CH4 data 0 |
| `turntable.reloop` | button-action | Reloop | `{"type":"button-action","modelEnabled":"turntable.song.masterLoopRegion.hasRange"}` |  |
| `turntable.resetFilter` | button-action | Reset Filter | `{"type":"button-action","modelState":"turntable.effects.filter.isReset","modelStateIsFlipped":true}` | Note CH1 data 20; Note CH2 data 20; Note CH3 data 20; Note CH4 data 20 |
| `turntable.resetGain` | button-action | Reset Gain | `{"type":"button-action","modelState":"turntable.effects.hardwareGain.isReset","modelStateIsFlipped":true}` |  |
| `turntable.resetHighEQ` | button-action | Reset High EQ | `{"type":"button-action","modelState":"turntable.highEQEffect.isReset","modelStateIsFlipped":true}` |  |
| `turntable.resetLowEQ` | button-action | Reset Low EQ | `{"type":"button-action","modelState":"turntable.lowEQEffect.isReset","modelStateIsFlipped":true}` |  |
| `turntable.resetMidEQ` | button-action | Reset Mid EQ | `{"type":"button-action","modelState":"turntable.midEQEffect.isReset","modelStateIsFlipped":true}` |  |
| `turntable.resetPan` | button-action | Reset Pan | `{"type":"button-action","modelState":"turntable.effects.pan.isReset","modelStateIsFlipped":true}` |  |
| `turntable.resetPitch` | button-action | Reset Key | `{"type":"button-action","modelState":"turntable.effects.pitch.isReset","modelStateIsFlipped":true}` |  |
| `turntable.resetSpeed` | button-action | Reset Tempo | `{"type":"button-action","modelState":"turntable.effects.tempo.isReset"}` |  |
| `turntable.reverse` | button-toggle | Reverse | `{"type":"button-toggle","modelState":"turntable.reverse","modelEnabled":"turntable.playbackControlsAvailable"}` | Note CH1 data 3; Note CH2 data 3; Note CH3 data 3; Note CH4 data 3 |
| `turntable.reverseHold` | button-hold | [Inferred] turntable / reverse Hold | `{"type":"button-hold","modelState":"turntable.reverse","modelEnabled":"turntable.playbackControlsAvailable"}` |  |
| `turntable.saveLoopIfLoopingOrActivate1` | button-action | Saved Loop Activate 1 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion1.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion1.isLoopingRange","modelState":"turntable.song.loopRegion1.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrActivate2` | button-action | Saved Loop Activate 2 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion2.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion2.isLoopingRange","modelState":"turntable.song.loopRegion2.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrActivate3` | button-action | Saved Loop Activate 3 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion3.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion3.isLoopingRange","modelState":"turntable.song.loopRegion3.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrActivate4` | button-action | Saved Loop Activate 4 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion4.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion4.isLoopingRange","modelState":"turntable.song.loopRegion4.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrActivate5` | button-action | Saved Loop Activate 5 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion5.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion5.isLoopingRange","modelState":"turntable.song.loopRegion5.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrActivate6` | button-action | Saved Loop Activate 6 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion6.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion6.isLoopingRange","modelState":"turntable.song.loopRegion6.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrActivate7` | button-action | Saved Loop Activate 7 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion7.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion7.isLoopingRange","modelState":"turntable.song.loopRegion7.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrActivate8` | button-action | Saved Loop Activate 8 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion8.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion8.isLoopingRange","modelState":"turntable.song.loopRegion8.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrReloop1` | button-action | Saved Loop Reloop 1 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion1.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion1.isLoopingRange","modelState":"turntable.song.loopRegion1.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrReloop2` | button-action | Saved Loop Reloop 2 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion2.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion2.isLoopingRange","modelState":"turntable.song.loopRegion2.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrReloop3` | button-action | Saved Loop Reloop 3 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion3.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion3.isLoopingRange","modelState":"turntable.song.loopRegion3.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrReloop4` | button-action | Saved Loop Reloop 4 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion4.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion4.isLoopingRange","modelState":"turntable.song.loopRegion4.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrReloop5` | button-action | Saved Loop Reloop 5 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion5.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion5.isLoopingRange","modelState":"turntable.song.loopRegion5.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrReloop6` | button-action | Saved Loop Reloop 6 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion6.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion6.isLoopingRange","modelState":"turntable.song.loopRegion6.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrReloop7` | button-action | Saved Loop Reloop 7 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion7.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion7.isLoopingRange","modelState":"turntable.song.loopRegion7.hasRange"}` |  |
| `turntable.saveLoopIfLoopingOrReloop8` | button-action | Saved Loop Reloop 8 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion8.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion8.isLoopingRange","modelState":"turntable.song.loopRegion8.hasRange"}` |  |
| `turntable.saveLoopOrActivate1` | button-action | [Inferred] turntable / save Loop Or Activate 1 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion1.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion1.isLoopingRange","modelState":"turntable.song.loopRegion1.hasRange"}` |  |
| `turntable.saveLoopOrActivate2` | button-action | [Inferred] turntable / save Loop Or Activate 2 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion2.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion2.isLoopingRange","modelState":"turntable.song.loopRegion2.hasRange"}` |  |
| `turntable.saveLoopOrActivate3` | button-action | [Inferred] turntable / save Loop Or Activate 3 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion3.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion3.isLoopingRange","modelState":"turntable.song.loopRegion3.hasRange"}` |  |
| `turntable.saveLoopOrActivate4` | button-action | [Inferred] turntable / save Loop Or Activate 4 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion4.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion4.isLoopingRange","modelState":"turntable.song.loopRegion4.hasRange"}` |  |
| `turntable.saveLoopOrActivate5` | button-action | [Inferred] turntable / save Loop Or Activate 5 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion5.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion5.isLoopingRange","modelState":"turntable.song.loopRegion5.hasRange"}` |  |
| `turntable.saveLoopOrActivate6` | button-action | [Inferred] turntable / save Loop Or Activate 6 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion6.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion6.isLoopingRange","modelState":"turntable.song.loopRegion6.hasRange"}` |  |
| `turntable.saveLoopOrActivate7` | button-action | [Inferred] turntable / save Loop Or Activate 7 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion7.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion7.isLoopingRange","modelState":"turntable.song.loopRegion7.hasRange"}` |  |
| `turntable.saveLoopOrActivate8` | button-action | [Inferred] turntable / save Loop Or Activate 8 | `{"modelEnabled":"turntable.transportControlsAvailable","modelDescription":"turntable.song.loopRegion8.name","type":"button-action","modelBlinkingState":"turntable.song.loopRegion8.isLoopingRange","modelState":"turntable.song.loopRegion8.hasRange"}` |  |
| `turntable.scrubbing` | control | Scrub | `{"type":"control","modelValue":"turntable.display.songProgress"}` |  |
| `turntable.sectionRotary` | not declared | Move Library Section (Rotary) | `{"steppedRotary":true}` |  |
| `turntable.selectTool1` | control | [Inferred] turntable / select Tool 1 | `{"type":"control","modelValue":"turntable.view.state.tool1.type"}` |  |
| `turntable.selectTool1CuePoints` | control | [Inferred] turntable / select Tool 1 Cue Points | `{"type":"control","modelValue":"turntable.view.state.tool1.cuePointsType"}` |  |
| `turntable.selectTool1CuePointsSection` | button-action | [Inferred] turntable / select Tool 1 Cue Points Section | `{"type":"button-action","modelState":"turntable.view.state.tool1.isCuePointsSectionSelected"}` |  |
| `turntable.selectTool1Effects` | control | [Inferred] turntable / select Tool 1 Effects | `{"type":"control","modelValue":"turntable.view.state.tool1.effectsType"}` |  |
| `turntable.selectTool1EffectsSection` | button-action | [Inferred] turntable / select Tool 1 Effects Section | `{"type":"button-action","modelState":"turntable.view.state.tool1.isEffectsSectionSelected"}` |  |
| `turntable.selectTool1EQSection` | button-action | [Inferred] turntable / select Tool 1 EQ Section | `{"type":"button-action","modelState":"turntable.view.state.tool1.isEQSectionSelected"}` |  |
| `turntable.selectTool1Looping` | control | [Inferred] turntable / select Tool 1 Looping | `{"type":"control","modelValue":"turntable.view.state.tool1.loopingType"}` |  |
| `turntable.selectTool1LoopingSection` | button-action | [Inferred] turntable / select Tool 1 Looping Section | `{"type":"button-action","modelState":"turntable.view.state.tool1.isLoopingSectionSelected"}` |  |
| `turntable.selectTool1UnmixerSection` | button-action | [Inferred] turntable / select Tool 1 Unmixer Section | `{"type":"button-action","modelState":"turntable.view.state.tool1.isUnmixerSectionSelected"}` |  |
| `turntable.selectTool2` | control | [Inferred] turntable / select Tool 2 | `{"type":"control","modelValue":"turntable.view.state.tool2.type"}` |  |
| `turntable.selectTool2CuePoints` | control | [Inferred] turntable / select Tool 2 Cue Points | `{"type":"control","modelValue":"turntable.view.state.tool2.cuePointsType"}` |  |
| `turntable.selectTool2CuePointsSection` | button-action | [Inferred] turntable / select Tool 2 Cue Points Section | `{"type":"button-action","modelState":"turntable.view.state.tool2.isCuePointsSectionSelected"}` |  |
| `turntable.selectTool2Effects` | control | [Inferred] turntable / select Tool 2 Effects | `{"type":"control","modelValue":"turntable.view.state.tool2.effectsType"}` |  |
| `turntable.selectTool2EffectsSection` | button-action | [Inferred] turntable / select Tool 2 Effects Section | `{"type":"button-action","modelState":"turntable.view.state.tool2.isEffectsSectionSelected"}` |  |
| `turntable.selectTool2Looping` | control | [Inferred] turntable / select Tool 2 Looping | `{"type":"control","modelValue":"turntable.view.state.tool2.loopingType"}` |  |
| `turntable.selectTool2LoopingSection` | button-action | [Inferred] turntable / select Tool 2 Looping Section | `{"type":"button-action","modelState":"turntable.view.state.tool2.isLoopingSectionSelected"}` |  |
| `turntable.selectTool2UnmixerSection` | button-action | [Inferred] turntable / select Tool 2 Unmixer Section | `{"type":"button-action","modelState":"turntable.view.state.tool2.isUnmixerSectionSelected"}` |  |
| `turntable.skipBackward` | button-action | Skip Backward | `{"type":"button-action","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.skipDuration0125Beats` | button-action | Skip Duration 1/8 Beat | `{"type":"button-action","modelState":"turntable.skipping0125BeatInterval"}` |  |
| `turntable.skipDuration025Beats` | button-action | Skip Duration 1/4 Beat | `{"type":"button-action","modelState":"turntable.skipping025BeatInterval"}` |  |
| `turntable.skipDuration05Beats` | button-action | Skip Duration 1/2 Beat | `{"type":"button-action","modelState":"turntable.skipping05BeatInterval"}` |  |
| `turntable.skipDuration16Beats` | button-action | Skip Duration 16 Beats | `{"type":"button-action","modelState":"turntable.skipping16BeatInterval"}` |  |
| `turntable.skipDuration1Beat` | button-action | Skip Duration 1 Beat | `{"type":"button-action","modelState":"turntable.skipping1BeatInterval"}` |  |
| `turntable.skipDuration2Beats` | button-action | Skip Duration 2 Beats | `{"type":"button-action","modelState":"turntable.skipping2BeatInterval"}` |  |
| `turntable.skipDuration32Beats` | button-action | Skip Duration 32 Beats | `{"type":"button-action","modelState":"turntable.skipping32BeatInterval"}` |  |
| `turntable.skipDuration4Beats` | button-action | Skip Duration 4 Beats | `{"type":"button-action","modelState":"turntable.skipping4BeatInterval"}` |  |
| `turntable.skipDuration64Beats` | button-action | Skip Duration 64 Beats | `{"type":"button-action","modelState":"turntable.skipping64BeatInterval"}` |  |
| `turntable.skipDuration8Beats` | button-action | Skip Duration 8 Beats | `{"type":"button-action","modelState":"turntable.skipping8BeatInterval"}` |  |
| `turntable.skipDurationRotary` | not declared | Skip Duration (Rotary) | `{"steppedRotary":true}` |  |
| `turntable.skipForward` | button-action | Skip Forward | `{"type":"button-action","modelEnabled":"turntable.transportControlsAvailable"}` |  |
| `turntable.skipRotary` | not declared | Skip (Rotary) | `{"steppedRotary":true}` | CC CH10 data 2 (rotary-64); CC CH11 data 2 (rotary-64); CC CH12 data 2 (rotary-64); CC CH9 data 2 (rotary-64) |
| `turntable.slicer4Slice1` | button-action | [Inferred] turntable / slicer 4 Slice 1 | `{"type":"button-action","modelState":"turntable.slicer4Slice1Active"}` |  |
| `turntable.slicer4Slice2` | button-action | [Inferred] turntable / slicer 4 Slice 2 | `{"type":"button-action","modelState":"turntable.slicer4Slice2Active"}` |  |
| `turntable.slicer4Slice3` | button-action | [Inferred] turntable / slicer 4 Slice 3 | `{"type":"button-action","modelState":"turntable.slicer4Slice3Active"}` |  |
| `turntable.slicer4Slice4` | button-action | [Inferred] turntable / slicer 4 Slice 4 | `{"type":"button-action","modelState":"turntable.slicer4Slice4Active"}` |  |
| `turntable.slicer8Slice1` | button-action | Slice 1 | `{"type":"button-action","modelState":"turntable.slicer8Slice1Active"}` |  |
| `turntable.slicer8Slice2` | button-action | Slice 2 | `{"type":"button-action","modelState":"turntable.slicer8Slice2Active"}` |  |
| `turntable.slicer8Slice3` | button-action | Slice 3 | `{"type":"button-action","modelState":"turntable.slicer8Slice3Active"}` |  |
| `turntable.slicer8Slice4` | button-action | Slice 4 | `{"type":"button-action","modelState":"turntable.slicer8Slice4Active"}` |  |
| `turntable.slicer8Slice5` | button-action | Slice 5 | `{"type":"button-action","modelState":"turntable.slicer8Slice5Active"}` |  |
| `turntable.slicer8Slice6` | button-action | Slice 6 | `{"type":"button-action","modelState":"turntable.slicer8Slice6Active"}` |  |
| `turntable.slicer8Slice7` | button-action | Slice 7 | `{"type":"button-action","modelState":"turntable.slicer8Slice7Active"}` |  |
| `turntable.slicer8Slice8` | button-action | Slice 8 | `{"type":"button-action","modelState":"turntable.slicer8Slice8Active"}` |  |
| `turntable.slicerLoopModeToggle` | button-toggle | Slicer Loop Activate | `{"type":"button-toggle","modelState":"turntable.slicerLoopMode"}` |  |
| `turntable.sourceRotary` | not declared | Select Library Source (Rotary) | `{"steppedRotary":true}` |  |
| `turntable.speed` | control | Tempo | `{"pickupMode":true,"modelValue":"turntable.effects.tempo.currentValue","type":"control","modelState":"turntable.effects.tempo.isReset","modelValueIsFlipped":true}` | CC CH1 data 0; pickup; flipped; CC CH2 data 0; pickup; flipped; CC CH3 data 0; pickup; flipped; CC CH4 data 0; pickup; flipped |
| `turntable.toggleControlMode` | button-toggle | Toggle Control Mode | `{"type":"button-toggle","modelState":"turntable.controlMode"}` |  |
| `turntable.toggleQuantize` | button-toggle | Toggle Quantize | `{"type":"button-toggle","modelState":"turntable.deckQuantize"}` | Note CH1 data 15; Note CH2 data 15; Note CH3 data 15; Note CH4 data 15 |
| `turntable.toggleQuantizeCueJump` | button-toggle | [Inferred] turntable / toggle Quantize Cue Jump | `{"type":"button-toggle","modelState":"turntable.shouldQuantizeCueJump"}` |  |
| `turntable.toggleQuantizeLoopJump` | button-toggle | [Inferred] turntable / toggle Quantize Loop Jump | `{"type":"button-toggle","modelState":"turntable.shouldQuantizeLoopJump"}` |  |
| `turntable.toggleQuantizeSliceJump` | button-toggle | [Inferred] turntable / toggle Quantize Slice Jump | `{"type":"button-toggle","modelState":"turntable.shouldQuantizeSliceJump"}` |  |
| `turntable.toggleShowTools` | button-toggle | [Inferred] turntable / toggle Show Tools | `{"type":"button-toggle","modelState":"turntable.view.state.showTools"}` |  |
| `turntable.toggleShowUnmixerPopup` | button-toggle | [Inferred] turntable / toggle Show Unmixer Popup | `{"type":"button-toggle","modelState":"turntable.view.state.showUnmixerCrossfader"}` |  |
| `turntable.toggleShowUnmixerSliderPopup` | button-toggle | [Inferred] turntable / toggle Show Unmixer Slider Popup | `{"type":"button-toggle","modelState":"turntable.view.state.unmixerCrossfaderModeSlider"}` |  |
| `turntable.toggleTimeDisplay` | button-toggle | Toggle Elapsed/Remaining Time | `{"type":"button-toggle","modelState":"turntable.display.showElapsedTime"}` |  |
| `turntable.toggleUnmixerCrossfadePercussiveMode` | button-toggle | Toggle Percussive - Instrumental Fade | `{"type":"button-toggle","modelState":"turntable.unmixer.crossfadePercussiveMode"}` |  |
| `turntable.toggleUnmixerEQMode` | button-toggle | Toggle Neural Mix - EQ | `{"type":"button-toggle","modelState":"turntable.enableUnmixerEQ"}` |  |
| `turntable.toggleUnmixerMuteFxEnabled` | button-toggle | Toggle Neural Mix - Mute/Solo Echo Out | `{"type":"button-toggle","modelState":"turntable.unmixer.muteWithFx"}` |  |
| `turntable.toggleUnmixerPercussiveMode` | button-toggle | [Inferred] turntable / toggle Unmixer Percussive Mode | `{"type":"button-toggle","modelState":"turntable.unmixer.percussiveMode"}` |  |
| `turntable.toggleWaveformAlternateMode` | button-toggle | Toggle Waveform Mode | `{"type":"button-toggle","modelState":"turntable.waveformAlternateMode"}` |  |
| `turntable.toggleWaveSlice` | button-toggle | [Inferred] turntable / toggle Wave Slice | `{"type":"button-toggle","modelState":"turntable.view.state.waveSlice"}` |  |
| `turntable.turntableIsSyncMaster` | button-action | Set Sync Master | `{"type":"button-action","modelState":"turntable.isSyncMaster","modelEnabled":"turntable.canSync"}` | Note CH1 data 8; Note CH2 data 8; Note CH3 data 8; Note CH4 data 8 |
| `turntable.unload` | button-action | Eject | `{"type":"button-action","modelEnabled":"turntable.hasSongTrack"}` |  |
| `turntable.unmixerAcapellaMuted` | button-toggle | Mute Vocals | `{"type":"button-toggle","modelState":"turntable.unmixer.acapellaMuted"}` |  |
| `turntable.unmixerAcapellaSolo` | button-toggle | Solo Vocals | `{"type":"button-toggle","modelState":"turntable.unmixer.acapellaSoloActive"}` |  |
| `turntable.unmixerAcapellaSwap` | button-toggle | Swap Vocals | `{"type":"button-toggle","modelState":"turntable.unmixer.acapellaSwapped"}` |  |
| `turntable.unmixerAcapellaVolume` | control | Vocals Volume | `{"type":"control","modelValue":"turntable.unmixer.acapellaVolume"}` |  |
| `turntable.unmixerCrossfade` | control | Neural Mix Filter (Instrumental / Acappella) | `{"type":"control","modelValue":"turntable.unmixer.crossfadeValue"}` |  |
| `turntable.unmixerCrossfadeSwitchLeft` | button-toggle | [Inferred] turntable / unmixer Crossfade Switch Left | `{"type":"button-toggle","modelState":"turntable.unmixer.crossfadeLeftSwitchActive"}` |  |
| `turntable.unmixerCrossfadeSwitchLeftInstrumental` | button-toggle | Neural Mix Instrumental | `{"type":"button-toggle","modelState":"turntable.unmixer.crossfadeLeftSwitchActive"}` |  |
| `turntable.unmixerCrossfadeSwitchRight` | button-toggle | [Inferred] turntable / unmixer Crossfade Switch Right | `{"type":"button-toggle","modelState":"turntable.unmixer.crossfadeRightSwitchActive"}` |  |
| `turntable.unmixerCrossfadeSwitchRightAcapella` | button-toggle | Neural Mix Acapella | `{"type":"button-toggle","modelState":"turntable.unmixer.crossfadeRightSwitchActive"}` |  |
| `turntable.unmixerDrumsMuted` | button-toggle | Mute Drums | `{"type":"button-toggle","modelState":"turntable.unmixer.instrumentalMuted"}` |  |
| `turntable.unmixerDrumsSolo` | button-toggle | Solo Drums | `{"type":"button-toggle","modelState":"turntable.unmixer.instrumentalSoloActive"}` |  |
| `turntable.unmixerDrumsSwap` | button-toggle | Swap Drums | `{"type":"button-toggle","modelState":"turntable.unmixer.instrumentalSwapped"}` |  |
| `turntable.unmixerDrumsVolume` | control | Drums Volume | `{"type":"control","modelValue":"turntable.unmixer.instrumentalVolume"}` |  |
| `turntable.unmixerEQAcapella` | control | Neural Mix Vocals Gain | `{"type":"control","modelState":"turntable.unmixer.eqThreeTrackChannel3VolumeEffect.isReset","modelValue":"turntable.unmixer.eqThreeTrackChannel3VolumeEffect.currentValue"}` |  |
| `turntable.unmixerEQDrums` | control | Neural Mix Drums Gain | `{"type":"control","modelState":"turntable.unmixer.eqThreeTrackChannel1VolumeEffect.isReset","modelValue":"turntable.unmixer.eqThreeTrackChannel1VolumeEffect.currentValue"}` |  |
| `turntable.unmixerEQHarmonic` | control | Neural Mix Harmonic Gain | `{"type":"control","modelState":"turntable.unmixer.eqThreeTrackChannel2VolumeEffect.isReset","modelValue":"turntable.unmixer.eqThreeTrackChannel2VolumeEffect.currentValue"}` |  |
| `turntable.unmixerFourTrackChannel1Active` | button-toggle | Neural Mix Active (4ch: Drums) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel1Active","modelDescription":"turntable.unmixer.fourTrackChannel1Name"}` |  |
| `turntable.unmixerFourTrackChannel1Muted` | button-toggle | Neural Mix Mute (4ch: Drums) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel1Muted","modelDescription":"turntable.unmixer.fourTrackChannel1Name"}` | Note CH1 data 56; Note CH2 data 56; Note CH3 data 56; Note CH4 data 56 |
| `turntable.unmixerFourTrackChannel1Solo` | button-toggle | Neural Mix Solo (4ch: Drums) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel1Solo","modelDescription":"turntable.unmixer.fourTrackChannel1Name"}` | Note CH1 data 60; Note CH2 data 60; Note CH3 data 60; Note CH4 data 60 |
| `turntable.unmixerFourTrackChannel1SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (4ch: Drums) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel1Solo","modelDescription":"turntable.unmixer.fourTrackChannel1Name"}` | Note CH10 data 56; Note CH11 data 56; Note CH12 data 56; Note CH9 data 56 |
| `turntable.unmixerFourTrackChannel1Swapped` | button-toggle | Neural Mix Swap (4ch: Drums) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel1Swapped","modelDescription":"turntable.unmixer.fourTrackChannel1Name"}` |  |
| `turntable.unmixerFourTrackChannel1Volume` | control | Neural Mix Volume (4ch: Drums) | `{"type":"control","modelValue":"turntable.unmixer.fourTrackChannel1Volume","modelDescription":"turntable.unmixer.fourTrackChannel1Name"}` |  |
| `turntable.unmixerFourTrackChannel1VolumeEQ` | control | Neural Mix EQ (4ch: Drums) | `{"modelValue":"turntable.unmixer.eqFourTrackChannel1VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqFourTrackChannel1VolumeEffect.isReset","modelDescription":"turntable.unmixer.fourTrackChannel1Name"}` |  |
| `turntable.unmixerFourTrackChannel2Active` | button-toggle | Neural Mix Active (4ch: Bass) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel2Active","modelDescription":"turntable.unmixer.fourTrackChannel2Name"}` |  |
| `turntable.unmixerFourTrackChannel2Muted` | button-toggle | Neural Mix Mute (4ch: Bass) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel2Muted","modelDescription":"turntable.unmixer.fourTrackChannel2Name"}` | Note CH1 data 57; Note CH2 data 57; Note CH3 data 57; Note CH4 data 57 |
| `turntable.unmixerFourTrackChannel2Solo` | button-toggle | Neural Mix Solo (4ch: Bass) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel2Solo","modelDescription":"turntable.unmixer.fourTrackChannel2Name"}` | Note CH1 data 61; Note CH2 data 61; Note CH3 data 61; Note CH4 data 61 |
| `turntable.unmixerFourTrackChannel2SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (4ch: Bass) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel2Solo","modelDescription":"turntable.unmixer.fourTrackChannel2Name"}` | Note CH10 data 57; Note CH11 data 57; Note CH12 data 57; Note CH9 data 57 |
| `turntable.unmixerFourTrackChannel2Swapped` | button-toggle | Neural Mix Swap (4ch: Bass) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel2Swapped","modelDescription":"turntable.unmixer.fourTrackChannel2Name"}` |  |
| `turntable.unmixerFourTrackChannel2Volume` | control | Neural Mix Volume (4ch: Bass) | `{"type":"control","modelValue":"turntable.unmixer.fourTrackChannel2Volume","modelDescription":"turntable.unmixer.fourTrackChannel2Name"}` |  |
| `turntable.unmixerFourTrackChannel2VolumeEQ` | control | Neural Mix EQ (4ch: Bass) | `{"modelValue":"turntable.unmixer.eqFourTrackChannel2VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqFourTrackChannel2VolumeEffect.isReset","modelDescription":"turntable.unmixer.fourTrackChannel2Name"}` |  |
| `turntable.unmixerFourTrackChannel3Active` | button-toggle | Neural Mix Active (4ch: Harmonic) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel3Active","modelDescription":"turntable.unmixer.fourTrackChannel3Name"}` |  |
| `turntable.unmixerFourTrackChannel3Muted` | button-toggle | Neural Mix Mute (4ch: Harmonic) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel3Muted","modelDescription":"turntable.unmixer.fourTrackChannel3Name"}` | Note CH1 data 58; Note CH2 data 58; Note CH3 data 58; Note CH4 data 58 |
| `turntable.unmixerFourTrackChannel3Solo` | button-toggle | Neural Mix Solo (4ch: Harmonic) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel3Solo","modelDescription":"turntable.unmixer.fourTrackChannel3Name"}` | Note CH1 data 62; Note CH2 data 62; Note CH3 data 62; Note CH4 data 62 |
| `turntable.unmixerFourTrackChannel3SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (4ch: Harmonic) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel3Solo","modelDescription":"turntable.unmixer.fourTrackChannel3Name"}` | Note CH10 data 58; Note CH11 data 58; Note CH12 data 58; Note CH9 data 58 |
| `turntable.unmixerFourTrackChannel3Swapped` | button-toggle | Neural Mix Swap (4ch: Harmonic) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel3Swapped","modelDescription":"turntable.unmixer.fourTrackChannel3Name"}` |  |
| `turntable.unmixerFourTrackChannel3Volume` | control | Neural Mix Volume (4ch: Harmonic) | `{"type":"control","modelValue":"turntable.unmixer.fourTrackChannel3Volume","modelDescription":"turntable.unmixer.fourTrackChannel3Name"}` |  |
| `turntable.unmixerFourTrackChannel3VolumeEQ` | control | Neural Mix EQ (4ch: Harmonic) | `{"modelValue":"turntable.unmixer.eqFourTrackChannel3VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqFourTrackChannel3VolumeEffect.isReset","modelDescription":"turntable.unmixer.fourTrackChannel3Name"}` |  |
| `turntable.unmixerFourTrackChannel4Active` | button-toggle | Neural Mix Active (4ch: Vocals) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel4Active","modelDescription":"turntable.unmixer.fourTrackChannel4Name"}` |  |
| `turntable.unmixerFourTrackChannel4Muted` | button-toggle | Neural Mix Mute (4ch: Vocals) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel4Muted","modelDescription":"turntable.unmixer.fourTrackChannel4Name"}` | Note CH1 data 59; Note CH2 data 59; Note CH3 data 59; Note CH4 data 59 |
| `turntable.unmixerFourTrackChannel4Solo` | button-toggle | Neural Mix Solo (4ch: Vocals) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel4Solo","modelDescription":"turntable.unmixer.fourTrackChannel4Name"}` | Note CH1 data 63; Note CH2 data 63; Note CH3 data 63; Note CH4 data 63 |
| `turntable.unmixerFourTrackChannel4SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (4ch: Vocals) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel4Solo","modelDescription":"turntable.unmixer.fourTrackChannel4Name"}` | Note CH10 data 59; Note CH11 data 59; Note CH12 data 59; Note CH9 data 59 |
| `turntable.unmixerFourTrackChannel4Swapped` | button-toggle | Neural Mix Swap (4ch: Vocals) | `{"type":"button-toggle","modelState":"turntable.unmixer.fourTrackChannel4Swapped","modelDescription":"turntable.unmixer.fourTrackChannel4Name"}` |  |
| `turntable.unmixerFourTrackChannel4Volume` | control | Neural Mix Volume (4ch: Vocals) | `{"type":"control","modelValue":"turntable.unmixer.fourTrackChannel4Volume","modelDescription":"turntable.unmixer.fourTrackChannel4Name"}` |  |
| `turntable.unmixerFourTrackChannel4VolumeEQ` | control | Neural Mix EQ (4ch: Vocals) | `{"modelValue":"turntable.unmixer.eqFourTrackChannel4VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqFourTrackChannel4VolumeEffect.isReset","modelDescription":"turntable.unmixer.fourTrackChannel4Name"}` |  |
| `turntable.unmixerGenericChannel1Active` | button-toggle | Neural Mix Active (1) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeOne","type":"button-toggle","modelState":"turntable.unmixer.genericChannel1Active","modelDescription":"turntable.unmixer.genericChannel1Name"}` |  |
| `turntable.unmixerGenericChannel1Muted` | button-toggle | Neural Mix Mute (1) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeOne","type":"button-toggle","modelState":"turntable.unmixer.genericChannel1Muted","modelDescription":"turntable.unmixer.genericChannel1Name"}` |  |
| `turntable.unmixerGenericChannel1Solo` | button-toggle | Neural Mix Solo (1) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeOne","type":"button-toggle","modelState":"turntable.unmixer.genericChannel1Solo","modelDescription":"turntable.unmixer.genericChannel1Name"}` |  |
| `turntable.unmixerGenericChannel1SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (1) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeOne","type":"button-toggle","modelState":"turntable.unmixer.genericChannel1Solo","modelDescription":"turntable.unmixer.genericChannel1Name"}` |  |
| `turntable.unmixerGenericChannel1Volume` | control | Neural Mix Volume (1) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeOne","modelValue":"turntable.unmixer.genericChannel1Volume","type":"control","modelDescription":"turntable.unmixer.genericChannel1Name"}` |  |
| `turntable.unmixerGenericChannel1VolumeEQ` | control | Neural Mix EQ (1) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeOne","modelValue":"turntable.unmixer.eqGenericChannel1VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqGenericChannel1VolumeEffect.isReset","modelDescription":"turntable.unmixer.genericChannel1Name"}` |  |
| `turntable.unmixerGenericChannel2Active` | button-toggle | Neural Mix Active (2) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeTwo","type":"button-toggle","modelState":"turntable.unmixer.genericChannel2Active","modelDescription":"turntable.unmixer.genericChannel2Name"}` |  |
| `turntable.unmixerGenericChannel2Muted` | button-toggle | Neural Mix Mute (2) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeTwo","type":"button-toggle","modelState":"turntable.unmixer.genericChannel2Muted","modelDescription":"turntable.unmixer.genericChannel2Name"}` |  |
| `turntable.unmixerGenericChannel2Solo` | button-toggle | Neural Mix Solo (2) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeTwo","type":"button-toggle","modelState":"turntable.unmixer.genericChannel2Solo","modelDescription":"turntable.unmixer.genericChannel2Name"}` |  |
| `turntable.unmixerGenericChannel2SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (2) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeTwo","type":"button-toggle","modelState":"turntable.unmixer.genericChannel2Solo","modelDescription":"turntable.unmixer.genericChannel2Name"}` |  |
| `turntable.unmixerGenericChannel2Volume` | control | Neural Mix Volume (2) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeTwo","modelValue":"turntable.unmixer.genericChannel2Volume","type":"control","modelDescription":"turntable.unmixer.genericChannel2Name"}` |  |
| `turntable.unmixerGenericChannel2VolumeEQ` | control | Neural Mix EQ (2) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeTwo","modelValue":"turntable.unmixer.eqGenericChannel2VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqGenericChannel2VolumeEffect.isReset","modelDescription":"turntable.unmixer.genericChannel2Name"}` |  |
| `turntable.unmixerGenericChannel3Active` | button-toggle | Neural Mix Active (3) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeThree","type":"button-toggle","modelState":"turntable.unmixer.genericChannel3Active","modelDescription":"turntable.unmixer.genericChannel3Name"}` |  |
| `turntable.unmixerGenericChannel3Muted` | button-toggle | Neural Mix Mute (3) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeThree","type":"button-toggle","modelState":"turntable.unmixer.genericChannel3Muted","modelDescription":"turntable.unmixer.genericChannel3Name"}` |  |
| `turntable.unmixerGenericChannel3Solo` | button-toggle | Neural Mix Solo (3) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeThree","type":"button-toggle","modelState":"turntable.unmixer.genericChannel3Solo","modelDescription":"turntable.unmixer.genericChannel3Name"}` |  |
| `turntable.unmixerGenericChannel3SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (3) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeThree","type":"button-toggle","modelState":"turntable.unmixer.genericChannel3Solo","modelDescription":"turntable.unmixer.genericChannel3Name"}` |  |
| `turntable.unmixerGenericChannel3Volume` | control | Neural Mix Volume (3) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeThree","modelValue":"turntable.unmixer.genericChannel3Volume","type":"control","modelDescription":"turntable.unmixer.genericChannel3Name"}` |  |
| `turntable.unmixerGenericChannel3VolumeEQ` | control | Neural Mix EQ (3) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeThree","modelValue":"turntable.unmixer.eqGenericChannel3VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqGenericChannel3VolumeEffect.isReset","modelDescription":"turntable.unmixer.genericChannel3Name"}` |  |
| `turntable.unmixerGenericChannel4Active` | button-toggle | Neural Mix Active (4) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeFour","type":"button-toggle","modelState":"turntable.unmixer.genericChannel4Active","modelDescription":"turntable.unmixer.genericChannel4Name"}` |  |
| `turntable.unmixerGenericChannel4Muted` | button-toggle | Neural Mix Mute (4) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeFour","type":"button-toggle","modelState":"turntable.unmixer.genericChannel4Muted","modelDescription":"turntable.unmixer.genericChannel4Name"}` |  |
| `turntable.unmixerGenericChannel4Solo` | button-toggle | Neural Mix Solo (4) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeFour","type":"button-toggle","modelState":"turntable.unmixer.genericChannel4Solo","modelDescription":"turntable.unmixer.genericChannel4Name"}` |  |
| `turntable.unmixerGenericChannel4SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (4) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeFour","type":"button-toggle","modelState":"turntable.unmixer.genericChannel4Solo","modelDescription":"turntable.unmixer.genericChannel4Name"}` |  |
| `turntable.unmixerGenericChannel4Volume` | control | Neural Mix Volume (4) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeFour","modelValue":"turntable.unmixer.genericChannel4Volume","type":"control","modelDescription":"turntable.unmixer.genericChannel4Name"}` |  |
| `turntable.unmixerGenericChannel4VolumeEQ` | control | Neural Mix EQ (4) | `{"modelEnabled":"turntable.view.state.hasUnmixerToolTrackModeFour","modelValue":"turntable.unmixer.eqGenericChannel4VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqGenericChannel4VolumeEffect.isReset","modelDescription":"turntable.unmixer.genericChannel4Name"}` |  |
| `turntable.unmixerHarmonicMuted` | button-toggle | Mute Harmonic | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel2Muted"}` |  |
| `turntable.unmixerHarmonicSolo` | button-toggle | Solo Harmonic | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel2Solo"}` |  |
| `turntable.unmixerHarmonicSwap` | button-toggle | Swap Harmonic | `{"type":"button-toggle","modelState":"turntable.unmixer.harmonicSwapped"}` |  |
| `turntable.unmixerHarmonicVolume` | control | Harmonic Volume | `{"type":"control","modelValue":"turntable.unmixer.threeTrackChannel2Volume"}` |  |
| `turntable.unmixerInstrumentalMuted` | button-toggle | Mute Instrumental | `{"type":"button-toggle","modelState":"turntable.unmixer.instrumentalMuted"}` |  |
| `turntable.unmixerInstrumentalSolo` | button-toggle | Solo Instrumental | `{"type":"button-toggle","modelState":"turntable.unmixer.instrumentalSoloActive"}` |  |
| `turntable.unmixerInstrumentalVolume` | control | Instrumental Volume | `{"type":"control","modelValue":"turntable.unmixer.instrumentalVolume"}` |  |
| `turntable.unmixerThreeTrackChannel1Active` | button-toggle | Neural Mix Active (3ch: Drums) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel1Active","modelDescription":"turntable.unmixer.threeTrackChannel1Name"}` |  |
| `turntable.unmixerThreeTrackChannel1Muted` | button-toggle | Neural Mix Mute (3ch: Drums) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel1Muted","modelDescription":"turntable.unmixer.threeTrackChannel1Name"}` |  |
| `turntable.unmixerThreeTrackChannel1Solo` | button-toggle | Neural Mix Solo (3ch: Drums) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel1Solo","modelDescription":"turntable.unmixer.threeTrackChannel1Name"}` |  |
| `turntable.unmixerThreeTrackChannel1SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (3ch: Drums) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel1Solo","modelDescription":"turntable.unmixer.threeTrackChannel1Name"}` |  |
| `turntable.unmixerThreeTrackChannel1Swapped` | button-toggle | Neural Mix Swap (3ch: Drums) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel1Swapped","modelDescription":"turntable.unmixer.threeTrackChannel1Name"}` |  |
| `turntable.unmixerThreeTrackChannel1Volume` | control | Neural Mix Volume (3ch: Drums) | `{"type":"control","modelValue":"turntable.unmixer.threeTrackChannel1Volume","modelDescription":"turntable.unmixer.threeTrackChannel1Name"}` |  |
| `turntable.unmixerThreeTrackChannel1VolumeEQ` | control | Neural Mix EQ (3ch: Drums) | `{"modelValue":"turntable.unmixer.eqThreeTrackChannel1VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqThreeTrackChannel1VolumeEffect.isReset","modelDescription":"turntable.unmixer.threeTrackChannel1Name"}` |  |
| `turntable.unmixerThreeTrackChannel2Active` | button-toggle | Neural Mix Active (3ch: Harmonic) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel2Active","modelDescription":"turntable.unmixer.threeTrackChannel2Name"}` |  |
| `turntable.unmixerThreeTrackChannel2Muted` | button-toggle | Neural Mix Mute (3ch: Harmonic) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel2Muted","modelDescription":"turntable.unmixer.threeTrackChannel2Name"}` |  |
| `turntable.unmixerThreeTrackChannel2Solo` | button-toggle | Neural Mix Solo (3ch: Harmonic) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel2Solo","modelDescription":"turntable.unmixer.threeTrackChannel2Name"}` |  |
| `turntable.unmixerThreeTrackChannel2SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (3ch: Harmonic) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel2Solo","modelDescription":"turntable.unmixer.threeTrackChannel2Name"}` |  |
| `turntable.unmixerThreeTrackChannel2Swapped` | button-toggle | Neural Mix Swap (3ch: Harmonic) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel2Swapped","modelDescription":"turntable.unmixer.threeTrackChannel2Name"}` |  |
| `turntable.unmixerThreeTrackChannel2Volume` | control | Neural Mix Volume (3ch: Harmonic) | `{"type":"control","modelValue":"turntable.unmixer.threeTrackChannel2Volume","modelDescription":"turntable.unmixer.threeTrackChannel2Name"}` |  |
| `turntable.unmixerThreeTrackChannel2VolumeEQ` | control | Neural Mix EQ (3ch: Harmonic) | `{"modelValue":"turntable.unmixer.eqThreeTrackChannel2VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqThreeTrackChannel2VolumeEffect.isReset","modelDescription":"turntable.unmixer.threeTrackChannel2Name"}` |  |
| `turntable.unmixerThreeTrackChannel3Active` | button-toggle | Neural Mix Active (3ch: Vocals) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel3Active","modelDescription":"turntable.unmixer.threeTrackChannel3Name"}` |  |
| `turntable.unmixerThreeTrackChannel3Muted` | button-toggle | Neural Mix Mute (3ch: Vocals) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel3Muted","modelDescription":"turntable.unmixer.threeTrackChannel3Name"}` |  |
| `turntable.unmixerThreeTrackChannel3Solo` | button-toggle | Neural Mix Solo (3ch: Vocals) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel3Solo","modelDescription":"turntable.unmixer.threeTrackChannel3Name"}` |  |
| `turntable.unmixerThreeTrackChannel3SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (3ch: Vocals) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel3Solo","modelDescription":"turntable.unmixer.threeTrackChannel3Name"}` |  |
| `turntable.unmixerThreeTrackChannel3Swapped` | button-toggle | Neural Mix Swap (3ch: Vocals) | `{"type":"button-toggle","modelState":"turntable.unmixer.threeTrackChannel3Swapped","modelDescription":"turntable.unmixer.threeTrackChannel3Name"}` |  |
| `turntable.unmixerThreeTrackChannel3Volume` | control | Neural Mix Volume (3ch: Vocals) | `{"type":"control","modelValue":"turntable.unmixer.threeTrackChannel3Volume","modelDescription":"turntable.unmixer.threeTrackChannel3Name"}` |  |
| `turntable.unmixerThreeTrackChannel3VolumeEQ` | control | Neural Mix EQ (3ch: Vocals) | `{"modelValue":"turntable.unmixer.eqThreeTrackChannel3VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqThreeTrackChannel3VolumeEffect.isReset","modelDescription":"turntable.unmixer.threeTrackChannel3Name"}` |  |
| `turntable.unmixerTwoTrackChannel1Active` | button-toggle | Neural Mix Active (2ch: Instrumental) | `{"type":"button-toggle","modelState":"turntable.unmixer.twoTrackChannel1Active","modelDescription":"turntable.unmixer.twoTrackChannel1Name"}` |  |
| `turntable.unmixerTwoTrackChannel1Muted` | button-toggle | Neural Mix Mute (2ch: Instrumental) | `{"type":"button-toggle","modelState":"turntable.unmixer.twoTrackChannel1Muted","modelDescription":"turntable.unmixer.twoTrackChannel1Name"}` |  |
| `turntable.unmixerTwoTrackChannel1Solo` | button-toggle | Neural Mix Solo (2ch: Instrumental) | `{"type":"button-toggle","modelState":"turntable.unmixer.twoTrackChannel1Solo","modelDescription":"turntable.unmixer.twoTrackChannel1Name"}` |  |
| `turntable.unmixerTwoTrackChannel1SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (2ch: Instrumental) | `{"type":"button-toggle","modelState":"turntable.unmixer.twoTrackChannel1Solo","modelDescription":"turntable.unmixer.twoTrackChannel1Name"}` |  |
| `turntable.unmixerTwoTrackChannel1Swapped` | button-toggle | Neural Mix Swap (2ch: Instrumental) | `{"type":"button-toggle","modelState":"turntable.unmixer.twoTrackChannel1Swapped","modelDescription":"turntable.unmixer.twoTrackChannel1Name"}` |  |
| `turntable.unmixerTwoTrackChannel1Volume` | control | Neural Mix Volume (2ch: Instrumental) | `{"type":"control","modelValue":"turntable.unmixer.twoTrackChannel1Volume","modelDescription":"turntable.unmixer.twoTrackChannel1Name"}` |  |
| `turntable.unmixerTwoTrackChannel1VolumeEQ` | control | Neural Mix EQ (2ch: Instrumental) | `{"modelValue":"turntable.unmixer.eqTwoTrackChannel1VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqTwoTrackChannel1VolumeEffect.isReset","modelDescription":"turntable.unmixer.twoTrackChannel1Name"}` |  |
| `turntable.unmixerTwoTrackChannel2Active` | button-toggle | Neural Mix Active (2ch: Acappella) | `{"type":"button-toggle","modelState":"turntable.unmixer.twoTrackChannel2Active","modelDescription":"turntable.unmixer.twoTrackChannel2Name"}` |  |
| `turntable.unmixerTwoTrackChannel2Muted` | button-toggle | Neural Mix Mute (2ch: Acappella) | `{"type":"button-toggle","modelState":"turntable.unmixer.twoTrackChannel2Muted","modelDescription":"turntable.unmixer.twoTrackChannel2Name"}` |  |
| `turntable.unmixerTwoTrackChannel2Solo` | button-toggle | Neural Mix Solo (2ch: Acappella) | `{"type":"button-toggle","modelState":"turntable.unmixer.twoTrackChannel2Solo","modelDescription":"turntable.unmixer.twoTrackChannel2Name"}` |  |
| `turntable.unmixerTwoTrackChannel2SoloExclusive` | button-toggle | Neural Mix Solo Exclusive (2ch: Acappella) | `{"type":"button-toggle","modelState":"turntable.unmixer.twoTrackChannel2Solo","modelDescription":"turntable.unmixer.twoTrackChannel2Name"}` |  |
| `turntable.unmixerTwoTrackChannel2Swapped` | button-toggle | Neural Mix Swap (2ch: Acappella) | `{"type":"button-toggle","modelState":"turntable.unmixer.twoTrackChannel2Swapped","modelDescription":"turntable.unmixer.twoTrackChannel2Name"}` |  |
| `turntable.unmixerTwoTrackChannel2Volume` | control | Neural Mix Volume (2ch: Acappella) | `{"type":"control","modelValue":"turntable.unmixer.twoTrackChannel2Volume","modelDescription":"turntable.unmixer.twoTrackChannel2Name"}` |  |
| `turntable.unmixerTwoTrackChannel2VolumeEQ` | control | Neural Mix EQ (2ch: Acappella) | `{"modelValue":"turntable.unmixer.eqTwoTrackChannel2VolumeEffect.currentValue","type":"control","modelState":"turntable.unmixer.eqTwoTrackChannel2VolumeEffect.isReset","modelDescription":"turntable.unmixer.twoTrackChannel2Name"}` |  |
| `turntable.waveSlice` | button-hold | [Inferred] turntable / wave Slice | `{"type":"button-hold","modelState":"turntable.view.state.waveSlice"}` |  |
| `turntable.waveSlip` | button-hold | [Inferred] turntable / wave Slip | `{"type":"button-hold","modelState":"turntable.waveSlip"}` |  |
| `turntable.waveSlipToggle` | button-toggle | [Inferred] turntable / wave Slip Toggle | `{"type":"button-toggle","modelState":"turntable.anySlip"}` |  |

## Preset-observed input key paths outside the metadata dictionary

This table normalizes deck families for readability: `turntable1`–`turntable4` and `turntableSelected` are shown as `turntable`; `sampler.turntable1/2` as `sampler.turntable`. The “Concrete paths seen” column preserves every distinct exact spelling found in the installed preset controls. Rows are present because at least one shipped preset uses the path, not because metadata declares a general-purpose contract. **578** normalized paths (**874** concrete spellings) are outside the 944-entry metadata catalog. S4 usage is blank unless the generated mapping actually assigns it.

| Preset-observed key path | Human description | Concrete paths seen in presets | Embedded-output control records | S4 MIDI usage |
|---|---|---|---:|---|
| `application` | General | `application` | 11 |  |
| `application.cuttingModifier` | [Inferred] application / cutting Modifier | `application.cuttingModifier` |  |  |
| `application.modifier` | Controller Shift Key | `application.modifier` | 26 |  |
| `application.startDelay` | Start Delay | `application.startDelay` |  |  |
| `application.startStopDelay` | Start/Stop Delay | `application.startStopDelay` |  |  |
| `application.stopDelay` | Stop Delay | `application.stopDelay` |  |  |
| `application.tempoSliderRangeMinus` | Decrease Tempo Slider Range | `application.tempoSliderRangeMinus` | 14 |  |
| `application.tempoSliderRangeNext` | Switch Tempo Slider Range | `application.tempoSliderRangeNext` | 105 |  |
| `application.tempoSliderRangePlus` | Increase Tempo Slider Range | `application.tempoSliderRangePlus` | 14 |  |
| `application.viewModeNext` | Next View Mode | `application.viewModeNext` | 1 |  |
| `application.viewModePrev` | Previous View Mode | `application.viewModePrev` |  |  |
| `customProcessor` | Device Custom | `customProcessor` |  |  |
| `customProcessor.alphaThetaDDJFLX2Deck1ResetTempo` | [Inferred] custom Processor / alpha Theta DDJFLX 2 Deck 1 Reset Tempo | `customProcessor.alphaThetaDDJFLX2Deck1ResetTempo` |  |  |
| `customProcessor.alphaThetaDDJFLX2Deck2ResetTempo` | [Inferred] custom Processor / alpha Theta DDJFLX 2 Deck 2 Reset Tempo | `customProcessor.alphaThetaDDJFLX2Deck2ResetTempo` |  |  |
| `customProcessor.alphaThetaDDJFLX2SmartCFX1` | [Inferred] custom Processor / alpha Theta DDJFLX 2 Smart CFX 1 | `customProcessor.alphaThetaDDJFLX2SmartCFX1` |  |  |
| `customProcessor.alphaThetaDDJFLX2SmartCFX2` | [Inferred] custom Processor / alpha Theta DDJFLX 2 Smart CFX 2 | `customProcessor.alphaThetaDDJFLX2SmartCFX2` |  |  |
| `customProcessor.alphaThetaDDJFLX2SmartCFXEnabled` | [Inferred] custom Processor / alpha Theta DDJFLX 2 Smart CFX Enabled | `customProcessor.alphaThetaDDJFLX2SmartCFXEnabled` | 1 |  |
| `customProcessor.alphaThetaDDJFLX2SmartCFXPreset1` | [Inferred] custom Processor / alpha Theta DDJFLX 2 Smart CFX Preset 1 | `customProcessor.alphaThetaDDJFLX2SmartCFXPreset1` | 1 |  |
| `customProcessor.alphaThetaDDJFLX2SmartCFXPreset2` | [Inferred] custom Processor / alpha Theta DDJFLX 2 Smart CFX Preset 2 | `customProcessor.alphaThetaDDJFLX2SmartCFXPreset2` | 1 |  |
| `customProcessor.alphaThetaXDJANTempoRange10` | [Inferred] custom Processor / alpha Theta XDJAN Tempo Range 10 | `customProcessor.alphaThetaXDJANTempoRange10` | 2 |  |
| `customProcessor.alphaThetaXDJANTempoRange16` | [Inferred] custom Processor / alpha Theta XDJAN Tempo Range 16 | `customProcessor.alphaThetaXDJANTempoRange16` | 2 |  |
| `customProcessor.alphaThetaXDJANTempoRange6` | [Inferred] custom Processor / alpha Theta XDJAN Tempo Range 6 | `customProcessor.alphaThetaXDJANTempoRange6` | 2 |  |
| `customProcessor.alphaThetaXDJANTempoRange75` | [Inferred] custom Processor / alpha Theta XDJAN Tempo Range 75 | `customProcessor.alphaThetaXDJANTempoRange75` | 2 |  |
| `customProcessor.beatFXDepth` | [Inferred] custom Processor / beat FX Depth | `customProcessor.beatFXDepth` |  |  |
| `customProcessor.beatFXDestination1` | [Inferred] custom Processor / beat FX Destination 1 | `customProcessor.beatFXDestination1` |  |  |
| `customProcessor.beatFXDestination2` | [Inferred] custom Processor / beat FX Destination 2 | `customProcessor.beatFXDestination2` |  |  |
| `customProcessor.beatFXDestination3` | [Inferred] custom Processor / beat FX Destination 3 | `customProcessor.beatFXDestination3` |  |  |
| `customProcessor.beatFXDestination4` | [Inferred] custom Processor / beat FX Destination 4 | `customProcessor.beatFXDestination4` |  |  |
| `customProcessor.beatFXDestinationCrossfadeLeft` | [Inferred] custom Processor / beat FX Destination Crossfade Left | `customProcessor.beatFXDestinationCrossfadeLeft` |  |  |
| `customProcessor.beatFXDestinationCrossfadeRight` | [Inferred] custom Processor / beat FX Destination Crossfade Right | `customProcessor.beatFXDestinationCrossfadeRight` |  |  |
| `customProcessor.beatFXDestinationMain` | [Inferred] custom Processor / beat FX Destination Main | `customProcessor.beatFXDestinationMain` |  |  |
| `customProcessor.beatFXDestinationMic` | [Inferred] custom Processor / beat FX Destination Mic | `customProcessor.beatFXDestinationMic` |  |  |
| `customProcessor.beatFXDestinationSampler` | [Inferred] custom Processor / beat FX Destination Sampler | `customProcessor.beatFXDestinationSampler` |  |  |
| `customProcessor.beatFXEffectDelay` | [Inferred] custom Processor / beat FX Effect Delay | `customProcessor.beatFXEffectDelay` |  |  |
| `customProcessor.beatFXEffectEcho` | [Inferred] custom Processor / beat FX Effect Echo | `customProcessor.beatFXEffectEcho` |  |  |
| `customProcessor.beatFXEffectFilter` | [Inferred] custom Processor / beat FX Effect Filter | `customProcessor.beatFXEffectFilter` |  |  |
| `customProcessor.beatFXEffectFlanger` | [Inferred] custom Processor / beat FX Effect Flanger | `customProcessor.beatFXEffectFlanger` |  |  |
| `customProcessor.beatFXEffectHelix` | [Inferred] custom Processor / beat FX Effect Helix | `customProcessor.beatFXEffectHelix` |  |  |
| `customProcessor.beatFXEffectLowCutEcho` | [Inferred] custom Processor / beat FX Effect Low Cut Echo | `customProcessor.beatFXEffectLowCutEcho` |  |  |
| `customProcessor.beatFXEffectMobius` | [Inferred] custom Processor / beat FX Effect Mobius | `customProcessor.beatFXEffectMobius` |  |  |
| `customProcessor.beatFXEffectMobiusSaw` | [Inferred] custom Processor / beat FX Effect Mobius Saw | `customProcessor.beatFXEffectMobiusSaw` |  |  |
| `customProcessor.beatFXEffectMobiusTri` | [Inferred] custom Processor / beat FX Effect Mobius Tri | `customProcessor.beatFXEffectMobiusTri` |  |  |
| `customProcessor.beatFXEffectPhaser` | [Inferred] custom Processor / beat FX Effect Phaser | `customProcessor.beatFXEffectPhaser` |  |  |
| `customProcessor.beatFXEffectPingPong` | [Inferred] custom Processor / beat FX Effect Ping Pong | `customProcessor.beatFXEffectPingPong` |  |  |
| `customProcessor.beatFXEffectPitch` | [Inferred] custom Processor / beat FX Effect Pitch | `customProcessor.beatFXEffectPitch` |  |  |
| `customProcessor.beatFXEffectReverb` | [Inferred] custom Processor / beat FX Effect Reverb | `customProcessor.beatFXEffectReverb` |  |  |
| `customProcessor.beatFXEffectRoll` | [Inferred] custom Processor / beat FX Effect Roll | `customProcessor.beatFXEffectRoll` |  |  |
| `customProcessor.beatFXEffectSpiral` | [Inferred] custom Processor / beat FX Effect Spiral | `customProcessor.beatFXEffectSpiral` |  |  |
| `customProcessor.beatFXEffectTrans` | [Inferred] custom Processor / beat FX Effect Trans | `customProcessor.beatFXEffectTrans` |  |  |
| `customProcessor.beatFXEffectTripletFilter` | [Inferred] custom Processor / beat FX Effect Triplet Filter | `customProcessor.beatFXEffectTripletFilter` |  |  |
| `customProcessor.beatFXEffectTripletRoll` | [Inferred] custom Processor / beat FX Effect Triplet Roll | `customProcessor.beatFXEffectTripletRoll` |  |  |
| `customProcessor.beatFXEnabled` | [Inferred] custom Processor / beat FX Enabled | `customProcessor.beatFXEnabled` | 1 |  |
| `customProcessor.beatFXHi` | [Inferred] custom Processor / beat FX Hi | `customProcessor.beatFXHi` |  |  |
| `customProcessor.beatFXLow` | [Inferred] custom Processor / beat FX Low | `customProcessor.beatFXLow` |  |  |
| `customProcessor.beatFXMid` | [Inferred] custom Processor / beat FX Mid | `customProcessor.beatFXMid` |  |  |
| `customProcessor.beatFXParameterValue` | [Inferred] custom Processor / beat FX Parameter Value | `customProcessor.beatFXParameterValue` |  |  |
| `customProcessor.beatFXParameterValueMinus` | [Inferred] custom Processor / beat FX Parameter Value Minus | `customProcessor.beatFXParameterValueMinus` |  |  |
| `customProcessor.beatFXParameterValuePlus` | [Inferred] custom Processor / beat FX Parameter Value Plus | `customProcessor.beatFXParameterValuePlus` |  |  |
| `customProcessor.cdjNxs2LoopInOrMoveInPoint` | [Inferred] custom Processor / cdj Nxs 2 Loop In Or Move In Point | `customProcessor.cdjNxs2LoopInOrMoveInPoint` | 1 |  |
| `customProcessor.cdjNxs2SyncOrInstantDouble` | [Inferred] custom Processor / cdj Nxs 2 Sync Or Instant Double | `customProcessor.cdjNxs2SyncOrInstantDouble` | 1 |  |
| `customProcessor.colorFXCrush` | [Inferred] custom Processor / color FX Crush | `customProcessor.colorFXCrush` |  |  |
| `customProcessor.colorFXDubEcho` | [Inferred] custom Processor / color FX Dub Echo | `customProcessor.colorFXDubEcho` |  |  |
| `customProcessor.colorFXEnabled` | [Inferred] custom Processor / color FX Enabled | `customProcessor.colorFXEnabled` | 1 |  |
| `customProcessor.colorFXFilter` | [Inferred] custom Processor / color FX Filter | `customProcessor.colorFXFilter` |  |  |
| `customProcessor.colorFXKnob1` | [Inferred] custom Processor / color FX Knob 1 | `customProcessor.colorFXKnob1` |  |  |
| `customProcessor.colorFXKnob2` | [Inferred] custom Processor / color FX Knob 2 | `customProcessor.colorFXKnob2` |  |  |
| `customProcessor.colorFXKnob3` | [Inferred] custom Processor / color FX Knob 3 | `customProcessor.colorFXKnob3` |  |  |
| `customProcessor.colorFXKnob4` | [Inferred] custom Processor / color FX Knob 4 | `customProcessor.colorFXKnob4` |  |  |
| `customProcessor.colorFXNoise` | [Inferred] custom Processor / color FX Noise | `customProcessor.colorFXNoise` |  |  |
| `customProcessor.colorFXParameter` | [Inferred] custom Processor / color FX Parameter | `customProcessor.colorFXParameter` |  |  |
| `customProcessor.colorFXSelect` | [Inferred] custom Processor / color FX Select | `customProcessor.colorFXSelect` |  |  |
| `customProcessor.colorFXSpace` | [Inferred] custom Processor / color FX Space | `customProcessor.colorFXSpace` |  |  |
| `customProcessor.colorFXSweep` | [Inferred] custom Processor / color FX Sweep | `customProcessor.colorFXSweep` |  |  |
| `customProcessor.crossfadeCurve` | [Inferred] custom Processor / crossfade Curve | `customProcessor.crossfadeCurve` |  |  |
| `customProcessor.ddj1000BeatFXAutoBPM` | [Inferred] custom Processor / ddj 1000 Beat FX Auto BPM | `customProcessor.ddj1000BeatFXAutoBPM` |  |  |
| `customProcessor.ddj1000BeatFXDepth` | [Inferred] custom Processor / ddj 1000 Beat FX Depth | `customProcessor.ddj1000BeatFXDepth` |  |  |
| `customProcessor.ddj1000BeatFXDestination1` | [Inferred] custom Processor / ddj 1000 Beat FX Destination 1 | `customProcessor.ddj1000BeatFXDestination1` |  |  |
| `customProcessor.ddj1000BeatFXDestination2` | [Inferred] custom Processor / ddj 1000 Beat FX Destination 2 | `customProcessor.ddj1000BeatFXDestination2` |  |  |
| `customProcessor.ddj1000BeatFXDestination3` | [Inferred] custom Processor / ddj 1000 Beat FX Destination 3 | `customProcessor.ddj1000BeatFXDestination3` |  |  |
| `customProcessor.ddj1000BeatFXDestination4` | [Inferred] custom Processor / ddj 1000 Beat FX Destination 4 | `customProcessor.ddj1000BeatFXDestination4` |  |  |
| `customProcessor.ddj1000BeatFXDestinationMain` | [Inferred] custom Processor / ddj 1000 Beat FX Destination Main | `customProcessor.ddj1000BeatFXDestinationMain` |  |  |
| `customProcessor.ddj1000BeatFXDestinationMic` | [Inferred] custom Processor / ddj 1000 Beat FX Destination Mic | `customProcessor.ddj1000BeatFXDestinationMic` |  |  |
| `customProcessor.ddj1000BeatFXDestinationSampler` | [Inferred] custom Processor / ddj 1000 Beat FX Destination Sampler | `customProcessor.ddj1000BeatFXDestinationSampler` |  |  |
| `customProcessor.ddj1000BeatFXEcho` | [Inferred] custom Processor / ddj 1000 Beat FX Echo | `customProcessor.ddj1000BeatFXEcho` |  |  |
| `customProcessor.ddj1000BeatFXEnabled` | [Inferred] custom Processor / ddj 1000 Beat FX Enabled | `customProcessor.ddj1000BeatFXEnabled` |  |  |
| `customProcessor.ddj1000BeatFXEnigmaJet` | [Inferred] custom Processor / ddj 1000 Beat FX Enigma Jet | `customProcessor.ddj1000BeatFXEnigmaJet` |  |  |
| `customProcessor.ddj1000BeatFXFlanger` | [Inferred] custom Processor / ddj 1000 Beat FX Flanger | `customProcessor.ddj1000BeatFXFlanger` |  |  |
| `customProcessor.ddj1000BeatFXLowCutEcho` | [Inferred] custom Processor / ddj 1000 Beat FX Low Cut Echo | `customProcessor.ddj1000BeatFXLowCutEcho` |  |  |
| `customProcessor.ddj1000BeatFXMobiusSaw` | [Inferred] custom Processor / ddj 1000 Beat FX Mobius Saw | `customProcessor.ddj1000BeatFXMobiusSaw` |  |  |
| `customProcessor.ddj1000BeatFXMobiusTri` | [Inferred] custom Processor / ddj 1000 Beat FX Mobius Tri | `customProcessor.ddj1000BeatFXMobiusTri` |  |  |
| `customProcessor.ddj1000BeatFXMTDelay` | [Inferred] custom Processor / ddj 1000 Beat FXMT Delay | `customProcessor.ddj1000BeatFXMTDelay` |  |  |
| `customProcessor.ddj1000BeatFXParameterMinus` | [Inferred] custom Processor / ddj 1000 Beat FX Parameter Minus | `customProcessor.ddj1000BeatFXParameterMinus` |  |  |
| `customProcessor.ddj1000BeatFXParameterPlus` | [Inferred] custom Processor / ddj 1000 Beat FX Parameter Plus | `customProcessor.ddj1000BeatFXParameterPlus` |  |  |
| `customProcessor.ddj1000BeatFXPhaser` | [Inferred] custom Processor / ddj 1000 Beat FX Phaser | `customProcessor.ddj1000BeatFXPhaser` |  |  |
| `customProcessor.ddj1000BeatFXPitch` | [Inferred] custom Processor / ddj 1000 Beat FX Pitch | `customProcessor.ddj1000BeatFXPitch` |  |  |
| `customProcessor.ddj1000BeatFXReverb` | [Inferred] custom Processor / ddj 1000 Beat FX Reverb | `customProcessor.ddj1000BeatFXReverb` |  |  |
| `customProcessor.ddj1000BeatFXRoll` | [Inferred] custom Processor / ddj 1000 Beat FX Roll | `customProcessor.ddj1000BeatFXRoll` |  |  |
| `customProcessor.ddj1000BeatFXSlipRoll` | [Inferred] custom Processor / ddj 1000 Beat FX Slip Roll | `customProcessor.ddj1000BeatFXSlipRoll` |  |  |
| `customProcessor.ddj1000BeatFXSpiral` | [Inferred] custom Processor / ddj 1000 Beat FX Spiral | `customProcessor.ddj1000BeatFXSpiral` |  |  |
| `customProcessor.ddj1000BeatFXTapBPM` | [Inferred] custom Processor / ddj 1000 Beat FX Tap BPM | `customProcessor.ddj1000BeatFXTapBPM` |  |  |
| `customProcessor.ddj1000BeatFXTransformer` | [Inferred] custom Processor / ddj 1000 Beat FX Transformer | `customProcessor.ddj1000BeatFXTransformer` |  |  |
| `customProcessor.ddj200TransitionFX` | [Inferred] custom Processor / ddj 200 Transition FX | `customProcessor.ddj200TransitionFX` | 1 |  |
| `customProcessor.ddjFlx4SmartFaderEcho` | [Inferred] custom Processor / ddj Flx 4 Smart Fader Echo | `customProcessor.ddjFlx4SmartFaderEcho` | 1 |  |
| `customProcessor.ddjFlx4SmartFaderReverb` | [Inferred] custom Processor / ddj Flx 4 Smart Fader Reverb | `customProcessor.ddjFlx4SmartFaderReverb` | 1 |  |
| `customProcessor.ddjRev5AutoLoopWithSize1` | [Inferred] custom Processor / ddj Rev 5 Auto Loop With Size 1 | `customProcessor.ddjRev5AutoLoopWithSize1` |  |  |
| `customProcessor.ddjRev5AutoLoopWithSize2` | [Inferred] custom Processor / ddj Rev 5 Auto Loop With Size 2 | `customProcessor.ddjRev5AutoLoopWithSize2` |  |  |
| `customProcessor.ddjRev5AutoLoopWithSize3` | [Inferred] custom Processor / ddj Rev 5 Auto Loop With Size 3 | `customProcessor.ddjRev5AutoLoopWithSize3` |  |  |
| `customProcessor.ddjRev5AutoLoopWithSize4` | [Inferred] custom Processor / ddj Rev 5 Auto Loop With Size 4 | `customProcessor.ddjRev5AutoLoopWithSize4` |  |  |
| `customProcessor.ddjRev5DeckAutoTransitionSelectButton` | [Inferred] custom Processor / ddj Rev 5 Deck Auto Transition Select Button | `customProcessor.ddjRev5DeckAutoTransitionSelectButton` |  |  |
| `customProcessor.ddjRev5StemSplitDeck1Bass` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 1 Bass | `customProcessor.ddjRev5StemSplitDeck1Bass` |  |  |
| `customProcessor.ddjRev5StemSplitDeck1Drums` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 1 Drums | `customProcessor.ddjRev5StemSplitDeck1Drums` |  |  |
| `customProcessor.ddjRev5StemSplitDeck1Harmonic` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 1 Harmonic | `customProcessor.ddjRev5StemSplitDeck1Harmonic` |  |  |
| `customProcessor.ddjRev5StemSplitDeck1Vocals` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 1 Vocals | `customProcessor.ddjRev5StemSplitDeck1Vocals` |  |  |
| `customProcessor.ddjRev5StemSplitDeck2Bass` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 2 Bass | `customProcessor.ddjRev5StemSplitDeck2Bass` |  |  |
| `customProcessor.ddjRev5StemSplitDeck2Drums` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 2 Drums | `customProcessor.ddjRev5StemSplitDeck2Drums` |  |  |
| `customProcessor.ddjRev5StemSplitDeck2Harmonic` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 2 Harmonic | `customProcessor.ddjRev5StemSplitDeck2Harmonic` |  |  |
| `customProcessor.ddjRev5StemSplitDeck2Vocals` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 2 Vocals | `customProcessor.ddjRev5StemSplitDeck2Vocals` |  |  |
| `customProcessor.ddjRev5StemSplitDeck3Bass` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 3 Bass | `customProcessor.ddjRev5StemSplitDeck3Bass` |  |  |
| `customProcessor.ddjRev5StemSplitDeck3Drums` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 3 Drums | `customProcessor.ddjRev5StemSplitDeck3Drums` |  |  |
| `customProcessor.ddjRev5StemSplitDeck3Harmonic` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 3 Harmonic | `customProcessor.ddjRev5StemSplitDeck3Harmonic` |  |  |
| `customProcessor.ddjRev5StemSplitDeck3Vocals` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 3 Vocals | `customProcessor.ddjRev5StemSplitDeck3Vocals` |  |  |
| `customProcessor.ddjRev5StemSplitDeck4Bass` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 4 Bass | `customProcessor.ddjRev5StemSplitDeck4Bass` |  |  |
| `customProcessor.ddjRev5StemSplitDeck4Drums` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 4 Drums | `customProcessor.ddjRev5StemSplitDeck4Drums` |  |  |
| `customProcessor.ddjRev5StemSplitDeck4Harmonic` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 4 Harmonic | `customProcessor.ddjRev5StemSplitDeck4Harmonic` |  |  |
| `customProcessor.ddjRev5StemSplitDeck4Vocals` | [Inferred] custom Processor / ddj Rev 5 Stem Split Deck 4 Vocals | `customProcessor.ddjRev5StemSplitDeck4Vocals` |  |  |
| `customProcessor.ddjRev7AutoLoopWithSize1` | [Inferred] custom Processor / ddj Rev 7 Auto Loop With Size 1 | `customProcessor.ddjRev7AutoLoopWithSize1` |  |  |
| `customProcessor.ddjRev7AutoLoopWithSize2` | [Inferred] custom Processor / ddj Rev 7 Auto Loop With Size 2 | `customProcessor.ddjRev7AutoLoopWithSize2` |  |  |
| `customProcessor.ddjRev7FxEnabled1` | [Inferred] custom Processor / ddj Rev 7 Fx Enabled 1 | `customProcessor.ddjRev7FxEnabled1` |  |  |
| `customProcessor.ddjRev7FxEnabled2` | [Inferred] custom Processor / ddj Rev 7 Fx Enabled 2 | `customProcessor.ddjRev7FxEnabled2` |  |  |
| `customProcessor.ddjRev7FxSelection1` | [Inferred] custom Processor / ddj Rev 7 Fx Selection 1 | `customProcessor.ddjRev7FxSelection1` |  |  |
| `customProcessor.ddjRev7FxSelection2` | [Inferred] custom Processor / ddj Rev 7 Fx Selection 2 | `customProcessor.ddjRev7FxSelection2` |  |  |
| `customProcessor.ddjRev7FxSelection3` | [Inferred] custom Processor / ddj Rev 7 Fx Selection 3 | `customProcessor.ddjRev7FxSelection3` |  |  |
| `customProcessor.ddjRev7FxSelection4` | [Inferred] custom Processor / ddj Rev 7 Fx Selection 4 | `customProcessor.ddjRev7FxSelection4` |  |  |
| `customProcessor.ddjRev7FxSelection5` | [Inferred] custom Processor / ddj Rev 7 Fx Selection 5 | `customProcessor.ddjRev7FxSelection5` |  |  |
| `customProcessor.ddjRev7FxSelection6` | [Inferred] custom Processor / ddj Rev 7 Fx Selection 6 | `customProcessor.ddjRev7FxSelection6` |  |  |
| `customProcessor.ddjS11AutoLoopWithSize1` | [Inferred] custom Processor / ddj S 11 Auto Loop With Size 1 | `customProcessor.ddjS11AutoLoopWithSize1` |  |  |
| `customProcessor.ddjS11AutoLoopWithSize2` | [Inferred] custom Processor / ddj S 11 Auto Loop With Size 2 | `customProcessor.ddjS11AutoLoopWithSize2` |  |  |
| `customProcessor.djmS11FxEnabled1` | [Inferred] custom Processor / djm S 11 Fx Enabled 1 | `customProcessor.djmS11FxEnabled1` |  |  |
| `customProcessor.djmS11FxEnabled2` | [Inferred] custom Processor / djm S 11 Fx Enabled 2 | `customProcessor.djmS11FxEnabled2` |  |  |
| `customProcessor.djmS11FxParameterMinus` | [Inferred] custom Processor / djm S 11 Fx Parameter Minus | `customProcessor.djmS11FxParameterMinus` |  |  |
| `customProcessor.djmS11FxParameterPlus` | [Inferred] custom Processor / djm S 11 Fx Parameter Plus | `customProcessor.djmS11FxParameterPlus` |  |  |
| `customProcessor.djmS11FxSelection1` | [Inferred] custom Processor / djm S 11 Fx Selection 1 | `customProcessor.djmS11FxSelection1` |  |  |
| `customProcessor.djmS11FxSelection2` | [Inferred] custom Processor / djm S 11 Fx Selection 2 | `customProcessor.djmS11FxSelection2` |  |  |
| `customProcessor.djmS11FxSelection3` | [Inferred] custom Processor / djm S 11 Fx Selection 3 | `customProcessor.djmS11FxSelection3` |  |  |
| `customProcessor.djmS11FxSelection4` | [Inferred] custom Processor / djm S 11 Fx Selection 4 | `customProcessor.djmS11FxSelection4` |  |  |
| `customProcessor.djmS11FxSelection5` | [Inferred] custom Processor / djm S 11 Fx Selection 5 | `customProcessor.djmS11FxSelection5` |  |  |
| `customProcessor.djmS11FxSelection6` | [Inferred] custom Processor / djm S 11 Fx Selection 6 | `customProcessor.djmS11FxSelection6` |  |  |
| `customProcessor.eqCurve` | [Inferred] custom Processor / eq Curve | `customProcessor.eqCurve` |  |  |
| `customProcessor.flx10BeatFXAutoBPM` | [Inferred] custom Processor / flx 10 Beat FX Auto BPM | `customProcessor.flx10BeatFXAutoBPM` |  |  |
| `customProcessor.flx10BeatFXDepth` | [Inferred] custom Processor / flx 10 Beat FX Depth | `customProcessor.flx10BeatFXDepth` |  |  |
| `customProcessor.flx10BeatFXDestination1` | [Inferred] custom Processor / flx 10 Beat FX Destination 1 | `customProcessor.flx10BeatFXDestination1` |  |  |
| `customProcessor.flx10BeatFXDestination2` | [Inferred] custom Processor / flx 10 Beat FX Destination 2 | `customProcessor.flx10BeatFXDestination2` |  |  |
| `customProcessor.flx10BeatFXDestination3` | [Inferred] custom Processor / flx 10 Beat FX Destination 3 | `customProcessor.flx10BeatFXDestination3` |  |  |
| `customProcessor.flx10BeatFXDestination4` | [Inferred] custom Processor / flx 10 Beat FX Destination 4 | `customProcessor.flx10BeatFXDestination4` |  |  |
| `customProcessor.flx10BeatFXDestinationMain` | [Inferred] custom Processor / flx 10 Beat FX Destination Main | `customProcessor.flx10BeatFXDestinationMain` |  |  |
| `customProcessor.flx10BeatFXDestinationMic` | [Inferred] custom Processor / flx 10 Beat FX Destination Mic | `customProcessor.flx10BeatFXDestinationMic` |  |  |
| `customProcessor.flx10BeatFXDestinationSampler` | [Inferred] custom Processor / flx 10 Beat FX Destination Sampler | `customProcessor.flx10BeatFXDestinationSampler` |  |  |
| `customProcessor.flx10BeatFXEcho` | [Inferred] custom Processor / flx 10 Beat FX Echo | `customProcessor.flx10BeatFXEcho` |  |  |
| `customProcessor.flx10BeatFXEnabled` | [Inferred] custom Processor / flx 10 Beat FX Enabled | `customProcessor.flx10BeatFXEnabled` |  |  |
| `customProcessor.flx10BeatFXEnigmaJet` | [Inferred] custom Processor / flx 10 Beat FX Enigma Jet | `customProcessor.flx10BeatFXEnigmaJet` |  |  |
| `customProcessor.flx10BeatFXFlanger` | [Inferred] custom Processor / flx 10 Beat FX Flanger | `customProcessor.flx10BeatFXFlanger` |  |  |
| `customProcessor.flx10BeatFXLowCutEcho` | [Inferred] custom Processor / flx 10 Beat FX Low Cut Echo | `customProcessor.flx10BeatFXLowCutEcho` |  |  |
| `customProcessor.flx10BeatFXMobiusSaw` | [Inferred] custom Processor / flx 10 Beat FX Mobius Saw | `customProcessor.flx10BeatFXMobiusSaw` |  |  |
| `customProcessor.flx10BeatFXMobiusTri` | [Inferred] custom Processor / flx 10 Beat FX Mobius Tri | `customProcessor.flx10BeatFXMobiusTri` |  |  |
| `customProcessor.flx10BeatFXMTDelay` | [Inferred] custom Processor / flx 10 Beat FXMT Delay | `customProcessor.flx10BeatFXMTDelay` |  |  |
| `customProcessor.flx10BeatFXParameterMinus` | [Inferred] custom Processor / flx 10 Beat FX Parameter Minus | `customProcessor.flx10BeatFXParameterMinus` |  |  |
| `customProcessor.flx10BeatFXParameterPlus` | [Inferred] custom Processor / flx 10 Beat FX Parameter Plus | `customProcessor.flx10BeatFXParameterPlus` |  |  |
| `customProcessor.flx10BeatFXPhaser` | [Inferred] custom Processor / flx 10 Beat FX Phaser | `customProcessor.flx10BeatFXPhaser` |  |  |
| `customProcessor.flx10BeatFXReverb` | [Inferred] custom Processor / flx 10 Beat FX Reverb | `customProcessor.flx10BeatFXReverb` |  |  |
| `customProcessor.flx10BeatFXRoll` | [Inferred] custom Processor / flx 10 Beat FX Roll | `customProcessor.flx10BeatFXRoll` |  |  |
| `customProcessor.flx10BeatFXSlipRoll` | [Inferred] custom Processor / flx 10 Beat FX Slip Roll | `customProcessor.flx10BeatFXSlipRoll` |  |  |
| `customProcessor.flx10BeatFXSpiral` | [Inferred] custom Processor / flx 10 Beat FX Spiral | `customProcessor.flx10BeatFXSpiral` |  |  |
| `customProcessor.flx10BeatFXStretch` | [Inferred] custom Processor / flx 10 Beat FX Stretch | `customProcessor.flx10BeatFXStretch` |  |  |
| `customProcessor.flx10BeatFXTapBPM` | [Inferred] custom Processor / flx 10 Beat FX Tap BPM | `customProcessor.flx10BeatFXTapBPM` |  |  |
| `customProcessor.flx10BeatFXToggle` | [Inferred] custom Processor / flx 10 Beat FX Toggle | `customProcessor.flx10BeatFXToggle` |  |  |
| `customProcessor.flx10BeatFXTransformer` | [Inferred] custom Processor / flx 10 Beat FX Transformer | `customProcessor.flx10BeatFXTransformer` |  |  |
| `customProcessor.flx10ColorFXCrush` | [Inferred] custom Processor / flx 10 Color FX Crush | `customProcessor.flx10ColorFXCrush` |  |  |
| `customProcessor.flx10ColorFXDEcho` | [Inferred] custom Processor / flx 10 Color FXD Echo | `customProcessor.flx10ColorFXDEcho` |  |  |
| `customProcessor.flx10ColorFXFilter` | [Inferred] custom Processor / flx 10 Color FX Filter | `customProcessor.flx10ColorFXFilter` |  |  |
| `customProcessor.flx10ColorFXNoise` | [Inferred] custom Processor / flx 10 Color FX Noise | `customProcessor.flx10ColorFXNoise` |  |  |
| `customProcessor.flx10ColorFXPitch` | [Inferred] custom Processor / flx 10 Color FX Pitch | `customProcessor.flx10ColorFXPitch` |  |  |
| `customProcessor.flx10ColorFXSpace` | [Inferred] custom Processor / flx 10 Color FX Space | `customProcessor.flx10ColorFXSpace` |  |  |
| `customProcessor.flx10FilterKnob1` | [Inferred] custom Processor / flx 10 Filter Knob 1 | `customProcessor.flx10FilterKnob1` |  |  |
| `customProcessor.flx10FilterKnob2` | [Inferred] custom Processor / flx 10 Filter Knob 2 | `customProcessor.flx10FilterKnob2` |  |  |
| `customProcessor.flx10FilterKnob3` | [Inferred] custom Processor / flx 10 Filter Knob 3 | `customProcessor.flx10FilterKnob3` |  |  |
| `customProcessor.flx10FilterKnob4` | [Inferred] custom Processor / flx 10 Filter Knob 4 | `customProcessor.flx10FilterKnob4` |  |  |
| `customProcessor.flx10FXStemDrums` | [Inferred] custom Processor / flx 10 FX Stem Drums | `customProcessor.flx10FXStemDrums` |  |  |
| `customProcessor.flx10FXStemInstrumentals` | [Inferred] custom Processor / flx 10 FX Stem Instrumentals | `customProcessor.flx10FXStemInstrumentals` |  |  |
| `customProcessor.flx10FXStemVocal` | [Inferred] custom Processor / flx 10 FX Stem Vocal | `customProcessor.flx10FXStemVocal` |  |  |
| `customProcessor.flx10InstantDoubleDrums1` | [Inferred] custom Processor / flx 10 Instant Double Drums 1 | `customProcessor.flx10InstantDoubleDrums1` |  |  |
| `customProcessor.flx10InstantDoubleDrums2` | [Inferred] custom Processor / flx 10 Instant Double Drums 2 | `customProcessor.flx10InstantDoubleDrums2` |  |  |
| `customProcessor.flx10InstantDoubleDrums3` | [Inferred] custom Processor / flx 10 Instant Double Drums 3 | `customProcessor.flx10InstantDoubleDrums3` |  |  |
| `customProcessor.flx10InstantDoubleDrums4` | [Inferred] custom Processor / flx 10 Instant Double Drums 4 | `customProcessor.flx10InstantDoubleDrums4` |  |  |
| `customProcessor.flx10InstantDoubleInstrumentals1` | [Inferred] custom Processor / flx 10 Instant Double Instrumentals 1 | `customProcessor.flx10InstantDoubleInstrumentals1` |  |  |
| `customProcessor.flx10InstantDoubleInstrumentals2` | [Inferred] custom Processor / flx 10 Instant Double Instrumentals 2 | `customProcessor.flx10InstantDoubleInstrumentals2` |  |  |
| `customProcessor.flx10InstantDoubleInstrumentals3` | [Inferred] custom Processor / flx 10 Instant Double Instrumentals 3 | `customProcessor.flx10InstantDoubleInstrumentals3` |  |  |
| `customProcessor.flx10InstantDoubleInstrumentals4` | [Inferred] custom Processor / flx 10 Instant Double Instrumentals 4 | `customProcessor.flx10InstantDoubleInstrumentals4` |  |  |
| `customProcessor.flx10InstantDoubleVocals1` | [Inferred] custom Processor / flx 10 Instant Double Vocals 1 | `customProcessor.flx10InstantDoubleVocals1` |  |  |
| `customProcessor.flx10InstantDoubleVocals2` | [Inferred] custom Processor / flx 10 Instant Double Vocals 2 | `customProcessor.flx10InstantDoubleVocals2` |  |  |
| `customProcessor.flx10InstantDoubleVocals3` | [Inferred] custom Processor / flx 10 Instant Double Vocals 3 | `customProcessor.flx10InstantDoubleVocals3` |  |  |
| `customProcessor.flx10InstantDoubleVocals4` | [Inferred] custom Processor / flx 10 Instant Double Vocals 4 | `customProcessor.flx10InstantDoubleVocals4` |  |  |
| `customProcessor.fourBeatLoopOnInPoint1` | [Inferred] custom Processor / four Beat Loop On In Point 1 | `customProcessor.fourBeatLoopOnInPoint1` | 1 |  |
| `customProcessor.fourBeatLoopOnInPoint2` | [Inferred] custom Processor / four Beat Loop On In Point 2 | `customProcessor.fourBeatLoopOnInPoint2` | 1 |  |
| `customProcessor.fourBeatLoopOnInPoint3` | [Inferred] custom Processor / four Beat Loop On In Point 3 | `customProcessor.fourBeatLoopOnInPoint3` | 1 |  |
| `customProcessor.fourBeatLoopOnInPoint4` | [Inferred] custom Processor / four Beat Loop On In Point 4 | `customProcessor.fourBeatLoopOnInPoint4` | 1 |  |
| `customProcessor.herculesT10ChannelFXDubEcho` | [Inferred] custom Processor / hercules T 10 Channel FX Dub Echo | `customProcessor.herculesT10ChannelFXDubEcho` | 1 |  |
| `customProcessor.herculesT10ChannelFXFilter` | [Inferred] custom Processor / hercules T 10 Channel FX Filter | `customProcessor.herculesT10ChannelFXFilter` | 1 |  |
| `customProcessor.herculesT10ChannelFXKnob1` | [Inferred] custom Processor / hercules T 10 Channel FX Knob 1 | `customProcessor.herculesT10ChannelFXKnob1` |  |  |
| `customProcessor.herculesT10ChannelFXKnob2` | [Inferred] custom Processor / hercules T 10 Channel FX Knob 2 | `customProcessor.herculesT10ChannelFXKnob2` |  |  |
| `customProcessor.herculesT10ChannelFXNoise` | [Inferred] custom Processor / hercules T 10 Channel FX Noise | `customProcessor.herculesT10ChannelFXNoise` | 1 |  |
| `customProcessor.herculesT10ChannelFXReverb` | [Inferred] custom Processor / hercules T 10 Channel FX Reverb | `customProcessor.herculesT10ChannelFXReverb` | 1 |  |
| `customProcessor.jumpToLastCuePointAndPauseCrossfadeLeft` | [Inferred] custom Processor / jump To Last Cue Point And Pause Crossfade Left | `customProcessor.jumpToLastCuePointAndPauseCrossfadeLeft` |  |  |
| `customProcessor.jumpToLastCuePointAndPauseCrossfadeRight` | [Inferred] custom Processor / jump To Last Cue Point And Pause Crossfade Right | `customProcessor.jumpToLastCuePointAndPauseCrossfadeRight` |  |  |
| `customProcessor.jumpToLastCuePointAndPlayCrossfadeLeft` | [Inferred] custom Processor / jump To Last Cue Point And Play Crossfade Left | `customProcessor.jumpToLastCuePointAndPlayCrossfadeLeft` |  |  |
| `customProcessor.jumpToLastCuePointAndPlayCrossfadeRight` | [Inferred] custom Processor / jump To Last Cue Point And Play Crossfade Right | `customProcessor.jumpToLastCuePointAndPlayCrossfadeRight` |  |  |
| `customProcessor.micFXEcho` | [Inferred] custom Processor / mic FX Echo | `customProcessor.micFXEcho` |  |  |
| `customProcessor.micFXMegaphone` | [Inferred] custom Processor / mic FX Megaphone | `customProcessor.micFXMegaphone` |  |  |
| `customProcessor.micFXParameter` | [Inferred] custom Processor / mic FX Parameter | `customProcessor.micFXParameter` |  |  |
| `customProcessor.micFXPitch` | [Inferred] custom Processor / mic FX Pitch | `customProcessor.micFXPitch` |  |  |
| `customProcessor.mixon8Turntable1SourceSwitch` | [Inferred] custom Processor / mixon 8 Turntable 1 Source Switch | `customProcessor.mixon8Turntable1SourceSwitch` |  |  |
| `customProcessor.mixon8Turntable2SourceSwitch` | [Inferred] custom Processor / mixon 8 Turntable 2 Source Switch | `customProcessor.mixon8Turntable2SourceSwitch` |  |  |
| `customProcessor.mixon8Turntable3SourceSwitch` | [Inferred] custom Processor / mixon 8 Turntable 3 Source Switch | `customProcessor.mixon8Turntable3SourceSwitch` |  |  |
| `customProcessor.mixon8Turntable4SourceSwitch` | [Inferred] custom Processor / mixon 8 Turntable 4 Source Switch | `customProcessor.mixon8Turntable4SourceSwitch` |  |  |
| `customProcessor.mixtrackGoCyclePadModeDeck1` | [Inferred] custom Processor / mixtrack Go Cycle Pad Mode Deck 1 | `customProcessor.mixtrackGoCyclePadModeDeck1` |  |  |
| `customProcessor.mixtrackGoCyclePadModeDeck2` | [Inferred] custom Processor / mixtrack Go Cycle Pad Mode Deck 2 | `customProcessor.mixtrackGoCyclePadModeDeck2` |  |  |
| `customProcessor.mixtrackGoToggleFilterLow` | [Inferred] custom Processor / mixtrack Go Toggle Filter Low | `customProcessor.mixtrackGoToggleFilterLow` | 1 |  |
| `customProcessor.numarkMixtrackGoMDeck1Acapella` | [Inferred] custom Processor / numark Mixtrack Go M Deck 1 Acapella | `customProcessor.numarkMixtrackGoMDeck1Acapella` | 1 |  |
| `customProcessor.numarkMixtrackGoMDeck1Instrumental` | [Inferred] custom Processor / numark Mixtrack Go M Deck 1 Instrumental | `customProcessor.numarkMixtrackGoMDeck1Instrumental` | 1 |  |
| `customProcessor.numarkMixtrackGoMDeck1StemFxAcapella` | [Inferred] custom Processor / numark Mixtrack Go M Deck 1 Stem Fx Acapella | `customProcessor.numarkMixtrackGoMDeck1StemFxAcapella` | 1 |  |
| `customProcessor.numarkMixtrackGoMDeck1StemFxInstrumental` | [Inferred] custom Processor / numark Mixtrack Go M Deck 1 Stem Fx Instrumental | `customProcessor.numarkMixtrackGoMDeck1StemFxInstrumental` | 1 |  |
| `customProcessor.numarkMixtrackGoMDeck2Acapella` | [Inferred] custom Processor / numark Mixtrack Go M Deck 2 Acapella | `customProcessor.numarkMixtrackGoMDeck2Acapella` | 1 |  |
| `customProcessor.numarkMixtrackGoMDeck2Instrumental` | [Inferred] custom Processor / numark Mixtrack Go M Deck 2 Instrumental | `customProcessor.numarkMixtrackGoMDeck2Instrumental` | 1 |  |
| `customProcessor.numarkMixtrackGoMDeck2StemFxAcapella` | [Inferred] custom Processor / numark Mixtrack Go M Deck 2 Stem Fx Acapella | `customProcessor.numarkMixtrackGoMDeck2StemFxAcapella` | 1 |  |
| `customProcessor.numarkMixtrackGoMDeck2StemFxInstrumental` | [Inferred] custom Processor / numark Mixtrack Go M Deck 2 Stem Fx Instrumental | `customProcessor.numarkMixtrackGoMDeck2StemFxInstrumental` | 1 |  |
| `customProcessor.PartyMixThreeCyclePadModeDeck1` | [Inferred] custom Processor / Party Mix Three Cycle Pad Mode Deck 1 | `customProcessor.PartyMixThreeCyclePadModeDeck1` |  |  |
| `customProcessor.PartyMixThreeCyclePadModeDeck2` | [Inferred] custom Processor / Party Mix Three Cycle Pad Mode Deck 2 | `customProcessor.PartyMixThreeCyclePadModeDeck2` |  |  |
| `customProcessor.pioneerDDJXP2SetDeck3` | [Inferred] custom Processor / pioneer DDJXP 2 Set Deck 3 | `customProcessor.pioneerDDJXP2SetDeck3` | 2 |  |
| `customProcessor.pioneerDDJXP2SetDeck4` | [Inferred] custom Processor / pioneer DDJXP 2 Set Deck 4 | `customProcessor.pioneerDDJXP2SetDeck4` | 2 |  |
| `customProcessor.raneOneMk2FxBeatParameter` | [Inferred] custom Processor / rane One Mk 2 Fx Beat Parameter | `customProcessor.raneOneMk2FxBeatParameter` |  |  |
| `customProcessor.raneOneMk2FxEnabled1` | [Inferred] custom Processor / rane One Mk 2 Fx Enabled 1 | `customProcessor.raneOneMk2FxEnabled1` |  |  |
| `customProcessor.raneOneMk2FxEnabled2` | [Inferred] custom Processor / rane One Mk 2 Fx Enabled 2 | `customProcessor.raneOneMk2FxEnabled2` |  |  |
| `customProcessor.raneOneMk2FxSelection1` | [Inferred] custom Processor / rane One Mk 2 Fx Selection 1 | `customProcessor.raneOneMk2FxSelection1` | 1 |  |
| `customProcessor.raneOneMk2FxSelection2` | [Inferred] custom Processor / rane One Mk 2 Fx Selection 2 | `customProcessor.raneOneMk2FxSelection2` | 1 |  |
| `customProcessor.raneOneMk2FxSelection3` | [Inferred] custom Processor / rane One Mk 2 Fx Selection 3 | `customProcessor.raneOneMk2FxSelection3` | 1 |  |
| `customProcessor.raneOneMk2FxSelection4` | [Inferred] custom Processor / rane One Mk 2 Fx Selection 4 | `customProcessor.raneOneMk2FxSelection4` | 1 |  |
| `customProcessor.raneOneMk2FxSelection5` | [Inferred] custom Processor / rane One Mk 2 Fx Selection 5 | `customProcessor.raneOneMk2FxSelection5` | 1 |  |
| `customProcessor.raneOneMk2FxSelection6` | [Inferred] custom Processor / rane One Mk 2 Fx Selection 6 | `customProcessor.raneOneMk2FxSelection6` | 1 |  |
| `customProcessor.raneOneMk2FxTapTurntable1` | [Inferred] custom Processor / rane One Mk 2 Fx Tap Turntable 1 | `customProcessor.raneOneMk2FxTapTurntable1` |  |  |
| `customProcessor.raneOneMk2FxTapTurntable2` | [Inferred] custom Processor / rane One Mk 2 Fx Tap Turntable 2 | `customProcessor.raneOneMk2FxTapTurntable2` |  |  |
| `customProcessor.raneOneMk2SoftwareFxEnabled` | [Inferred] custom Processor / rane One Mk 2 Software Fx Enabled | `customProcessor.raneOneMk2SoftwareFxEnabled` |  |  |
| `customProcessor.raneOneMk2Turntable1Acapella` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Acapella | `customProcessor.raneOneMk2Turntable1Acapella` | 1 |  |
| `customProcessor.raneOneMk2Turntable1Instrumental` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Instrumental | `customProcessor.raneOneMk2Turntable1Instrumental` | 1 |  |
| `customProcessor.raneOneMk2Turntable1MidiBounceLoopRouting.routingGenericChannel1` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Midi Bounce Loop Routing / routing Generic Channel 1 | `customProcessor.raneOneMk2Turntable1MidiBounceLoopRouting.routingGenericChannel1` | 1 |  |
| `customProcessor.raneOneMk2Turntable1MidiBounceLoopRouting.routingGenericChannel2` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Midi Bounce Loop Routing / routing Generic Channel 2 | `customProcessor.raneOneMk2Turntable1MidiBounceLoopRouting.routingGenericChannel2` | 1 |  |
| `customProcessor.raneOneMk2Turntable1MidiBounceLoopRouting.routingGenericChannel3` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Midi Bounce Loop Routing / routing Generic Channel 3 | `customProcessor.raneOneMk2Turntable1MidiBounceLoopRouting.routingGenericChannel3` | 1 |  |
| `customProcessor.raneOneMk2Turntable1MidiBounceLoopRouting.routingGenericChannel4` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Midi Bounce Loop Routing / routing Generic Channel 4 | `customProcessor.raneOneMk2Turntable1MidiBounceLoopRouting.routingGenericChannel4` | 1 |  |
| `customProcessor.raneOneMk2Turntable1PadFxBraker` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Pad Fx Braker | `customProcessor.raneOneMk2Turntable1PadFxBraker` | 1 |  |
| `customProcessor.raneOneMk2Turntable1PadFxDelay` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Pad Fx Delay | `customProcessor.raneOneMk2Turntable1PadFxDelay` | 1 |  |
| `customProcessor.raneOneMk2Turntable1PadFxEchoOut` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Pad Fx Echo Out | `customProcessor.raneOneMk2Turntable1PadFxEchoOut` | 1 |  |
| `customProcessor.raneOneMk2Turntable1PadFxReverbOut` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Pad Fx Reverb Out | `customProcessor.raneOneMk2Turntable1PadFxReverbOut` | 1 |  |
| `customProcessor.raneOneMk2Turntable1PadModeHotCue` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Pad Mode Hot Cue | `customProcessor.raneOneMk2Turntable1PadModeHotCue` | 1 |  |
| `customProcessor.raneOneMk2Turntable1PadModeRoll` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Pad Mode Roll | `customProcessor.raneOneMk2Turntable1PadModeRoll` | 1 |  |
| `customProcessor.raneOneMk2Turntable1PadModeSampler` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Pad Mode Sampler | `customProcessor.raneOneMk2Turntable1PadModeSampler` | 1 |  |
| `customProcessor.raneOneMk2Turntable1PadModeSavedLoop` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Pad Mode Saved Loop | `customProcessor.raneOneMk2Turntable1PadModeSavedLoop` | 1 |  |
| `customProcessor.raneOneMk2Turntable1PadModeStems` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Pad Mode Stems | `customProcessor.raneOneMk2Turntable1PadModeStems` | 1 |  |
| `customProcessor.raneOneMk2Turntable1SmallPadModeButton` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Small Pad Mode Button | `customProcessor.raneOneMk2Turntable1SmallPadModeButton` |  |  |
| `customProcessor.raneOneMk2Turntable1SmallPadModeHotCue` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Small Pad Mode Hot Cue | `customProcessor.raneOneMk2Turntable1SmallPadModeHotCue` | 1 |  |
| `customProcessor.raneOneMk2Turntable1SmallPadModeSampler` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Small Pad Mode Sampler | `customProcessor.raneOneMk2Turntable1SmallPadModeSampler` | 1 |  |
| `customProcessor.raneOneMk2Turntable1SmallPadModeSBanks` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Small Pad Mode S Banks | `customProcessor.raneOneMk2Turntable1SmallPadModeSBanks` | 1 |  |
| `customProcessor.raneOneMk2Turntable1SmallPadModeStems` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Small Pad Mode Stems | `customProcessor.raneOneMk2Turntable1SmallPadModeStems` | 1 |  |
| `customProcessor.raneOneMk2Turntable1SmallPadsBounceLoop1` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Small Pads Bounce Loop 1 | `customProcessor.raneOneMk2Turntable1SmallPadsBounceLoop1` | 1 |  |
| `customProcessor.raneOneMk2Turntable1SmallPadsBounceLoop1_2` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Small Pads Bounce Loop 1 2 | `customProcessor.raneOneMk2Turntable1SmallPadsBounceLoop1_2` | 1 |  |
| `customProcessor.raneOneMk2Turntable1SmallPadsBounceLoop1_4` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Small Pads Bounce Loop 1 4 | `customProcessor.raneOneMk2Turntable1SmallPadsBounceLoop1_4` | 1 |  |
| `customProcessor.raneOneMk2Turntable1SmallPadsBounceLoop1_8` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Small Pads Bounce Loop 1 8 | `customProcessor.raneOneMk2Turntable1SmallPadsBounceLoop1_8` | 1 |  |
| `customProcessor.raneOneMk2Turntable1StemFxAcapella` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Stem Fx Acapella | `customProcessor.raneOneMk2Turntable1StemFxAcapella` | 1 |  |
| `customProcessor.raneOneMk2Turntable1StemFxInstrumental` | [Inferred] custom Processor / rane One Mk 2 Turntable 1 Stem Fx Instrumental | `customProcessor.raneOneMk2Turntable1StemFxInstrumental` | 1 |  |
| `customProcessor.raneOneMk2Turntable2Acapella` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Acapella | `customProcessor.raneOneMk2Turntable2Acapella` | 1 |  |
| `customProcessor.raneOneMk2Turntable2Instrumental` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Instrumental | `customProcessor.raneOneMk2Turntable2Instrumental` | 1 |  |
| `customProcessor.raneOneMk2Turntable2MidiBounceLoopRouting.routingGenericChannel1` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Midi Bounce Loop Routing / routing Generic Channel 1 | `customProcessor.raneOneMk2Turntable2MidiBounceLoopRouting.routingGenericChannel1` | 1 |  |
| `customProcessor.raneOneMk2Turntable2MidiBounceLoopRouting.routingGenericChannel2` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Midi Bounce Loop Routing / routing Generic Channel 2 | `customProcessor.raneOneMk2Turntable2MidiBounceLoopRouting.routingGenericChannel2` | 1 |  |
| `customProcessor.raneOneMk2Turntable2MidiBounceLoopRouting.routingGenericChannel3` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Midi Bounce Loop Routing / routing Generic Channel 3 | `customProcessor.raneOneMk2Turntable2MidiBounceLoopRouting.routingGenericChannel3` | 1 |  |
| `customProcessor.raneOneMk2Turntable2MidiBounceLoopRouting.routingGenericChannel4` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Midi Bounce Loop Routing / routing Generic Channel 4 | `customProcessor.raneOneMk2Turntable2MidiBounceLoopRouting.routingGenericChannel4` | 1 |  |
| `customProcessor.raneOneMk2Turntable2PadFxBraker` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Pad Fx Braker | `customProcessor.raneOneMk2Turntable2PadFxBraker` | 1 |  |
| `customProcessor.raneOneMk2Turntable2PadFxDelay` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Pad Fx Delay | `customProcessor.raneOneMk2Turntable2PadFxDelay` | 1 |  |
| `customProcessor.raneOneMk2Turntable2PadFxEchoOut` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Pad Fx Echo Out | `customProcessor.raneOneMk2Turntable2PadFxEchoOut` | 1 |  |
| `customProcessor.raneOneMk2Turntable2PadFxReverbOut` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Pad Fx Reverb Out | `customProcessor.raneOneMk2Turntable2PadFxReverbOut` | 1 |  |
| `customProcessor.raneOneMk2Turntable2PadModeHotCue` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Pad Mode Hot Cue | `customProcessor.raneOneMk2Turntable2PadModeHotCue` | 1 |  |
| `customProcessor.raneOneMk2Turntable2PadModeRoll` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Pad Mode Roll | `customProcessor.raneOneMk2Turntable2PadModeRoll` | 1 |  |
| `customProcessor.raneOneMk2Turntable2PadModeSampler` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Pad Mode Sampler | `customProcessor.raneOneMk2Turntable2PadModeSampler` | 1 |  |
| `customProcessor.raneOneMk2Turntable2PadModeSavedLoop` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Pad Mode Saved Loop | `customProcessor.raneOneMk2Turntable2PadModeSavedLoop` | 1 |  |
| `customProcessor.raneOneMk2Turntable2PadModeStems` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Pad Mode Stems | `customProcessor.raneOneMk2Turntable2PadModeStems` | 1 |  |
| `customProcessor.raneOneMk2Turntable2SmallPadModeButton` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Small Pad Mode Button | `customProcessor.raneOneMk2Turntable2SmallPadModeButton` |  |  |
| `customProcessor.raneOneMk2Turntable2SmallPadModeHotCue` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Small Pad Mode Hot Cue | `customProcessor.raneOneMk2Turntable2SmallPadModeHotCue` | 1 |  |
| `customProcessor.raneOneMk2Turntable2SmallPadModeSampler` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Small Pad Mode Sampler | `customProcessor.raneOneMk2Turntable2SmallPadModeSampler` | 1 |  |
| `customProcessor.raneOneMk2Turntable2SmallPadModeSBanks` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Small Pad Mode S Banks | `customProcessor.raneOneMk2Turntable2SmallPadModeSBanks` | 1 |  |
| `customProcessor.raneOneMk2Turntable2SmallPadModeStems` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Small Pad Mode Stems | `customProcessor.raneOneMk2Turntable2SmallPadModeStems` | 1 |  |
| `customProcessor.raneOneMk2Turntable2SmallPadsBounceLoop1` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Small Pads Bounce Loop 1 | `customProcessor.raneOneMk2Turntable2SmallPadsBounceLoop1` | 1 |  |
| `customProcessor.raneOneMk2Turntable2SmallPadsBounceLoop1_2` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Small Pads Bounce Loop 1 2 | `customProcessor.raneOneMk2Turntable2SmallPadsBounceLoop1_2` | 1 |  |
| `customProcessor.raneOneMk2Turntable2SmallPadsBounceLoop1_4` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Small Pads Bounce Loop 1 4 | `customProcessor.raneOneMk2Turntable2SmallPadsBounceLoop1_4` | 1 |  |
| `customProcessor.raneOneMk2Turntable2SmallPadsBounceLoop1_8` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Small Pads Bounce Loop 1 8 | `customProcessor.raneOneMk2Turntable2SmallPadsBounceLoop1_8` | 1 |  |
| `customProcessor.raneOneMk2Turntable2StemFxAcapella` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Stem Fx Acapella | `customProcessor.raneOneMk2Turntable2StemFxAcapella` | 1 |  |
| `customProcessor.raneOneMk2Turntable2StemFxInstrumental` | [Inferred] custom Processor / rane One Mk 2 Turntable 2 Stem Fx Instrumental | `customProcessor.raneOneMk2Turntable2StemFxInstrumental` | 1 |  |
| `customProcessor.ranePerformerMotorActive1` | [Inferred] custom Processor / rane Performer Motor Active 1 | `customProcessor.ranePerformerMotorActive1` | 1 |  |
| `customProcessor.ranePerformerMotorActive2` | [Inferred] custom Processor / rane Performer Motor Active 2 | `customProcessor.ranePerformerMotorActive2` | 1 |  |
| `customProcessor.ranePerformerMotorActive3` | [Inferred] custom Processor / rane Performer Motor Active 3 | `customProcessor.ranePerformerMotorActive3` | 1 |  |
| `customProcessor.ranePerformerMotorActive4` | [Inferred] custom Processor / rane Performer Motor Active 4 | `customProcessor.ranePerformerMotorActive4` | 1 |  |
| `customProcessor.RaneSystemOneDeck1Acapella` | [Inferred] custom Processor / Rane System One Deck 1 Acapella | `customProcessor.RaneSystemOneDeck1Acapella` | 1 |  |
| `customProcessor.raneSystemOneDeck1Censor` | [Inferred] custom Processor / rane System One Deck 1 Censor | `customProcessor.raneSystemOneDeck1Censor` | 1 |  |
| `customProcessor.RaneSystemOneDeck1Instrumental` | [Inferred] custom Processor / Rane System One Deck 1 Instrumental | `customProcessor.RaneSystemOneDeck1Instrumental` | 1 |  |
| `customProcessor.raneSystemOneDeck1KeyAdjustDown` | [Inferred] custom Processor / rane System One Deck 1 Key Adjust Down | `customProcessor.raneSystemOneDeck1KeyAdjustDown` | 1 |  |
| `customProcessor.raneSystemOneDeck1KeyAdjustUp` | [Inferred] custom Processor / rane System One Deck 1 Key Adjust Up | `customProcessor.raneSystemOneDeck1KeyAdjustUp` | 1 |  |
| `customProcessor.raneSystemOneDeck1Motor45` | [Inferred] custom Processor / rane System One Deck 1 Motor 45 | `customProcessor.raneSystemOneDeck1Motor45` |  |  |
| `customProcessor.raneSystemOneDeck1MotorOn` | [Inferred] custom Processor / rane System One Deck 1 Motor On | `customProcessor.raneSystemOneDeck1MotorOn` | 1 |  |
| `customProcessor.raneSystemOneDeck1MotorReverse` | [Inferred] custom Processor / rane System One Deck 1 Motor Reverse | `customProcessor.raneSystemOneDeck1MotorReverse` | 1 |  |
| `customProcessor.raneSystemOneDeck1PadModeHotCue` | [Inferred] custom Processor / rane System One Deck 1 Pad Mode Hot Cue | `customProcessor.raneSystemOneDeck1PadModeHotCue` | 1 |  |
| `customProcessor.raneSystemOneDeck1PadModeRoll` | [Inferred] custom Processor / rane System One Deck 1 Pad Mode Roll | `customProcessor.raneSystemOneDeck1PadModeRoll` | 1 |  |
| `customProcessor.raneSystemOneDeck1PadModeSampler` | [Inferred] custom Processor / rane System One Deck 1 Pad Mode Sampler | `customProcessor.raneSystemOneDeck1PadModeSampler` | 1 |  |
| `customProcessor.raneSystemOneDeck1PadModeStems` | [Inferred] custom Processor / rane System One Deck 1 Pad Mode Stems | `customProcessor.raneSystemOneDeck1PadModeStems` | 1 |  |
| `customProcessor.raneSystemOneDeck1StartStop` | [Inferred] custom Processor / rane System One Deck 1 Start Stop | `customProcessor.raneSystemOneDeck1StartStop` | 1 |  |
| `customProcessor.RaneSystemOneDeck1StemFxAcapella` | [Inferred] custom Processor / Rane System One Deck 1 Stem Fx Acapella | `customProcessor.RaneSystemOneDeck1StemFxAcapella` | 1 |  |
| `customProcessor.RaneSystemOneDeck1StemFxInstrumental` | [Inferred] custom Processor / Rane System One Deck 1 Stem Fx Instrumental | `customProcessor.RaneSystemOneDeck1StemFxInstrumental` | 1 |  |
| `customProcessor.RaneSystemOneDeck1StemLevel` | [Inferred] custom Processor / Rane System One Deck 1 Stem Level | `customProcessor.RaneSystemOneDeck1StemLevel` | 1 |  |
| `customProcessor.RaneSystemOneDeck2Acapella` | [Inferred] custom Processor / Rane System One Deck 2 Acapella | `customProcessor.RaneSystemOneDeck2Acapella` | 1 |  |
| `customProcessor.raneSystemOneDeck2Censor` | [Inferred] custom Processor / rane System One Deck 2 Censor | `customProcessor.raneSystemOneDeck2Censor` | 1 |  |
| `customProcessor.RaneSystemOneDeck2Instrumental` | [Inferred] custom Processor / Rane System One Deck 2 Instrumental | `customProcessor.RaneSystemOneDeck2Instrumental` | 1 |  |
| `customProcessor.raneSystemOneDeck2KeyAdjustDown` | [Inferred] custom Processor / rane System One Deck 2 Key Adjust Down | `customProcessor.raneSystemOneDeck2KeyAdjustDown` | 1 |  |
| `customProcessor.raneSystemOneDeck2KeyAdjustUp` | [Inferred] custom Processor / rane System One Deck 2 Key Adjust Up | `customProcessor.raneSystemOneDeck2KeyAdjustUp` | 1 |  |
| `customProcessor.raneSystemOneDeck2Motor45` | [Inferred] custom Processor / rane System One Deck 2 Motor 45 | `customProcessor.raneSystemOneDeck2Motor45` |  |  |
| `customProcessor.raneSystemOneDeck2MotorOn` | [Inferred] custom Processor / rane System One Deck 2 Motor On | `customProcessor.raneSystemOneDeck2MotorOn` | 1 |  |
| `customProcessor.raneSystemOneDeck2MotorReverse` | [Inferred] custom Processor / rane System One Deck 2 Motor Reverse | `customProcessor.raneSystemOneDeck2MotorReverse` | 1 |  |
| `customProcessor.raneSystemOneDeck2PadModeHotCue` | [Inferred] custom Processor / rane System One Deck 2 Pad Mode Hot Cue | `customProcessor.raneSystemOneDeck2PadModeHotCue` | 1 |  |
| `customProcessor.raneSystemOneDeck2PadModeRoll` | [Inferred] custom Processor / rane System One Deck 2 Pad Mode Roll | `customProcessor.raneSystemOneDeck2PadModeRoll` | 1 |  |
| `customProcessor.raneSystemOneDeck2PadModeSampler` | [Inferred] custom Processor / rane System One Deck 2 Pad Mode Sampler | `customProcessor.raneSystemOneDeck2PadModeSampler` | 1 |  |
| `customProcessor.raneSystemOneDeck2PadModeStems` | [Inferred] custom Processor / rane System One Deck 2 Pad Mode Stems | `customProcessor.raneSystemOneDeck2PadModeStems` | 1 |  |
| `customProcessor.raneSystemOneDeck2StartStop` | [Inferred] custom Processor / rane System One Deck 2 Start Stop | `customProcessor.raneSystemOneDeck2StartStop` | 1 |  |
| `customProcessor.RaneSystemOneDeck2StemFxAcapella` | [Inferred] custom Processor / Rane System One Deck 2 Stem Fx Acapella | `customProcessor.RaneSystemOneDeck2StemFxAcapella` | 1 |  |
| `customProcessor.RaneSystemOneDeck2StemFxInstrumental` | [Inferred] custom Processor / Rane System One Deck 2 Stem Fx Instrumental | `customProcessor.RaneSystemOneDeck2StemFxInstrumental` | 1 |  |
| `customProcessor.RaneSystemOneDeck2StemLevel` | [Inferred] custom Processor / Rane System One Deck 2 Stem Level | `customProcessor.RaneSystemOneDeck2StemLevel` | 1 |  |
| `customProcessor.raneSystemOneFxPaddle1` | [Inferred] custom Processor / rane System One Fx Paddle 1 | `customProcessor.raneSystemOneFxPaddle1` | 1 |  |
| `customProcessor.raneSystemOneFxPaddle2` | [Inferred] custom Processor / rane System One Fx Paddle 2 | `customProcessor.raneSystemOneFxPaddle2` | 1 |  |
| `customProcessor.raneSystemOneFxParameter` | [Inferred] custom Processor / rane System One Fx Parameter | `customProcessor.raneSystemOneFxParameter` |  |  |
| `customProcessor.raneSystemOneFxParameterMinus` | [Inferred] custom Processor / rane System One Fx Parameter Minus | `customProcessor.raneSystemOneFxParameterMinus` |  |  |
| `customProcessor.raneSystemOneFxParameterPlus` | [Inferred] custom Processor / rane System One Fx Parameter Plus | `customProcessor.raneSystemOneFxParameterPlus` |  |  |
| `customProcessor.raneSystemOneFxSelection1` | [Inferred] custom Processor / rane System One Fx Selection 1 | `customProcessor.raneSystemOneFxSelection1` | 1 |  |
| `customProcessor.raneSystemOneFxSelection2` | [Inferred] custom Processor / rane System One Fx Selection 2 | `customProcessor.raneSystemOneFxSelection2` | 1 |  |
| `customProcessor.raneSystemOneFxSelection3` | [Inferred] custom Processor / rane System One Fx Selection 3 | `customProcessor.raneSystemOneFxSelection3` | 1 |  |
| `customProcessor.raneSystemOneFxSelection4` | [Inferred] custom Processor / rane System One Fx Selection 4 | `customProcessor.raneSystemOneFxSelection4` | 1 |  |
| `customProcessor.raneSystemOneFxSelection5` | [Inferred] custom Processor / rane System One Fx Selection 5 | `customProcessor.raneSystemOneFxSelection5` | 1 |  |
| `customProcessor.raneSystemOneFxSelection6` | [Inferred] custom Processor / rane System One Fx Selection 6 | `customProcessor.raneSystemOneFxSelection6` | 1 |  |
| `customProcessor.setTempoRange10` | [Inferred] custom Processor / set Tempo Range 10 | `customProcessor.setTempoRange10` |  |  |
| `customProcessor.setTempoRange16` | [Inferred] custom Processor / set Tempo Range 16 | `customProcessor.setTempoRange16` |  |  |
| `customProcessor.setTempoRange6` | [Inferred] custom Processor / set Tempo Range 6 | `customProcessor.setTempoRange6` |  |  |
| `customProcessor.setTempoRange75` | [Inferred] custom Processor / set Tempo Range 75 | `customProcessor.setTempoRange75` |  |  |
| `customProcessor.shift` | [Inferred] custom Processor / shift | `customProcessor.shift` |  |  |
| `customProcessor.stemsFxTurntable1Bass` | [Inferred] custom Processor / stems Fx Turntable 1 Bass | `customProcessor.stemsFxTurntable1Bass` | 1 |  |
| `customProcessor.stemsFxTurntable1Drums` | [Inferred] custom Processor / stems Fx Turntable 1 Drums | `customProcessor.stemsFxTurntable1Drums` | 1 |  |
| `customProcessor.stemsFxTurntable1HalfBeat` | [Inferred] custom Processor / stems Fx Turntable 1 Half Beat | `customProcessor.stemsFxTurntable1HalfBeat` | 1 |  |
| `customProcessor.stemsFxTurntable1Melody` | [Inferred] custom Processor / stems Fx Turntable 1 Melody | `customProcessor.stemsFxTurntable1Melody` | 1 |  |
| `customProcessor.stemsFxTurntable1TypeBraker` | [Inferred] custom Processor / stems Fx Turntable 1 Type Braker | `customProcessor.stemsFxTurntable1TypeBraker` | 1 |  |
| `customProcessor.stemsFxTurntable1TypeDelay` | [Inferred] custom Processor / stems Fx Turntable 1 Type Delay | `customProcessor.stemsFxTurntable1TypeDelay` | 1 |  |
| `customProcessor.stemsFxTurntable1TypeEchoOut` | [Inferred] custom Processor / stems Fx Turntable 1 Type Echo Out | `customProcessor.stemsFxTurntable1TypeEchoOut` | 1 |  |
| `customProcessor.stemsFxTurntable1TypeReverbOut` | [Inferred] custom Processor / stems Fx Turntable 1 Type Reverb Out | `customProcessor.stemsFxTurntable1TypeReverbOut` | 1 |  |
| `customProcessor.stemsFxTurntable1Vocal` | [Inferred] custom Processor / stems Fx Turntable 1 Vocal | `customProcessor.stemsFxTurntable1Vocal` | 1 |  |
| `customProcessor.stemsFxTurntable2Bass` | [Inferred] custom Processor / stems Fx Turntable 2 Bass | `customProcessor.stemsFxTurntable2Bass` | 1 |  |
| `customProcessor.stemsFxTurntable2Drums` | [Inferred] custom Processor / stems Fx Turntable 2 Drums | `customProcessor.stemsFxTurntable2Drums` | 1 |  |
| `customProcessor.stemsFxTurntable2HalfBeat` | [Inferred] custom Processor / stems Fx Turntable 2 Half Beat | `customProcessor.stemsFxTurntable2HalfBeat` | 1 |  |
| `customProcessor.stemsFxTurntable2Melody` | [Inferred] custom Processor / stems Fx Turntable 2 Melody | `customProcessor.stemsFxTurntable2Melody` | 1 |  |
| `customProcessor.stemsFxTurntable2TypeBraker` | [Inferred] custom Processor / stems Fx Turntable 2 Type Braker | `customProcessor.stemsFxTurntable2TypeBraker` | 1 |  |
| `customProcessor.stemsFxTurntable2TypeDelay` | [Inferred] custom Processor / stems Fx Turntable 2 Type Delay | `customProcessor.stemsFxTurntable2TypeDelay` | 1 |  |
| `customProcessor.stemsFxTurntable2TypeEchoOut` | [Inferred] custom Processor / stems Fx Turntable 2 Type Echo Out | `customProcessor.stemsFxTurntable2TypeEchoOut` | 1 |  |
| `customProcessor.stemsFxTurntable2TypeReverbOut` | [Inferred] custom Processor / stems Fx Turntable 2 Type Reverb Out | `customProcessor.stemsFxTurntable2TypeReverbOut` | 1 |  |
| `customProcessor.stemsFxTurntable2Vocal` | [Inferred] custom Processor / stems Fx Turntable 2 Vocal | `customProcessor.stemsFxTurntable2Vocal` | 1 |  |
| `customProcessor.stemsFxTurntable3Bass` | [Inferred] custom Processor / stems Fx Turntable 3 Bass | `customProcessor.stemsFxTurntable3Bass` | 1 |  |
| `customProcessor.stemsFxTurntable3Drums` | [Inferred] custom Processor / stems Fx Turntable 3 Drums | `customProcessor.stemsFxTurntable3Drums` | 1 |  |
| `customProcessor.stemsFxTurntable3HalfBeat` | [Inferred] custom Processor / stems Fx Turntable 3 Half Beat | `customProcessor.stemsFxTurntable3HalfBeat` | 1 |  |
| `customProcessor.stemsFxTurntable3Melody` | [Inferred] custom Processor / stems Fx Turntable 3 Melody | `customProcessor.stemsFxTurntable3Melody` | 1 |  |
| `customProcessor.stemsFxTurntable3TypeBraker` | [Inferred] custom Processor / stems Fx Turntable 3 Type Braker | `customProcessor.stemsFxTurntable3TypeBraker` | 1 |  |
| `customProcessor.stemsFxTurntable3TypeDelay` | [Inferred] custom Processor / stems Fx Turntable 3 Type Delay | `customProcessor.stemsFxTurntable3TypeDelay` | 1 |  |
| `customProcessor.stemsFxTurntable3TypeEchoOut` | [Inferred] custom Processor / stems Fx Turntable 3 Type Echo Out | `customProcessor.stemsFxTurntable3TypeEchoOut` | 1 |  |
| `customProcessor.stemsFxTurntable3TypeReverbOut` | [Inferred] custom Processor / stems Fx Turntable 3 Type Reverb Out | `customProcessor.stemsFxTurntable3TypeReverbOut` | 1 |  |
| `customProcessor.stemsFxTurntable3Vocal` | [Inferred] custom Processor / stems Fx Turntable 3 Vocal | `customProcessor.stemsFxTurntable3Vocal` | 1 |  |
| `customProcessor.stemsFxTurntable4Bass` | [Inferred] custom Processor / stems Fx Turntable 4 Bass | `customProcessor.stemsFxTurntable4Bass` | 1 |  |
| `customProcessor.stemsFxTurntable4Drums` | [Inferred] custom Processor / stems Fx Turntable 4 Drums | `customProcessor.stemsFxTurntable4Drums` | 1 |  |
| `customProcessor.stemsFxTurntable4HalfBeat` | [Inferred] custom Processor / stems Fx Turntable 4 Half Beat | `customProcessor.stemsFxTurntable4HalfBeat` | 1 |  |
| `customProcessor.stemsFxTurntable4Melody` | [Inferred] custom Processor / stems Fx Turntable 4 Melody | `customProcessor.stemsFxTurntable4Melody` | 1 |  |
| `customProcessor.stemsFxTurntable4TypeBraker` | [Inferred] custom Processor / stems Fx Turntable 4 Type Braker | `customProcessor.stemsFxTurntable4TypeBraker` | 1 |  |
| `customProcessor.stemsFxTurntable4TypeDelay` | [Inferred] custom Processor / stems Fx Turntable 4 Type Delay | `customProcessor.stemsFxTurntable4TypeDelay` | 1 |  |
| `customProcessor.stemsFxTurntable4TypeEchoOut` | [Inferred] custom Processor / stems Fx Turntable 4 Type Echo Out | `customProcessor.stemsFxTurntable4TypeEchoOut` | 1 |  |
| `customProcessor.stemsFxTurntable4TypeReverbOut` | [Inferred] custom Processor / stems Fx Turntable 4 Type Reverb Out | `customProcessor.stemsFxTurntable4TypeReverbOut` | 1 |  |
| `customProcessor.stemsFxTurntable4Vocal` | [Inferred] custom Processor / stems Fx Turntable 4 Vocal | `customProcessor.stemsFxTurntable4Vocal` | 1 |  |
| `customProcessor.tempoSliderRangeNext` | [Inferred] custom Processor / tempo Slider Range Next | `customProcessor.tempoSliderRangeNext` | 4 |  |
| `customProcessor.xdjAnBeatFXAutoBPM` | [Inferred] custom Processor / xdj An Beat FX Auto BPM | `customProcessor.xdjAnBeatFXAutoBPM` |  |  |
| `customProcessor.xdjAnBeatFXDepth` | [Inferred] custom Processor / xdj An Beat FX Depth | `customProcessor.xdjAnBeatFXDepth` |  |  |
| `customProcessor.xdjAnBeatFXDestination1` | [Inferred] custom Processor / xdj An Beat FX Destination 1 | `customProcessor.xdjAnBeatFXDestination1` |  |  |
| `customProcessor.xdjAnBeatFXDestination2` | [Inferred] custom Processor / xdj An Beat FX Destination 2 | `customProcessor.xdjAnBeatFXDestination2` |  |  |
| `customProcessor.xdjAnBeatFXDestinationMain` | [Inferred] custom Processor / xdj An Beat FX Destination Main | `customProcessor.xdjAnBeatFXDestinationMain` |  |  |
| `customProcessor.xdjAnBeatFXEffectEcho` | [Inferred] custom Processor / xdj An Beat FX Effect Echo | `customProcessor.xdjAnBeatFXEffectEcho` |  |  |
| `customProcessor.xdjAnBeatFXEffectFlanger` | [Inferred] custom Processor / xdj An Beat FX Effect Flanger | `customProcessor.xdjAnBeatFXEffectFlanger` |  |  |
| `customProcessor.xdjAnBeatFXEffectReverb` | [Inferred] custom Processor / xdj An Beat FX Effect Reverb | `customProcessor.xdjAnBeatFXEffectReverb` |  |  |
| `customProcessor.xdjAnBeatFXParameterMinus` | [Inferred] custom Processor / xdj An Beat FX Parameter Minus | `customProcessor.xdjAnBeatFXParameterMinus` |  |  |
| `customProcessor.xdjAnBeatFXParameterPlus` | [Inferred] custom Processor / xdj An Beat FX Parameter Plus | `customProcessor.xdjAnBeatFXParameterPlus` |  |  |
| `customProcessor.xdjAnBeatFXTap` | [Inferred] custom Processor / xdj An Beat FX Tap | `customProcessor.xdjAnBeatFXTap` |  |  |
| `customProcessor.xdjAnBeatFXToggle` | [Inferred] custom Processor / xdj An Beat FX Toggle | `customProcessor.xdjAnBeatFXToggle` | 1 |  |
| `customProcessor.xdjAnMasterPreCue` | [Inferred] custom Processor / xdj An Master Pre Cue | `customProcessor.xdjAnMasterPreCue` | 1 |  |
| `looper` | Looper | `looper` | 144 |  |
| `microphone` | Microphone | `microphone` |  |  |
| `microphone.echoDuration` | Echo Duration | `microphone.echoDuration` |  |  |
| `microphone.level` | Level | `microphone.level` |  |  |
| `microphone.pitch` | Pitch | `microphone.pitch` |  |  |
| `mixer` | Mixer | `mixer` | 18 |  |
| `mixer.crossfadeAssignment1Left` | Deck 1: Assign Left | `mixer.crossfadeAssignment1Left` |  | Note CH5 data 82 |
| `mixer.crossfadeAssignment1Right` | Deck 1: Assign Right | `mixer.crossfadeAssignment1Right` |  | Note CH5 data 80 |
| `mixer.crossfadeAssignment1Switch` | Deck 1: Assignment Switch | `mixer.crossfadeAssignment1Switch` |  |  |
| `mixer.crossfadeAssignment1Through` | Deck 1: Assign Through | `mixer.crossfadeAssignment1Through` |  | Note CH5 data 81 |
| `mixer.crossfadeAssignment2Left` | Deck 2: Assign Left | `mixer.crossfadeAssignment2Left` |  | Note CH5 data 85 |
| `mixer.crossfadeAssignment2Right` | Deck 2: Assign Right | `mixer.crossfadeAssignment2Right` |  | Note CH5 data 83 |
| `mixer.crossfadeAssignment2Switch` | Deck 2: Assignment Switch | `mixer.crossfadeAssignment2Switch` |  |  |
| `mixer.crossfadeAssignment2Through` | Deck 2: Assign Through | `mixer.crossfadeAssignment2Through` |  | Note CH5 data 84 |
| `mixer.crossfadeAssignment3Left` | Deck 3: Assign Left | `mixer.crossfadeAssignment3Left` |  | Note CH5 data 88 |
| `mixer.crossfadeAssignment3Right` | Deck 3: Assign Right | `mixer.crossfadeAssignment3Right` |  | Note CH5 data 86 |
| `mixer.crossfadeAssignment3Switch` | Deck 3: Assignment Switch | `mixer.crossfadeAssignment3Switch` |  |  |
| `mixer.crossfadeAssignment3Through` | Deck 3: Assign Through | `mixer.crossfadeAssignment3Through` |  | Note CH5 data 87 |
| `mixer.crossfadeAssignment4Left` | Deck 4: Assign Left | `mixer.crossfadeAssignment4Left` |  | Note CH5 data 91 |
| `mixer.crossfadeAssignment4Right` | Deck 4: Assign Right | `mixer.crossfadeAssignment4Right` |  | Note CH5 data 89 |
| `mixer.crossfadeAssignment4Switch` | Deck 4: Assignment Switch | `mixer.crossfadeAssignment4Switch` |  |  |
| `mixer.crossfadeAssignment4Through` | Deck 4: Assign Through | `mixer.crossfadeAssignment4Through` |  | Note CH5 data 90 |
| `mixer.crossfadeFXNext` | Crossfader FX Select Next | `mixer.crossfadeFXNext` | 2 |  |
| `mixer.crossfadeFXPrevious` | Crossfader FX Select Previous | `mixer.crossfadeFXPrevious` |  |  |
| `mixer.crossfadeFXTransitionDurationDown` | Transition Duration - | `mixer.crossfadeFXTransitionDurationDown` |  |  |
| `mixer.crossfadeFXTransitionDurationUp` | Transition Duration + | `mixer.crossfadeFXTransitionDurationUp` |  |  |
| `mixer.crossfadeStyle` | Crossfader Curve | `mixer.crossfadeStyle` |  | CC CH5 data 25 |
| `mixer.externalLineVolume1` | Deck 1: External Line Volume | `mixer.externalLineVolume1` |  |  |
| `mixer.externalLineVolume2` | Deck 2: External Line Volume | `mixer.externalLineVolume2` |  |  |
| `mixer.externalLineVolume3` | Deck 3: External Line Volume | `mixer.externalLineVolume3` |  |  |
| `mixer.externalLineVolume4` | Deck 4: External Line Volume | `mixer.externalLineVolume4` |  |  |
| `mixer.lineVolume1Kill` | Deck 1: Kill Line Volume | `mixer.lineVolume1Kill` | 1 |  |
| `mixer.lineVolume2Kill` | Deck 2: Kill Line Volume | `mixer.lineVolume2Kill` | 1 |  |
| `mixer.monitorActive1` | Deck 1: Monitor Active | `mixer.monitorActive1` | 121 | Note CH5 data 0 |
| `mixer.monitorActive2` | Deck 2: Monitor Active | `mixer.monitorActive2` | 120 | Note CH5 data 1 |
| `mixer.monitorActive3` | Deck 3: Monitor Active | `mixer.monitorActive3` | 44 | Note CH5 data 2 |
| `mixer.monitorActive4` | Deck 4: Monitor Active | `mixer.monitorActive4` | 43 | Note CH5 data 3 |
| `mixer.monitorLevelMinus` | Monitor Volume - | `mixer.monitorLevelMinus` | 1 |  |
| `mixer.monitorLevelPlus` | Monitor Volume + | `mixer.monitorLevelPlus` | 1 |  |
| `mixer.videoTransitionTypeNext` | Next Video Transition | `mixer.videoTransitionTypeNext` |  |  |
| `musicLibrary` | Music Library | `musicLibrary` | 25 |  |
| `musicLibrary.deckMove1To4` | Deck Move 1 to 4 | `musicLibrary.deckMove1To4` |  |  |
| `musicLibrary.deckMove2To3` | Deck Move 2 to 3 | `musicLibrary.deckMove2To3` |  |  |
| `musicLibrary.focusQueue` | Toggle Library Panel | `musicLibrary.focusQueue` | 15 | Note CH1 data 10; Note CH2 data 10; Note CH3 data 10; Note CH4 data 10 |
| `musicLibrary.focusSources` | Focus Playlists | `musicLibrary.focusSources` | 12 |  |
| `musicLibrary.focusTracks` | Focus Tracks | `musicLibrary.focusTracks` | 10 |  |
| `musicLibrary.libraryBack` | Switch Back | `musicLibrary.libraryBack` | 18 | Note CH10 data 12; Note CH11 data 12; Note CH12 data 12; Note CH9 data 12 |
| `musicLibrary.libraryDown` | Select Next Track | `musicLibrary.libraryDown` | 3 |  |
| `musicLibrary.librarySlider` | Move Library Selection (Slider) | `musicLibrary.librarySlider` |  |  |
| `musicLibrary.libraryUp` | Select Previous Track | `musicLibrary.libraryUp` | 4 |  |
| `musicLibrary.load1` | Load Track on Deck 1 | `musicLibrary.load1` | 26 | Note CH1 data 12 |
| `musicLibrary.load1NoInstantDouble` | Load Track on Deck 1 (No Instant Double) | `musicLibrary.load1NoInstantDouble` |  |  |
| `musicLibrary.load2` | Load Track on Deck 2 | `musicLibrary.load2` | 26 | Note CH2 data 12 |
| `musicLibrary.load2NoInstantDouble` | Load Track on Deck 2 (No Instant Double) | `musicLibrary.load2NoInstantDouble` |  |  |
| `musicLibrary.load3` | Load Track on Deck 3 | `musicLibrary.load3` | 8 | Note CH3 data 12 |
| `musicLibrary.load4` | Load Track on Deck 4 | `musicLibrary.load4` | 8 | Note CH4 data 12 |
| `musicLibrary.loadSelection` | Load Selected Track | `musicLibrary.loadSelection` | 2 |  |
| `musicLibrary.markSelectedSongs` | Add Selected Tracks to Queue | `musicLibrary.markSelectedSongs` | 8 |  |
| `musicLibrary.markUnmarkSelectedSongs` | Add/Remove Selected Tracks | `musicLibrary.markUnmarkSelectedSongs` | 19 | Note CH1 data 11; Note CH2 data 11; Note CH3 data 11; Note CH4 data 11 |
| `musicLibrary.nextSection` | Select Next Section | `musicLibrary.nextSection` | 6 |  |
| `musicLibrary.nextSource` | Select Next Source | `musicLibrary.nextSource` | 4 |  |
| `musicLibrary.previousSection` | Select Previous Section | `musicLibrary.previousSection` | 4 |  |
| `musicLibrary.skipForwardPreview` | Skip Forward Preview | `musicLibrary.skipForwardPreview` |  |  |
| `musicLibrary.toggleLibrarySource` | Switch Library Table | `musicLibrary.toggleLibrarySource` | 13 |  |
| `musicLibrary.toggleLibraryVisible` | Toggle Library Expanded | `musicLibrary.toggleLibraryVisible` | 19 | Note CH1 data 9; Note CH2 data 9; Note CH3 data 9; Note CH4 data 9 |
| `musicLibrary.togglePreview` | Preview Selected Track | `musicLibrary.togglePreview` | 5 | Note CH1 data 13; Note CH2 data 13; Note CH3 data 13; Note CH4 data 13 |
| `musicLibrary.toggleSplitLayout` | Toggle Split Library | `musicLibrary.toggleSplitLayout` |  |  |
| `musicLibrary.unmarkSelectedSongs` | Remove Selected Tracks from Queue | `musicLibrary.unmarkSelectedSongs` | 6 |  |
| `sampler` | Sampler | `sampler` | 162 |  |
| `sampler.playing1` | [Inferred] sampler / playing 1 | `sampler.playing1` | 3 |  |
| `sampler.playing2` | [Inferred] sampler / playing 2 | `sampler.playing2` | 1 |  |
| `sampler.recording1` | [Inferred] sampler / recording 1 | `sampler.recording1` | 3 |  |
| `sampler.recording2` | [Inferred] sampler / recording 2 | `sampler.recording2` | 3 |  |
| `turntable` | Deck | `turntable1`, `turntable2`, `turntable3`, `turntable4`, `turntableSelected` | 5473 |  |
| `turntable.autoLoopDurationDouble` | Loop Double | `turntable1.autoLoopDurationDouble`, `turntable2.autoLoopDurationDouble`, `turntable3.autoLoopDurationDouble`, `turntable4.autoLoopDurationDouble`, `turntableSelected.autoLoopDurationDouble` | 242 |  |
| `turntable.autoLoopDurationHalf` | Loop Half | `turntable1.autoLoopDurationHalf`, `turntable2.autoLoopDurationHalf`, `turntable3.autoLoopDurationHalf`, `turntable4.autoLoopDurationHalf`, `turntableSelected.autoLoopDurationHalf` | 250 |  |
| `turntable.beatShiftLeft` | Shift Beat Grid Left | `turntable1.beatShiftLeft`, `turntable2.beatShiftLeft`, `turntable3.beatShiftLeft`, `turntable4.beatShiftLeft` | 4 |  |
| `turntable.beatShiftRight` | Shift Beat Grid Right | `turntable1.beatShiftRight`, `turntable2.beatShiftRight`, `turntable3.beatShiftRight`, `turntable4.beatShiftRight` | 4 |  |
| `turntable.bpmAndDownBeatRestoreAnalyzed` | Restore Analyzed BPM and Grid | `turntable1.bpmAndDownBeatRestoreAnalyzed`, `turntable2.bpmAndDownBeatRestoreAnalyzed`, `turntable3.bpmAndDownBeatRestoreAnalyzed`, `turntable4.bpmAndDownBeatRestoreAnalyzed` | 4 |  |
| `turntable.bpmTap` | BPM Tap | `turntable1.bpmTap`, `turntable2.bpmTap`, `turntable3.bpmTap`, `turntable4.bpmTap` | 38 |  |
| `turntable.cancelLoopIn` | Cancel Loop In | `turntable1.cancelLoopIn`, `turntable2.cancelLoopIn`, `turntable4.cancelLoopIn` | 4 |  |
| `turntable.clearOutPoint1` | [Inferred] turntable / clear Out Point 1 | `turntable1.clearOutPoint1`, `turntable2.clearOutPoint1` |  |  |
| `turntable.clearOutPoint2` | [Inferred] turntable / clear Out Point 2 | `turntable1.clearOutPoint2`, `turntable2.clearOutPoint2` |  |  |
| `turntable.clearOutPoint3` | [Inferred] turntable / clear Out Point 3 | `turntable1.clearOutPoint3`, `turntable2.clearOutPoint3` |  |  |
| `turntable.clearSavedLoop1` | Clear Saved Loop 1 | `turntable1.clearSavedLoop1`, `turntable2.clearSavedLoop1` | 4 |  |
| `turntable.clearSavedLoop2` | Clear Saved Loop 2 | `turntable1.clearSavedLoop2`, `turntable2.clearSavedLoop2` | 4 |  |
| `turntable.clearSavedLoop3` | Clear Saved Loop 3 | `turntable1.clearSavedLoop3`, `turntable2.clearSavedLoop3` | 4 |  |
| `turntable.clearSavedLoop4` | Clear Saved Loop 4 | `turntable1.clearSavedLoop4`, `turntable2.clearSavedLoop4` | 4 |  |
| `turntable.clearSavedLoop5` | Clear Saved Loop 5 | `turntable1.clearSavedLoop5`, `turntable2.clearSavedLoop5` | 2 |  |
| `turntable.clearSavedLoop6` | Clear Saved Loop 6 | `turntable1.clearSavedLoop6`, `turntable2.clearSavedLoop6` | 2 |  |
| `turntable.clearSavedLoop7` | Clear Saved Loop 7 | `turntable1.clearSavedLoop7`, `turntable2.clearSavedLoop7` | 2 |  |
| `turntable.clearSavedLoop8` | Clear Saved Loop 8 | `turntable1.clearSavedLoop8`, `turntable2.clearSavedLoop8` | 2 |  |
| `turntable.cueOutLoop1` | [Inferred] turntable / cue Out Loop 1 | `turntable1.cueOutLoop1`, `turntable2.cueOutLoop1` | 4 |  |
| `turntable.cueOutLoop2` | [Inferred] turntable / cue Out Loop 2 | `turntable1.cueOutLoop2`, `turntable2.cueOutLoop2` | 4 |  |
| `turntable.cueOutLoop3` | [Inferred] turntable / cue Out Loop 3 | `turntable1.cueOutLoop3`, `turntable2.cueOutLoop3` | 2 |  |
| `turntable.cuePlay1` | [Inferred] turntable / cue Play 1 | `turntable1.cuePlay1`, `turntable2.cuePlay1` | 2 |  |
| `turntable.cuePointPageDown` | Cue Point Range 1-8 | `turntable1.cuePointPageDown`, `turntable2.cuePointPageDown`, `turntable3.cuePointPageDown`, `turntable4.cuePointPageDown` | 16 |  |
| `turntable.cuePointPageUp` | Cue Point Range 9-16 | `turntable1.cuePointPageUp`, `turntable2.cuePointPageUp`, `turntable3.cuePointPageUp`, `turntable4.cuePointPageUp` | 16 |  |
| `turntable.cuePosition1` | Set Cue Point 1 | `turntable1.cuePosition1`, `turntable2.cuePosition1`, `turntableSelected.cuePosition1` | 6 |  |
| `turntable.cuePosition2` | Set Cue Point 2 | `turntable1.cuePosition2`, `turntable2.cuePosition2`, `turntableSelected.cuePosition2` | 6 |  |
| `turntable.cuePosition3` | Set Cue Point 3 | `turntable1.cuePosition3`, `turntable2.cuePosition3`, `turntableSelected.cuePosition3` | 6 |  |
| `turntable.echoTransitionEffect` | Echo Transition FX | `turntable1.echoTransitionEffect`, `turntable2.echoTransitionEffect`, `turntable3.echoTransitionEffect`, `turntable4.echoTransitionEffect` | 4 |  |
| `turntable.fx1ParameterValueMinus` | FX 1 Parameter - | `turntable1.fx1ParameterValueMinus`, `turntable2.fx1ParameterValueMinus`, `turntable3.fx1ParameterValueMinus`, `turntable4.fx1ParameterValueMinus` | 4 |  |
| `turntable.fx1ParameterValuePlus` | FX 1 Parameter + | `turntable1.fx1ParameterValuePlus`, `turntable2.fx1ParameterValuePlus`, `turntable3.fx1ParameterValuePlus`, `turntable4.fx1ParameterValuePlus` | 4 |  |
| `turntable.fx1SelectNext` | FX 1 Select Next | `turntable1.fx1SelectNext`, `turntable2.fx1SelectNext`, `turntable3.fx1SelectNext`, `turntable4.fx1SelectNext`, `turntableSelected.fx1SelectNext` | 95 | Note CH10 data 33; Note CH10 data 37; Note CH11 data 33; Note CH11 data 37; Note CH12 data 33; Note CH12 data 37; Note CH9 data 33; Note CH9 data 37 |
| `turntable.fx1SelectPrev` | FX 1 Select Previous | `turntable1.fx1SelectPrev`, `turntable2.fx1SelectPrev`, `turntable3.fx1SelectPrev`, `turntable4.fx1SelectPrev`, `turntableSelected.fx1SelectPrev` | 14 |  |
| `turntable.fx2ParameterValueMinus` | FX 2 Parameter - | `turntable1.fx2ParameterValueMinus`, `turntable2.fx2ParameterValueMinus`, `turntable3.fx2ParameterValueMinus`, `turntable4.fx2ParameterValueMinus` |  |  |
| `turntable.fx2ParameterValuePlus` | FX 2 Parameter + | `turntable1.fx2ParameterValuePlus`, `turntable2.fx2ParameterValuePlus`, `turntable3.fx2ParameterValuePlus`, `turntable4.fx2ParameterValuePlus` |  |  |
| `turntable.fx2SelectNext` | FX 2 Select Next | `turntable1.fx2SelectNext`, `turntable2.fx2SelectNext`, `turntable3.fx2SelectNext`, `turntable4.fx2SelectNext`, `turntableSelected.fx2SelectNext` | 71 | Note CH10 data 34; Note CH10 data 38; Note CH11 data 34; Note CH11 data 38; Note CH12 data 34; Note CH12 data 38; Note CH9 data 34; Note CH9 data 38 |
| `turntable.fx2SelectPrev` | FX 2 Select Previous | `turntable1.fx2SelectPrev`, `turntable2.fx2SelectPrev`, `turntable3.fx2SelectPrev`, `turntable4.fx2SelectPrev`, `turntableSelected.fx2SelectPrev` | 2 |  |
| `turntable.fx3ParameterValueMinus` | FX 3 Parameter - | `turntable1.fx3ParameterValueMinus`, `turntable2.fx3ParameterValueMinus`, `turntable3.fx3ParameterValueMinus`, `turntable4.fx3ParameterValueMinus` |  |  |
| `turntable.fx3ParameterValuePlus` | FX 3 Parameter + | `turntable1.fx3ParameterValuePlus`, `turntable2.fx3ParameterValuePlus`, `turntable3.fx3ParameterValuePlus`, `turntable4.fx3ParameterValuePlus` |  |  |
| `turntable.fx3SelectNext` | FX 3 Select Next | `turntable1.fx3SelectNext`, `turntable2.fx3SelectNext`, `turntable3.fx3SelectNext`, `turntable4.fx3SelectNext`, `turntableSelected.fx3SelectNext` | 69 | Note CH10 data 35; Note CH10 data 39; Note CH11 data 35; Note CH11 data 39; Note CH12 data 35; Note CH12 data 39; Note CH9 data 35; Note CH9 data 39 |
| `turntable.fx3SelectPrev` | FX 3 Select Previous | `turntable1.fx3SelectPrev`, `turntable2.fx3SelectPrev`, `turntable3.fx3SelectPrev`, `turntable4.fx3SelectPrev`, `turntableSelected.fx3SelectPrev` |  |  |
| `turntable.fxBounceBitCrusher` | Crush FX | `turntable1.fxBounceBitCrusher`, `turntable2.fxBounceBitCrusher`, `turntable3.fxBounceBitCrusher`, `turntable4.fxBounceBitCrusher`, `turntableSelected.fxBounceBitCrusher` | 25 |  |
| `turntable.fxBounceEchoCensor` | Twist FX | `turntable1.fxBounceEchoCensor`, `turntable2.fxBounceEchoCensor`, `turntable3.fxBounceEchoCensor`, `turntable4.fxBounceEchoCensor` | 8 |  |
| `turntable.fxBounceEchoExtreme` | Punch FX | `turntable1.fxBounceEchoExtreme`, `turntable2.fxBounceEchoExtreme`, `turntable3.fxBounceEchoExtreme`, `turntable4.fxBounceEchoExtreme` | 6 |  |
| `turntable.fxBounceFlanger` | Sway FX | `turntable1.fxBounceFlanger`, `turntable2.fxBounceFlanger`, `turntable3.fxBounceFlanger`, `turntable4.fxBounceFlanger`, `turntableSelected.fxBounceFlanger` | 29 |  |
| `turntable.fxBounceGate` | Gate FX | `turntable1.fxBounceGate`, `turntable2.fxBounceGate`, `turntable3.fxBounceGate`, `turntable4.fxBounceGate` | 4 |  |
| `turntable.fxBounceHighPass` | High-Pass | `turntable1.fxBounceHighPass`, `turntable2.fxBounceHighPass`, `turntable3.fxBounceHighPass`, `turntable4.fxBounceHighPass` | 19 |  |
| `turntable.fxBounceHighPassEcho` | Drift FX | `turntable1.fxBounceHighPassEcho`, `turntable2.fxBounceHighPassEcho`, `turntable3.fxBounceHighPassEcho`, `turntable4.fxBounceHighPassEcho`, `turntableSelected.fxBounceHighPassEcho` | 49 |  |
| `turntable.fxBounceLowPass` | Low-Pass | `turntable1.fxBounceLowPass`, `turntable2.fxBounceLowPass`, `turntable3.fxBounceLowPass`, `turntable4.fxBounceLowPass` | 15 |  |
| `turntable.fxBounceLowPassEcho` | Absorb FX | `turntable1.fxBounceLowPassEcho`, `turntable2.fxBounceLowPassEcho`, `turntable3.fxBounceLowPassEcho`, `turntable4.fxBounceLowPassEcho` | 52 |  |
| `turntable.fxBpmRotary` | FX BPM | `turntable1.fxBpmRotary`, `turntable2.fxBpmRotary` |  |  |
| `turntable.jogSeekMove` | Jog Seek | `turntable1.jogSeekMove`, `turntable2.jogSeekMove`, `turntable3.jogSeekMove`, `turntable4.jogSeekMove`, `turntableSelected.jogSeekMove` |  | CC CH1 data 6 (rotary-64); sensitivity 20; CC CH2 data 6 (rotary-64); sensitivity 20; CC CH3 data 6 (rotary-64); sensitivity 20; CC CH4 data 6 (rotary-64); sensitivity 20 |
| `turntable.jumpToCueAndPause1` | Jump to Cue Point 1 and Pause | `turntable1.jumpToCueAndPause1`, `turntable2.jumpToCueAndPause1` | 2 |  |
| `turntable.jumpToCueAndPlay1` | Jump to Cue Point 1 and Play | `turntable1.jumpToCueAndPlay1`, `turntable2.jumpToCueAndPlay1`, `turntable3.jumpToCueAndPlay1`, `turntable4.jumpToCueAndPlay1` |  |  |
| `turntable.jumpToLastJumpedToCuePoint` | Jump to Last Cue Point | `turntable1.jumpToLastJumpedToCuePoint`, `turntable2.jumpToLastJumpedToCuePoint` |  |  |
| `turntable.jumpToLastJumpedToCuePointAndPause` | Jump to Most Recent Cue Point and Pause | `turntable1.jumpToLastJumpedToCuePointAndPause`, `turntable2.jumpToLastJumpedToCuePointAndPause`, `turntable3.jumpToLastJumpedToCuePointAndPause`, `turntable4.jumpToLastJumpedToCuePointAndPause` |  |  |
| `turntable.jumpToLastJumpedToCuePointAndPlay` | Jump to Most Recent Cue Point and Play | `turntable1.jumpToLastJumpedToCuePointAndPlay`, `turntable2.jumpToLastJumpedToCuePointAndPlay`, `turntable3.jumpToLastJumpedToCuePointAndPlay`, `turntable4.jumpToLastJumpedToCuePointAndPlay` |  |  |
| `turntable.loadDouble` | Instant Double | `turntable1.loadDouble`, `turntable2.loadDouble`, `turntable3.loadDouble`, `turntable4.loadDouble` |  |  |
| `turntable.loadNextTrack` | Load Next Track | `turntable1.loadNextTrack`, `turntable2.loadNextTrack`, `turntable3.loadNextTrack`, `turntable4.loadNextTrack`, `turntableSelected.loadNextTrack` | 12 |  |
| `turntable.loadPreviousTrack` | Load Previous Track | `turntable1.loadPreviousTrack`, `turntable2.loadPreviousTrack`, `turntable3.loadPreviousTrack`, `turntable4.loadPreviousTrack`, `turntableSelected.loadPreviousTrack` | 20 |  |
| `turntable.loadTrack` | Load Selected Track | `turntable1.loadTrack`, `turntable2.loadTrack`, `turntable3.loadTrack`, `turntable4.loadTrack`, `turntableSelected.loadTrack` | 16 |  |
| `turntable.loopOutPointMoveRotary` | Move Loop Out Point (Rotary) | `turntable1.loopOutPointMoveRotary`, `turntable2.loopOutPointMoveRotary`, `turntable3.loopOutPointMoveRotary`, `turntable4.loopOutPointMoveRotary` |  |  |
| `turntable.pitchBendMove` | Pitch Bend | `turntable1.pitchBendMove`, `turntable2.pitchBendMove`, `turntable3.pitchBendMove`, `turntable4.pitchBendMove`, `turntableSelected.pitchBendMove` | 9 | CC CH1 data 4 (rotary-64); sensitivity 7; CC CH2 data 4 (rotary-64); sensitivity 7; CC CH3 data 4 (rotary-64); sensitivity 7; CC CH4 data 4 (rotary-64); sensitivity 7 |
| `turntable.pitchMinus` | Key - | `turntable1.pitchMinus`, `turntable2.pitchMinus`, `turntable3.pitchMinus`, `turntable4.pitchMinus`, `turntableSelected.pitchMinus` | 74 |  |
| `turntable.pitchPlus` | Key + | `turntable1.pitchPlus`, `turntable2.pitchPlus`, `turntable3.pitchPlus`, `turntable4.pitchPlus`, `turntableSelected.pitchPlus` | 74 |  |
| `turntable.recordSample` | Record Sample | `turntable1.recordSample`, `turntable2.recordSample`, `turntable3.recordSample`, `turntable4.recordSample` |  | Note CH1 data 22; Note CH2 data 22; Note CH3 data 22; Note CH4 data 22 |
| `turntable.resetFxBpm` | Reset FX BPM | `turntable1.resetFxBpm`, `turntable2.resetFxBpm`, `turntable3.resetFxBpm`, `turntable4.resetFxBpm` |  |  |
| `turntable.reverb` | [Inferred] turntable / reverb | `turntable1.reverb`, `turntable2.reverb` | 2 |  |
| `turntable.scratchingMode` | Scratching Mode | `turntable1.scratchingMode`, `turntable2.scratchingMode`, `turntable3.scratchingMode`, `turntable4.scratchingMode`, `turntableSelected.scratchingMode` | 8 | Note CH1 data 7; Note CH2 data 7; Note CH3 data 7; Note CH4 data 7 |
| `turntable.scratchingMove` | Scratch | `turntable1.scratchingMove`, `turntable2.scratchingMove`, `turntable3.scratchingMove`, `turntable4.scratchingMove`, `turntableSelected.scratchingMove` | 8 | CC CH1 data 5 (rotary-64); sensitivity 25; CC CH2 data 5 (rotary-64); sensitivity 25; CC CH3 data 5 (rotary-64); sensitivity 25; CC CH4 data 5 (rotary-64); sensitivity 25 |
| `turntable.scratchingNoTouchMove` | Scratch (no touch detection) | `turntable1.scratchingNoTouchMove`, `turntable2.scratchingNoTouchMove` |  |  |
| `turntable.seekBackward` | Seek Backward | `turntable1.seekBackward`, `turntable2.seekBackward`, `turntable3.seekBackward`, `turntable4.seekBackward` | 3 |  |
| `turntable.seekForward` | Seek Forward | `turntable1.seekForward`, `turntable2.seekForward`, `turntable3.seekForward`, `turntable4.seekForward` | 4 |  |
| `turntable.skipBackward0125Beats` | Skip 1/8 Beat Backward | `turntable1.skipBackward0125Beats`, `turntable2.skipBackward0125Beats`, `turntable3.skipBackward0125Beats`, `turntable4.skipBackward0125Beats` | 52 |  |
| `turntable.skipBackward025Beats` | Skip 1/4 Beat Backward | `turntable1.skipBackward025Beats`, `turntable2.skipBackward025Beats`, `turntable3.skipBackward025Beats`, `turntable4.skipBackward025Beats` | 70 |  |
| `turntable.skipBackward05Beats` | Skip 1/2 Beat Backward | `turntable1.skipBackward05Beats`, `turntable2.skipBackward05Beats`, `turntable3.skipBackward05Beats`, `turntable4.skipBackward05Beats`, `turntableSelected.skipBackward05Beats` | 74 |  |
| `turntable.skipBackward16Beats` | Skip 16 Beats Backward | `turntable1.skipBackward16Beats`, `turntable2.skipBackward16Beats`, `turntable3.skipBackward16Beats`, `turntable4.skipBackward16Beats`, `turntableSelected.skipBackward16Beats` | 26 |  |
| `turntable.skipBackward1Beat` | Skip 1 Beat Backward | `turntable1.skipBackward1Beat`, `turntable2.skipBackward1Beat`, `turntable3.skipBackward1Beat`, `turntable4.skipBackward1Beat`, `turntableSelected.skipBackward1Beat` | 134 |  |
| `turntable.skipBackward2Beats` | Skip 2 Beats Backward | `turntable1.skipBackward2Beats`, `turntable2.skipBackward2Beats`, `turntable3.skipBackward2Beats`, `turntable4.skipBackward2Beats`, `turntableSelected.skipBackward2Beats` | 82 |  |
| `turntable.skipBackward32Beats` | Skip 32 Beats Backward | `turntable1.skipBackward32Beats`, `turntable2.skipBackward32Beats`, `turntable3.skipBackward32Beats`, `turntable4.skipBackward32Beats`, `turntableSelected.skipBackward32Beats` | 26 |  |
| `turntable.skipBackward4Beats` | Skip 4 Beats Backward | `turntable1.skipBackward4Beats`, `turntable2.skipBackward4Beats`, `turntable3.skipBackward4Beats`, `turntable4.skipBackward4Beats`, `turntableSelected.skipBackward4Beats` | 100 |  |
| `turntable.skipBackward64Beats` | Skip 64 Beats Backward | `turntable1.skipBackward64Beats`, `turntable2.skipBackward64Beats`, `turntable3.skipBackward64Beats`, `turntable4.skipBackward64Beats`, `turntableSelected.skipBackward64Beats` | 8 |  |
| `turntable.skipBackward8Beats` | Skip 8 Beats Backward | `turntable1.skipBackward8Beats`, `turntable2.skipBackward8Beats`, `turntable3.skipBackward8Beats`, `turntable4.skipBackward8Beats`, `turntableSelected.skipBackward8Beats` | 86 |  |
| `turntable.skipDurationDown` | Skip Duration - | `turntable1.skipDurationDown`, `turntable2.skipDurationDown`, `turntable3.skipDurationDown`, `turntable4.skipDurationDown` | 20 |  |
| `turntable.skipDurationUp` | Skip Duration + | `turntable1.skipDurationUp`, `turntable2.skipDurationUp`, `turntable3.skipDurationUp`, `turntable4.skipDurationUp` | 20 |  |
| `turntable.skipForward0125Beats` | Skip 1/8 Beat Forward | `turntable1.skipForward0125Beats`, `turntable2.skipForward0125Beats`, `turntable3.skipForward0125Beats`, `turntable4.skipForward0125Beats` | 52 |  |
| `turntable.skipForward025Beats` | Skip 1/4 Beat Forward | `turntable1.skipForward025Beats`, `turntable2.skipForward025Beats`, `turntable3.skipForward025Beats`, `turntable4.skipForward025Beats` | 70 |  |
| `turntable.skipForward05Beats` | Skip 1/2 Beat Forward | `turntable1.skipForward05Beats`, `turntable2.skipForward05Beats`, `turntable3.skipForward05Beats`, `turntable4.skipForward05Beats`, `turntableSelected.skipForward05Beats` | 74 |  |
| `turntable.skipForward16Beats` | Skip 16 Beats Forward | `turntable1.skipForward16Beats`, `turntable2.skipForward16Beats`, `turntable3.skipForward16Beats`, `turntable4.skipForward16Beats`, `turntableSelected.skipForward16Beats` | 26 |  |
| `turntable.skipForward1Beat` | Skip 1 Beat Forward | `turntable1.skipForward1Beat`, `turntable2.skipForward1Beat`, `turntable3.skipForward1Beat`, `turntable4.skipForward1Beat`, `turntableSelected.skipForward1Beat` | 134 |  |
| `turntable.skipForward2Beats` | Skip 2 Beats Forward | `turntable1.skipForward2Beats`, `turntable2.skipForward2Beats`, `turntable3.skipForward2Beats`, `turntable4.skipForward2Beats`, `turntableSelected.skipForward2Beats` | 82 |  |
| `turntable.skipForward32Beats` | Skip 32 Beats Forward | `turntable1.skipForward32Beats`, `turntable2.skipForward32Beats`, `turntable3.skipForward32Beats`, `turntable4.skipForward32Beats`, `turntableSelected.skipForward32Beats` | 26 |  |
| `turntable.skipForward4Beats` | Skip 4 Beats Forward | `turntable1.skipForward4Beats`, `turntable2.skipForward4Beats`, `turntable3.skipForward4Beats`, `turntable4.skipForward4Beats`, `turntableSelected.skipForward4Beats` | 100 |  |
| `turntable.skipForward64Beats` | Skip 64 Beats Forward | `turntable1.skipForward64Beats`, `turntable2.skipForward64Beats`, `turntable3.skipForward64Beats`, `turntable4.skipForward64Beats`, `turntableSelected.skipForward64Beats` | 8 |  |
| `turntable.skipForward8Beats` | Skip 8 Beats Forward | `turntable1.skipForward8Beats`, `turntable2.skipForward8Beats`, `turntable3.skipForward8Beats`, `turntable4.skipForward8Beats`, `turntableSelected.skipForward8Beats` | 86 |  |
| `turntable.slicerRepeatDurationDouble` | Slicer Repeat Interval + | `turntable1.slicerRepeatDurationDouble`, `turntable2.slicerRepeatDurationDouble`, `turntable3.slicerRepeatDurationDouble`, `turntable4.slicerRepeatDurationDouble`, `turntableSelected.slicerRepeatDurationDouble` | 15 |  |
| `turntable.slicerRepeatDurationHalf` | Slicer Repeat Interval - | `turntable1.slicerRepeatDurationHalf`, `turntable2.slicerRepeatDurationHalf`, `turntable3.slicerRepeatDurationHalf`, `turntable4.slicerRepeatDurationHalf`, `turntableSelected.slicerRepeatDurationHalf` | 15 |  |
| `turntable.speedMinus` | Tempo - | `turntable1.speedMinus`, `turntable2.speedMinus`, `turntable3.speedMinus`, `turntable4.speedMinus` | 24 |  |
| `turntable.speedPlus` | Tempo + | `turntable1.speedPlus`, `turntable2.speedPlus`, `turntable3.speedPlus`, `turntable4.speedPlus` | 24 |  |
| `turntable.speedRelative` | Tempo | `turntable1.speedRelative`, `turntable2.speedRelative`, `turntable3.speedRelative`, `turntable4.speedRelative` |  | CC CH10 data 0; pickup; flipped; CC CH11 data 0; pickup; flipped; CC CH12 data 0; pickup; flipped; CC CH9 data 0; pickup; flipped |
| `turntable.startPoint` | Set Start CUE | `turntable1.startPoint`, `turntable2.startPoint` | 34 |  |

## Metadata-declared model state/value references

Every distinct path appearing in an input entry's `modelState` or `modelValue` field appears once below. These are Djay model references rather than physical MIDI addresses. “S4 feedback address” is populated only when the generated S4 mapping embeds output feedback on a metadata input linked to that state/value path; otherwise it is blank.

| Djay state/value key | Referenced as | Native/inferred description | Metadata input keys referencing it | S4 feedback address |
|---|---|---|---|---|
| `automix` | modelState | [Inferred] automix | `application.automix` |  |
| `isAnyTurntablePlaying` | modelState | [Inferred] is Any Turntable Playing | `application.playPause` |  |
| `isInSyncMode` | modelState | [Inferred] is In Sync Mode | `application.syncAll` |  |
| `looper.playing` | modelState | [Inferred] looper / playing | `looper.playPause` |  |
| `looper.track1.sample1.statePlaying` | modelState | [Inferred] looper / track 1 / sample 1 / state Playing | `looper.track1.sample1.trigger` |  |
| `looper.track1.sample2.statePlaying` | modelState | [Inferred] looper / track 1 / sample 2 / state Playing | `looper.track1.sample2.trigger` |  |
| `looper.track1.sample3.statePlaying` | modelState | [Inferred] looper / track 1 / sample 3 / state Playing | `looper.track1.sample3.trigger` |  |
| `looper.track1.sample4.statePlaying` | modelState | [Inferred] looper / track 1 / sample 4 / state Playing | `looper.track1.sample4.trigger` |  |
| `looper.track1.sample5.statePlaying` | modelState | [Inferred] looper / track 1 / sample 5 / state Playing | `looper.track1.sample5.trigger` |  |
| `looper.track1.sample6.statePlaying` | modelState | [Inferred] looper / track 1 / sample 6 / state Playing | `looper.track1.sample6.trigger` |  |
| `looper.track1.volume` | modelValue | Track 1 Volume | `looper.track1.volume` |  |
| `looper.track2.sample1.statePlaying` | modelState | [Inferred] looper / track 2 / sample 1 / state Playing | `looper.track2.sample1.trigger` |  |
| `looper.track2.sample2.statePlaying` | modelState | [Inferred] looper / track 2 / sample 2 / state Playing | `looper.track2.sample2.trigger` |  |
| `looper.track2.sample3.statePlaying` | modelState | [Inferred] looper / track 2 / sample 3 / state Playing | `looper.track2.sample3.trigger` |  |
| `looper.track2.sample4.statePlaying` | modelState | [Inferred] looper / track 2 / sample 4 / state Playing | `looper.track2.sample4.trigger` |  |
| `looper.track2.sample5.statePlaying` | modelState | [Inferred] looper / track 2 / sample 5 / state Playing | `looper.track2.sample5.trigger` |  |
| `looper.track2.sample6.statePlaying` | modelState | [Inferred] looper / track 2 / sample 6 / state Playing | `looper.track2.sample6.trigger` |  |
| `looper.track2.volume` | modelValue | Track 2 Volume | `looper.track2.volume` |  |
| `looper.track3.sample1.statePlaying` | modelState | [Inferred] looper / track 3 / sample 1 / state Playing | `looper.track3.sample1.trigger` |  |
| `looper.track3.sample2.statePlaying` | modelState | [Inferred] looper / track 3 / sample 2 / state Playing | `looper.track3.sample2.trigger` |  |
| `looper.track3.sample3.statePlaying` | modelState | [Inferred] looper / track 3 / sample 3 / state Playing | `looper.track3.sample3.trigger` |  |
| `looper.track3.sample4.statePlaying` | modelState | [Inferred] looper / track 3 / sample 4 / state Playing | `looper.track3.sample4.trigger` |  |
| `looper.track3.sample5.statePlaying` | modelState | [Inferred] looper / track 3 / sample 5 / state Playing | `looper.track3.sample5.trigger` |  |
| `looper.track3.sample6.statePlaying` | modelState | [Inferred] looper / track 3 / sample 6 / state Playing | `looper.track3.sample6.trigger` |  |
| `looper.track3.volume` | modelValue | Track 3 Volume | `looper.track3.volume` |  |
| `looper.track4.sample1.statePlaying` | modelState | [Inferred] looper / track 4 / sample 1 / state Playing | `looper.track4.sample1.trigger` |  |
| `looper.track4.sample2.statePlaying` | modelState | [Inferred] looper / track 4 / sample 2 / state Playing | `looper.track4.sample2.trigger` |  |
| `looper.track4.sample3.statePlaying` | modelState | [Inferred] looper / track 4 / sample 3 / state Playing | `looper.track4.sample3.trigger` |  |
| `looper.track4.sample4.statePlaying` | modelState | [Inferred] looper / track 4 / sample 4 / state Playing | `looper.track4.sample4.trigger` |  |
| `looper.track4.sample5.statePlaying` | modelState | [Inferred] looper / track 4 / sample 5 / state Playing | `looper.track4.sample5.trigger` |  |
| `looper.track4.sample6.statePlaying` | modelState | [Inferred] looper / track 4 / sample 6 / state Playing | `looper.track4.sample6.trigger` |  |
| `looper.track4.volume` | modelValue | Track 4 Volume | `looper.track4.volume` |  |
| `looper.track5.sample1.statePlaying` | modelState | [Inferred] looper / track 5 / sample 1 / state Playing | `looper.track5.sample1.trigger` |  |
| `looper.track5.sample2.statePlaying` | modelState | [Inferred] looper / track 5 / sample 2 / state Playing | `looper.track5.sample2.trigger` |  |
| `looper.track5.sample3.statePlaying` | modelState | [Inferred] looper / track 5 / sample 3 / state Playing | `looper.track5.sample3.trigger` |  |
| `looper.track5.sample4.statePlaying` | modelState | [Inferred] looper / track 5 / sample 4 / state Playing | `looper.track5.sample4.trigger` |  |
| `looper.track5.sample5.statePlaying` | modelState | [Inferred] looper / track 5 / sample 5 / state Playing | `looper.track5.sample5.trigger` |  |
| `looper.track5.sample6.statePlaying` | modelState | [Inferred] looper / track 5 / sample 6 / state Playing | `looper.track5.sample6.trigger` |  |
| `looper.track5.volume` | modelValue | Track 5 Volume | `looper.track5.volume` |  |
| `looper.track6.sample1.statePlaying` | modelState | [Inferred] looper / track 6 / sample 1 / state Playing | `looper.track6.sample1.trigger` |  |
| `looper.track6.sample2.statePlaying` | modelState | [Inferred] looper / track 6 / sample 2 / state Playing | `looper.track6.sample2.trigger` |  |
| `looper.track6.sample3.statePlaying` | modelState | [Inferred] looper / track 6 / sample 3 / state Playing | `looper.track6.sample3.trigger` |  |
| `looper.track6.sample4.statePlaying` | modelState | [Inferred] looper / track 6 / sample 4 / state Playing | `looper.track6.sample4.trigger` |  |
| `looper.track6.sample5.statePlaying` | modelState | [Inferred] looper / track 6 / sample 5 / state Playing | `looper.track6.sample5.trigger` |  |
| `looper.track6.sample6.statePlaying` | modelState | [Inferred] looper / track 6 / sample 6 / state Playing | `looper.track6.sample6.trigger` |  |
| `looper.track6.volume` | modelValue | Track 6 Volume | `looper.track6.volume` |  |
| `looper.track7.sample1.statePlaying` | modelState | [Inferred] looper / track 7 / sample 1 / state Playing | `looper.track7.sample1.trigger` |  |
| `looper.track7.sample2.statePlaying` | modelState | [Inferred] looper / track 7 / sample 2 / state Playing | `looper.track7.sample2.trigger` |  |
| `looper.track7.sample3.statePlaying` | modelState | [Inferred] looper / track 7 / sample 3 / state Playing | `looper.track7.sample3.trigger` |  |
| `looper.track7.sample4.statePlaying` | modelState | [Inferred] looper / track 7 / sample 4 / state Playing | `looper.track7.sample4.trigger` |  |
| `looper.track7.sample5.statePlaying` | modelState | [Inferred] looper / track 7 / sample 5 / state Playing | `looper.track7.sample5.trigger` |  |
| `looper.track7.sample6.statePlaying` | modelState | [Inferred] looper / track 7 / sample 6 / state Playing | `looper.track7.sample6.trigger` |  |
| `looper.track7.volume` | modelValue | Track 7 Volume | `looper.track7.volume` |  |
| `looper.track8.sample1.statePlaying` | modelState | [Inferred] looper / track 8 / sample 1 / state Playing | `looper.track8.sample1.trigger` |  |
| `looper.track8.sample2.statePlaying` | modelState | [Inferred] looper / track 8 / sample 2 / state Playing | `looper.track8.sample2.trigger` |  |
| `looper.track8.sample3.statePlaying` | modelState | [Inferred] looper / track 8 / sample 3 / state Playing | `looper.track8.sample3.trigger` |  |
| `looper.track8.sample4.statePlaying` | modelState | [Inferred] looper / track 8 / sample 4 / state Playing | `looper.track8.sample4.trigger` |  |
| `looper.track8.sample5.statePlaying` | modelState | [Inferred] looper / track 8 / sample 5 / state Playing | `looper.track8.sample5.trigger` |  |
| `looper.track8.sample6.statePlaying` | modelState | [Inferred] looper / track 8 / sample 6 / state Playing | `looper.track8.sample6.trigger` |  |
| `looper.track8.volume` | modelValue | Track 8 Volume | `looper.track8.volume` |  |
| `looper.volume` | modelValue | Volume | `looper.volume` |  |
| `microphone.active` | modelState | Microphone on-off | `microphone.active` |  |
| `midiModel.turntable.padModeIsAutoLoop` | modelState | [Inferred] midi Model / turntable / pad Mode Is Auto Loop | `turntable.padModeAutoLoop` |  |
| `midiModel.turntable.padModeIsBounceLoop` | modelState | [Inferred] midi Model / turntable / pad Mode Is Bounce Loop | `turntable.padModeBounceLoop` |  |
| `midiModel.turntable.padModeIsCueLoop` | modelState | [Inferred] midi Model / turntable / pad Mode Is Cue Loop | `turntable.padModeCueLoop` |  |
| `midiModel.turntable.padModeIsEmpty` | modelState | [Inferred] midi Model / turntable / pad Mode Is Empty | `turntable.padModeEmpty` |  |
| `midiModel.turntable.padModeIsGatedCue` | modelState | [Inferred] midi Model / turntable / pad Mode Is Gated Cue | `turntable.padModeGatedCue` |  |
| `midiModel.turntable.padModeIsHotCue` | modelState | [Inferred] midi Model / turntable / pad Mode Is Hot Cue | `turntable.padModeHotCue` |  |
| `midiModel.turntable.padModeIsInstantFX` | modelState | [Inferred] midi Model / turntable / pad Mode Is Instant FX | `turntable.padModeInstantFX` |  |
| `midiModel.turntable.padModeIsLooper` | modelState | [Inferred] midi Model / turntable / pad Mode Is Looper | `turntable.padModeLooper` |  |
| `midiModel.turntable.padModeIsManualLoop` | modelState | [Inferred] midi Model / turntable / pad Mode Is Manual Loop | `turntable.padModeManualLoop` |  |
| `midiModel.turntable.padModeIsPitchPlay` | modelState | [Inferred] midi Model / turntable / pad Mode Is Pitch Play | `turntable.padModePitchPlay` |  |
| `midiModel.turntable.padModeIsPitchShift` | modelState | [Inferred] midi Model / turntable / pad Mode Is Pitch Shift | `turntable.padModePitchShift` |  |
| `midiModel.turntable.padModeIsSampler` | modelState | [Inferred] midi Model / turntable / pad Mode Is Sampler | `turntable.padModeSampler` |  |
| `midiModel.turntable.padModeIsSavedLoop` | modelState | [Inferred] midi Model / turntable / pad Mode Is Saved Loop | `turntable.padModeSavedLoop` |  |
| `midiModel.turntable.padModeIsSkipping` | modelState | [Inferred] midi Model / turntable / pad Mode Is Skipping | `turntable.padModeSkipping` |  |
| `midiModel.turntable.padModeIsSlicer` | modelState | [Inferred] midi Model / turntable / pad Mode Is Slicer | `turntable.padModeSlicer` |  |
| `midiModel.turntable.padModeIsSlicerLoop` | modelState | [Inferred] midi Model / turntable / pad Mode Is Slicer Loop | `turntable.padModeSlicerLoop` |  |
| `midiModel.turntable.padModeIsUnmixer` | modelState | [Inferred] midi Model / turntable / pad Mode Is Unmixer | `turntable.padModeUnmixer` |  |
| `midiModel.turntable.padModeIsUser1` | modelState | [Inferred] midi Model / turntable / pad Mode Is User 1 | `turntable.padModeUser1` |  |
| `midiModel.turntable.padModeIsUser2` | modelState | [Inferred] midi Model / turntable / pad Mode Is User 2 | `turntable.padModeUser2` |  |
| `midiModel.turntable.padModeIsUser3` | modelState | [Inferred] midi Model / turntable / pad Mode Is User 3 | `turntable.padModeUser3` |  |
| `midiModel.turntable.padModeIsUser4` | modelState | [Inferred] midi Model / turntable / pad Mode Is User 4 | `turntable.padModeUser4` |  |
| `mixer.audioConfigSplitOutput` | modelState | [Inferred] mixer / audio Config Split Output | `mixer.monitorSplitOutputToggle` |  |
| `mixer.boothVolume` | modelValue | [Inferred] mixer / booth Volume | `mixer.boothLevel` |  |
| `mixer.crossfadeAssignment1` | modelState | [Inferred] mixer / crossfade Assignment 1 | `mixer.crossfadeAssignment1Toggle` |  |
| `mixer.crossfadeAssignment2` | modelState | [Inferred] mixer / crossfade Assignment 2 | `mixer.crossfadeAssignment2Toggle` |  |
| `mixer.crossfadeAssignment3` | modelState | [Inferred] mixer / crossfade Assignment 3 | `mixer.crossfadeAssignment3Toggle` |  |
| `mixer.crossfadeAssignment4` | modelState | [Inferred] mixer / crossfade Assignment 4 | `mixer.crossfadeAssignment4Toggle` |  |
| `mixer.crossfadeFXAutoPlay` | modelState | [Inferred] mixer / crossfade FX Auto Play | `mixer.crossfadeFXAutoPlayToggle` |  |
| `mixer.crossfadeFXTempoAdjustModeIsMorph` | modelState | [Inferred] mixer / crossfade FX Tempo Adjust Mode Is Morph | `mixer.crossfadeFXTempoBlendToggle` |  |
| `mixer.enableCrossfadeFX` | modelState | [Inferred] mixer / enable Crossfade FX | `mixer.crossfadeFXToggle` |  |
| `mixer.isPrecueing` | modelState | [Inferred] mixer / is Precueing | `mixer.monitorActive` |  |
| `mixer.manualTransition.isRunning` | modelState | [Inferred] mixer / manual Transition / is Running | `mixer.crossfadeFXTransition` |  |
| `mixer.masterAcapellaCrossfade` | modelValue | [Inferred] mixer / master Acapella Crossfade | `mixer.crossfadeAcapella`, `mixer.crossfadeFourTrackChannel4`, `mixer.crossfadeThreeTrackChannel3`, `mixer.crossfadeTwoTrackChannel2` |  |
| `mixer.masterBassFourCrossfade` | modelValue | [Inferred] mixer / master Bass Four Crossfade | `mixer.crossfadeFourTrackChannel2` |  |
| `mixer.masterCrossfade` | modelValue | [Inferred] mixer / master Crossfade | `mixer.crossfade` |  |
| `mixer.masterHarmonicCrossfade` | modelValue | [Inferred] mixer / master Harmonic Crossfade | `mixer.crossfadeFourTrackChannel3`, `mixer.crossfadeHarmonic`, `mixer.crossfadeThreeTrackChannel2` |  |
| `mixer.masterInstrumentalCrossfade` | modelValue | [Inferred] mixer / master Instrumental Crossfade | `mixer.crossfadeFourTrackChannel1`, `mixer.crossfadeInstrumental`, `mixer.crossfadeThreeTrackChannel1`, `mixer.crossfadeTwoTrackChannel1` |  |
| `mixer.masterVideoCrossfade` | modelValue | [Inferred] mixer / master Video Crossfade | `mixer.videoCrossfade` |  |
| `mixer.masterVolume` | modelValue | [Inferred] mixer / master Volume | `mixer.masterLevel` |  |
| `mixer.preCueCrossfade` | modelValue | [Inferred] mixer / pre Cue Crossfade | `mixer.monitorSelect` |  |
| `mixer.preCueMix` | modelValue | [Inferred] mixer / pre Cue Mix | `mixer.monitorMix` |  |
| `mixer.preCueMixIsCue` | modelState | [Inferred] mixer / pre Cue Mix Is Cue | `mixer.monitorMixToCue` |  |
| `mixer.preCueMixIsMiddle` | modelState | [Inferred] mixer / pre Cue Mix Is Middle | `mixer.monitorMixToMiddle` |  |
| `mixer.preCueMixIsMix` | modelState | [Inferred] mixer / pre Cue Mix Is Mix | `mixer.monitorMixToMix`, `mixer.monitorMixToggle` |  |
| `mixer.preCueVolume` | modelValue | [Inferred] mixer / pre Cue Volume | `mixer.monitorLevel` |  |
| `mixer.selectedActiveCrossfadeFXIndex` | modelState | [Inferred] mixer / selected Active Crossfade FX Index | `mixer.crossfadeFXSelectAndToggle` |  |
| `mixer.selectedCrossfadeFXIndex` | modelState | [Inferred] mixer / selected Crossfade FX Index | `mixer.crossfadeFXSelect` |  |
| `mixer.splitCueMode` | modelState | [Inferred] mixer / split Cue Mode | `mixer.monitorSplitCueMode` |  |
| `mixer.unmixerCrossfadeModeSeparate` | modelState | [Inferred] mixer / unmixer Crossfade Mode Separate | `mixer.unmixerCrossfadeModeSeparate` |  |
| `mixer.volumeEffectChannel.currentValue` | modelValue | [Inferred] mixer / volume Effect Channel / current Value | `mixer.lineVolume` |  |
| `mixer.volumeEffectChannel1.currentValue` | modelValue | [Inferred] mixer / volume Effect Channel 1 / current Value | `mixer.lineVolume1` |  |
| `mixer.volumeEffectChannel2.currentValue` | modelValue | [Inferred] mixer / volume Effect Channel 2 / current Value | `mixer.lineVolume2` |  |
| `mixer.volumeEffectChannel3.currentValue` | modelValue | [Inferred] mixer / volume Effect Channel 3 / current Value | `mixer.lineVolume3` |  |
| `mixer.volumeEffectChannel4.currentValue` | modelValue | [Inferred] mixer / volume Effect Channel 4 / current Value | `mixer.lineVolume4` |  |
| `permanentGlobalSyncMode` | modelState | [Inferred] permanent Global Sync Mode | `application.permanentGlobalSyncMode` |  |
| `recorder.isRecording` | modelState | [Inferred] recorder / is Recording | `application.recording` |  |
| `sampler.player1.gain` | modelValue | [Inferred] sampler / player 1 / gain | `sampler.player1.gain` |  |
| `sampler.player1.statePlaying` | modelState | [Inferred] sampler / player 1 / state Playing | `sampler.player1.paused`, `sampler.player1.playing`, `sampler.player1.playingConsideringHoldSetting`, `sampler.player1.playingHold` |  |
| `sampler.player1.volume` | modelValue | Sampler 1 Volume | `sampler.player1.volume` |  |
| `sampler.player10.gain` | modelValue | [Inferred] sampler / player 10 / gain | `sampler.player10.gain` |  |
| `sampler.player10.playing` | modelState | [Inferred] sampler / player 10 / playing | `sampler.player10.paused`, `sampler.player10.playing`, `sampler.player10.playingConsideringHoldSetting`, `sampler.player10.playingHold` |  |
| `sampler.player10.volume` | modelValue | Sampler 10 Volume | `sampler.player10.volume` |  |
| `sampler.player11.gain` | modelValue | [Inferred] sampler / player 11 / gain | `sampler.player11.gain` |  |
| `sampler.player11.playing` | modelState | [Inferred] sampler / player 11 / playing | `sampler.player11.paused`, `sampler.player11.playing`, `sampler.player11.playingConsideringHoldSetting`, `sampler.player11.playingHold` |  |
| `sampler.player11.volume` | modelValue | Sampler 11 Volume | `sampler.player11.volume` |  |
| `sampler.player12.gain` | modelValue | [Inferred] sampler / player 12 / gain | `sampler.player12.gain` |  |
| `sampler.player12.playing` | modelState | [Inferred] sampler / player 12 / playing | `sampler.player12.paused`, `sampler.player12.playing`, `sampler.player12.playingConsideringHoldSetting`, `sampler.player12.playingHold` |  |
| `sampler.player12.volume` | modelValue | Sampler 12 Volume | `sampler.player12.volume` |  |
| `sampler.player13.gain` | modelValue | [Inferred] sampler / player 13 / gain | `sampler.player13.gain` |  |
| `sampler.player13.statePlaying` | modelState | [Inferred] sampler / player 13 / state Playing | `sampler.player13.paused`, `sampler.player13.playing`, `sampler.player13.playingConsideringHoldSetting`, `sampler.player13.playingHold` |  |
| `sampler.player13.volume` | modelValue | Sampler 13 Volume | `sampler.player13.volume` |  |
| `sampler.player14.gain` | modelValue | [Inferred] sampler / player 14 / gain | `sampler.player14.gain` |  |
| `sampler.player14.statePlaying` | modelState | [Inferred] sampler / player 14 / state Playing | `sampler.player14.paused`, `sampler.player14.playing`, `sampler.player14.playingConsideringHoldSetting`, `sampler.player14.playingHold` |  |
| `sampler.player14.volume` | modelValue | Sampler 14 Volume | `sampler.player14.volume` |  |
| `sampler.player15.gain` | modelValue | [Inferred] sampler / player 15 / gain | `sampler.player15.gain` |  |
| `sampler.player15.statePlaying` | modelState | [Inferred] sampler / player 15 / state Playing | `sampler.player15.paused`, `sampler.player15.playing`, `sampler.player15.playingConsideringHoldSetting`, `sampler.player15.playingHold` |  |
| `sampler.player15.volume` | modelValue | Sampler 15 Volume | `sampler.player15.volume` |  |
| `sampler.player16.gain` | modelValue | [Inferred] sampler / player 16 / gain | `sampler.player16.gain` |  |
| `sampler.player16.statePlaying` | modelState | [Inferred] sampler / player 16 / state Playing | `sampler.player16.paused`, `sampler.player16.playing`, `sampler.player16.playingConsideringHoldSetting`, `sampler.player16.playingHold` |  |
| `sampler.player16.volume` | modelValue | Sampler 16 Volume | `sampler.player16.volume` |  |
| `sampler.player2.gain` | modelValue | [Inferred] sampler / player 2 / gain | `sampler.player2.gain` |  |
| `sampler.player2.statePlaying` | modelState | [Inferred] sampler / player 2 / state Playing | `sampler.player2.paused`, `sampler.player2.playing`, `sampler.player2.playingConsideringHoldSetting`, `sampler.player2.playingHold` |  |
| `sampler.player2.volume` | modelValue | Sampler 2 Volume | `sampler.player2.volume` |  |
| `sampler.player3.gain` | modelValue | [Inferred] sampler / player 3 / gain | `sampler.player3.gain` |  |
| `sampler.player3.statePlaying` | modelState | [Inferred] sampler / player 3 / state Playing | `sampler.player3.paused`, `sampler.player3.playing`, `sampler.player3.playingConsideringHoldSetting`, `sampler.player3.playingHold` |  |
| `sampler.player3.volume` | modelValue | Sampler 3 Volume | `sampler.player3.volume` |  |
| `sampler.player4.gain` | modelValue | [Inferred] sampler / player 4 / gain | `sampler.player4.gain` |  |
| `sampler.player4.statePlaying` | modelState | [Inferred] sampler / player 4 / state Playing | `sampler.player4.paused`, `sampler.player4.playing`, `sampler.player4.playingConsideringHoldSetting`, `sampler.player4.playingHold` |  |
| `sampler.player4.volume` | modelValue | Sampler 4 Volume | `sampler.player4.volume` |  |
| `sampler.player5.gain` | modelValue | [Inferred] sampler / player 5 / gain | `sampler.player5.gain` |  |
| `sampler.player5.statePlaying` | modelState | [Inferred] sampler / player 5 / state Playing | `sampler.player5.paused`, `sampler.player5.playing`, `sampler.player5.playingConsideringHoldSetting`, `sampler.player5.playingHold` |  |
| `sampler.player5.volume` | modelValue | Sampler 5 Volume | `sampler.player5.volume` |  |
| `sampler.player6.gain` | modelValue | [Inferred] sampler / player 6 / gain | `sampler.player6.gain` |  |
| `sampler.player6.statePlaying` | modelState | [Inferred] sampler / player 6 / state Playing | `sampler.player6.paused`, `sampler.player6.playing`, `sampler.player6.playingConsideringHoldSetting`, `sampler.player6.playingHold` |  |
| `sampler.player6.volume` | modelValue | Sampler 6 Volume | `sampler.player6.volume` |  |
| `sampler.player7.gain` | modelValue | [Inferred] sampler / player 7 / gain | `sampler.player7.gain` |  |
| `sampler.player7.statePlaying` | modelState | [Inferred] sampler / player 7 / state Playing | `sampler.player7.paused`, `sampler.player7.playing`, `sampler.player7.playingConsideringHoldSetting`, `sampler.player7.playingHold` |  |
| `sampler.player7.volume` | modelValue | Sampler 7 Volume | `sampler.player7.volume` |  |
| `sampler.player8.gain` | modelValue | [Inferred] sampler / player 8 / gain | `sampler.player8.gain` |  |
| `sampler.player8.statePlaying` | modelState | [Inferred] sampler / player 8 / state Playing | `sampler.player8.paused`, `sampler.player8.playing`, `sampler.player8.playingConsideringHoldSetting`, `sampler.player8.playingHold` |  |
| `sampler.player8.volume` | modelValue | Sampler 8 Volume | `sampler.player8.volume` |  |
| `sampler.player9.gain` | modelValue | [Inferred] sampler / player 9 / gain | `sampler.player9.gain` |  |
| `sampler.player9.statePlaying` | modelState | [Inferred] sampler / player 9 / state Playing | `sampler.player9.paused`, `sampler.player9.playing`, `sampler.player9.playingConsideringHoldSetting`, `sampler.player9.playingHold` |  |
| `sampler.player9.volume` | modelValue | Sampler 9 Volume | `sampler.player9.volume` |  |
| `sampler.sequencePlayerPlaying` | modelState | [Inferred] sampler / sequence Player Playing | `sampler.clearSequenceRecording` |  |
| `sampler.sequencePlaying` | modelState | [Inferred] sampler / sequence Playing | `sampler.toggleSequencePlaying` |  |
| `sampler.sequenceRecording` | modelState | [Inferred] sampler / sequence Recording | `sampler.toggleSequencePlayAndRecord`, `sampler.toggleSequenceRecording` |  |
| `sampler.turntable.player1.gain` | modelValue | [Inferred] sampler / turntable / player 1 / gain | `sampler.turntable.player1.gain` |  |
| `sampler.turntable.player1.statePlaying` | modelState | [Inferred] sampler / turntable / player 1 / state Playing | `sampler.turntable.player1.paused`, `sampler.turntable.player1.playing`, `sampler.turntable.player1.playingConsideringHoldSetting`, `sampler.turntable.player1.playingHold` | Note CH1 data 48; Note CH2 data 48; Note CH3 data 48; Note CH4 data 48 |
| `sampler.turntable.player1.volume` | modelValue | [Inferred] sampler / turntable / player 1 / volume | `sampler.turntable.player1.volume` |  |
| `sampler.turntable.player2.gain` | modelValue | [Inferred] sampler / turntable / player 2 / gain | `sampler.turntable.player2.gain` |  |
| `sampler.turntable.player2.statePlaying` | modelState | [Inferred] sampler / turntable / player 2 / state Playing | `sampler.turntable.player2.paused`, `sampler.turntable.player2.playing`, `sampler.turntable.player2.playingConsideringHoldSetting`, `sampler.turntable.player2.playingHold` | Note CH1 data 49; Note CH2 data 49; Note CH3 data 49; Note CH4 data 49 |
| `sampler.turntable.player2.volume` | modelValue | [Inferred] sampler / turntable / player 2 / volume | `sampler.turntable.player2.volume` |  |
| `sampler.turntable.player3.gain` | modelValue | [Inferred] sampler / turntable / player 3 / gain | `sampler.turntable.player3.gain` |  |
| `sampler.turntable.player3.statePlaying` | modelState | [Inferred] sampler / turntable / player 3 / state Playing | `sampler.turntable.player3.paused`, `sampler.turntable.player3.playing`, `sampler.turntable.player3.playingConsideringHoldSetting`, `sampler.turntable.player3.playingHold` | Note CH1 data 50; Note CH2 data 50; Note CH3 data 50; Note CH4 data 50 |
| `sampler.turntable.player3.volume` | modelValue | [Inferred] sampler / turntable / player 3 / volume | `sampler.turntable.player3.volume` |  |
| `sampler.turntable.player4.gain` | modelValue | [Inferred] sampler / turntable / player 4 / gain | `sampler.turntable.player4.gain` |  |
| `sampler.turntable.player4.statePlaying` | modelState | [Inferred] sampler / turntable / player 4 / state Playing | `sampler.turntable.player4.paused`, `sampler.turntable.player4.playing`, `sampler.turntable.player4.playingConsideringHoldSetting`, `sampler.turntable.player4.playingHold` | Note CH1 data 51; Note CH2 data 51; Note CH3 data 51; Note CH4 data 51 |
| `sampler.turntable.player4.volume` | modelValue | [Inferred] sampler / turntable / player 4 / volume | `sampler.turntable.player4.volume` |  |
| `sampler.turntable.player5.gain` | modelValue | [Inferred] sampler / turntable / player 5 / gain | `sampler.turntable.player5.gain` |  |
| `sampler.turntable.player5.statePlaying` | modelState | [Inferred] sampler / turntable / player 5 / state Playing | `sampler.turntable.player5.paused`, `sampler.turntable.player5.playing`, `sampler.turntable.player5.playingConsideringHoldSetting`, `sampler.turntable.player5.playingHold` | Note CH1 data 52; Note CH2 data 52; Note CH3 data 52; Note CH4 data 52 |
| `sampler.turntable.player5.volume` | modelValue | [Inferred] sampler / turntable / player 5 / volume | `sampler.turntable.player5.volume` |  |
| `sampler.turntable.player6.gain` | modelValue | [Inferred] sampler / turntable / player 6 / gain | `sampler.turntable.player6.gain` |  |
| `sampler.turntable.player6.statePlaying` | modelState | [Inferred] sampler / turntable / player 6 / state Playing | `sampler.turntable.player6.paused`, `sampler.turntable.player6.playing`, `sampler.turntable.player6.playingConsideringHoldSetting`, `sampler.turntable.player6.playingHold` | Note CH1 data 53; Note CH2 data 53; Note CH3 data 53; Note CH4 data 53 |
| `sampler.turntable.player6.volume` | modelValue | [Inferred] sampler / turntable / player 6 / volume | `sampler.turntable.player6.volume` |  |
| `sampler.turntable.player7.gain` | modelValue | [Inferred] sampler / turntable / player 7 / gain | `sampler.turntable.player7.gain` |  |
| `sampler.turntable.player7.statePlaying` | modelState | [Inferred] sampler / turntable / player 7 / state Playing | `sampler.turntable.player7.paused`, `sampler.turntable.player7.playing`, `sampler.turntable.player7.playingConsideringHoldSetting`, `sampler.turntable.player7.playingHold` | Note CH1 data 54; Note CH2 data 54; Note CH3 data 54; Note CH4 data 54 |
| `sampler.turntable.player7.volume` | modelValue | [Inferred] sampler / turntable / player 7 / volume | `sampler.turntable.player7.volume` |  |
| `sampler.turntable.player8.gain` | modelValue | [Inferred] sampler / turntable / player 8 / gain | `sampler.turntable.player8.gain` |  |
| `sampler.turntable.player8.statePlaying` | modelState | [Inferred] sampler / turntable / player 8 / state Playing | `sampler.turntable.player8.paused`, `sampler.turntable.player8.playing`, `sampler.turntable.player8.playingConsideringHoldSetting`, `sampler.turntable.player8.playingHold` | Note CH1 data 55; Note CH2 data 55; Note CH3 data 55; Note CH4 data 55 |
| `sampler.turntable.player8.volume` | modelValue | [Inferred] sampler / turntable / player 8 / volume | `sampler.turntable.player8.volume` |  |
| `sampler.volume` | modelValue | Volume | `sampler.volume` |  |
| `sequencer.metronomeEnabled` | modelState | [Inferred] sequencer / metronome Enabled | `sequencer.toggleMetronomeEnabled` |  |
| `turntable.anySlip` | modelState | [Inferred] turntable / any Slip | `turntable.waveSlipToggle` |  |
| `turntable.autoLoop003125BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 003125 Beat Interval Active | `turntable.autoLoop003125BeatInterval` |  |
| `turntable.autoLoop00625BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 00625 Beat Interval Active | `turntable.autoLoop00625BeatInterval` |  |
| `turntable.autoLoop0125BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 0125 Beat Interval Active | `turntable.autoLoop0125BeatInterval` |  |
| `turntable.autoLoop025BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 025 Beat Interval Active | `turntable.autoLoop025BeatInterval` |  |
| `turntable.autoLoop033BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 033 Beat Interval Active | `turntable.autoLoop033BeatInterval` |  |
| `turntable.autoLoop05BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 05 Beat Interval Active | `turntable.autoLoop05BeatInterval` |  |
| `turntable.autoLoop075BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 075 Beat Interval Active | `turntable.autoLoop075BeatInterval` |  |
| `turntable.autoLoop1_5BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 1 5 Beat Interval Active | `turntable.autoLoop1_5BeatInterval` |  |
| `turntable.autoLoop128BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 128 Beat Interval Active | `turntable.autoLoop128BeatInterval` |  |
| `turntable.autoLoop12BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 12 Beat Interval Active | `turntable.autoLoop12BeatInterval` |  |
| `turntable.autoLoop16BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 16 Beat Interval Active | `turntable.autoLoop16BeatInterval` |  |
| `turntable.autoLoop1BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 1 Beat Interval Active | `turntable.autoLoop1BeatInterval` |  |
| `turntable.autoLoop2BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 2 Beat Interval Active | `turntable.autoLoop2BeatInterval` |  |
| `turntable.autoLoop32BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 32 Beat Interval Active | `turntable.autoLoop32BeatInterval` |  |
| `turntable.autoLoop3BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 3 Beat Interval Active | `turntable.autoLoop3BeatInterval` |  |
| `turntable.autoLoop4BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 4 Beat Interval Active | `turntable.autoLoop4BeatInterval` |  |
| `turntable.autoLoop5BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 5 Beat Interval Active | `turntable.autoLoop5BeatInterval` |  |
| `turntable.autoLoop64BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 64 Beat Interval Active | `turntable.autoLoop64BeatInterval` |  |
| `turntable.autoLoop6BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 6 Beat Interval Active | `turntable.autoLoop6BeatInterval` |  |
| `turntable.autoLoop7BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 7 Beat Interval Active | `turntable.autoLoop7BeatInterval` |  |
| `turntable.autoLoop8BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 8 Beat Interval Active | `turntable.autoLoop8BeatInterval` |  |
| `turntable.autoLoop9BeatIntervalActive` | modelState | [Inferred] turntable / auto Loop 9 Beat Interval Active | `turntable.autoLoop9BeatInterval` |  |
| `turntable.autoRepeat` | modelState | [Inferred] turntable / auto Repeat | `turntable.autoRepeat` |  |
| `turntable.bounceLoop003125BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 003125 Beat Interval Active | `turntable.bounceLoop003125BeatInterval` |  |
| `turntable.bounceLoop00625BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 00625 Beat Interval Active | `turntable.bounceLoop00625BeatInterval` |  |
| `turntable.bounceLoop0125BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 0125 Beat Interval Active | `turntable.bounceLoop0125BeatInterval` |  |
| `turntable.bounceLoop025BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 025 Beat Interval Active | `turntable.bounceLoop025BeatInterval` |  |
| `turntable.bounceLoop033BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 033 Beat Interval Active | `turntable.bounceLoop033BeatInterval` |  |
| `turntable.bounceLoop05BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 05 Beat Interval Active | `turntable.bounceLoop05BeatInterval` |  |
| `turntable.bounceLoop075BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 075 Beat Interval Active | `turntable.bounceLoop075BeatInterval` |  |
| `turntable.bounceLoop16BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 16 Beat Interval Active | `turntable.bounceLoop16BeatInterval` |  |
| `turntable.bounceLoop1BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 1 Beat Interval Active | `turntable.bounceLoop1BeatInterval` |  |
| `turntable.bounceLoop2BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 2 Beat Interval Active | `turntable.bounceLoop2BeatInterval` |  |
| `turntable.bounceLoop32BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 32 Beat Interval Active | `turntable.bounceLoop32BeatInterval` |  |
| `turntable.bounceLoop4BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 4 Beat Interval Active | `turntable.bounceLoop4BeatInterval` |  |
| `turntable.bounceLoop8BeatIntervalActive` | modelState | [Inferred] turntable / bounce Loop 8 Beat Interval Active | `turntable.bounceLoop8BeatInterval` |  |
| `turntable.bounceLoopRouting.fourTrackChannel1` | modelState | [Inferred] turntable / bounce Loop Routing / four Track Channel 1 | `turntable.bounceLoopRouting.routingFourTrackChannel1` |  |
| `turntable.bounceLoopRouting.fourTrackChannel2` | modelState | [Inferred] turntable / bounce Loop Routing / four Track Channel 2 | `turntable.bounceLoopRouting.routingFourTrackChannel2` |  |
| `turntable.bounceLoopRouting.fourTrackChannel3` | modelState | [Inferred] turntable / bounce Loop Routing / four Track Channel 3 | `turntable.bounceLoopRouting.routingFourTrackChannel3` |  |
| `turntable.bounceLoopRouting.fourTrackChannel4` | modelState | [Inferred] turntable / bounce Loop Routing / four Track Channel 4 | `turntable.bounceLoopRouting.routingFourTrackChannel4` |  |
| `turntable.bounceLoopRouting.genericChannel1` | modelState | [Inferred] turntable / bounce Loop Routing / generic Channel 1 | `turntable.bounceLoopRouting.routingGenericChannel1` |  |
| `turntable.bounceLoopRouting.genericChannel2` | modelState | [Inferred] turntable / bounce Loop Routing / generic Channel 2 | `turntable.bounceLoopRouting.routingGenericChannel2` |  |
| `turntable.bounceLoopRouting.genericChannel3` | modelState | [Inferred] turntable / bounce Loop Routing / generic Channel 3 | `turntable.bounceLoopRouting.routingGenericChannel3` |  |
| `turntable.bounceLoopRouting.genericChannel4` | modelState | [Inferred] turntable / bounce Loop Routing / generic Channel 4 | `turntable.bounceLoopRouting.routingGenericChannel4` |  |
| `turntable.bounceLoopRouting.isDeck` | modelState | [Inferred] turntable / bounce Loop Routing / is Deck | `turntable.bounceLoopRouting.routingDeck`, `turntable.bounceLoopRoutingActiveToggle` |  |
| `turntable.bounceLoopRouting.threeTrackChannel1` | modelState | [Inferred] turntable / bounce Loop Routing / three Track Channel 1 | `turntable.bounceLoopRouting.routingThreeTrackChannel1` |  |
| `turntable.bounceLoopRouting.threeTrackChannel2` | modelState | [Inferred] turntable / bounce Loop Routing / three Track Channel 2 | `turntable.bounceLoopRouting.routingThreeTrackChannel2` |  |
| `turntable.bounceLoopRouting.threeTrackChannel3` | modelState | [Inferred] turntable / bounce Loop Routing / three Track Channel 3 | `turntable.bounceLoopRouting.routingThreeTrackChannel3` |  |
| `turntable.bounceLoopRouting.twoTrackChannel1` | modelState | [Inferred] turntable / bounce Loop Routing / two Track Channel 1 | `turntable.bounceLoopRouting.routingTwoTrackChannel1`, `turntable.bounceLoopRouting.routingTwoTrackChannel1Instrumental` |  |
| `turntable.bounceLoopRouting.twoTrackChannel2` | modelState | [Inferred] turntable / bounce Loop Routing / two Track Channel 2 | `turntable.bounceLoopRouting.routingTwoTrackChannel2`, `turntable.bounceLoopRouting.routingTwoTrackChannel2Acapella` |  |
| `turntable.controlMode` | modelState | [Inferred] turntable / control Mode | `turntable.toggleControlMode` |  |
| `turntable.controlModeIsAbsolute` | modelState | [Inferred] turntable / control Mode Is Absolute | `turntable.controlModeAbsolute` |  |
| `turntable.controlModeIsInternal` | modelState | [Inferred] turntable / control Mode Is Internal | `turntable.controlModeInternal` |  |
| `turntable.controlModeIsRelative` | modelState | [Inferred] turntable / control Mode Is Relative | `turntable.controlModeRelative` |  |
| `turntable.controlModeIsThru` | modelState | [Inferred] turntable / control Mode Is Thru | `turntable.controlModeThru` |  |
| `turntable.controlModeIsWireless` | modelState | [Inferred] turntable / control Mode Is Wireless | `turntable.controlModeWireless` |  |
| `turntable.cuePointPadIsSet1` | modelState | [Inferred] turntable / cue Point Pad Is Set 1 | `turntable.clearCuePoint1`, `turntable.cueOrJumpIfAlreadySet1`, `turntable.cueOrJumpIfAlreadySetAndAutoloop1`, `turntable.cueOrJumpIfAlreadySetAndReloop1` | Note CH1 data 40; Note CH2 data 40; Note CH3 data 40; Note CH4 data 40 |
| `turntable.cuePointPadIsSet2` | modelState | [Inferred] turntable / cue Point Pad Is Set 2 | `turntable.clearCuePoint2`, `turntable.cueOrJumpIfAlreadySet2`, `turntable.cueOrJumpIfAlreadySetAndAutoloop2`, `turntable.cueOrJumpIfAlreadySetAndReloop2` | Note CH1 data 41; Note CH2 data 41; Note CH3 data 41; Note CH4 data 41 |
| `turntable.cuePointPadIsSet3` | modelState | [Inferred] turntable / cue Point Pad Is Set 3 | `turntable.clearCuePoint3`, `turntable.cueOrJumpIfAlreadySet3`, `turntable.cueOrJumpIfAlreadySetAndAutoloop3`, `turntable.cueOrJumpIfAlreadySetAndReloop3` | Note CH1 data 42; Note CH2 data 42; Note CH3 data 42; Note CH4 data 42 |
| `turntable.cuePointPadIsSet4` | modelState | [Inferred] turntable / cue Point Pad Is Set 4 | `turntable.clearCuePoint4`, `turntable.cueOrJumpIfAlreadySet4`, `turntable.cueOrJumpIfAlreadySetAndAutoloop4`, `turntable.cueOrJumpIfAlreadySetAndReloop4` | Note CH1 data 43; Note CH2 data 43; Note CH3 data 43; Note CH4 data 43 |
| `turntable.cuePointPadIsSet5` | modelState | [Inferred] turntable / cue Point Pad Is Set 5 | `turntable.clearCuePoint5`, `turntable.cueOrJumpIfAlreadySet5`, `turntable.cueOrJumpIfAlreadySetAndAutoloop5`, `turntable.cueOrJumpIfAlreadySetAndReloop5` | Note CH1 data 44; Note CH2 data 44; Note CH3 data 44; Note CH4 data 44 |
| `turntable.cuePointPadIsSet6` | modelState | [Inferred] turntable / cue Point Pad Is Set 6 | `turntable.clearCuePoint6`, `turntable.cueOrJumpIfAlreadySet6`, `turntable.cueOrJumpIfAlreadySetAndAutoloop6`, `turntable.cueOrJumpIfAlreadySetAndReloop6` | Note CH1 data 45; Note CH2 data 45; Note CH3 data 45; Note CH4 data 45 |
| `turntable.cuePointPadIsSet7` | modelState | [Inferred] turntable / cue Point Pad Is Set 7 | `turntable.clearCuePoint7`, `turntable.cueOrJumpIfAlreadySet7`, `turntable.cueOrJumpIfAlreadySetAndAutoloop7`, `turntable.cueOrJumpIfAlreadySetAndReloop7` | Note CH1 data 46; Note CH2 data 46; Note CH3 data 46; Note CH4 data 46 |
| `turntable.cuePointPadIsSet8` | modelState | [Inferred] turntable / cue Point Pad Is Set 8 | `turntable.clearCuePoint8`, `turntable.cueOrJumpIfAlreadySet8`, `turntable.cueOrJumpIfAlreadySetAndAutoloop8`, `turntable.cueOrJumpIfAlreadySetAndReloop8` | Note CH1 data 47; Note CH2 data 47; Note CH3 data 47; Note CH4 data 47 |
| `turntable.deckQuantize` | modelState | [Inferred] turntable / deck Quantize | `turntable.toggleQuantize` | Note CH1 data 15; Note CH2 data 15; Note CH3 data 15; Note CH4 data 15 |
| `turntable.deckSlipButtonState` | modelState | [Inferred] turntable / deck Slip Button State | `turntable.deckSlip`, `turntable.deckSlipToggle` | Note CH1 data 4; Note CH2 data 4; Note CH3 data 4; Note CH4 data 4 |
| `turntable.display.showElapsedTime` | modelState | [Inferred] turntable / display / show Elapsed Time | `turntable.toggleTimeDisplay` |  |
| `turntable.display.songProgress` | modelValue | [Inferred] turntable / display / song Progress | `turntable.scrubbing` |  |
| `turntable.effects.filter.currentValue` | modelValue | [Inferred] turntable / effects / filter / current Value | `turntable.filter` |  |
| `turntable.effects.filter.isReset` | modelState | [Inferred] turntable / effects / filter / is Reset | `turntable.filter`, `turntable.filterKill`, `turntable.resetFilter` |  |
| `turntable.effects.gain.currentValue` | modelValue | [Inferred] turntable / effects / gain / current Value | `turntable.gain` |  |
| `turntable.effects.hardwareGain.isReset` | modelState | [Inferred] turntable / effects / hardware Gain / is Reset | `turntable.resetGain` |  |
| `turntable.effects.highEQ.currentValue` | modelValue | [Inferred] turntable / effects / high EQ / current Value | `turntable.equalizerEQHigh` |  |
| `turntable.effects.highEQ.isReset` | modelState | [Inferred] turntable / effects / high EQ / is Reset | `turntable.equalizerEQHigh` |  |
| `turntable.effects.lowEQ.currentValue` | modelValue | [Inferred] turntable / effects / low EQ / current Value | `turntable.equalizerEQLow` |  |
| `turntable.effects.lowEQ.isReset` | modelState | [Inferred] turntable / effects / low EQ / is Reset | `turntable.equalizerEQLow` |  |
| `turntable.effects.midEQ.currentValue` | modelValue | [Inferred] turntable / effects / mid EQ / current Value | `turntable.equalizerEQMid` |  |
| `turntable.effects.midEQ.isReset` | modelState | [Inferred] turntable / effects / mid EQ / is Reset | `turntable.equalizerEQMid` |  |
| `turntable.effects.pan.currentValue` | modelValue | [Inferred] turntable / effects / pan / current Value | `turntable.pan` |  |
| `turntable.effects.pan.isReset` | modelState | [Inferred] turntable / effects / pan / is Reset | `turntable.pan`, `turntable.resetPan` |  |
| `turntable.effects.pitch.currentValue` | modelState, modelValue | [Inferred] turntable / effects / pitch / current Value | `turntable.pitch`, `turntable.pitchSelect` |  |
| `turntable.effects.pitch.isEnabled` | modelState | [Inferred] turntable / effects / pitch / is Enabled | `turntable.pitchOnOff` |  |
| `turntable.effects.pitch.isReset` | modelState | [Inferred] turntable / effects / pitch / is Reset | `turntable.resetPitch` |  |
| `turntable.effects.tempo.currentValue` | modelValue | [Inferred] turntable / effects / tempo / current Value | `turntable.speed` |  |
| `turntable.effects.tempo.isReset` | modelState | [Inferred] turntable / effects / tempo / is Reset | `turntable.resetSpeed`, `turntable.speed` |  |
| `turntable.enableUnmixerEQ` | modelState | [Inferred] turntable / enable Unmixer EQ | `turntable.toggleUnmixerEQMode` |  |
| `turntable.fx1.enabled` | modelState | [Inferred] turntable / fx 1 / enabled | `turntable.echo`, `turntable.fx1Enabled`, `turntable.fx1EnabledHold`, `turntable.fx1ParameterDefaultValue` | Note CH1 data 33; Note CH2 data 33; Note CH3 data 33; Note CH4 data 33 |
| `turntable.fx1.parameterValue` | modelValue | [Inferred] turntable / fx 1 / parameter Value | `turntable.fx1ParameterValue` |  |
| `turntable.fx1.routing.fourTrackChannel1` | modelState | [Inferred] turntable / fx 1 / routing / four Track Channel 1 | `turntable.fx1Routing.routingFourTrackChannel1` |  |
| `turntable.fx1.routing.fourTrackChannel2` | modelState | [Inferred] turntable / fx 1 / routing / four Track Channel 2 | `turntable.fx1Routing.routingFourTrackChannel2` |  |
| `turntable.fx1.routing.fourTrackChannel3` | modelState | [Inferred] turntable / fx 1 / routing / four Track Channel 3 | `turntable.fx1Routing.routingFourTrackChannel3` |  |
| `turntable.fx1.routing.fourTrackChannel4` | modelState | [Inferred] turntable / fx 1 / routing / four Track Channel 4 | `turntable.fx1Routing.routingFourTrackChannel4` |  |
| `turntable.fx1.routing.genericChannel1` | modelState | [Inferred] turntable / fx 1 / routing / generic Channel 1 | `turntable.fx1Routing.routingGenericChannel1` |  |
| `turntable.fx1.routing.genericChannel2` | modelState | [Inferred] turntable / fx 1 / routing / generic Channel 2 | `turntable.fx1Routing.routingGenericChannel2` |  |
| `turntable.fx1.routing.genericChannel3` | modelState | [Inferred] turntable / fx 1 / routing / generic Channel 3 | `turntable.fx1Routing.routingGenericChannel3` |  |
| `turntable.fx1.routing.genericChannel4` | modelState | [Inferred] turntable / fx 1 / routing / generic Channel 4 | `turntable.fx1Routing.routingGenericChannel4` |  |
| `turntable.fx1.routing.isDeck` | modelState | [Inferred] turntable / fx 1 / routing / is Deck | `turntable.fx1Routing.routingDeck`, `turntable.fx1RoutingDeck` |  |
| `turntable.fx1.routing.threeTrackChannel1` | modelState | [Inferred] turntable / fx 1 / routing / three Track Channel 1 | `turntable.fx1Routing.routingThreeTrackChannel1`, `turntable.fx1RoutingInstrumental` |  |
| `turntable.fx1.routing.threeTrackChannel2` | modelState | [Inferred] turntable / fx 1 / routing / three Track Channel 2 | `turntable.fx1Routing.routingThreeTrackChannel2`, `turntable.fx1RoutingHarmonic` |  |
| `turntable.fx1.routing.threeTrackChannel3` | modelState | [Inferred] turntable / fx 1 / routing / three Track Channel 3 | `turntable.fx1Routing.routingThreeTrackChannel3`, `turntable.fx1RoutingAcapella` |  |
| `turntable.fx1.routing.twoTrackChannel1` | modelState | [Inferred] turntable / fx 1 / routing / two Track Channel 1 | `turntable.fx1Routing.routingTwoTrackChannel1`, `turntable.fx1Routing.routingTwoTrackChannel1Instrumental` |  |
| `turntable.fx1.routing.twoTrackChannel2` | modelState | [Inferred] turntable / fx 1 / routing / two Track Channel 2 | `turntable.fx1Routing.routingTwoTrackChannel2`, `turntable.fx1Routing.routingTwoTrackChannel2Acapella` |  |
| `turntable.fx1.typeIndexInSourceFavorites` | modelState | [Inferred] turntable / fx 1 / type Index In Source Favorites | `turntable.fx1SelectFavorite` |  |
| `turntable.fx1.typeIndexInSourcePack` | modelState | [Inferred] turntable / fx 1 / type Index In Source Pack | `turntable.fx1Select` |  |
| `turntable.fx1.wetDryValue` | modelValue | [Inferred] turntable / fx 1 / wet Dry Value | `turntable.fx1WetDryValue`, `turntable.fxWetDryValue` |  |
| `turntable.fx2.enabled` | modelState | [Inferred] turntable / fx 2 / enabled | `turntable.fx2Enabled`, `turntable.fx2EnabledHold`, `turntable.fx2ParameterDefaultValue` | Note CH1 data 34; Note CH2 data 34; Note CH3 data 34; Note CH4 data 34 |
| `turntable.fx2.parameterValue` | modelValue | [Inferred] turntable / fx 2 / parameter Value | `turntable.fx2ParameterValue` |  |
| `turntable.fx2.routing.fourTrackChannel1` | modelState | [Inferred] turntable / fx 2 / routing / four Track Channel 1 | `turntable.fx2Routing.routingFourTrackChannel1` |  |
| `turntable.fx2.routing.fourTrackChannel2` | modelState | [Inferred] turntable / fx 2 / routing / four Track Channel 2 | `turntable.fx2Routing.routingFourTrackChannel2` |  |
| `turntable.fx2.routing.fourTrackChannel3` | modelState | [Inferred] turntable / fx 2 / routing / four Track Channel 3 | `turntable.fx2Routing.routingFourTrackChannel3` |  |
| `turntable.fx2.routing.fourTrackChannel4` | modelState | [Inferred] turntable / fx 2 / routing / four Track Channel 4 | `turntable.fx2Routing.routingFourTrackChannel4` |  |
| `turntable.fx2.routing.genericChannel1` | modelState | [Inferred] turntable / fx 2 / routing / generic Channel 1 | `turntable.fx2Routing.routingGenericChannel1` |  |
| `turntable.fx2.routing.genericChannel2` | modelState | [Inferred] turntable / fx 2 / routing / generic Channel 2 | `turntable.fx2Routing.routingGenericChannel2` |  |
| `turntable.fx2.routing.genericChannel3` | modelState | [Inferred] turntable / fx 2 / routing / generic Channel 3 | `turntable.fx2Routing.routingGenericChannel3` |  |
| `turntable.fx2.routing.genericChannel4` | modelState | [Inferred] turntable / fx 2 / routing / generic Channel 4 | `turntable.fx2Routing.routingGenericChannel4` |  |
| `turntable.fx2.routing.isDeck` | modelState | [Inferred] turntable / fx 2 / routing / is Deck | `turntable.fx2Routing.routingDeck`, `turntable.fx2RoutingDeck` |  |
| `turntable.fx2.routing.threeTrackChannel1` | modelState | [Inferred] turntable / fx 2 / routing / three Track Channel 1 | `turntable.fx2Routing.routingThreeTrackChannel1`, `turntable.fx2RoutingInstrumental` |  |
| `turntable.fx2.routing.threeTrackChannel2` | modelState | [Inferred] turntable / fx 2 / routing / three Track Channel 2 | `turntable.fx2Routing.routingThreeTrackChannel2`, `turntable.fx2RoutingHarmonic` |  |
| `turntable.fx2.routing.threeTrackChannel3` | modelState | [Inferred] turntable / fx 2 / routing / three Track Channel 3 | `turntable.fx2Routing.routingThreeTrackChannel3`, `turntable.fx2RoutingAcapella` |  |
| `turntable.fx2.routing.twoTrackChannel1` | modelState | [Inferred] turntable / fx 2 / routing / two Track Channel 1 | `turntable.fx2Routing.routingTwoTrackChannel1`, `turntable.fx2Routing.routingTwoTrackChannel1Instrumental` |  |
| `turntable.fx2.routing.twoTrackChannel2` | modelState | [Inferred] turntable / fx 2 / routing / two Track Channel 2 | `turntable.fx2Routing.routingTwoTrackChannel2`, `turntable.fx2Routing.routingTwoTrackChannel2Acapella` |  |
| `turntable.fx2.typeIndexInSourceFavorites` | modelState | [Inferred] turntable / fx 2 / type Index In Source Favorites | `turntable.fx2SelectFavorite` |  |
| `turntable.fx2.typeIndexInSourcePack` | modelState | [Inferred] turntable / fx 2 / type Index In Source Pack | `turntable.fx2Select` |  |
| `turntable.fx2.wetDryValue` | modelValue | [Inferred] turntable / fx 2 / wet Dry Value | `turntable.fx2WetDryValue` |  |
| `turntable.fx3.enabled` | modelState | [Inferred] turntable / fx 3 / enabled | `turntable.fx3Enabled`, `turntable.fx3EnabledHold`, `turntable.fx3ParameterDefaultValue` | Note CH1 data 35; Note CH2 data 35; Note CH3 data 35; Note CH4 data 35 |
| `turntable.fx3.parameterValue` | modelValue | [Inferred] turntable / fx 3 / parameter Value | `turntable.fx3ParameterValue` |  |
| `turntable.fx3.routing.fourTrackChannel1` | modelState | [Inferred] turntable / fx 3 / routing / four Track Channel 1 | `turntable.fx3Routing.routingFourTrackChannel1` |  |
| `turntable.fx3.routing.fourTrackChannel2` | modelState | [Inferred] turntable / fx 3 / routing / four Track Channel 2 | `turntable.fx3Routing.routingFourTrackChannel2` |  |
| `turntable.fx3.routing.fourTrackChannel3` | modelState | [Inferred] turntable / fx 3 / routing / four Track Channel 3 | `turntable.fx3Routing.routingFourTrackChannel3` |  |
| `turntable.fx3.routing.fourTrackChannel4` | modelState | [Inferred] turntable / fx 3 / routing / four Track Channel 4 | `turntable.fx3Routing.routingFourTrackChannel4` |  |
| `turntable.fx3.routing.genericChannel1` | modelState | [Inferred] turntable / fx 3 / routing / generic Channel 1 | `turntable.fx3Routing.routingGenericChannel1` |  |
| `turntable.fx3.routing.genericChannel2` | modelState | [Inferred] turntable / fx 3 / routing / generic Channel 2 | `turntable.fx3Routing.routingGenericChannel2` |  |
| `turntable.fx3.routing.genericChannel3` | modelState | [Inferred] turntable / fx 3 / routing / generic Channel 3 | `turntable.fx3Routing.routingGenericChannel3` |  |
| `turntable.fx3.routing.genericChannel4` | modelState | [Inferred] turntable / fx 3 / routing / generic Channel 4 | `turntable.fx3Routing.routingGenericChannel4` |  |
| `turntable.fx3.routing.isDeck` | modelState | [Inferred] turntable / fx 3 / routing / is Deck | `turntable.fx3Routing.routingDeck`, `turntable.fx3RoutingDeck` |  |
| `turntable.fx3.routing.threeTrackChannel1` | modelState | [Inferred] turntable / fx 3 / routing / three Track Channel 1 | `turntable.fx3Routing.routingThreeTrackChannel1`, `turntable.fx3RoutingInstrumental` |  |
| `turntable.fx3.routing.threeTrackChannel2` | modelState | [Inferred] turntable / fx 3 / routing / three Track Channel 2 | `turntable.fx3Routing.routingThreeTrackChannel2`, `turntable.fx3RoutingHarmonic` |  |
| `turntable.fx3.routing.threeTrackChannel3` | modelState | [Inferred] turntable / fx 3 / routing / three Track Channel 3 | `turntable.fx3Routing.routingThreeTrackChannel3`, `turntable.fx3RoutingAcapella` |  |
| `turntable.fx3.routing.twoTrackChannel1` | modelState | [Inferred] turntable / fx 3 / routing / two Track Channel 1 | `turntable.fx3Routing.routingTwoTrackChannel1`, `turntable.fx3Routing.routingTwoTrackChannel1Instrumental` |  |
| `turntable.fx3.routing.twoTrackChannel2` | modelState | [Inferred] turntable / fx 3 / routing / two Track Channel 2 | `turntable.fx3Routing.routingTwoTrackChannel2`, `turntable.fx3Routing.routingTwoTrackChannel2Acapella` |  |
| `turntable.fx3.typeIndexInSourceFavorites` | modelState | [Inferred] turntable / fx 3 / type Index In Source Favorites | `turntable.fx3SelectFavorite` |  |
| `turntable.fx3.typeIndexInSourcePack` | modelState | [Inferred] turntable / fx 3 / type Index In Source Pack | `turntable.fx3Select` |  |
| `turntable.fx3.wetDryValue` | modelValue | [Inferred] turntable / fx 3 / wet Dry Value | `turntable.fx3WetDryValue` |  |
| `turntable.fxActive` | modelState | FX Active | `turntable.fxActive` | Note CH1 data 32; Note CH2 data 32; Note CH3 data 32; Note CH4 data 32 |
| `turntable.highEQEffect.currentValue` | modelValue | [Inferred] turntable / high EQ Effect / current Value | `turntable.highEQ` |  |
| `turntable.highEQEffect.isReset` | modelState | [Inferred] turntable / high EQ Effect / is Reset | `turntable.highEQ`, `turntable.highEQKill`, `turntable.resetHighEQ` |  |
| `turntable.instantFx1.enabled` | modelState | [Inferred] turntable / instant Fx 1 / enabled | `turntable.instantFx1` | Note CH1 data 16; Note CH2 data 16; Note CH3 data 16; Note CH4 data 16 |
| `turntable.instantFx2.enabled` | modelState | [Inferred] turntable / instant Fx 2 / enabled | `turntable.instantFx2` | Note CH1 data 17; Note CH2 data 17; Note CH3 data 17; Note CH4 data 17 |
| `turntable.instantFx3.enabled` | modelState | [Inferred] turntable / instant Fx 3 / enabled | `turntable.instantFx3` | Note CH1 data 18; Note CH2 data 18; Note CH3 data 18; Note CH4 data 18 |
| `turntable.instantFx4.enabled` | modelState | [Inferred] turntable / instant Fx 4 / enabled | `turntable.instantFx4` | Note CH1 data 19; Note CH2 data 19; Note CH3 data 19; Note CH4 data 19 |
| `turntable.instantFx5.enabled` | modelState | [Inferred] turntable / instant Fx 5 / enabled | `turntable.instantFx5` |  |
| `turntable.instantFx6.enabled` | modelState | [Inferred] turntable / instant Fx 6 / enabled | `turntable.instantFx6` |  |
| `turntable.instantFx7.enabled` | modelState | [Inferred] turntable / instant Fx 7 / enabled | `turntable.instantFx7` |  |
| `turntable.instantFx8.enabled` | modelState | [Inferred] turntable / instant Fx 8 / enabled | `turntable.instantFx8` |  |
| `turntable.instantFxMain1.enabled` | modelState | [Inferred] turntable / instant Fx Main 1 / enabled | `turntable.instantFxMain1` |  |
| `turntable.instantFxMain2.enabled` | modelState | [Inferred] turntable / instant Fx Main 2 / enabled | `turntable.instantFxMain2` |  |
| `turntable.instantFxToggleMode` | modelState | Instant FX Toggle Mode | `turntable.instantFxToggleMode` |  |
| `turntable.isGatedCueEnabled` | modelState | [Inferred] turntable / is Gated Cue Enabled | `turntable.gatedCue` |  |
| `turntable.isKeyMatched` | modelState | [Inferred] turntable / is Key Matched | `turntable.matchKey` |  |
| `turntable.isMutedUntilCue` | modelState | [Inferred] turntable / is Muted Until Cue | `turntable.muteUntilCue` |  |
| `turntable.isPlayingButtonState` | modelState | [Inferred] turntable / is Playing Button State | `turntable.playPause` | Note CH1 data 0; Note CH2 data 0; Note CH3 data 0; Note CH4 data 0 |
| `turntable.isSyncMaster` | modelState | [Inferred] turntable / is Sync Master | `turntable.turntableIsSyncMaster` | Note CH1 data 8; Note CH2 data 8; Note CH3 data 8; Note CH4 data 8 |
| `turntable.isTraditionalCueSettableButtonState` | modelState | [Inferred] turntable / is Traditional Cue Settable Button State | `turntable.cuePositionOrJumpConsideringPlayState1` | Note CH1 data 1; Note CH2 data 1; Note CH3 data 1; Note CH4 data 1 |
| `turntable.jogPitchBendMode` | modelState | [Inferred] turntable / jog Pitch Bend Mode | `turntable.jogPitchBendMode`, `turntable.jogPitchBendModeToggle` |  |
| `turntable.jogSeekMode` | modelState | [Inferred] turntable / jog Seek Mode | `turntable.jogSeekMode`, `turntable.jogSeekModeToggle` |  |
| `turntable.loopingEnabledInView` | modelState | [Inferred] turntable / looping Enabled In View | `turntable.autoLoopOnOff`, `turntable.loopOut`, `turntable.loopOutAndReloopOrUnloop`, `turntable.loopOutOrDouble`, `turntable.loopingActive`, `turntable.loopingActiveAndReloopWhenOff` | Note CH1 data 6; Note CH2 data 6; Note CH3 data 6; Note CH4 data 6 |
| `turntable.loopInOrMoveInPointMidiButtonState` | modelState | [Inferred] turntable / loop In Or Move In Point Midi Button State | `turntable.loopInOrMoveInPoint` |  |
| `turntable.loopOutOrMoveOutPointMidiButtonState` | modelState | [Inferred] turntable / loop Out Or Move Out Point Midi Button State | `turntable.loopOutOrMoveOutPoint` |  |
| `turntable.lowEQEffect.currentValue` | modelValue | [Inferred] turntable / low EQ Effect / current Value | `turntable.lowEQ` |  |
| `turntable.lowEQEffect.isReset` | modelState | [Inferred] turntable / low EQ Effect / is Reset | `turntable.lowEQ`, `turntable.lowEQKill`, `turntable.resetLowEQ` |  |
| `turntable.midEQEffect.currentValue` | modelValue | [Inferred] turntable / mid EQ Effect / current Value | `turntable.midEQ` |  |
| `turntable.midEQEffect.isReset` | modelState | [Inferred] turntable / mid EQ Effect / is Reset | `turntable.midEQ`, `turntable.midEQKill`, `turntable.resetMidEQ` |  |
| `turntable.padFx.enabled` | modelState | [Inferred] turntable / pad Fx / enabled | `turntable.padFxEnabled`, `turntable.padFxParameterDefaultValue` |  |
| `turntable.padFx.parameterValue` | modelValue | [Inferred] turntable / pad Fx / parameter Value | `turntable.padFxParameterValue` |  |
| `turntable.padFx.routing.fourTrackChannel1` | modelState | [Inferred] turntable / pad Fx / routing / four Track Channel 1 | `turntable.padFxRouting.routingFourTrackChannel1` |  |
| `turntable.padFx.routing.fourTrackChannel2` | modelState | [Inferred] turntable / pad Fx / routing / four Track Channel 2 | `turntable.padFxRouting.routingFourTrackChannel2` |  |
| `turntable.padFx.routing.fourTrackChannel3` | modelState | [Inferred] turntable / pad Fx / routing / four Track Channel 3 | `turntable.padFxRouting.routingFourTrackChannel3` |  |
| `turntable.padFx.routing.fourTrackChannel4` | modelState | [Inferred] turntable / pad Fx / routing / four Track Channel 4 | `turntable.padFxRouting.routingFourTrackChannel4` |  |
| `turntable.padFx.routing.genericChannel1` | modelState | [Inferred] turntable / pad Fx / routing / generic Channel 1 | `turntable.padFxRouting.routingGenericChannel1` |  |
| `turntable.padFx.routing.genericChannel2` | modelState | [Inferred] turntable / pad Fx / routing / generic Channel 2 | `turntable.padFxRouting.routingGenericChannel2` |  |
| `turntable.padFx.routing.genericChannel3` | modelState | [Inferred] turntable / pad Fx / routing / generic Channel 3 | `turntable.padFxRouting.routingGenericChannel3` |  |
| `turntable.padFx.routing.genericChannel4` | modelState | [Inferred] turntable / pad Fx / routing / generic Channel 4 | `turntable.padFxRouting.routingGenericChannel4` |  |
| `turntable.padFx.routing.isDeck` | modelState | [Inferred] turntable / pad Fx / routing / is Deck | `turntable.padFxRouting.routingDeck`, `turntable.padFxRoutingDeck` |  |
| `turntable.padFx.routing.threeTrackChannel1` | modelState | [Inferred] turntable / pad Fx / routing / three Track Channel 1 | `turntable.padFxRouting.routingThreeTrackChannel1`, `turntable.padFxRoutingInstrumental` |  |
| `turntable.padFx.routing.threeTrackChannel2` | modelState | [Inferred] turntable / pad Fx / routing / three Track Channel 2 | `turntable.padFxRouting.routingThreeTrackChannel2`, `turntable.padFxRoutingHarmonic` |  |
| `turntable.padFx.routing.threeTrackChannel3` | modelState | [Inferred] turntable / pad Fx / routing / three Track Channel 3 | `turntable.padFxRouting.routingThreeTrackChannel3`, `turntable.padFxRoutingAcapella` |  |
| `turntable.padFx.routing.twoTrackChannel1` | modelState | [Inferred] turntable / pad Fx / routing / two Track Channel 1 | `turntable.padFxRouting.routingTwoTrackChannel1` |  |
| `turntable.padFx.routing.twoTrackChannel2` | modelState | [Inferred] turntable / pad Fx / routing / two Track Channel 2 | `turntable.padFxRouting.routingTwoTrackChannel2` |  |
| `turntable.padFx.wetDryValue` | modelValue | [Inferred] turntable / pad Fx / wet Dry Value | `turntable.padFxWetDryValue` |  |
| `turntable.pitchPlayButtonState1` | modelState | [Inferred] turntable / pitch Play Button State 1 | `turntable.pitchPlayAction1` |  |
| `turntable.pitchPlayButtonState2` | modelState | [Inferred] turntable / pitch Play Button State 2 | `turntable.pitchPlayAction2` |  |
| `turntable.pitchPlayButtonState3` | modelState | [Inferred] turntable / pitch Play Button State 3 | `turntable.pitchPlayAction3` |  |
| `turntable.pitchPlayButtonState4` | modelState | [Inferred] turntable / pitch Play Button State 4 | `turntable.pitchPlayAction4` |  |
| `turntable.pitchPlayButtonState5` | modelState | [Inferred] turntable / pitch Play Button State 5 | `turntable.pitchPlayAction5` |  |
| `turntable.pitchPlayButtonState6` | modelState | [Inferred] turntable / pitch Play Button State 6 | `turntable.pitchPlayAction6` |  |
| `turntable.pitchPlayButtonState7` | modelState | [Inferred] turntable / pitch Play Button State 7 | `turntable.pitchPlayAction7` |  |
| `turntable.pitchPlayButtonState8` | modelState | [Inferred] turntable / pitch Play Button State 8 | `turntable.pitchPlayAction8` |  |
| `turntable.pitchPlayCuePointState1` | modelState | [Inferred] turntable / pitch Play Cue Point State 1 | `turntable.pitchPlayCuePoint1` |  |
| `turntable.pitchPlayCuePointState2` | modelState | [Inferred] turntable / pitch Play Cue Point State 2 | `turntable.pitchPlayCuePoint2` |  |
| `turntable.pitchPlayCuePointState3` | modelState | [Inferred] turntable / pitch Play Cue Point State 3 | `turntable.pitchPlayCuePoint3` |  |
| `turntable.pitchPlayCuePointState4` | modelState | [Inferred] turntable / pitch Play Cue Point State 4 | `turntable.pitchPlayCuePoint4` |  |
| `turntable.pitchPlayCuePointState5` | modelState | [Inferred] turntable / pitch Play Cue Point State 5 | `turntable.pitchPlayCuePoint5` |  |
| `turntable.pitchPlayCuePointState6` | modelState | [Inferred] turntable / pitch Play Cue Point State 6 | `turntable.pitchPlayCuePoint6` |  |
| `turntable.pitchPlayCuePointState7` | modelState | [Inferred] turntable / pitch Play Cue Point State 7 | `turntable.pitchPlayCuePoint7` |  |
| `turntable.pitchPlayCuePointState8` | modelState | [Inferred] turntable / pitch Play Cue Point State 8 | `turntable.pitchPlayCuePoint8` |  |
| `turntable.pitchPlayCuePointStateCuePoint1` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 1 | `turntable.pitchPlayCuePointCuePoint1` |  |
| `turntable.pitchPlayCuePointStateCuePoint10` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 10 | `turntable.pitchPlayCuePointCuePoint10` |  |
| `turntable.pitchPlayCuePointStateCuePoint11` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 11 | `turntable.pitchPlayCuePointCuePoint11` |  |
| `turntable.pitchPlayCuePointStateCuePoint12` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 12 | `turntable.pitchPlayCuePointCuePoint12` |  |
| `turntable.pitchPlayCuePointStateCuePoint13` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 13 | `turntable.pitchPlayCuePointCuePoint13` |  |
| `turntable.pitchPlayCuePointStateCuePoint14` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 14 | `turntable.pitchPlayCuePointCuePoint14` |  |
| `turntable.pitchPlayCuePointStateCuePoint15` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 15 | `turntable.pitchPlayCuePointCuePoint15` |  |
| `turntable.pitchPlayCuePointStateCuePoint16` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 16 | `turntable.pitchPlayCuePointCuePoint16` |  |
| `turntable.pitchPlayCuePointStateCuePoint2` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 2 | `turntable.pitchPlayCuePointCuePoint2` |  |
| `turntable.pitchPlayCuePointStateCuePoint3` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 3 | `turntable.pitchPlayCuePointCuePoint3` |  |
| `turntable.pitchPlayCuePointStateCuePoint4` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 4 | `turntable.pitchPlayCuePointCuePoint4` |  |
| `turntable.pitchPlayCuePointStateCuePoint5` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 5 | `turntable.pitchPlayCuePointCuePoint5` |  |
| `turntable.pitchPlayCuePointStateCuePoint6` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 6 | `turntable.pitchPlayCuePointCuePoint6` |  |
| `turntable.pitchPlayCuePointStateCuePoint7` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 7 | `turntable.pitchPlayCuePointCuePoint7` |  |
| `turntable.pitchPlayCuePointStateCuePoint8` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 8 | `turntable.pitchPlayCuePointCuePoint8` |  |
| `turntable.pitchPlayCuePointStateCuePoint9` | modelState | [Inferred] turntable / pitch Play Cue Point State Cue Point 9 | `turntable.pitchPlayCuePointCuePoint9` |  |
| `turntable.pitchPlayRangeIsDown` | modelState | [Inferred] turntable / pitch Play Range Is Down | `turntable.pitchPlayRangeDown` |  |
| `turntable.pitchPlayRangeIsUp` | modelState | [Inferred] turntable / pitch Play Range Is Up | `turntable.pitchPlayRangeUp` |  |
| `turntable.preservesPitch` | modelState | [Inferred] turntable / preserves Pitch | `turntable.key` |  |
| `turntable.reverse` | modelState | Reverse | `turntable.censor`, `turntable.reverse`, `turntable.reverseHold` | Note CH1 data 3; Note CH2 data 3; Note CH3 data 3; Note CH4 data 3 |
| `turntable.shouldQuantizeCueJump` | modelState | [Inferred] turntable / should Quantize Cue Jump | `turntable.toggleQuantizeCueJump` |  |
| `turntable.shouldQuantizeLoopJump` | modelState | [Inferred] turntable / should Quantize Loop Jump | `turntable.toggleQuantizeLoopJump` |  |
| `turntable.shouldQuantizeSliceJump` | modelState | [Inferred] turntable / should Quantize Slice Jump | `turntable.toggleQuantizeSliceJump` |  |
| `turntable.skipping0125BeatInterval` | modelState | [Inferred] turntable / skipping 0125 Beat Interval | `turntable.skipDuration0125Beats` |  |
| `turntable.skipping025BeatInterval` | modelState | [Inferred] turntable / skipping 025 Beat Interval | `turntable.skipDuration025Beats` |  |
| `turntable.skipping05BeatInterval` | modelState | [Inferred] turntable / skipping 05 Beat Interval | `turntable.skipDuration05Beats` |  |
| `turntable.skipping16BeatInterval` | modelState | [Inferred] turntable / skipping 16 Beat Interval | `turntable.skipDuration16Beats` |  |
| `turntable.skipping1BeatInterval` | modelState | [Inferred] turntable / skipping 1 Beat Interval | `turntable.skipDuration1Beat` |  |
| `turntable.skipping2BeatInterval` | modelState | [Inferred] turntable / skipping 2 Beat Interval | `turntable.skipDuration2Beats` |  |
| `turntable.skipping32BeatInterval` | modelState | [Inferred] turntable / skipping 32 Beat Interval | `turntable.skipDuration32Beats` |  |
| `turntable.skipping4BeatInterval` | modelState | [Inferred] turntable / skipping 4 Beat Interval | `turntable.skipDuration4Beats` |  |
| `turntable.skipping64BeatInterval` | modelState | [Inferred] turntable / skipping 64 Beat Interval | `turntable.skipDuration64Beats` |  |
| `turntable.skipping8BeatInterval` | modelState | [Inferred] turntable / skipping 8 Beat Interval | `turntable.skipDuration8Beats` |  |
| `turntable.slicer4Slice1Active` | modelState | [Inferred] turntable / slicer 4 Slice 1 Active | `turntable.slicer4Slice1` |  |
| `turntable.slicer4Slice2Active` | modelState | [Inferred] turntable / slicer 4 Slice 2 Active | `turntable.slicer4Slice2` |  |
| `turntable.slicer4Slice3Active` | modelState | [Inferred] turntable / slicer 4 Slice 3 Active | `turntable.slicer4Slice3` |  |
| `turntable.slicer4Slice4Active` | modelState | [Inferred] turntable / slicer 4 Slice 4 Active | `turntable.slicer4Slice4` |  |
| `turntable.slicer8Slice1Active` | modelState | [Inferred] turntable / slicer 8 Slice 1 Active | `turntable.slicer8Slice1` |  |
| `turntable.slicer8Slice2Active` | modelState | [Inferred] turntable / slicer 8 Slice 2 Active | `turntable.slicer8Slice2` |  |
| `turntable.slicer8Slice3Active` | modelState | [Inferred] turntable / slicer 8 Slice 3 Active | `turntable.slicer8Slice3` |  |
| `turntable.slicer8Slice4Active` | modelState | [Inferred] turntable / slicer 8 Slice 4 Active | `turntable.slicer8Slice4` |  |
| `turntable.slicer8Slice5Active` | modelState | [Inferred] turntable / slicer 8 Slice 5 Active | `turntable.slicer8Slice5` |  |
| `turntable.slicer8Slice6Active` | modelState | [Inferred] turntable / slicer 8 Slice 6 Active | `turntable.slicer8Slice6` |  |
| `turntable.slicer8Slice7Active` | modelState | [Inferred] turntable / slicer 8 Slice 7 Active | `turntable.slicer8Slice7` |  |
| `turntable.slicer8Slice8Active` | modelState | [Inferred] turntable / slicer 8 Slice 8 Active | `turntable.slicer8Slice8` |  |
| `turntable.slicerLoopMode` | modelState | [Inferred] turntable / slicer Loop Mode | `turntable.slicerLoopModeToggle` |  |
| `turntable.song.automixEndPoint.hasStart` | modelState | [Inferred] turntable / song / automix End Point / has Start | `turntable.clearEndPoint`, `turntable.cueOrJumpIfAlreadySetEnd`, `turntable.gotoEnd`, `turntable.gotoEndConsideringPlayState` |  |
| `turntable.song.automixStartPoint.hasStart` | modelState | [Inferred] turntable / song / automix Start Point / has Start | `turntable.clearAutomixStartPoint`, `turntable.cueOrJumpIfAlreadySetAutomixStart`, `turntable.gotoAutomixStartConsideringPlayState` |  |
| `turntable.song.cuePoint1.hasStart` | modelState | [Inferred] turntable / song / cue Point 1 / has Start | `turntable.clearCuePointCuePoint1`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint1`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint1`, `turntable.cueOrJumpIfAlreadySetCuePoint1`, `turntable.jumpToCueConsideringPlayState1` |  |
| `turntable.song.cuePoint10.hasStart` | modelState | [Inferred] turntable / song / cue Point 10 / has Start | `turntable.clearCuePointCuePoint10`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint10`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint10`, `turntable.cueOrJumpIfAlreadySetCuePoint10` |  |
| `turntable.song.cuePoint11.hasStart` | modelState | [Inferred] turntable / song / cue Point 11 / has Start | `turntable.clearCuePointCuePoint11`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint11`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint11`, `turntable.cueOrJumpIfAlreadySetCuePoint11` |  |
| `turntable.song.cuePoint12.hasStart` | modelState | [Inferred] turntable / song / cue Point 12 / has Start | `turntable.clearCuePointCuePoint12`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint12`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint12`, `turntable.cueOrJumpIfAlreadySetCuePoint12` |  |
| `turntable.song.cuePoint13.hasStart` | modelState | [Inferred] turntable / song / cue Point 13 / has Start | `turntable.clearCuePointCuePoint13`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint13`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint13`, `turntable.cueOrJumpIfAlreadySetCuePoint13` |  |
| `turntable.song.cuePoint14.hasStart` | modelState | [Inferred] turntable / song / cue Point 14 / has Start | `turntable.clearCuePointCuePoint14`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint14`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint14`, `turntable.cueOrJumpIfAlreadySetCuePoint14` |  |
| `turntable.song.cuePoint15.hasStart` | modelState | [Inferred] turntable / song / cue Point 15 / has Start | `turntable.clearCuePointCuePoint15`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint15`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint15`, `turntable.cueOrJumpIfAlreadySetCuePoint15` |  |
| `turntable.song.cuePoint16.hasStart` | modelState | [Inferred] turntable / song / cue Point 16 / has Start | `turntable.clearCuePointCuePoint16`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint16`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint16`, `turntable.cueOrJumpIfAlreadySetCuePoint16` |  |
| `turntable.song.cuePoint2.hasStart` | modelState | [Inferred] turntable / song / cue Point 2 / has Start | `turntable.clearCuePointCuePoint2`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint2`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint2`, `turntable.cueOrJumpIfAlreadySetCuePoint2`, `turntable.jumpToCueConsideringPlayState2` |  |
| `turntable.song.cuePoint3.hasStart` | modelState | [Inferred] turntable / song / cue Point 3 / has Start | `turntable.clearCuePointCuePoint3`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint3`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint3`, `turntable.cueOrJumpIfAlreadySetCuePoint3`, `turntable.jumpToCueConsideringPlayState3` |  |
| `turntable.song.cuePoint4.hasStart` | modelState | [Inferred] turntable / song / cue Point 4 / has Start | `turntable.clearCuePointCuePoint4`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint4`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint4`, `turntable.cueOrJumpIfAlreadySetCuePoint4`, `turntable.jumpToCueConsideringPlayState4` |  |
| `turntable.song.cuePoint5.hasStart` | modelState | [Inferred] turntable / song / cue Point 5 / has Start | `turntable.clearCuePointCuePoint5`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint5`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint5`, `turntable.cueOrJumpIfAlreadySetCuePoint5`, `turntable.jumpToCueConsideringPlayState5` |  |
| `turntable.song.cuePoint6.hasStart` | modelState | [Inferred] turntable / song / cue Point 6 / has Start | `turntable.clearCuePointCuePoint6`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint6`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint6`, `turntable.cueOrJumpIfAlreadySetCuePoint6`, `turntable.jumpToCueConsideringPlayState6` |  |
| `turntable.song.cuePoint7.hasStart` | modelState | [Inferred] turntable / song / cue Point 7 / has Start | `turntable.clearCuePointCuePoint7`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint7`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint7`, `turntable.cueOrJumpIfAlreadySetCuePoint7`, `turntable.jumpToCueConsideringPlayState7` |  |
| `turntable.song.cuePoint8.hasStart` | modelState | [Inferred] turntable / song / cue Point 8 / has Start | `turntable.clearCuePointCuePoint8`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint8`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint8`, `turntable.cueOrJumpIfAlreadySetCuePoint8`, `turntable.jumpToCueConsideringPlayState8` |  |
| `turntable.song.cuePoint9.hasStart` | modelState | [Inferred] turntable / song / cue Point 9 / has Start | `turntable.clearCuePointCuePoint9`, `turntable.cueOrJumpIfAlreadySetAndAutoloopCuePoint9`, `turntable.cueOrJumpIfAlreadySetAndReloopCuePoint9`, `turntable.cueOrJumpIfAlreadySetCuePoint9` |  |
| `turntable.song.cuePointStart.hasStart` | modelState | [Inferred] turntable / song / cue Point Start / has Start | `turntable.clearStartPoint`, `turntable.cueOrJumpIfAlreadySetStart`, `turntable.gotoBeginning`, `turntable.gotoBeginningAndPause`, `turntable.gotoBeginningAndPlay`, `turntable.gotoBeginningConsideringPlayState` |  |
| `turntable.song.loopRegion1.hasRange` | modelState | [Inferred] turntable / song / loop Region 1 / has Range | `turntable.saveLoopIfLoopingOrActivate1`, `turntable.saveLoopIfLoopingOrReloop1`, `turntable.saveLoopOrActivate1` |  |
| `turntable.song.loopRegion2.hasRange` | modelState | [Inferred] turntable / song / loop Region 2 / has Range | `turntable.saveLoopIfLoopingOrActivate2`, `turntable.saveLoopIfLoopingOrReloop2`, `turntable.saveLoopOrActivate2` |  |
| `turntable.song.loopRegion3.hasRange` | modelState | [Inferred] turntable / song / loop Region 3 / has Range | `turntable.saveLoopIfLoopingOrActivate3`, `turntable.saveLoopIfLoopingOrReloop3`, `turntable.saveLoopOrActivate3` |  |
| `turntable.song.loopRegion4.hasRange` | modelState | [Inferred] turntable / song / loop Region 4 / has Range | `turntable.saveLoopIfLoopingOrActivate4`, `turntable.saveLoopIfLoopingOrReloop4`, `turntable.saveLoopOrActivate4` |  |
| `turntable.song.loopRegion5.hasRange` | modelState | [Inferred] turntable / song / loop Region 5 / has Range | `turntable.saveLoopIfLoopingOrActivate5`, `turntable.saveLoopIfLoopingOrReloop5`, `turntable.saveLoopOrActivate5` |  |
| `turntable.song.loopRegion6.hasRange` | modelState | [Inferred] turntable / song / loop Region 6 / has Range | `turntable.saveLoopIfLoopingOrActivate6`, `turntable.saveLoopIfLoopingOrReloop6`, `turntable.saveLoopOrActivate6` |  |
| `turntable.song.loopRegion7.hasRange` | modelState | [Inferred] turntable / song / loop Region 7 / has Range | `turntable.saveLoopIfLoopingOrActivate7`, `turntable.saveLoopIfLoopingOrReloop7`, `turntable.saveLoopOrActivate7` |  |
| `turntable.song.loopRegion8.hasRange` | modelState | [Inferred] turntable / song / loop Region 8 / has Range | `turntable.saveLoopIfLoopingOrActivate8`, `turntable.saveLoopIfLoopingOrReloop8`, `turntable.saveLoopOrActivate8` |  |
| `turntable.song.showLoopRegionLiveUpdateOrLoopingInView` | modelState | [Inferred] turntable / song / show Loop Region Live Update Or Looping In View | `turntable.loopIn`, `turntable.loopInOut` |  |
| `turntable.syncButtonState` | modelState | [Inferred] turntable / sync Button State | `turntable.bpmSync` | Note CH1 data 2; Note CH2 data 2; Note CH3 data 2; Note CH4 data 2 |
| `turntable.unmixer.acapellaMuted` | modelState | [Inferred] turntable / unmixer / acapella Muted | `turntable.unmixerAcapellaMuted` |  |
| `turntable.unmixer.acapellaSoloActive` | modelState | [Inferred] turntable / unmixer / acapella Solo Active | `turntable.unmixerAcapellaSolo` |  |
| `turntable.unmixer.acapellaSwapped` | modelState | [Inferred] turntable / unmixer / acapella Swapped | `turntable.unmixerAcapellaSwap` |  |
| `turntable.unmixer.acapellaVolume` | modelValue | [Inferred] turntable / unmixer / acapella Volume | `turntable.unmixerAcapellaVolume` |  |
| `turntable.unmixer.crossfadeLeftSwitchActive` | modelState | [Inferred] turntable / unmixer / crossfade Left Switch Active | `turntable.unmixerCrossfadeSwitchLeft`, `turntable.unmixerCrossfadeSwitchLeftInstrumental` |  |
| `turntable.unmixer.crossfadePercussiveMode` | modelState | [Inferred] turntable / unmixer / crossfade Percussive Mode | `turntable.toggleUnmixerCrossfadePercussiveMode` |  |
| `turntable.unmixer.crossfadeRightSwitchActive` | modelState | [Inferred] turntable / unmixer / crossfade Right Switch Active | `turntable.unmixerCrossfadeSwitchRight`, `turntable.unmixerCrossfadeSwitchRightAcapella` |  |
| `turntable.unmixer.crossfadeValue` | modelValue | [Inferred] turntable / unmixer / crossfade Value | `turntable.unmixerCrossfade` |  |
| `turntable.unmixer.eqFourTrackChannel1VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Four Track Channel 1 Volume Effect / current Value | `turntable.unmixerFourTrackChannel1VolumeEQ` |  |
| `turntable.unmixer.eqFourTrackChannel1VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Four Track Channel 1 Volume Effect / is Reset | `turntable.unmixerFourTrackChannel1VolumeEQ` |  |
| `turntable.unmixer.eqFourTrackChannel2VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Four Track Channel 2 Volume Effect / current Value | `turntable.unmixerFourTrackChannel2VolumeEQ` |  |
| `turntable.unmixer.eqFourTrackChannel2VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Four Track Channel 2 Volume Effect / is Reset | `turntable.unmixerFourTrackChannel2VolumeEQ` |  |
| `turntable.unmixer.eqFourTrackChannel3VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Four Track Channel 3 Volume Effect / current Value | `turntable.unmixerFourTrackChannel3VolumeEQ` |  |
| `turntable.unmixer.eqFourTrackChannel3VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Four Track Channel 3 Volume Effect / is Reset | `turntable.unmixerFourTrackChannel3VolumeEQ` |  |
| `turntable.unmixer.eqFourTrackChannel4VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Four Track Channel 4 Volume Effect / current Value | `turntable.unmixerFourTrackChannel4VolumeEQ` |  |
| `turntable.unmixer.eqFourTrackChannel4VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Four Track Channel 4 Volume Effect / is Reset | `turntable.unmixerFourTrackChannel4VolumeEQ` |  |
| `turntable.unmixer.eqGenericChannel1VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Generic Channel 1 Volume Effect / current Value | `turntable.unmixerGenericChannel1VolumeEQ` |  |
| `turntable.unmixer.eqGenericChannel1VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Generic Channel 1 Volume Effect / is Reset | `turntable.unmixerGenericChannel1VolumeEQ` |  |
| `turntable.unmixer.eqGenericChannel2VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Generic Channel 2 Volume Effect / current Value | `turntable.unmixerGenericChannel2VolumeEQ` |  |
| `turntable.unmixer.eqGenericChannel2VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Generic Channel 2 Volume Effect / is Reset | `turntable.unmixerGenericChannel2VolumeEQ` |  |
| `turntable.unmixer.eqGenericChannel3VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Generic Channel 3 Volume Effect / current Value | `turntable.unmixerGenericChannel3VolumeEQ` |  |
| `turntable.unmixer.eqGenericChannel3VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Generic Channel 3 Volume Effect / is Reset | `turntable.unmixerGenericChannel3VolumeEQ` |  |
| `turntable.unmixer.eqGenericChannel4VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Generic Channel 4 Volume Effect / current Value | `turntable.unmixerGenericChannel4VolumeEQ` |  |
| `turntable.unmixer.eqGenericChannel4VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Generic Channel 4 Volume Effect / is Reset | `turntable.unmixerGenericChannel4VolumeEQ` |  |
| `turntable.unmixer.eqThreeTrackChannel1VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Three Track Channel 1 Volume Effect / current Value | `turntable.unmixerEQDrums`, `turntable.unmixerThreeTrackChannel1VolumeEQ` |  |
| `turntable.unmixer.eqThreeTrackChannel1VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Three Track Channel 1 Volume Effect / is Reset | `turntable.unmixerEQDrums`, `turntable.unmixerThreeTrackChannel1VolumeEQ` |  |
| `turntable.unmixer.eqThreeTrackChannel2VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Three Track Channel 2 Volume Effect / current Value | `turntable.unmixerEQHarmonic`, `turntable.unmixerThreeTrackChannel2VolumeEQ` |  |
| `turntable.unmixer.eqThreeTrackChannel2VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Three Track Channel 2 Volume Effect / is Reset | `turntable.unmixerEQHarmonic`, `turntable.unmixerThreeTrackChannel2VolumeEQ` |  |
| `turntable.unmixer.eqThreeTrackChannel3VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Three Track Channel 3 Volume Effect / current Value | `turntable.unmixerEQAcapella`, `turntable.unmixerThreeTrackChannel3VolumeEQ` |  |
| `turntable.unmixer.eqThreeTrackChannel3VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Three Track Channel 3 Volume Effect / is Reset | `turntable.unmixerEQAcapella`, `turntable.unmixerThreeTrackChannel3VolumeEQ` |  |
| `turntable.unmixer.eqTwoTrackChannel1VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Two Track Channel 1 Volume Effect / current Value | `turntable.unmixerTwoTrackChannel1VolumeEQ` |  |
| `turntable.unmixer.eqTwoTrackChannel1VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Two Track Channel 1 Volume Effect / is Reset | `turntable.unmixerTwoTrackChannel1VolumeEQ` |  |
| `turntable.unmixer.eqTwoTrackChannel2VolumeEffect.currentValue` | modelValue | [Inferred] turntable / unmixer / eq Two Track Channel 2 Volume Effect / current Value | `turntable.unmixerTwoTrackChannel2VolumeEQ` |  |
| `turntable.unmixer.eqTwoTrackChannel2VolumeEffect.isReset` | modelState | [Inferred] turntable / unmixer / eq Two Track Channel 2 Volume Effect / is Reset | `turntable.unmixerTwoTrackChannel2VolumeEQ` |  |
| `turntable.unmixer.fourTrackChannel1Active` | modelState | [Inferred] turntable / unmixer / four Track Channel 1 Active | `turntable.unmixerFourTrackChannel1Active` |  |
| `turntable.unmixer.fourTrackChannel1Muted` | modelState | [Inferred] turntable / unmixer / four Track Channel 1 Muted | `turntable.unmixerFourTrackChannel1Muted` | Note CH1 data 56; Note CH2 data 56; Note CH3 data 56; Note CH4 data 56 |
| `turntable.unmixer.fourTrackChannel1Solo` | modelState | [Inferred] turntable / unmixer / four Track Channel 1 Solo | `turntable.unmixerFourTrackChannel1Solo`, `turntable.unmixerFourTrackChannel1SoloExclusive` | Note CH1 data 60; Note CH2 data 60; Note CH3 data 60; Note CH4 data 60 |
| `turntable.unmixer.fourTrackChannel1Swapped` | modelState | [Inferred] turntable / unmixer / four Track Channel 1 Swapped | `turntable.unmixerFourTrackChannel1Swapped` |  |
| `turntable.unmixer.fourTrackChannel1Volume` | modelValue | [Inferred] turntable / unmixer / four Track Channel 1 Volume | `turntable.unmixerFourTrackChannel1Volume` |  |
| `turntable.unmixer.fourTrackChannel2Active` | modelState | [Inferred] turntable / unmixer / four Track Channel 2 Active | `turntable.unmixerFourTrackChannel2Active` |  |
| `turntable.unmixer.fourTrackChannel2Muted` | modelState | [Inferred] turntable / unmixer / four Track Channel 2 Muted | `turntable.unmixerFourTrackChannel2Muted` | Note CH1 data 57; Note CH2 data 57; Note CH3 data 57; Note CH4 data 57 |
| `turntable.unmixer.fourTrackChannel2Solo` | modelState | [Inferred] turntable / unmixer / four Track Channel 2 Solo | `turntable.unmixerFourTrackChannel2Solo`, `turntable.unmixerFourTrackChannel2SoloExclusive` | Note CH1 data 61; Note CH2 data 61; Note CH3 data 61; Note CH4 data 61 |
| `turntable.unmixer.fourTrackChannel2Swapped` | modelState | [Inferred] turntable / unmixer / four Track Channel 2 Swapped | `turntable.unmixerFourTrackChannel2Swapped` |  |
| `turntable.unmixer.fourTrackChannel2Volume` | modelValue | [Inferred] turntable / unmixer / four Track Channel 2 Volume | `turntable.unmixerFourTrackChannel2Volume` |  |
| `turntable.unmixer.fourTrackChannel3Active` | modelState | [Inferred] turntable / unmixer / four Track Channel 3 Active | `turntable.unmixerFourTrackChannel3Active` |  |
| `turntable.unmixer.fourTrackChannel3Muted` | modelState | [Inferred] turntable / unmixer / four Track Channel 3 Muted | `turntable.unmixerFourTrackChannel3Muted` | Note CH1 data 58; Note CH2 data 58; Note CH3 data 58; Note CH4 data 58 |
| `turntable.unmixer.fourTrackChannel3Solo` | modelState | [Inferred] turntable / unmixer / four Track Channel 3 Solo | `turntable.unmixerFourTrackChannel3Solo`, `turntable.unmixerFourTrackChannel3SoloExclusive` | Note CH1 data 62; Note CH2 data 62; Note CH3 data 62; Note CH4 data 62 |
| `turntable.unmixer.fourTrackChannel3Swapped` | modelState | [Inferred] turntable / unmixer / four Track Channel 3 Swapped | `turntable.unmixerFourTrackChannel3Swapped` |  |
| `turntable.unmixer.fourTrackChannel3Volume` | modelValue | [Inferred] turntable / unmixer / four Track Channel 3 Volume | `turntable.unmixerFourTrackChannel3Volume` |  |
| `turntable.unmixer.fourTrackChannel4Active` | modelState | [Inferred] turntable / unmixer / four Track Channel 4 Active | `turntable.unmixerFourTrackChannel4Active` |  |
| `turntable.unmixer.fourTrackChannel4Muted` | modelState | [Inferred] turntable / unmixer / four Track Channel 4 Muted | `turntable.unmixerFourTrackChannel4Muted` | Note CH1 data 59; Note CH2 data 59; Note CH3 data 59; Note CH4 data 59 |
| `turntable.unmixer.fourTrackChannel4Solo` | modelState | [Inferred] turntable / unmixer / four Track Channel 4 Solo | `turntable.unmixerFourTrackChannel4Solo`, `turntable.unmixerFourTrackChannel4SoloExclusive` | Note CH1 data 63; Note CH2 data 63; Note CH3 data 63; Note CH4 data 63 |
| `turntable.unmixer.fourTrackChannel4Swapped` | modelState | [Inferred] turntable / unmixer / four Track Channel 4 Swapped | `turntable.unmixerFourTrackChannel4Swapped` |  |
| `turntable.unmixer.fourTrackChannel4Volume` | modelValue | [Inferred] turntable / unmixer / four Track Channel 4 Volume | `turntable.unmixerFourTrackChannel4Volume` |  |
| `turntable.unmixer.genericChannel1Active` | modelState | [Inferred] turntable / unmixer / generic Channel 1 Active | `turntable.unmixerGenericChannel1Active` |  |
| `turntable.unmixer.genericChannel1Muted` | modelState | [Inferred] turntable / unmixer / generic Channel 1 Muted | `turntable.unmixerGenericChannel1Muted` |  |
| `turntable.unmixer.genericChannel1Solo` | modelState | [Inferred] turntable / unmixer / generic Channel 1 Solo | `turntable.unmixerGenericChannel1Solo`, `turntable.unmixerGenericChannel1SoloExclusive` |  |
| `turntable.unmixer.genericChannel1Volume` | modelValue | [Inferred] turntable / unmixer / generic Channel 1 Volume | `turntable.unmixerGenericChannel1Volume` |  |
| `turntable.unmixer.genericChannel2Active` | modelState | [Inferred] turntable / unmixer / generic Channel 2 Active | `turntable.unmixerGenericChannel2Active` |  |
| `turntable.unmixer.genericChannel2Muted` | modelState | [Inferred] turntable / unmixer / generic Channel 2 Muted | `turntable.unmixerGenericChannel2Muted` |  |
| `turntable.unmixer.genericChannel2Solo` | modelState | [Inferred] turntable / unmixer / generic Channel 2 Solo | `turntable.unmixerGenericChannel2Solo`, `turntable.unmixerGenericChannel2SoloExclusive` |  |
| `turntable.unmixer.genericChannel2Volume` | modelValue | [Inferred] turntable / unmixer / generic Channel 2 Volume | `turntable.unmixerGenericChannel2Volume` |  |
| `turntable.unmixer.genericChannel3Active` | modelState | [Inferred] turntable / unmixer / generic Channel 3 Active | `turntable.unmixerGenericChannel3Active` |  |
| `turntable.unmixer.genericChannel3Muted` | modelState | [Inferred] turntable / unmixer / generic Channel 3 Muted | `turntable.unmixerGenericChannel3Muted` |  |
| `turntable.unmixer.genericChannel3Solo` | modelState | [Inferred] turntable / unmixer / generic Channel 3 Solo | `turntable.unmixerGenericChannel3Solo`, `turntable.unmixerGenericChannel3SoloExclusive` |  |
| `turntable.unmixer.genericChannel3Volume` | modelValue | [Inferred] turntable / unmixer / generic Channel 3 Volume | `turntable.unmixerGenericChannel3Volume` |  |
| `turntable.unmixer.genericChannel4Active` | modelState | [Inferred] turntable / unmixer / generic Channel 4 Active | `turntable.unmixerGenericChannel4Active` |  |
| `turntable.unmixer.genericChannel4Muted` | modelState | [Inferred] turntable / unmixer / generic Channel 4 Muted | `turntable.unmixerGenericChannel4Muted` |  |
| `turntable.unmixer.genericChannel4Solo` | modelState | [Inferred] turntable / unmixer / generic Channel 4 Solo | `turntable.unmixerGenericChannel4Solo`, `turntable.unmixerGenericChannel4SoloExclusive` |  |
| `turntable.unmixer.genericChannel4Volume` | modelValue | [Inferred] turntable / unmixer / generic Channel 4 Volume | `turntable.unmixerGenericChannel4Volume` |  |
| `turntable.unmixer.harmonicSwapped` | modelState | [Inferred] turntable / unmixer / harmonic Swapped | `turntable.unmixerHarmonicSwap` |  |
| `turntable.unmixer.instrumentalMuted` | modelState | [Inferred] turntable / unmixer / instrumental Muted | `turntable.unmixerDrumsMuted`, `turntable.unmixerInstrumentalMuted` |  |
| `turntable.unmixer.instrumentalSoloActive` | modelState | [Inferred] turntable / unmixer / instrumental Solo Active | `turntable.unmixerDrumsSolo`, `turntable.unmixerInstrumentalSolo` |  |
| `turntable.unmixer.instrumentalSwapped` | modelState | [Inferred] turntable / unmixer / instrumental Swapped | `turntable.unmixerDrumsSwap` |  |
| `turntable.unmixer.instrumentalVolume` | modelValue | [Inferred] turntable / unmixer / instrumental Volume | `turntable.unmixerDrumsVolume`, `turntable.unmixerInstrumentalVolume` |  |
| `turntable.unmixer.muteWithFx` | modelState | [Inferred] turntable / unmixer / mute With Fx | `turntable.toggleUnmixerMuteFxEnabled` |  |
| `turntable.unmixer.percussiveMode` | modelState | [Inferred] turntable / unmixer / percussive Mode | `turntable.toggleUnmixerPercussiveMode` |  |
| `turntable.unmixer.threeTrackChannel1Active` | modelState | [Inferred] turntable / unmixer / three Track Channel 1 Active | `turntable.unmixerThreeTrackChannel1Active` |  |
| `turntable.unmixer.threeTrackChannel1Muted` | modelState | [Inferred] turntable / unmixer / three Track Channel 1 Muted | `turntable.unmixerThreeTrackChannel1Muted` |  |
| `turntable.unmixer.threeTrackChannel1Solo` | modelState | [Inferred] turntable / unmixer / three Track Channel 1 Solo | `turntable.unmixerThreeTrackChannel1Solo`, `turntable.unmixerThreeTrackChannel1SoloExclusive` |  |
| `turntable.unmixer.threeTrackChannel1Swapped` | modelState | [Inferred] turntable / unmixer / three Track Channel 1 Swapped | `turntable.unmixerThreeTrackChannel1Swapped` |  |
| `turntable.unmixer.threeTrackChannel1Volume` | modelValue | [Inferred] turntable / unmixer / three Track Channel 1 Volume | `turntable.unmixerThreeTrackChannel1Volume` |  |
| `turntable.unmixer.threeTrackChannel2Active` | modelState | [Inferred] turntable / unmixer / three Track Channel 2 Active | `turntable.unmixerThreeTrackChannel2Active` |  |
| `turntable.unmixer.threeTrackChannel2Muted` | modelState | [Inferred] turntable / unmixer / three Track Channel 2 Muted | `turntable.unmixerHarmonicMuted`, `turntable.unmixerThreeTrackChannel2Muted` |  |
| `turntable.unmixer.threeTrackChannel2Solo` | modelState | [Inferred] turntable / unmixer / three Track Channel 2 Solo | `turntable.unmixerHarmonicSolo`, `turntable.unmixerThreeTrackChannel2Solo`, `turntable.unmixerThreeTrackChannel2SoloExclusive` |  |
| `turntable.unmixer.threeTrackChannel2Swapped` | modelState | [Inferred] turntable / unmixer / three Track Channel 2 Swapped | `turntable.unmixerThreeTrackChannel2Swapped` |  |
| `turntable.unmixer.threeTrackChannel2Volume` | modelValue | [Inferred] turntable / unmixer / three Track Channel 2 Volume | `turntable.unmixerHarmonicVolume`, `turntable.unmixerThreeTrackChannel2Volume` |  |
| `turntable.unmixer.threeTrackChannel3Active` | modelState | [Inferred] turntable / unmixer / three Track Channel 3 Active | `turntable.unmixerThreeTrackChannel3Active` |  |
| `turntable.unmixer.threeTrackChannel3Muted` | modelState | [Inferred] turntable / unmixer / three Track Channel 3 Muted | `turntable.unmixerThreeTrackChannel3Muted` |  |
| `turntable.unmixer.threeTrackChannel3Solo` | modelState | [Inferred] turntable / unmixer / three Track Channel 3 Solo | `turntable.unmixerThreeTrackChannel3Solo`, `turntable.unmixerThreeTrackChannel3SoloExclusive` |  |
| `turntable.unmixer.threeTrackChannel3Swapped` | modelState | [Inferred] turntable / unmixer / three Track Channel 3 Swapped | `turntable.unmixerThreeTrackChannel3Swapped` |  |
| `turntable.unmixer.threeTrackChannel3Volume` | modelValue | [Inferred] turntable / unmixer / three Track Channel 3 Volume | `turntable.unmixerThreeTrackChannel3Volume` |  |
| `turntable.unmixer.twoTrackChannel1Active` | modelState | [Inferred] turntable / unmixer / two Track Channel 1 Active | `turntable.unmixerTwoTrackChannel1Active` |  |
| `turntable.unmixer.twoTrackChannel1Muted` | modelState | [Inferred] turntable / unmixer / two Track Channel 1 Muted | `turntable.unmixerTwoTrackChannel1Muted` |  |
| `turntable.unmixer.twoTrackChannel1Solo` | modelState | [Inferred] turntable / unmixer / two Track Channel 1 Solo | `turntable.unmixerTwoTrackChannel1Solo`, `turntable.unmixerTwoTrackChannel1SoloExclusive` |  |
| `turntable.unmixer.twoTrackChannel1Swapped` | modelState | [Inferred] turntable / unmixer / two Track Channel 1 Swapped | `turntable.unmixerTwoTrackChannel1Swapped` |  |
| `turntable.unmixer.twoTrackChannel1Volume` | modelValue | [Inferred] turntable / unmixer / two Track Channel 1 Volume | `turntable.unmixerTwoTrackChannel1Volume` |  |
| `turntable.unmixer.twoTrackChannel2Active` | modelState | [Inferred] turntable / unmixer / two Track Channel 2 Active | `turntable.unmixerTwoTrackChannel2Active` |  |
| `turntable.unmixer.twoTrackChannel2Muted` | modelState | [Inferred] turntable / unmixer / two Track Channel 2 Muted | `turntable.unmixerTwoTrackChannel2Muted` |  |
| `turntable.unmixer.twoTrackChannel2Solo` | modelState | [Inferred] turntable / unmixer / two Track Channel 2 Solo | `turntable.unmixerTwoTrackChannel2Solo`, `turntable.unmixerTwoTrackChannel2SoloExclusive` |  |
| `turntable.unmixer.twoTrackChannel2Swapped` | modelState | [Inferred] turntable / unmixer / two Track Channel 2 Swapped | `turntable.unmixerTwoTrackChannel2Swapped` |  |
| `turntable.unmixer.twoTrackChannel2Volume` | modelValue | [Inferred] turntable / unmixer / two Track Channel 2 Volume | `turntable.unmixerTwoTrackChannel2Volume` |  |
| `turntable.useFxBpmValue` | modelState | [Inferred] turntable / use Fx Bpm Value | `turntable.fxBpmTap` |  |
| `turntable.view.state.jogwheelSlice` | modelState | [Inferred] turntable / view / state / jogwheel Slice | `turntable.jogwheelSlice` |  |
| `turntable.view.state.showTools` | modelState | [Inferred] turntable / view / state / show Tools | `turntable.toggleShowTools` |  |
| `turntable.view.state.showUnmixerCrossfader` | modelState | [Inferred] turntable / view / state / show Unmixer Crossfader | `turntable.toggleShowUnmixerPopup` |  |
| `turntable.view.state.tool1.cuePointsType` | modelValue | [Inferred] turntable / view / state / tool 1 / cue Points Type | `turntable.selectTool1CuePoints` |  |
| `turntable.view.state.tool1.effectsType` | modelValue | [Inferred] turntable / view / state / tool 1 / effects Type | `turntable.selectTool1Effects` |  |
| `turntable.view.state.tool1.isCuePointsSectionSelected` | modelState | [Inferred] turntable / view / state / tool 1 / is Cue Points Section Selected | `turntable.selectTool1CuePointsSection` |  |
| `turntable.view.state.tool1.isEffectsSectionSelected` | modelState | [Inferred] turntable / view / state / tool 1 / is Effects Section Selected | `turntable.selectTool1EffectsSection` |  |
| `turntable.view.state.tool1.isEQSectionSelected` | modelState | [Inferred] turntable / view / state / tool 1 / is EQ Section Selected | `turntable.selectTool1EQSection` |  |
| `turntable.view.state.tool1.isLoopingSectionSelected` | modelState | [Inferred] turntable / view / state / tool 1 / is Looping Section Selected | `turntable.selectTool1LoopingSection` |  |
| `turntable.view.state.tool1.isUnmixerSectionSelected` | modelState | [Inferred] turntable / view / state / tool 1 / is Unmixer Section Selected | `turntable.selectTool1UnmixerSection` |  |
| `turntable.view.state.tool1.loopingType` | modelValue | [Inferred] turntable / view / state / tool 1 / looping Type | `turntable.selectTool1Looping` |  |
| `turntable.view.state.tool1.type` | modelValue | [Inferred] turntable / view / state / tool 1 / type | `turntable.selectTool1` |  |
| `turntable.view.state.tool2.cuePointsType` | modelValue | [Inferred] turntable / view / state / tool 2 / cue Points Type | `turntable.selectTool2CuePoints` |  |
| `turntable.view.state.tool2.effectsType` | modelValue | [Inferred] turntable / view / state / tool 2 / effects Type | `turntable.selectTool2Effects` |  |
| `turntable.view.state.tool2.isCuePointsSectionSelected` | modelState | [Inferred] turntable / view / state / tool 2 / is Cue Points Section Selected | `turntable.selectTool2CuePointsSection` |  |
| `turntable.view.state.tool2.isEffectsSectionSelected` | modelState | [Inferred] turntable / view / state / tool 2 / is Effects Section Selected | `turntable.selectTool2EffectsSection` |  |
| `turntable.view.state.tool2.isLoopingSectionSelected` | modelState | [Inferred] turntable / view / state / tool 2 / is Looping Section Selected | `turntable.selectTool2LoopingSection` |  |
| `turntable.view.state.tool2.isUnmixerSectionSelected` | modelState | [Inferred] turntable / view / state / tool 2 / is Unmixer Section Selected | `turntable.selectTool2UnmixerSection` |  |
| `turntable.view.state.tool2.loopingType` | modelValue | [Inferred] turntable / view / state / tool 2 / looping Type | `turntable.selectTool2Looping` |  |
| `turntable.view.state.tool2.type` | modelValue | [Inferred] turntable / view / state / tool 2 / type | `turntable.selectTool2` |  |
| `turntable.view.state.unmixerCrossfaderModeSlider` | modelState | [Inferred] turntable / view / state / unmixer Crossfader Mode Slider | `turntable.toggleShowUnmixerSliderPopup` |  |
| `turntable.view.state.waveSlice` | modelState | [Inferred] turntable / view / state / wave Slice | `turntable.toggleWaveSlice`, `turntable.waveSlice` |  |
| `turntable.waveformAlternateMode` | modelState | [Inferred] turntable / waveform Alternate Mode | `turntable.toggleWaveformAlternateMode` |  |
| `turntable.waveSlip` | modelState | [Inferred] turntable / wave Slip | `turntable.waveSlip` |  |
| `turntable1.deckQuantize` | modelState | [Inferred] turntable 1 / deck Quantize | `application.quantize`, `application.quantizeToggle` |  |
| `turntable1.jogPitchBendMode` | modelState | [Inferred] turntable 1 / jog Pitch Bend Mode | `application.jogPitchBendMode`, `application.jogPitchBendModeToggle` |  |
| `turntable1.jogSeekMode` | modelState | [Inferred] turntable 1 / jog Seek Mode | `application.jogSeekMode`, `application.jogSeekModeToggle` |  |
| `turntable1.unmixer.muteWithFx` | modelState | [Inferred] turntable 1 / unmixer / mute With Fx | `mixer.toggleUnmixerMuteFxEnabled` |  |
| `turntable1Selected` | modelState | [Inferred] turntable 1 Selected | `application.turntable1Selected` |  |
| `turntable2Selected` | modelState | [Inferred] turntable 2 Selected | `application.turntable2Selected`, `application.turntableToggleSelected` |  |
| `turntable3Selected` | modelState | [Inferred] turntable 3 Selected | `application.turntable3Selected` |  |
| `turntable4Selected` | modelState | [Inferred] turntable 4 Selected | `application.turntable4Selected` |  |
| `userDefaults.DJMidiCrossfadeCuttingMode` | modelState | [Inferred] user Defaults / DJ Midi Crossfade Cutting Mode | `mixer.midiCrossfadeCuttingModeToggle` |  |
| `userDefaults.DJMidiHamsterSwitch` | modelState | [Inferred] user Defaults / DJ Midi Hamster Switch | `mixer.midiHamsterSwitchToggle` |  |
| `userDefaults.VJVideoCrossfadeModeSeparate` | modelValue | [Inferred] user Defaults / VJ Video Crossfade Mode Separate | `mixer.videoCrossfadeModeSeparate` |  |
| `view.isAutomixViewMode` | modelState | [Inferred] view / is Automix View Mode | `application.viewModeOneDeckAutomix` |  |
| `view.isFourDeckMode` | modelState | [Inferred] view / is Four Deck Mode | `application.viewModeFourDeckWaveform` |  |
| `view.isVideoMode` | modelState | [Inferred] view / is Video Mode | `application.viewModeVideo` |  |
| `view.isViewModeOneDeckTurntable` | modelState | [Inferred] view / is View Mode One Deck Turntable | `application.viewModeOneDeckTurntable` |  |
| `view.isViewModeTwoDeckPro` | modelState | [Inferred] view / is View Mode Two Deck Pro | `application.viewModeTwoDeckPro` |  |
| `view.isViewModeTwoDeckSequencer` | modelState | [Inferred] view / is View Mode Two Deck Sequencer | `application.viewModeTwoDeckSequencer` |  |
| `view.isViewModeTwoDeckSimple` | modelState | [Inferred] view / is View Mode Two Deck Simple | `application.viewModeTwoDeckSimple` |  |
| `view.isViewModeTwoDeckTurntable` | modelState | [Inferred] view / is View Mode Two Deck Turntable | `application.viewModeTwoDeckTurntable` |  |
| `view.state.canResetWaveformZoomFactor` | modelState | [Inferred] view / state / can Reset Waveform Zoom Factor | `application.waveformZoomFactorReset` |  |
| `view.state.modeIndex` | modelState | [Inferred] view / state / mode Index | `application.viewModeSelect` |  |
| `view.state.showCueBar` | modelState | [Inferred] view / state / show Cue Bar | `application.toggleShowCueBar` |  |
| `view.state.showFXBar` | modelState | [Inferred] view / state / show FX Bar | `application.toggleShowEffects` |  |
| `view.state.showLooper` | modelState | [Inferred] view / state / show Looper | `looper.showLooper`, `looper.toggleLooperShown` |  |
| `view.state.showSampler` | modelState | [Inferred] view / state / show Sampler | `sampler.showSampler`, `sampler.toggleSamplerShown` |  |
| `view.state.showTools` | modelState | [Inferred] view / state / show Tools | `application.toggleShowTools` |  |
| `view.state.showUnmixerBar` | modelState | [Inferred] view / state / show Unmixer Bar | `application.toggleShowUnmixerBar` |  |
| `view.state.showUnmixerCrossfader` | modelState | [Inferred] view / state / show Unmixer Crossfader | `application.toggleShowUnmixerPopup` |  |
| `view.state.showWaveform` | modelState | [Inferred] view / state / show Waveform | `application.toggleShowWaveforms` |  |
| `view.state.waveformZoomFactor` | modelState | [Inferred] view / state / waveform Zoom Factor | `application.toggleWaveformZoomed`, `application.waveformZoomed` |  |
| `view.state.waveSlice` | modelState | [Inferred] view / state / wave Slice | `application.toggleWaveSlice` |  |

## Top-level output key paths observed in installed presets

One row per unique `outputs[].keyPath` across the 210 scanned preset files. These paths are distinct from embedded `controls[].output` dictionaries. “Preset output records” is the number of matching entries across the scanned preset files; “S4 top-level address” is blank unless this path occurs in the generated S4 mapping. Controller-specific `customProcessor.*` paths are included as observed and are not generalized.

| Djay top-level output key path | Human description | Preset output records | S4 top-level address |
|---|---|---:|---|
| `customProcessor.beatFxDisplayIndex` | [Inferred] custom Processor / beat Fx Display Index | 1 |  |
| `customProcessor.beatLEDController.beatMatchingDeckOneAhead` | [Inferred] custom Processor / beat LED Controller / beat Matching Deck One Ahead | 2 |  |
| `customProcessor.beatLEDController.beatMatchingDeckTwoAhead` | [Inferred] custom Processor / beat LED Controller / beat Matching Deck Two Ahead | 2 |  |
| `customProcessor.beatLEDController.beatMatchingIsMatched` | [Inferred] custom Processor / beat LED Controller / beat Matching Is Matched | 2 |  |
| `customProcessor.bpmShouldBeDisplayed` | [Inferred] custom Processor / bpm Should Be Displayed | 1 |  |
| `customProcessor.deck1Meter` | [Inferred] custom Processor / deck 1 Meter | 1 |  |
| `customProcessor.deck2Meter` | [Inferred] custom Processor / deck 2 Meter | 1 |  |
| `customProcessor.deck3Meter` | [Inferred] custom Processor / deck 3 Meter | 1 |  |
| `customProcessor.deck4Meter` | [Inferred] custom Processor / deck 4 Meter | 1 |  |
| `customProcessor.mixtrackGoPadMode1LoopLED` | [Inferred] custom Processor / mixtrack Go Pad Mode 1 Loop LED | 1 |  |
| `customProcessor.mixtrackGoPadMode1SampleLED` | [Inferred] custom Processor / mixtrack Go Pad Mode 1 Sample LED | 1 |  |
| `customProcessor.mixtrackGoPadMode2LoopLED` | [Inferred] custom Processor / mixtrack Go Pad Mode 2 Loop LED | 1 |  |
| `customProcessor.mixtrackGoPadMode2SampleLED` | [Inferred] custom Processor / mixtrack Go Pad Mode 2 Sample LED | 1 |  |
| `customProcessor.partyMix3PadMode1LoopLED` | [Inferred] custom Processor / party Mix 3 Pad Mode 1 Loop LED | 1 |  |
| `customProcessor.partyMix3PadMode1SampleLED` | [Inferred] custom Processor / party Mix 3 Pad Mode 1 Sample LED | 1 |  |
| `customProcessor.partyMix3PadMode2LoopLED` | [Inferred] custom Processor / party Mix 3 Pad Mode 2 Loop LED | 1 |  |
| `customProcessor.partyMix3PadMode2SampleLED` | [Inferred] custom Processor / party Mix 3 Pad Mode 2 Sample LED | 1 |  |
| `customProcessor.pitchFaderLEDModelLeft.arrowDownLED` | [Inferred] custom Processor / pitch Fader LED Model Left / arrow Down LED | 1 |  |
| `customProcessor.pitchFaderLEDModelLeft.arrowUpLED` | [Inferred] custom Processor / pitch Fader LED Model Left / arrow Up LED | 1 |  |
| `customProcessor.pitchFaderLEDModelLeft.centerLED` | [Inferred] custom Processor / pitch Fader LED Model Left / center LED | 1 |  |
| `customProcessor.pitchFaderLEDModelRight.arrowDownLED` | [Inferred] custom Processor / pitch Fader LED Model Right / arrow Down LED | 1 |  |
| `customProcessor.pitchFaderLEDModelRight.arrowUpLED` | [Inferred] custom Processor / pitch Fader LED Model Right / arrow Up LED | 1 |  |
| `customProcessor.pitchFaderLEDModelRight.centerLED` | [Inferred] custom Processor / pitch Fader LED Model Right / center LED | 1 |  |
| `midiModel.turntable1.padModeIsHotCue` | [Inferred] midi Model / turntable 1 / pad Mode Is Hot Cue | 2 |  |
| `midiModel.turntable1.padModeIsUnmixer` | [Inferred] midi Model / turntable 1 / pad Mode Is Unmixer | 2 |  |
| `midiModel.turntable2.padModeIsHotCue` | [Inferred] midi Model / turntable 2 / pad Mode Is Hot Cue | 2 |  |
| `midiModel.turntable2.padModeIsUnmixer` | [Inferred] midi Model / turntable 2 / pad Mode Is Unmixer | 2 |  |
| `mixer.masterLeftMeter` | [Inferred] mixer / master Left Meter | 7 |  |
| `mixer.masterRightMeter` | [Inferred] mixer / master Right Meter | 7 |  |
| `modifierState.modifier1` | [Inferred] modifier State / modifier 1 | 3 |  |
| `sampler.turntable1.player1.loadingSuccess` | [Inferred] sampler / turntable 1 / player 1 / loading Success | 1 |  |
| `sampler.turntable1.player1.statePlaying` | [Inferred] sampler / turntable 1 / player 1 / state Playing | 4 |  |
| `sampler.turntable1.player2.loadingSuccess` | [Inferred] sampler / turntable 1 / player 2 / loading Success | 1 |  |
| `sampler.turntable1.player2.statePlaying` | [Inferred] sampler / turntable 1 / player 2 / state Playing | 4 |  |
| `sampler.turntable1.player3.loadingSuccess` | [Inferred] sampler / turntable 1 / player 3 / loading Success | 1 |  |
| `sampler.turntable1.player3.statePlaying` | [Inferred] sampler / turntable 1 / player 3 / state Playing | 4 |  |
| `sampler.turntable1.player4.loadingSuccess` | [Inferred] sampler / turntable 1 / player 4 / loading Success | 1 |  |
| `sampler.turntable1.player4.statePlaying` | [Inferred] sampler / turntable 1 / player 4 / state Playing | 4 |  |
| `sampler.turntable1.player5.statePlaying` | [Inferred] sampler / turntable 1 / player 5 / state Playing | 1 |  |
| `sampler.turntable1.player6.statePlaying` | [Inferred] sampler / turntable 1 / player 6 / state Playing | 1 |  |
| `sampler.turntable1.player7.statePlaying` | [Inferred] sampler / turntable 1 / player 7 / state Playing | 1 |  |
| `sampler.turntable1.player8.statePlaying` | [Inferred] sampler / turntable 1 / player 8 / state Playing | 1 |  |
| `sampler.turntable2.player1.statePlaying` | [Inferred] sampler / turntable 2 / player 1 / state Playing | 4 |  |
| `sampler.turntable2.player2.statePlaying` | [Inferred] sampler / turntable 2 / player 2 / state Playing | 4 |  |
| `sampler.turntable2.player3.statePlaying` | [Inferred] sampler / turntable 2 / player 3 / state Playing | 4 |  |
| `sampler.turntable2.player4.statePlaying` | [Inferred] sampler / turntable 2 / player 4 / state Playing | 4 |  |
| `sampler.turntable2.player5.statePlaying` | [Inferred] sampler / turntable 2 / player 5 / state Playing | 1 |  |
| `sampler.turntable2.player6.statePlaying` | [Inferred] sampler / turntable 2 / player 6 / state Playing | 1 |  |
| `sampler.turntable2.player7.statePlaying` | [Inferred] sampler / turntable 2 / player 7 / state Playing | 1 |  |
| `sampler.turntable2.player8.statePlaying` | [Inferred] sampler / turntable 2 / player 8 / state Playing | 1 |  |
| `turntable1.deckSlip` | [Inferred] turntable 1 / deck Slip | 3 |  |
| `turntable1.deckSlipButtonState` | [Inferred] turntable 1 / deck Slip Button State | 2 |  |
| `turntable1.display.isApproachingEndOfSong` | [Inferred] turntable 1 / display / is Approaching End Of Song | 3 |  |
| `turntable1.display.showElapsedTime` | [Inferred] turntable 1 / display / show Elapsed Time | 5 |  |
| `turntable1.display.songProgress` | [Inferred] turntable 1 / display / song Progress | 7 |  |
| `turntable1.effects.tempo.isReset` | [Inferred] turntable 1 / effects / tempo / is Reset | 1 |  |
| `turntable1.isAudible` | [Inferred] turntable 1 / is Audible | 1 |  |
| `turntable1.isPlaying` | [Inferred] turntable 1 / is Playing | 5 |  |
| `turntable1.isPlayingButtonState` | [Inferred] turntable 1 / is Playing Button State | 2 |  |
| `turntable1.isPlayingRamped` | [Inferred] turntable 1 / is Playing Ramped | 5 |  |
| `turntable1.isSyncMaster` | [Inferred] turntable 1 / is Sync Master | 3 |  |
| `turntable1.isTouchOnTurntable` | [Inferred] turntable 1 / is Touch On Turntable | 5 |  |
| `turntable1.jogPitchBendMode` | [Inferred] turntable 1 / jog Pitch Bend Mode | 1 |  |
| `turntable1.jogSeekMode` | [Inferred] turntable 1 / jog Seek Mode | 1 |  |
| `turntable1.loadingSuccess` | [Inferred] turntable 1 / loading Success | 2 |  |
| `turntable1.looping` | [Inferred] turntable 1 / looping | 1 |  |
| `turntable1.loopingEnabledInView` | [Inferred] turntable 1 / looping Enabled In View | 2 |  |
| `turntable1.monoMeter` | [Inferred] turntable 1 / mono Meter | 54 | CC CH7 data 0 (control) |
| `turntable1.preservesPitch` | [Inferred] turntable 1 / preserves Pitch | 9 |  |
| `turntable1.slipMode` | [Inferred] turntable 1 / slip Mode | 1 |  |
| `turntable1.song.cuePointStart.hasStart` | [Inferred] turntable 1 / song / cue Point Start / has Start | 1 |  |
| `turntable1.song.cueRegionActive1` | [Inferred] turntable 1 / song / cue Region Active 1 | 1 |  |
| `turntable1.song.cueRegionActive2` | [Inferred] turntable 1 / song / cue Region Active 2 | 1 |  |
| `turntable1.song.cueRegionActive3` | [Inferred] turntable 1 / song / cue Region Active 3 | 1 |  |
| `turntable1.song.loadingSuccess` | [Inferred] turntable 1 / song / loading Success | 22 | Note CH7 data 0 |
| `turntable1.song.masterLoopRegion.hasRange` | [Inferred] turntable 1 / song / master Loop Region / has Range | 1 |  |
| `turntable1.syncButtonState` | [Inferred] turntable 1 / sync Button State | 1 |  |
| `turntable1.syncMode` | [Inferred] turntable 1 / sync Mode | 2 |  |
| `turntable2.deckSlip` | [Inferred] turntable 2 / deck Slip | 3 |  |
| `turntable2.deckSlipButtonState` | [Inferred] turntable 2 / deck Slip Button State | 2 |  |
| `turntable2.display.isApproachingEndOfSong` | [Inferred] turntable 2 / display / is Approaching End Of Song | 3 |  |
| `turntable2.display.showElapsedTime` | [Inferred] turntable 2 / display / show Elapsed Time | 5 |  |
| `turntable2.display.songProgress` | [Inferred] turntable 2 / display / song Progress | 7 |  |
| `turntable2.effects.tempo.isReset` | [Inferred] turntable 2 / effects / tempo / is Reset | 1 |  |
| `turntable2.isAudible` | [Inferred] turntable 2 / is Audible | 1 |  |
| `turntable2.isPlaying` | [Inferred] turntable 2 / is Playing | 5 |  |
| `turntable2.isPlayingButtonState` | [Inferred] turntable 2 / is Playing Button State | 2 |  |
| `turntable2.isPlayingRamped` | [Inferred] turntable 2 / is Playing Ramped | 5 |  |
| `turntable2.isSyncMaster` | [Inferred] turntable 2 / is Sync Master | 3 |  |
| `turntable2.isTouchOnTurntable` | [Inferred] turntable 2 / is Touch On Turntable | 5 |  |
| `turntable2.jogPitchBendMode` | [Inferred] turntable 2 / jog Pitch Bend Mode | 1 |  |
| `turntable2.jogSeekMode` | [Inferred] turntable 2 / jog Seek Mode | 1 |  |
| `turntable2.loadingSuccess` | [Inferred] turntable 2 / loading Success | 2 |  |
| `turntable2.looping` | [Inferred] turntable 2 / looping | 1 |  |
| `turntable2.loopingEnabledInView` | [Inferred] turntable 2 / looping Enabled In View | 2 |  |
| `turntable2.monoMeter` | [Inferred] turntable 2 / mono Meter | 54 | CC CH7 data 1 (control) |
| `turntable2.preservesPitch` | [Inferred] turntable 2 / preserves Pitch | 9 |  |
| `turntable2.slipMode` | [Inferred] turntable 2 / slip Mode | 1 |  |
| `turntable2.song.cuePointStart.hasStart` | [Inferred] turntable 2 / song / cue Point Start / has Start | 1 |  |
| `turntable2.song.cueRegionActive1` | [Inferred] turntable 2 / song / cue Region Active 1 | 1 |  |
| `turntable2.song.cueRegionActive2` | [Inferred] turntable 2 / song / cue Region Active 2 | 1 |  |
| `turntable2.song.cueRegionActive3` | [Inferred] turntable 2 / song / cue Region Active 3 | 1 |  |
| `turntable2.song.loadingSuccess` | [Inferred] turntable 2 / song / loading Success | 22 | Note CH7 data 1 |
| `turntable2.song.masterLoopRegion.hasRange` | [Inferred] turntable 2 / song / master Loop Region / has Range | 1 |  |
| `turntable2.syncButtonState` | [Inferred] turntable 2 / sync Button State | 1 |  |
| `turntable2.syncMode` | [Inferred] turntable 2 / sync Mode | 2 |  |
| `turntable3.deckSlip` | [Inferred] turntable 3 / deck Slip | 2 |  |
| `turntable3.deckSlipButtonState` | [Inferred] turntable 3 / deck Slip Button State | 2 |  |
| `turntable3.display.isApproachingEndOfSong` | [Inferred] turntable 3 / display / is Approaching End Of Song | 1 |  |
| `turntable3.display.showElapsedTime` | [Inferred] turntable 3 / display / show Elapsed Time | 5 |  |
| `turntable3.display.songProgress` | [Inferred] turntable 3 / display / song Progress | 7 |  |
| `turntable3.effects.tempo.isReset` | [Inferred] turntable 3 / effects / tempo / is Reset | 1 |  |
| `turntable3.isAudible` | [Inferred] turntable 3 / is Audible | 1 |  |
| `turntable3.isPlaying` | [Inferred] turntable 3 / is Playing | 3 |  |
| `turntable3.isPlayingRamped` | [Inferred] turntable 3 / is Playing Ramped | 3 |  |
| `turntable3.isSyncMaster` | [Inferred] turntable 3 / is Sync Master | 3 |  |
| `turntable3.isTouchOnTurntable` | [Inferred] turntable 3 / is Touch On Turntable | 5 |  |
| `turntable3.jogPitchBendMode` | [Inferred] turntable 3 / jog Pitch Bend Mode | 1 |  |
| `turntable3.loadingSuccess` | [Inferred] turntable 3 / loading Success | 1 |  |
| `turntable3.looping` | [Inferred] turntable 3 / looping | 1 |  |
| `turntable3.monoMeter` | [Inferred] turntable 3 / mono Meter | 26 | CC CH7 data 2 (control) |
| `turntable3.preservesPitch` | [Inferred] turntable 3 / preserves Pitch | 5 |  |
| `turntable3.slipMode` | [Inferred] turntable 3 / slip Mode | 1 |  |
| `turntable3.song.cuePointStart.hasStart` | [Inferred] turntable 3 / song / cue Point Start / has Start | 1 |  |
| `turntable3.song.loadingSuccess` | [Inferred] turntable 3 / song / loading Success | 18 | Note CH7 data 2 |
| `turntable3.song.masterLoopRegion.hasRange` | [Inferred] turntable 3 / song / master Loop Region / has Range | 1 |  |
| `turntable3.syncButtonState` | [Inferred] turntable 3 / sync Button State | 1 |  |
| `turntable3.syncMode` | [Inferred] turntable 3 / sync Mode | 2 |  |
| `turntable4.deckSlip` | [Inferred] turntable 4 / deck Slip | 2 |  |
| `turntable4.deckSlipButtonState` | [Inferred] turntable 4 / deck Slip Button State | 2 |  |
| `turntable4.display.isApproachingEndOfSong` | [Inferred] turntable 4 / display / is Approaching End Of Song | 1 |  |
| `turntable4.display.showElapsedTime` | [Inferred] turntable 4 / display / show Elapsed Time | 5 |  |
| `turntable4.display.songProgress` | [Inferred] turntable 4 / display / song Progress | 7 |  |
| `turntable4.effects.tempo.isReset` | [Inferred] turntable 4 / effects / tempo / is Reset | 1 |  |
| `turntable4.isAudible` | [Inferred] turntable 4 / is Audible | 1 |  |
| `turntable4.isPlaying` | [Inferred] turntable 4 / is Playing | 3 |  |
| `turntable4.isPlayingRamped` | [Inferred] turntable 4 / is Playing Ramped | 3 |  |
| `turntable4.isSyncMaster` | [Inferred] turntable 4 / is Sync Master | 3 |  |
| `turntable4.isTouchOnTurntable` | [Inferred] turntable 4 / is Touch On Turntable | 5 |  |
| `turntable4.jogPitchBendMode` | [Inferred] turntable 4 / jog Pitch Bend Mode | 1 |  |
| `turntable4.loadingSuccess` | [Inferred] turntable 4 / loading Success | 1 |  |
| `turntable4.looping` | [Inferred] turntable 4 / looping | 1 |  |
| `turntable4.monoMeter` | [Inferred] turntable 4 / mono Meter | 26 | CC CH7 data 3 (control) |
| `turntable4.preservesPitch` | [Inferred] turntable 4 / preserves Pitch | 5 |  |
| `turntable4.slipMode` | [Inferred] turntable 4 / slip Mode | 1 |  |
| `turntable4.song.cuePointStart.hasStart` | [Inferred] turntable 4 / song / cue Point Start / has Start | 1 |  |
| `turntable4.song.loadingSuccess` | [Inferred] turntable 4 / song / loading Success | 18 | Note CH7 data 3 |
| `turntable4.song.masterLoopRegion.hasRange` | [Inferred] turntable 4 / song / master Loop Region / has Range | 1 |  |
| `turntable4.syncButtonState` | [Inferred] turntable 4 / sync Button State | 1 |  |
| `turntable4.syncMode` | [Inferred] turntable 4 / sync Mode | 2 |  |
| `turntableSelected.autoLoop1BeatIntervalActive` | [Inferred] turntable Selected / auto Loop 1 Beat Interval Active | 2 |  |
| `turntableSelected.autoLoop2BeatIntervalActive` | [Inferred] turntable Selected / auto Loop 2 Beat Interval Active | 2 |  |
| `turntableSelected.autoLoop4BeatIntervalActive` | [Inferred] turntable Selected / auto Loop 4 Beat Interval Active | 2 |  |
| `turntableSelected.autoLoop8BeatIntervalActive` | [Inferred] turntable Selected / auto Loop 8 Beat Interval Active | 2 |  |
| `turntableSelected.deckSlip` | [Inferred] turntable Selected / deck Slip | 4 |  |
| `turntableSelected.displayKeyIndex` | [Inferred] turntable Selected / display Key Index | 3 |  |
| `turntableSelected.effects.pitch.currentValue` | [Inferred] turntable Selected / effects / pitch / current Value | 3 |  |
| `turntableSelected.effects.tempo.isReset` | [Inferred] turntable Selected / effects / tempo / is Reset | 4 |  |
| `turntableSelected.isSlipping` | [Inferred] turntable Selected / is Slipping | 2 |  |
| `turntableSelected.isTouchOnTurntable` | [Inferred] turntable Selected / is Touch On Turntable | 1 |  |
| `turntableSelected.isTraditionalCueSettable` | [Inferred] turntable Selected / is Traditional Cue Settable | 4 |  |
| `turntableSelected.looping` | [Inferred] turntable Selected / looping | 9 |  |
| `turntableSelected.movingLoopInPoint` | [Inferred] turntable Selected / moving Loop In Point | 1 |  |
| `turntableSelected.movingLoopOutPoint` | [Inferred] turntable Selected / moving Loop Out Point | 1 |  |
| `turntableSelected.reverse` | Reverse | 1 |  |
| `turntableSelected.song.keyIndex` | [Inferred] turntable Selected / song / key Index | 3 |  |
| `turntableSelected.song.loadingSuccess` | [Inferred] turntable Selected / song / loading Success | 35 |  |
| `turntableSelected.syncButtonState` | [Inferred] turntable Selected / sync Button State | 1 |  |
| `turntableSelected.syncMode` | [Inferred] turntable Selected / sync Mode | 3 |  |

## Source evidence paths

- `/Applications/djay Pro.app/Contents/Info.plist` (installed version)
- `/Applications/djay Pro.app/Contents/Resources/MidiModelMetadata.plist` (944 action/control/rotary metadata entries and their model references)
- `/Applications/djay Pro.app/Contents/Resources/MidiModelKeyPath.strings` (UTF-16 localized key-path labels)
- `/Applications/djay Pro.app/Contents/Resources/MIDI Mappings/` (all 210 scanned and indexed native preset mappings; preset catalog: `index.plist`)
- `src/full/catalog.rs` (physical control catalog and deck/pad/FX family expansion)
- `src/full/mapping.rs` (mapping schema, MIDI addresses, output generation)
- `src/full/feedback.rs` (accepted feedback channels/types and state decoding)
- `dist/S4 MK3 Bridge.app/Contents/Resources/S4 MK3 Bridge.djayMidiMapping` (generated bundled mapping used for the S4 address examples)
