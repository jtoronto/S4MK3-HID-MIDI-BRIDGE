# **Architectural and Technical Specification: Implementing a Traktor S4 MK3 HID-MIDI Bridge for Serato DJ Pro**

## Independent source review and bridge-specific assessment (October 30, 2026)

### Decision

Serato support is technically plausible through a separate XML mapping, but the
existing Djay mapping is not convertible by renaming its actions. Keep the
current Djay profile intact. Before adding a Serato mode or an XML generator,
test one copied Serato mapping with the existing bridge's left Play MIDI
message. Treat platter control, reactive LEDs, DVS/THRU audio and motorized
playback as separate experiments.

This review checks the claims that affect this bridge against current Serato
support pages, the supplied community XML examples, and the current source
tree. The Gemini-generated research below is preserved for provenance; claims
not specifically confirmed or corrected here should be treated as leads, not
as an implementation specification.

### What Serato documents, and what the XML community adds

Serato DJ Pro's supported MIDI workflow is in Setup > MIDI. A generic device
is enabled with **Enable MIDI Device**. **Allow Serato Hardware Remapping**
enables remapping for primary Serato hardware; it is not the checkbox required
to enable a generic MIDI device. The official guide describes third-party MIDI
controllers as secondary to primary Serato hardware. MIDI Assign lets the user
click a software control and press, turn or move the controller. A new preset
clears the current MIDI settings, so work only in a separate preset. The
official guides describe saving/loading presets and output lighting for
supported functions; output lighting is enabled per mapping with `L`.
([MIDI mapping with Serato DJ Pro](https://support.serato.com/hc/en-us/articles/209377487-MIDI-mapping-with-Serato-DJ-Pro);
[MIDI settings and output lighting](https://support.serato.com/hc/en-us/articles/223446808-MIDI))

Serato also documents five MIDI data modes: Absolute, Relative Signed Bit,
Relative Binary Offset, Relative 2's Complement and Relative On/Off. Which
modes appear depends on the mapped software function; the guide says to try
the relative modes for the specific controller. This is stronger evidence for
relative encoder feasibility than the Gemini draft's guesses, but it does not
identify the correct S4 encoding, sensitivity or Serato action for each
encoder. It also does not establish paired 14-bit pitch-fader support.
([MIDI Data Types](https://support.serato.com/hc/en-us/articles/225065768-MIDI-Data-Types))

The community [SERATO-XML-WIKI](https://github.com/Kovarsk/SERATO-XML-WIKI)
and [DJ TechTools XML guide](https://djtechtools.com/2018/04/11/hacking-serato-djs-midi-mapping-jogwheels-touchstrips-and-modifiers/)
provide useful reverse-engineered XML examples. The wiki's action list includes
`play`, library scrolling, FX and Stems examples; it explicitly marks some
items untested. Examples demonstrate `<control>`, `<userio event="click">`,
`<translation>` and action-specific `<userio event="output">` feedback.
This is a practical community reference, not a Serato-published complete
schema or a tested S4 preset.

Keep the versions and workflows distinct:

- The community articles show an enclosing `<midi>` element, hand-edited
  control fragments, and a community-reported `Music/_Serato_/MIDI/Xml` path.
  Copying a fragment does not prove it is a complete, importable profile for
  the current Serato release. Save a seed preset in the installed version and
  work on a copy.
- Serato's official “How to load custom MIDI Presets” page describes Scratch
  Live and an older `_Serato_/MIDI` path. Do not present that page as current
  Serato DJ Pro XML-generation documentation. Current Pro docs support
  managing presets in Setup > MIDI. ([historical custom-preset article](https://support.serato.com/hc/en-us/articles/219823647-How-to-load-custom-MIDI-Presets))
- The XML examples use device-specific MIDI channels, controls, relative
  encodings and ranges. The wiki README's “Relative 2's Complement” label and
  one FX example's “Relative Binary Offset” value are not interchangeable
  proof of S4 behavior. Several wiki actions are explicitly marked untested.
- Current Serato's official interface supports output lighting only for
  supported functions/devices. Community examples show output aliases for
  selected actions such as Play or Stems; they do not establish comprehensive
  state output, meters, or MIDI return to this bridge's virtual MIDI
  destination.

### Verified fit with this bridge

| Bridge surface | Current implementation | Serato assessment |
|---|---|---|
| HID decoding and physical controls | S4 report parsing and named physical events are separate from DJay targets (`src/main.rs`, `src/input.rs`, `src/full/catalog.rs`). | Reusable. No Serato mapping needs to know the S4's HID report layout. |
| MIDI port and ordinary controls | The full bridge exposes `S4 MK3 MIDI Full`; deck A Play is Note On channel 1/note 0/value 127 and Note Off channel 1/note 0/value 0 (`src/full/mod.rs:73-83, 257-262`). | Good first pilot. The port's visibility and Learn behavior in Serato remain untested. |
| Relative controls | The full translator emits relative CC values centered at 64, splitting large deltas into bounded messages (`src/full/mod.rs:1590-1597`); the catalog assigns MOVE, LOOP, Browse, jog, FX and other controls (`src/full/catalog.rs:126-190, 255-365`). | Serato has relative data modes, but its target functions and relative encoding must be selected/tested per control. Do not assume the Djay rotary action names transfer. |
| Generated mapping | `src/full/mapping.rs:1-75` serializes a Djay plist. Catalog entries carry Djay key paths, not software-neutral action identifiers (`src/full/catalog.rs:126-190`). | Requires a separate Serato XML output/profile or manual Serato preset. Never mutate the Djay plist to contain Serato XML. |
| Runtime gestures | Shift/deck/held-note ownership is bridge logic, but loop, jump, FX, preview and feedback behaviors are designed around Djay actions (`src/full/mod.rs`, `docs/mapping-audit.md:89-126`). | Reuse physical gesture interpretation only after mapping its meaning to Serato. The one-beat jump/restore workaround specifically responds to Djay's jump-size behavior; do not port it without a Serato test. |
| Software feedback and LEDs | `src/full/feedback.rs:29-185` and `src/full/feedback/info.rs:1-89` decode Djay-addressed play/load/meter/FX/THRU/jump/loop/key state. `src/full/leds.rs` and `src/full/output.rs` render S4 HID reports from that state. | MIDI decoding and S4 report writing are reusable patterns; current addresses and state assumptions are not. Community XML shows some action feedback, but no tested Serato-to-bridge state map. |
| Jog, screens and motor | Jog actions, platter-position CC, playback authorization, screen text and TT motor gates depend on Djay action/state data (`src/full/jog.rs`, `src/full/platter_position.rs`, `src/full/mod.rs:1385-1420`, `src/full/screens/key.rs`). | Experimental. A Serato feedback adapter would need real evidence for loaded/playing/mode states before it could drive motor or screen features. |
| Mixer audio, EXT and DVS | The bridge is HID/MIDI, not an audio router. Current EXT binds to Djay Internal/THRU actions (`docs/mapping-audit.md:31-45`). | Do not infer Serato THRU audio from a MIDI action or a changed UI label. Serato documents THRU as hardware-dependent and its virtual-deck modes as requiring DVS-enabled or DVS-upgrade-ready hardware. The S4 MK3 is absent from Serato's current hardware list. ([Virtual Deck Play Modes](https://support.serato.com/hc/en-us/articles/360004125475-Virtual-Deck-Play-Modes); [Through Mode](https://support.serato.com/hc/en-us/articles/234760848-Through-Mode); [hardware list](https://serato.com/dj/hardware)) |

### Source-level MIDI address inventory

The addresses below are the bridge's current full-profile inputs, before any
Serato action is selected. MIDI channel numbers are one-based here. Shifted
bindings use channels 9-12; do not copy Djay action names or generated plist
attributes into a Serato XML map. The source is `src/full/catalog.rs` plus
`src/full/mod.rs`; exact Serato XML actions remain a separate mapping decision.

| Physical function | Bridge message addresses | Implementation note |
|---|---|---|
| Play, Cue, Sync, Reverse, Flux | Notes 0-4 on deck channels 1-4 | Press is Note On velocity 127; release is Note Off velocity 0. Shifted Cue has a different bridge target but reuses Note 1 on channels 9-12. |
| Fixed one-beat forward/back | Notes 5 and 14 on deck channels 1-4 | Legacy aliases in the Djay map; current Shift+MOVE runtime does not use these as its jump operation. |
| LOOP press; wheel touch; master | Notes 6, 7 and 8 on deck channels 1-4 | Shifted LOOP press uses Note 6 on shifted channels for Djay Reloop. |
| Tempo fader | CC 0 on deck channels 1-4 | Absolute 7-bit MIDI; pickup is configured by the Djay map. |
| LOOP, MOVE, Browse encoders | Relative CC 1, 2 and 3 on deck channels 1-4 | Values are centered at 64; shifted LOOP uses CC 1 with a Djay move-loop target. Browse direction is flipped in the mapping. |
| Jog bend, scratch and seek | Relative CC 4, 5 and 6 on deck channels 1-4 | Wheel touch is a separate Note 7 event. The absolute platter-position mapping is CC 8 and is a separate Djay binding. |
| MOVE jump-size selector | Relative CC 7 on deck channels 1-4 | The selector press is bridge-local and emits no MIDI. The encoder's size-selection behavior is not itself a native MIDI action. |
| FX strips | Left strip Notes 32-35 / CC 32-37; right strip Notes 36-39 / CC 40-45 | Controls are deck-targeted; effect-browse encoders use channels 9-12. FX values and target selection are currently interpreted by Djay-specific feedback/runtime state. |
| Mixer faders, crossfader, gain, EQ and filter | Channel 5: CC 0-20 and CC 21-25 | CC 16-19 are channel faders; CC 20 is crossfader; CC 0-15 are gain/EQ; CC 21-24 are quick filter; CC 25 is crossfader curve. |
| Mixer PFL and Quick FX | Channel 5 Notes 0-3 for PFL; deck channels 1-4 Notes 16-19 for Quick FX | Quick FX is routed to a locally selected deck in the bridge, not directly to an audio bus. |
| Hotcue, sampler and Stems pads | Notes 40-47, 48-55 and 56-63 on deck channels 1-4 | Shifted pad commands use channels 9-12 where the catalog defines them. These are the bridge's Djay layer assignments, not Serato's default pad-mode layout. |
| Deck selection and crossfader assignment | Channel 6 Notes 0-3 select decks; channel 5 Notes 80-91 select assignment states | The address range is available to reuse only if Serato can bind those incoming messages to the intended software controls. |
| EXT Internal/THRU | Notes 27/28 on deck channels 1-4; feedback CC 21 | The actions are Djay DVS controls. Do not reuse their behavior for Serato without a supported input/audio route. |

The full Djay map also sends feedback addresses to the bridge: deck channels
1-4 use CC 32-37 for FX, 80-89 for jump size, 90-111 for active loop size,
and 112 for key; channel 7 carries meter/load/play state and tempo-range
feedback. `src/full/feedback.rs` interprets these as Djay-specific states.
This address layout is not a Serato contract. Serato XML examples demonstrate
some action-state outputs, but the bridge would need a separate Serato decoder
and a capture test before driving its LED/motor/screen state from them.

### Corrections to consequential claims in the Gemini draft

| Draft claim | Evidence-based status |
|---|---|
| The S4 HID protocol is “heavily encrypted.” | Not substantiated by the cited sources and inconsistent with this repository's report parsing/field-level decoding. Call it proprietary HID; do not claim encryption without evidence. The project is Rust plus a Swift macOS app, not a C++/Swift bridge. |
| Generic MIDI requires “Allow Serato Hardware Remapping.” | Incorrect. Serato's MIDI page says generic devices use **Enable MIDI Device**; “Allow Serato Hardware Remapping” applies to primary Serato hardware. Secondary MIDI mapping is documented alongside primary hardware. |
| Serato's XML has all the listed action names and exact pad defaults. | Unverified. The community action index has a small sample; some entries are marked untested. Names such as `play_pause`, `cue_set`, `pitch_slider`, `sampler_play_1` and the proposed fixed 8-pad Stems layout need an actual Serato mapping or official/device-specific source before they are treated as real Serato actions. The official Stems article documents MIDI Learn, not that universal pad assignment. ([MIDI-mapping Stems](https://support.serato.com/hc/en-us/articles/5732598890127-MIDI-mapping-Stems-with-Serato-DJ-Pro)) |
| A 1536 range is a standard platter multiplier, and `touch_strip_scrub` is the jogwheel solution. | Incorrectly generalized. The 2018 community example maps jog input with `<platter_move>` and `Relative Binary Offset`; the article's prose describes 1536 as an XDJ-700 experiment but its shown range attributes are `-1546` to `1546`. The article separately shows `touch_seek` for a touchstrip and says `touch_strip_scrub` can be tried there. It warns that the jog syntax is legacy and Serato may rewrite it on Save. This is not an S4 calibration or a current compatibility guarantee. ([DJ TechTools jog/touchstrip example](https://djtechtools.com/2018/04/11/hacking-serato-djs-midi-mapping-jogwheels-touchstrips-and-modifiers/)) |
| Generic MIDI platter control is simply impossible. | Too absolute. Serato's official MIDI Assign UI lists jogwheels/platters among features not remappable at this stage. A community XML hack demonstrates `<platter_move>` with an XDJ-700. That changes the conclusion to “unsupported in the official UI; community workaround exists; S4/current-version behavior unverified.” |
| `dvs_playback_mode_request` proves MIDI can enable usable THRU audio on an S4. | Not established by the cited official material. Serato says THRU is hardware-dependent and the deck modes require DVS-enabled or upgrade-ready hardware. Even if an XML action changes a software state, that does not prove the S4 input path is authorized or audio is passed through. The exact action token was not found in the wiki action index or official docs. ([Virtual Deck Play Modes](https://support.serato.com/hc/en-us/articles/360004125475-Virtual-Deck-Play-Modes); [DVS licensing](https://support.serato.com/hc/en-us/articles/202260274-Understanding-Serato-DJ-compatible-hardware-licensing)) |
| Generic 14-bit pitch, RGB velocity values, persistent “Red Hammer” feedback, or VU meters are definitively supported/blocked. | The report does not establish these universal claims. Serato's public MIDI types describe five modes for some dial/fader actions, but do not document paired 14-bit pitch or generic VU output. “Red Hammer” is a 2013 community forum technique with conflicting messages about behavior; it is not a current output contract. LED color/velocity values are controller-specific, not generic RGB. Mark these unknown until captured on the target Serato version and bridge port. ([MIDI Data Types](https://support.serato.com/hc/en-us/articles/225065768-MIDI-Data-Types); [historical Red Hammer thread](https://serato.com/forum/discussion/1066513)) |
| Delete `Auto_Save.xml`, mark a live map read-only, or spoof licensed hardware/SysEx to make the system work. | Not part of the recommended path. These are community-era workarounds or speculation, not validated deployment guidance. Use a separate copy, retain backups, and keep hardware licensing/audio behavior within documented supported configurations. |

### Recommended experiment sequence

No Serato app, hardware, audio or MIDI experiment was run for this documentation-only review.

1. With a supported Serato primary device available, start the S4 bridge and confirm `S4 MK3 MIDI Full` appears as a secondary MIDI device. Enable it with **Enable MIDI Device**. In a separate preset, map Serato's deck A Play/Pause and verify the S4 left Play event toggles deck A. Expected bridge bytes are `90 00 7F` / `80 00 00`. Test S4-only Practice Mode separately; do not assume the secondary-controller workflow works without primary hardware.
2. Save a throwaway preset, preserve its generated XML wrapper, and test one community-style `play` control on a copy. Confirm the current Serato version loads it, survives restart, and does not overwrite the user's existing mappings.
3. Add one output mapping for that same Play state. Capture MIDI at the bridge's virtual destination while toggling Play in Serato's GUI. This proves or refutes feedback delivery; it does not prove meters or all action states.
4. Test relative MOVE/LOOP and jog/touch separately. For jog, compare the S4 CC stream with the community `<platter_move>`/Relative Binary Offset example and tune only from observed range data. Keep a separate case for the Touchstrip command.
5. Test Beat Jump size retention, Stems, FX and pad modes only after the basic mapping is stable. Test DVS/THRU with supported DVS hardware and licensing; never infer audible input pass-through from a MIDI action.

### Bottom line

The Gemini draft is useful as a map of community experimentation, but it mixes official requirements, legacy XML hacks, controller-specific examples and speculative engineering recommendations. The decision-grade conclusion is narrower: standard MIDI buttons/faders and several relative controls are promising; a separate Serato XML generator is plausible; and output feedback, jog/platter mapping, audio modes, meters, screens and motor assistance need separate target-version tests. The current Djay profile remains untouched. Sources were checked October 30, 2026.

---

## Gemini research draft (preserved below; read with the review above)

## **1\. Introduction and Architectural Context**

The translation of proprietary Human Interface Device (HID) protocols into generic Musical Instrument Digital Interface (MIDI) architectures represents a highly complex systems engineering challenge, particularly within the closed-ecosystem paradigm of modern digital audio workstations and DJ performance software. When bridging a deeply integrated hardware device—such as the Native Instruments Traktor Kontrol S4 MK3—to a rigidly restricted software environment like Serato DJ Pro, the intermediary software bridge must function simultaneously as a protocol translator, a virtual MIDI port multiplexer, and a bidirectional state manager1.  
The Traktor Kontrol S4 MK3 natively utilizes a proprietary, high-resolution HID protocol to communicate exclusively with Traktor Pro. This proprietary protocol is heavily encrypted and structurally complex, allowing for advanced features such as the manipulation of motorized haptic jog wheels, high-refresh-rate OLED telemetry displays, and ultra-high-resolution 14-bit pitch faders that prevent tempo stepping during precise beatmatching. Serato DJ Pro, conversely, relies on a highly specific, officially licensed hardware ecosystem, utilizing its own proprietary handshake protocols to authenticate connected hardware before unlocking features like Digital Vinyl System (DVS) expansion, platter control, and high-resolution visual feedback3.  
However, Serato DJ Pro does expose a generic MIDI mapping interface via Extensible Markup Language (XML) to support secondary controllers, modular additions, and historically unsupported hardware5. For a bridge software developer transitioning a project from supporting Djay Pro to Serato DJ Pro, the primary engineering objective is mapping the translated 3-byte MIDI signals (Note On, Note Off, and Control Change) outputted by the bridge into Serato's heavily undocumented XML schema7. Because Serato does not formally publish an exhaustive MIDI mapping Software Development Kit (SDK) or XML syntax guide for generic devices, the DJ and developer community has relied on years of reverse-engineering, XML memory dumping, and trial-and-error to unlock features that are not exposed in the standard graphical user interface (GUI) MIDI mapping utility4.  
This report provides an exhaustive, expert-level breakdown of the Serato DJ Pro XML MIDI mapping infrastructure specifically tailored for the development of an HID-MIDI bridge for the Traktor S4 MK3. It details the comprehensive taxonomy of available MIDI actions, the complex and idiosyncratic implementation of LED feedback and output state management (including RGB color encoding and the hard limitations surrounding VU meters), the technical feasibility and XML configuration of audio THRU modes, and the rigid deployment pipelines required for successfully importing and sustaining custom XML maps within the software environment without triggering auto-save overwrites6.

## **2\. The Serato DJ Pro XML MIDI Mapping Schema**

Serato DJ Pro persists its MIDI configurations within a strictly formatted, hierarchical XML document. When an end-user utilizes the software’s internal, GUI-based "MIDI Learn" function, the application listens for incoming MIDI event flags and generates a rudimentary XML configuration file that maps the intercepted hardware commands to fundamental software functions6. However, this GUI-generated mapping is fundamentally limited. It severely restricts access to advanced state modifiers, complex multi-layered LED feedback loops, and hidden internal variables necessary for advanced performance. Direct, manual manipulation of the XML syntax via a plaintext editor is an absolute requirement to unlock the full potential of a comprehensive 4-channel controller like the S4 MK315.

### **2.1. XML Document Structure and Syntax Parsing**

The root of the mapping document must be encapsulated within a \<midi\> tag, which explicitly specifies the target application and exact version string. For example, the declaration \<midi app="Serato DJ Pro 3.0.8"\> informs the parser of the schema version17. Nested within this primary root are thousands of individual \<control\> and \<output\> tags that meticulously dictate how the software's internal audio engine interprets incoming MIDI data streams, and conversely, how the software's state machine responds with outgoing LED telemetry8.  
Every control definition requires a highly specific set of attributes to function correctly. The syntax must perfectly align with Serato’s internal parser; a single malformed tag, misplaced quotation mark, or incorrect attribute casing will cause the entire mapping file to fail silently or trigger a generic, non-descriptive error upon loading15.  
The critical attributes within a standard \<control\> tag include the following parameters, which the HID-MIDI bridge must target precisely:

> * **channel:** This dictates the MIDI channel (integer values 1-16). While internal C++ engines often represent these hexadecimally (0x00 to 0x0F), Serato's XML strictly utilizes standard integer strings, such as channel="1". It is vital that the HID-MIDI bridge consistently outputs on the channel defined in the XML4.  
> * **event\_type:** This attribute defines the specific type of MIDI message the parser should expect. Common and accepted string values include "Note On", "Note Off", and "Control Change" (CC). The bridge must perfectly translate the S4 MK3's button presses to Note On/Off messages, and its fader/knob movements to CC messages16.  
> * **control:** This defines the specific Note number or CC number (ranging from 0-127 in standard 7-bit MIDI) assigned to the physical hardware component13.  
> * **data\_type:** Arguably the most complex attribute, this dictates how the incoming 7-bit payload (0-127) is interpreted by the software engine mathematically. Incorrectly defining the data type will result in inverse fader movements, erratic knob behavior, or total control failure10.

### **2.2. Data Types and Endless Encoder Interpretation**

The data\_type attribute is particularly vital for the development of the S4 MK3 bridge, as the S4 relies heavily on endless rotary encoders for navigating libraries, manipulating loop sizes, and executing beat jumps. Standard generic MIDI maps hardware controls to absolute values (0-127), which is functionally useless for endless encoders that do not possess physical endpoints. Endless encoders simply send "increment" or "decrement" directional pulses. Serato supports several relative data types that the HID-MIDI bridge must output logically to ensure seamless operation.

| XML data\_type Attribute | Mathematical Functionality and Hardware Application |
| :---- | :---- |
| Absolute 7 | The standard 7-bit interpretation, mapping values linearly from 0 to 127\. This is strictly used for standard analog potentiometers (such as channel EQs, Filter sweeps, and Trim/Gain knobs) and standard linear faders (Upfaders and the Crossfader)10. |
| Relative 2's Complement | A mathematical format used predominantly for endless encoders. In this structure, clockwise increments are typically transmitted as low integer values (01, 02, 03), while counter-clockwise decrements are transmitted as high values rolling backward from the maximum (7F, 7E, 7D). This is ideal for library track scrolling and fractional loop size adjustments21. |
| Relative Signed Bit | An alternative endless encoder format where the most significant bit (MSB) dictates the direction of the turn, and the remaining lower bits indicate the speed, acceleration, or magnitude of the turn. The bridge must calculate these velocity curves internally if mapping to this data type10. |
| Relative OnOff | Often utilized for rotary encoders that send a static value depending on direction (e.g., continually sending a value of 65 for clockwise rotation, and 63 for counter-clockwise rotation). The bridge software must translate the S4's proprietary HID integer ticks into one of these relative MIDI formats so Serato can parse library navigation accurately10. |

### **2.3. The 14-Bit High-Resolution Pitch Fader Constraint**

A significant engineering hurdle in bridging modern DJ hardware to generic MIDI involves the pitch faders. Precise beatmatching requires high-resolution pitch control to prevent audible "tempo stepping"—where the tempo jumps by large fractions (e.g., from 120.00 BPM to 120.15 BPM) rather than smooth decimal increments (e.g., 120.00 to 120.01). The standard MIDI protocol achieves this by pairing two 7-bit CC messages (the Most Significant Byte, or MSB, and the Least Significant Byte, or LSB) to create a single 14-bit data stream yielding 16,384 distinct values8.  
While open-source platforms like Mixxx easily ingest 14-bit MSB/LSB pairs via explicit XML tags like fourteen-bit-lsb, Serato's support for 14-bit generic MIDI is notoriously limited and heavily obfuscated8. The software frequently locks true 14-bit high-resolution pitch processing to officially licensed hardware via its proprietary HID handshake, bypassing generic MIDI entirely4. The community consensus indicates that the pitch fader over generic MIDI cannot send or be interpreted as high-resolution messages4. The bridge developer may be forced to rely on standard 7-bit Absolute 7 mapping for the pitch faders, resulting in compromised tempo resolution, unless an undocumented legacy 14-bit syntax is successfully reverse-engineered and accepted by the Serato XML parser without throwing an exception4.

## **3\. Comprehensive Taxonomy of Available MIDI Actions**

To map the Traktor S4 MK3 effectively and exhaustively, the HID-MIDI bridge must synthesize specific MIDI CCs and Notes to target specific internal Serato functions. While Serato does not publish an official API or SDK list of these commands, years of community-driven XML memory dumping and systematic reverse-engineering have revealed a vast array of internal action names10.

### **3.1. Mixer, Transport, and Equalization Actions**

The foundational mixing controls of the DJ controller require precise mapping to Serato's digital signal processing (DSP) mixer and transport logic layers. The bridge must transmit Absolute 7 data for analog components and Note On/Off for binary switches.

| Serato XML Action Name | Hardware Target on S4 MK3 | Description and Implementation Notes |
| :---- | :---- | :---- |
| upfader | Channel Volume Faders | Controls the absolute audio volume of Decks 1 through 4\. Requires the bridge to output data\_type="Absolute 7"10. |
| crossfader | Main Crossfader | Blends audio linearly or logarithmically between assigned left and right decks23. Importantly, generic MIDI mappings often struggle to adjust the crossfader curve natively via XML syntax. Manipulating the curve (e.g., from a smooth mix to a sharp scratch cut) is typically restricted to the Serato GUI or handled via internal hardware-level DSP switching on licensed mixers. Generic MIDI users often lose the ability to assign curve parameters dynamically25. |
| eq\_bass | Low Frequency EQ Knob | Controls the low-frequency band (drums, sub-bass) via Absolute 710. |
| eq\_mid | Mid Frequency EQ Knob | Controls the mid-frequency band (vocals, lead synths) via Absolute 710. |
| eq\_treble | High Frequency EQ Knob | Controls the high-frequency band (hi-hats, cymbals) via Absolute 710. |
| play\_pause | Transport Play/Pause Button | Toggles the binary playback state of the assigned virtual deck. |
| cue\_set / cue\_play | Transport Cue Button | Sets a temporary anchor cue point or jumps to it momentarily, mimicking the standard Pioneer CDJ transport logic universally adapted by Serato. |
| pitch\_slider / rate | Tempo / Pitch Fader | Adjusts track playback speed. As noted, the bridge developer must rigorously test 14-bit MSB/LSB paired mapping, though Serato’s generic MIDI parser will likely restrict this to a 7-bit absolute translation, reducing accuracy4. |
| pitch\_bend\_up / pitch\_bend\_down | Jogwheel edge / Nudge buttons | Momentarily accelerates or decelerates the track tempo for phase alignment during beatmatching. This is triggered if the capacitive edge of the jogwheel is touched and rotated, outputting repeated Note On pulses28. |

### **3.2. Performance Pads and Real-Time Stems Integration**

Serato DJ Pro utilizes incredibly complex, multi-layered Pad Modes. The S4 MK3 features 8 high-quality RGB performance pads per deck. By default, these hardware pads are utilized for standard paradigms: Hot Cues, Loop Rolls, Beat Jumps, and Sampler triggering. More recently, Serato introduced "Stems," a machine-learning-powered feature that allows real-time isolation of acapellas and instrumentals by separating track frequencies on the fly30.  
The XML actions for the Stems feature allow the performance pads to instantly mute, solo, or apply specific vocal/instrumental effects to isolated frequency bands. When engineering the S4 MK3 bridge, the developer should create a specific MIDI multiplexing layer (e.g., accessed by pressing a custom "Pad Mode" shift button on the S4) that outputs unique MIDI notes exclusively mapped to these modern Stem actions.  
When a pad mode is mapped to Stems in the XML, the following routing applies:

> * **Pad 1 (Vocal Stem):** Mutes or solos the vocal track isolation.  
> * **Pad 2 (Melody Stem):** Mutes or solos the melodic and synth elements.  
> * **Pad 3 (Bass Stem):** Mutes or solos the bassline isolation.  
> * **Pad 4 (Drums Stem):** Mutes or solos the drum transient data.  
> * **Pads 5-8 (Stem FX):** Triggers complex macro effects including Vocal Echo, Instrumental Echo, Instrumental Braker, and Drums Echo respectively30.  
> * **Parameter Adjustments:** Left and Right parameter buttons can be mapped to toggle Acapella and Instrumental stems directly, or via a shift modifier, alter the Echo beat values (e.g., 1 beat or 1/2 beat timing)23.

Furthermore, the standard pad modes remain vital and must be mapped extensively in the XML. These include Hot Cues (set\_cue\_1 through set\_cue\_8), Sampler playback (sampler\_play\_1 through sampler\_play\_8 mapped directly to the internal SP-6 sampler engine), and Auto Loop / Loop Roll triggers, which allow rhythmic audio looping based on Serato's internal beatgrid analysis23.

### **3.3. The Platter and Motorized Jogwheel Limitation**

A major architectural hurdle in bridging generic hardware to Serato DJ Pro is the software's aggressively restrictive treatment of jogwheels and platters. Unlike competing software such as Traktor, Mixxx, or VirtualDJ, which allow seamless, native mapping of generic high-resolution MIDI encoders to the transport scratch engine, Serato actively blocks direct platter assignments for generic MIDI controllers6. Serato enforces a strict hardware licensing and Digital Rights Management (DRM) model; if a device is not officially authorized via its proprietary USB handshake, Serato will completely ignore high-resolution platter data sent via the standard \<platter\> XML tag.  
However, the reverse-engineering community has discovered highly effective XML workarounds to achieve basic scratching and track scrubbing using unsupported platters. The bridge software must translate the S4 MK3's proprietary high-resolution HID platter ticks into relative MIDI CC messages. In the XML file, the developer cannot use the restricted platter command. Instead, they must map the incoming relative CC to alternative, non-restricted scrubbing parameters.  
The most successful known hack is utilizing the touch\_strip\_scrub or needlesearch commands4. By mapping a relative CC to touch\_strip\_scrub, physically turning the S4's jogwheel forces the Serato playhead to scrub back and forth along the track timeline, effectively simulating a basic scratch. The XML code must specify an arbitrary range multiplier to adjust sensitivity. Community testing has shown that setting the range to 1536 (which represents 128 standard MIDI steps multiplied by a factor of 12\) yields the most organic, responsive scrubbing feel for standard capacitive platters4. While this hack inherently lacks the complex algorithmic inertia, advanced motor response, and absolute zero-latency audio manipulation of a natively licensed DVS or HID platter, it provides highly functional track searching, cueing, and basic scratching capabilities for a bridged, generic device4.

## **4\. Output Configuration: LED Feedback and State Management**

An HID-MIDI bridge is only partially complete if it merely acts as a one-way conduit sending data to the software. A professional DJ controller requires robust bi-directional communication to illuminate LEDs, change RGB pad colors to reflect track status, and provide visual feedback of internal software states. Serato DJ Pro handles these outputs via short 3-byte MIDI messages embedded in the XML7.

### **4.1. Basic Output Tag Structure and Velocity Encoding**

In the XML configuration file, outputs are defined similarly to inputs but are directed toward the hardware to trigger visual states. Serato sends a specific velocity value based on the software's binary or multi-state condition13.  
The developer must construct an output tag defining the event type and the exact decimal values for the on and off states. For example, if the HID-MIDI bridge expects a velocity of 42 to illuminate an S4 MK3 pad in the color blue, and a velocity of 0 to extinguish it, the XML output block for a specific deck's play button state would require the attributes on\_value="42" and off\_value="0". If the hardware utilizes Control Change messages for LED telemetry instead of Note commands, the event\_type is changed to "Control Change"16. Furthermore, if a pad supports variable brightness, the off\_value might not be 0, but rather a lower velocity integer that corresponds to a dim state, ensuring the pad remains visible in dark club environments16.

### **4.2. The "Red Hammer" Stateful Feedback Hack**

A well-documented idiosyncrasy and massive point of frustration within Serato’s XML parser is its failure to properly send complex state feedback using standard, GUI-generated output tags. Natively, if a user maps a pad to a Hot Cue using the MIDI Learn interface, the hardware pad will only illuminate momentarily while it is physically pressed down. It will not stay persistently illuminated to indicate that a cue point actually exists in the track timeline13.  
To solve this state management failure, the developer community reverse-engineered deep internal output commands, colloquially named the "Red Hammer" commands13. To force Serato to continuously broadcast persistent LED states for Cues and Sampler slots, the developer must manually inject these specific, undocumented command strings into the XML output block using a plaintext editor.  
To achieve continuous LED feedback when a sample is loaded into the SP-6 sampler, or a Cue Point is set on a track, the standard input mapping generated by the GUI must be ruthlessly deleted from the XML file. It must be completely replaced by a custom combination of a cleanly defined \<control\> input tag and the special "Red Hammer" \<output\> lines13.

> * **Conflict Resolution:** If both the standard GUI-generated output tag and the custom Red Hammer output tag exist simultaneously within the XML document, Serato's engine will attempt to process both. This results in conflicting MIDI messages being sent to the controller, causing the LED to flicker sporadically, illuminate in the wrong color, or fail to light up entirely13.  
> * **Bridge Translation:** The bridge software must be explicitly programmed to intercept and interpret these specific incoming velocity values from Serato. If the Red Hammer XML output dictates that a populated cue point sends a velocity of 112, the bridge software must catch the Note On, Velocity 112 message, translate it internally into the proprietary Traktor HID byte array for the color red (or whichever RGB hex code the user prefers), and push that byte array directly to the S4 MK3's physical hardware bus13.

### **4.3. VU Meters and High-Resolution Visual Feedback Limitations**

The Traktor S4 MK3 features prominent, multi-segment LED VU meters for each mixing channel, designed to allow the DJ to monitor audio gain staging and prevent digital clipping. Providing LED feedback for these VU meters via generic MIDI in Serato DJ Pro is a severe limitation that exposes the software's walled-garden approach.  
While competing DJ platforms like Traktor Pro and Mixxx natively calculate and export ChannelLevelMeter amplitude data over standard MIDI (allowing a software bridge to easily parse the amplitude integers and sequentially light the corresponding hardware LEDs)33, Serato DJ Pro absolutely does *not* support outputting VU meter data to generic, non-licensed MIDI controllers14. Serato locks this high-refresh-rate audio telemetry entirely behind its proprietary HID and SysEx protocols, intending for it to be utilized exclusively by its licensed hardware partners (e.g., Pioneer, Rane, Denon)35.  
Extensive community analysis, forum requests, and developer feedback confirm that internal functions representing VU levels are systematically restricted and stripped from the generic XML MIDI output engine14. Therefore, the HID-MIDI bridge developer attempting to illuminate the S4 MK3's VU meters must choose between two highly complex architectural workarounds:

> 1. **Hardware-Level Audio Emulation (Recommended):** The bridge software bypasses Serato's MIDI output entirely. Instead, it intercepts the audio stream directly at the operating system driver level (utilizing CoreAudio on macOS or ASIO on Windows). The bridge performs its own internal Fast Fourier Transform (FFT) or amplitude peak analysis on the audio outputs, calculates the decibel levels, and drives the S4 MK3 VU LEDs completely independently of Serato's internal software state.  
> 2. **SysEx Spoofing (Highly Discouraged):** The bridge could theoretically be programmed to spoof the complex System Exclusive (SysEx) handshake of a licensed Serato controller (e.g., pretending to be a Pioneer DJM mixer). By tricking Serato into believing an official device is connected, the software might output the encrypted VU data. However, this breaches standard generic MIDI compliance, introduces massive system instability, requires constant updating as Serato patches exploits, and often violates end-user license agreements (EULA) by actively circumventing DRM protocols.

For a standard generic MIDI XML mapping deployment, the architectural conclusion is absolute: **VU meter output feedback is not natively supported or accessible via Serato's generic XML MIDI interface.**14.

## **5\. Audio THRU Mode, DVS Architecture, and Deck State Management**

A critical query in the development specification is whether support for audio THRU mode (turning one of the virtual deck channels into a live analog audio input) is technically possible via MIDI mapping, and whether a specific XML action exists to trigger it.

### **5.1. Serato Audio Architecture and the Function of THRU Mode**

In the architecture of Serato DJ Pro, THRU mode acts as a software-level analog bypass. When a virtual deck is set to THRU, the software takes the incoming analog audio signal from the hardware’s connected audio interface (such as a live turntable playing standard vinyl, or a microphone) and routes it directly to the master output mix. Crucially, this bypasses Serato's internal digital signal processing (DSP), BPM analysis algorithms, beatgrids, and software EQ layers11. Visually, the Serato GUI indicates this state change by graying out the track waveform and explicitly labeling the virtual deck with a black and white "THRU" indicator11.

### **5.2. XML Implementation of Deck Modes**

Serato operates on four primary deck playback states: INT (Internal), REL (Relative), ABS (Absolute), and THRU3. The REL and ABS modes are specifically engineered for Digital Vinyl Systems (DVS), relying on a timecode control tone to dictate the playback speed and position of the digital file.  
The XML parser does indeed contain a specific, mappable MIDI action for cycling or requesting these state changes. The internal command is dvs\_playback\_mode\_request17. By mapping a physical button on the S4 MK3 (for instance, utilizing a Shift modifier combined with a Deck toggle button) to this XML attribute, the user can theoretically cycle the assigned virtual deck sequentially through INT, REL, ABS, and THRU modes17.  
The XML syntax for requesting a deck state change requires targeting the specific deck ID:

XML  
\<control channel="1" event\_type="Note On" control="24"\>  
    \<dvs\_playback\_mode\_request deck\_set="Default" deck\_id="0" /\>  
\</control\>

*(Note: deck\_id="0" typically corresponds to Deck 1, 1 corresponds to Deck 2, etc., following zero-indexed programming conventions).*

### **5.3. Hardware DRM Constraints on THRU Mode Feasibility**

While the *MIDI action* to trigger THRU mode clearly exists and is functional within the XML parser, the actual *audio execution* of THRU mode in Serato is strictly gated by hardware authentication3.  
Serato DJ Pro only permits DVS expansion features—and by extension, the complex audio routing required for REL, ABS, and THRU modes—if an officially licensed, DVS-capable soundcard or mixer is physically detected and authenticated upon software launch3. The Traktor S4 MK3 acts as a generic CoreAudio or ASIO soundcard. Serato will output digital audio to the S4 MK3’s master outputs perfectly fine in standard Internal (INT) playback mode. However, because the S4 MK3 is not a Serato-certified DVS interface, the software will silently disable the internal audio routing logic required for THRU mode.  
Even if the MIDI command successfully changes the GUI visual state to "THRU", no live audio will pass through the Serato mixing engine because the software restricts analog input routing for unauthorized interfaces3. If the user wishes to utilize true THRU mode (e.g., to play a vinyl record on a turntable connected to the S4), they would need to rely on the S4 MK3's internal hardware mixer capabilities. The bridge software would need to unlock the S4's hardware routing independently, routing the analog inputs directly to the analog outputs at the hardware level, bypassing Serato entirely. Therefore, while the MIDI XML trigger is available, the proprietary audio pipeline dependency within Serato severely restricts the functional execution of this feature for generic audio interfaces.

## **6\. Deployment: Importing and Managing Custom XML Mappings**

Once the HID-MIDI bridge is fully programmed in C++ or Swift, and the bespoke XML configuration file is meticulously authored, the mapping must be manually deployed into the Serato DJ Pro ecosystem by the end-user. Serato handles file paths, mapping persistence, and device recognition with strict, unforgiving directory rules.

### **6.1. XML File System Placement**

Serato DJ Pro completely ignores any XML mappings that are not placed in its specific, proprietary directory structure. It does not scan external folders or offer an "Import from File" button in the standard GUI. The bridge developer must instruct users (or utilize an automated installation script) to move the custom .xml file to the following exact path strings, dependent on their operating system:

> * **macOS / OS X Environment:** The path is strictly \~/Music/\_Serato\_/MIDI/Xml5.  
> * **Windows Environment:** The path is strictly %USERPROFILE%\\Music\\\_Serato\_\\MIDI\\Xml (which resolves to C:\\Users\\\[Username\]\\Music\\\_Serato\_\\MIDI\\Xml)5.

If the Xml folder does not currently exist within the MIDI directory (which occurs on fresh Serato installations where no MIDI mapping has ever been attempted), the user or the installer script must create the directory manually before placing the file40.

### **6.2. Enabling and Loading the Mapping in the Serato GUI**

To successfully activate the HID-MIDI bridge mapping, the user must navigate through a specific sequence within the Serato GUI settings:

> 1. **Initialize the Bridge:** Ensure the HID-MIDI bridge software is running in the background, exposing its virtual MIDI port to the operating system2.  
> 2. **Access MIDI Settings:** Open Serato DJ Pro and navigate to the **Setup** menu (represented by the gear icon in the top right corner), then select the **MIDI** tab5.  
> 3. **Select the Virtual Port:** In the *MIDI Devices* list, the virtual MIDI port created by the bridge software (e.g., labeled "S4 MK3 Bridge MIDI") will appear. The user must click this port to highlight it.  
> 4. **Hardware Remapping Flag (Crucial Step):** The user must check the box explicitly labeled **"Allow Serato Hardware Remapping"**. While this feature was historically designed to allow users to overwrite default mappings on official hardware, checking this box is a mandatory requirement to ensure that Serato actively listens to custom XML files for incoming generic MIDI devices5.  
> 5. **Enable Output Telemetry:** The user must ensure that **"Enable MIDI Output Lighting"** is checked. If this is disabled, the XML \<output\> tags are ignored, and no LED feedback will be transmitted back to the bridge41.  
> 6. **Load the Preset:** Under the *MIDI Presets* section, the user must select the custom XML file from the dropdown list and click **Load**5.

### **6.3. Troubleshooting Aggressive Auto-Save Overwrites**

A notorious, highly documented issue with Serato’s XML management system is its aggressive auto-save architecture. If a user manually edits the XML file in a plaintext editor to inject complex "Red Hammer" outputs or relative data types, and then opens Serato, the software may attempt to normalize the file. If the user touches the internal MIDI GUI, Serato will immediately generate a conflicting Auto\_Save.xml file or completely overwrite the custom file, destroying the complex syntax13.  
To prevent catastrophic mapping failures during deployment, bridge developers rely on specific mitigation strategies:

> * **Purging Cache:** Users must be explicitly instructed to delete any pre-existing Auto\_Save.xml files located in the MIDI folder *before* launching Serato with the newly installed bridge XML. Failure to do so forces Serato to read the cached, incorrect mapping13.  
> * **The Read-Only Hack:** Developers often mark the custom .xml file as "Read-Only" at the OS file-system level (via File Properties in Windows, or chmod in macOS). This forces Serato to read the highly complex, reverse-engineered syntax upon boot, while physically preventing the application from attempting to re-parse or overwrite it with its simpler, GUI-generated syntax4.

## **7\. Strategic Recommendations for Bridge Architecture**

Based on the exhaustive analysis of Serato's internal mapping engine, XML syntax requirements, and DRM-restricted audio architecture, the development of the S4 MK3 HID-MIDI bridge requires careful state abstraction. The bridge cannot rely on Serato for complex processing; it must act as an intelligent intermediary.  
Because Serato provides absolutely no native interface for high-resolution platters, VU meters, or onboard OLED screens via standard generic MIDI, the bridge software must bear the computational load of emulating these features externally.

> * **Managing OLED Telemetry:** The bridge should not rely on Serato for track metadata. Serato does not broadcast track names, waveforms, or BPM via generic MIDI. The bridge would have to hook into OS-level accessibility APIs, read Serato's protected memory space (which is highly volatile and prone to breaking on every software update), or alternatively, utilize the screens merely to display hardware states (e.g., current pad mode, master volume levels, or loop sizes, all handled internally by the bridge's state machine)9.  
> * **Executing Jogwheel Physics:** The touch\_strip\_scrub mapping via relative CC is the only viable path forward for platter movement in generic MIDI4. The bridge must intercept the S4 MK3's proprietary HID integer ticks, apply a mathematical sensitivity scaling algorithm (to match the required 128-value relative increments expected by Serato), and output the CC. Furthermore, the capacitive touch plate of the jogwheel must send a standard Note On message mapped to pause the deck and engage the scrub state, while the corresponding Note Off resumes playback seamlessly.  
> * **Abstracting Modifier and Shift States:** While Serato's XML allows for single-layer mapping easily, relying on it for complex multi-layer modifiers (e.g., holding Shift to delete a Cue Point) can cause ghost triggers if the XML is not perfectly sanitized. It is highly recommended that the bridge software handles *all* Shift states internally. Rather than sending a generic "Shift" MIDI note to Serato and relying on Serato's parser to decode the \<modifier\> tag, the bridge should intercept the physical Shift button press, and internally remap the outputted MIDI notes of the performance pads. Thus, Serato simply sees two entirely different MIDI notes (one for Cue Set, one for Cue Delete), drastically simplifying the XML file complexity, reducing software processing latency, and ensuring rock-solid stability42.

## **8\. Conclusion**

The development of a robust HID-MIDI bridge to integrate the Traktor Kontrol S4 MK3 into Serato DJ Pro is highly feasible, provided the software engineer meticulously navigates the strict architectural limitations of Serato's closed-hardware ecosystem. A successful deployment requires utilizing Serato's deeply undocumented XML schema to map both relative and absolute generic MIDI commands to internal DSP actions, such as crossfader, upfader, and dvs\_playback\_mode\_request for deck state toggling (including the audio-restricted THRU mode).  
Achieving bi-directional communication to drive the S4 MK3's complex RGB LED arrays requires manually injecting output configurations—most notably the "Red Hammer" stateful feedback syntaxes—directly into the XML document, bypassing the graphical interface entirely. While critical professional features like true high-resolution 14-bit pitch control, natively integrated motorized platters, and real-time VU meter telemetry are inherently walled off from generic MIDI by DRM protocols, strategic engineering workarounds can bridge the gap. By utilizing touch\_strip\_scrub for platters, abstracting shift layers into the bridge's internal state machine, and deploying a hardware-level audio monitor for VU metering, a developer can approximate a professional performance environment. Finally, by deploying a heavily customized, Read-Only XML file strictly into the \_Serato\_/MIDI/Xml directory, the bridge can successfully translate Native Instruments' proprietary hardware into a robust, highly versatile controller capable of commanding Serato DJ Pro.

#### **Works cited**

> 1. r/S4MK3\_HID\_MIDI\_BRIDGE \- Reddit, [https\://www\.reddit.com/r/S4MK3\_HID\_MIDI\_BRIDGE/](https://www.reddit.com/r/S4MK3_HID_MIDI_BRIDGE/)  
> 2. Traktor Kontrol S5 Support \- Algoriddim Community Forums \- DJay, [https\://community.algoriddim.com/t/traktor-kontrol-s5-support/28505](https://community.algoriddim.com/t/traktor-kontrol-s5-support/28505)  
> 3. User Manual \- Cloudfront.net, [https\://d1aeri3ty3izns.cloudfront.net/media/36/366330/download\_366330.pdf](https://d1aeri3ty3izns.cloudfront.net/media/36/366330/download_366330.pdf)  
> 4. Hacking Serato DJ's MIDI Mapping: Jogwheels, Touchstrips, and, [https\://djtechtools.com/2018/04/11/hacking-serato-djs-midi-mapping-jogwheels-touchstrips-and-modifiers/](https://djtechtools.com/2018/04/11/hacking-serato-djs-midi-mapping-jogwheels-touchstrips-and-modifiers/)  
> 5. marscanbueno/serato-dj-pro-midi-maps \- GitHub, [https\://github.com/marscanbueno/serato-dj-pro-midi-maps](https://github.com/marscanbueno/serato-dj-pro-midi-maps)  
> 6. MIDI mapping with Serato DJ Pro, [https\://support.serato.com/hc/en-us/articles/209377487-MIDI-mapping-with-Serato-DJ-Pro](https://support.serato.com/hc/en-us/articles/209377487-MIDI-mapping-with-Serato-DJ-Pro)  
> 7. MIDI Scripting · mixxxdj/mixxx Wiki \- GitHub, [https\://github.com/mixxxdj/mixxx/wiki/midi-scripting](https://github.com/mixxxdj/mixxx/wiki/midi-scripting)  
> 8. Midi Controller Mapping File Format · mixxxdj/mixxx Wiki \- GitHub, [https\://github.com/mixxxdj/mixxx/wiki/MIDI-controller-mapping-file-format](https://github.com/mixxxdj/mixxx/wiki/MIDI-controller-mapping-file-format)  
> 9. HID-MIDI bridge for Traktor S4 MK3 \- Algoriddim Community Forums, [https\://community.algoriddim.com/t/hid-midi-bridge-for-traktor-s4-mk3/43609](https://community.algoriddim.com/t/hid-midi-bridge-for-traktor-s4-mk3/43609)  
> 10. Here is a Serato DJ Pro midi dump for all of you, to map functions, [https\://www\.reddit.com/r/DJs/comments/w8a63c/here\_is\_a\_serato\_dj\_pro\_midi\_dump\_for\_all\_of\_you/](https://www.reddit.com/r/DJs/comments/w8a63c/here_is_a_serato_dj_pro_midi_dump_for_all_of_you/)  
> 11. Release Notes \- Scratch Live 2.3 \- DJ Software \- Serato, [https\://serato.com/scratchlive/downloads/2.3.0/releasenotes](https://serato.com/scratchlive/downloads/2.3.0/releasenotes)  
> 12. MIDI Troubleshooting Guide \- Serato Support, [https\://support.serato.com/hc/en-us/articles/202305134-MIDI-Troubleshooting-Guide](https://support.serato.com/hc/en-us/articles/202305134-MIDI-Troubleshooting-Guide)  
> 13. Advanced MIDI output functions (cue point lighting, etc.) tutorial \+, [https\://serato.com/forum/discussion/1066513](https://serato.com/forum/discussion/1066513)  
> 14. MIDI Mapping VU-meters and LED feedback in general \- DJ Hardware, [https\://community.algoriddim.com/t/midi-mapping-vu-meters-and-led-feedback-in-general/16253](https://community.algoriddim.com/t/midi-mapping-vu-meters-and-led-feedback-in-general/16253)  
> 15. Even More Serato DJ Pro MIDI Hacking: Deleting Cue Points, Tone, [https\://djtechtools.com/2018/10/29/even-more-serato-dj-pro-midi-hacking-deleting-cue-points-tone-play-multiple-modifiers/](https://djtechtools.com/2018/10/29/even-more-serato-dj-pro-midi-hacking-deleting-cue-points-tone-play-multiple-modifiers/)  
> 16. How to edit MIDI output with a text editor \- DJing Discussion \- Serato, [https\://serato.com/forum/discussion/1060373](https://serato.com/forum/discussion/1060373)  
> 17. Serato HID PhaseDJ WIR mode MIDI XML assignment ... \- Reddit, [https\://www\.reddit.com/r/Turntablists/comments/1g3y26l/serato\_hid\_phasedj\_wir\_mode\_midi\_xml\_assignment/](https://www.reddit.com/r/Turntablists/comments/1g3y26l/serato_hid_phasedj_wir_mode_midi_xml_assignment/)  
> 18. Serato DJ MIDI Mappings for Novation Launchpad \- GitHub Gist, [https\://gist.github.com/Themitchell/2fbeea02d3937cbb631a](https://gist.github.com/Themitchell/2fbeea02d3937cbb631a)  
> 19. ABLETON PUSH MK1 MAPPING FOR SERATO DJ PRO \- GitHub Gist, [https\://gist.github.com/rpong/6e70dcfb7242199d99e59eadf076a375](https://gist.github.com/rpong/6e70dcfb7242199d99e59eadf076a375)  
> 20. VCI-380 \- PowerOnPlay, [https\://www\.poweronplay.com/vci-380/](https://www.poweronplay.com/vci-380/)  
> 21. Adding library scroll functionality to Hercules Starlight DJ Controller, [https\://www\.reddit.com/r/Serato/comments/opfslf/adding\_library\_scroll\_functionality\_to\_hercules/](https://www.reddit.com/r/Serato/comments/opfslf/adding_library_scroll_functionality_to_hercules/)  
> 22. High-Resolution MIDI Controller \- Hackaday, [https\://hackaday.com/2024/01/05/high-resolution-midi-controller/](https://hackaday.com/2024/01/05/high-resolution-midi-controller/)  
> 23. Numark Party Mix III Quickstart Guide for Serato DJ Pro, [https\://support.serato.com/hc/en-us/articles/16577869540111-Numark-Party-Mix-III-Quickstart-Guide-for-Serato-DJ-Pro](https://support.serato.com/hc/en-us/articles/16577869540111-Numark-Party-Mix-III-Quickstart-Guide-for-Serato-DJ-Pro)  
> 24. Pioneer DJ DDJ-RB Quickstart Guide \- Serato Support, [https\://support.serato.com/hc/en-us/articles/12439975109647-Pioneer-DJ-DDJ-RB-Quickstart-Guide](https://support.serato.com/hc/en-us/articles/12439975109647-Pioneer-DJ-DDJ-RB-Quickstart-Guide)  
> 25. Crossfader or no crossfader? Why? : r/DJs \- Reddit, [https\://www\.reddit.com/r/DJs/comments/111kuro/crossfader\_or\_no\_crossfader\_why/](https://www.reddit.com/r/DJs/comments/111kuro/crossfader_or_no_crossfader_why/)  
> 26. no crossfader options inpulse 500 serato dj pro \- Hercules Forum, [https\://community.hercules.com/community/archives/no-crossfader-options-inpulse-500-serato-dj-pro/](https://community.hercules.com/community/archives/no-crossfader-options-inpulse-500-serato-dj-pro/)  
> 27. Mapping request for the Inpulse 500 \- Hercules Forum, [https\://community.hercules.com/community/djcontrol-inpulse-500/mapping-request-for-the-inpulse-500/](https://community.hercules.com/community/djcontrol-inpulse-500/mapping-request-for-the-inpulse-500/)  
> 28. Release Notes \- Scratch Live 1.9 \- DJ Software \- Serato, [https\://serato.com/scratchlive/downloads/1.9.0/releasenotes](https://serato.com/scratchlive/downloads/1.9.0/releasenotes)  
> 29. MIDI mapping pitch bend / nudge?\! : r/Serato \- Reddit, [https\://www\.reddit.com/r/Serato/comments/1h80d9t/midi\_mapping\_pitch\_bend\_nudge/](https://www.reddit.com/r/Serato/comments/1h80d9t/midi_mapping_pitch_bend_nudge/)  
> 30. Controlling Stems with your DJ hardware \- Serato Support, [https\://support.serato.com/hc/en-us/articles/5700970589583-Controlling-Stems-with-your-DJ-hardware](https://support.serato.com/hc/en-us/articles/5700970589583-Controlling-Stems-with-your-DJ-hardware)  
> 31. MIDI mapping Stems with Serato DJ Pro, [https\://support.serato.com/hc/en-us/articles/5732598890127-MIDI-mapping-Stems-with-Serato-DJ-Pro](https://support.serato.com/hc/en-us/articles/5732598890127-MIDI-mapping-Stems-with-Serato-DJ-Pro)  
> 32. Serato Custom MIDI User Modes for the Pioneer DJM-S9 \- GitHub, [https\://github.com/haveboard/s9-serato-custom-midi-user-modes-1](https://github.com/haveboard/s9-serato-custom-midi-user-modes-1)  
> 33. How To MIDI Map Traktor VU Meters To A DJ Controller, [https\://djtechtools.com/2013/10/27/how-to-midi-map-traktor-vu-meters-to-a-dj-controller/](https://djtechtools.com/2013/10/27/how-to-midi-map-traktor-vu-meters-to-a-dj-controller/)  
> 34. MIDI Controller XML Specification \- djDecks, [https\://djdecks.be/index.php?module=docs\&sub\_help=midi\_xml.php](https://djdecks.be/index.php?module=docs&sub_help=midi_xml.php)  
> 35. MIDI Map VU Meter \- Pioneer DJ forum, [https\://forums.pioneerdj.com/hc/en-us/community/posts/207729546-MIDI-Map-VU-Meter](https://forums.pioneerdj.com/hc/en-us/community/posts/207729546-MIDI-Map-VU-Meter)  
> 36. Mixxx DJ Tutorial PDF \| PDF \| Equalization (Audio) \- Scribd, [https\://www\.scribd.com/document/330480740/Mixxx-DJ-Tutorial-pdf](https://www.scribd.com/document/330480740/Mixxx-DJ-Tutorial-pdf)  
> 37. User Manual, [https\://www\.bhphotovideo.com/lit\_files/623206.pdf](https://www.bhphotovideo.com/lit_files/623206.pdf)  
> 38. PhaseDJ WIR mode MIDI XML assignment (same as REL/ABS mode), [https\://www\.reddit.com/r/Serato/comments/1g3y01f/phasedj\_wir\_mode\_midi\_xml\_assignment\_same\_as/](https://www.reddit.com/r/Serato/comments/1g3y01f/phasedj_wir_mode_midi_xml_assignment_same_as/)  
> 39. haveboard X As-One S9 Midi Mapping \- GitHub, [https\://github.com/haveboard/s9-serato-custom-midi-user-modes-haveboard-x-as-one](https://github.com/haveboard/s9-serato-custom-midi-user-modes-haveboard-x-as-one)  
> 40. How to load custom MIDI Presets \- Serato Support, [https\://support.serato.com/hc/en-us/articles/219823647-How-to-load-custom-MIDI-Presets](https://support.serato.com/hc/en-us/articles/219823647-How-to-load-custom-MIDI-Presets)  
> 41. MIDI \- Serato Support, [https\://support.serato.com/hc/en-us/articles/223446808-MIDI](https://support.serato.com/hc/en-us/articles/223446808-MIDI)  
> 42. Midi Mapping with a modifier : r/Serato \- Reddit, [https\://www\.reddit.com/r/Serato/comments/18ugu7d/midi\_mapping\_with\_a\_modifier/](https://www.reddit.com/r/Serato/comments/18ugu7d/midi_mapping_with_a_modifier/)  
> 43. Hacking Serato XML Midi Mapping with Shift, Hotcue ... \- YouTube, [https\://www\.youtube.com/watch?v=8NmwJi3q17U](https://www.youtube.com/watch?v=8NmwJi3q17U)  
> 44. HOW TO work with Modifiers in the MIDI Mapping XML \- DJ Hardware, [https\://community.algoriddim.com/t/how-to-work-with-modifiers-in-the-midi-mapping-xml/32183](https://community.algoriddim.com/t/how-to-work-with-modifiers-in-the-midi-mapping-xml/32183)
