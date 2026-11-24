# S4 hardware screen layout

## 1. Purpose

The fixed 320 x 240 screens show useful Djay feedback or bridge-owned values
not already indicated by an LED. STEMS is the exception: its reactive legend
preserves the physical four-column, two-row pad arrangement. No deck, pad-mode,
Shift, jog, or encoder-role bars surround ordinary content. Status headers are
the exception: reserve a 24-pixel header for an active wheel backspin
(`BACKSPIN A` through `BACKSPIN D`) and a 24-pixel header while a targeted deck
has mismatched or unknown physical/software FX positions (`FX SYNC`).

## 2. Color

RGB values are quantized through the existing RGB565 raster:

| Token | RGB | Purpose |
| --- | --- | --- |
| BACKGROUND | 10, 12, 18 | Whole screen and text on an active pad |
| PANEL | 24, 30, 42 | Inactive and unknown pad interiors |
| BORDER | 96, 108, 128 | Unheld pad boundary |
| PRIMARY | 244, 248, 255 | Labels and confirmed-active pad fill |
| SECONDARY | 164, 178, 202 | Unknown state |
| PRESSED | 252, 176, 36 | Thick physical-hold outline |
| LOOP_ACTIVE | 0, 220, 80 | Active loop quadrant |
| BLACK | 0, 0, 0 | Text on active loop and bright selection |
| DIVIDER | 60, 70, 90 | Info-page quadrant separators |

An active pad is inverted, not merely recolored. Every pad also says ON, OFF,
or ?. Press thickness and an inner dark separation remain visible on either
fill. MUTE ON describes the mute switch, not effective audibility under solo.
Screen state does not depend on the configurable pad-LED brightness policy.

## 3. Typography

Use the user-approved anti-aliased bold sans-serif direction on every screen,
including STEMS/MUTE, stopped output and the diagnostic patterns. S4 Screen Sans
is a pre-rendered glyph atlas derived from Liberation Sans Bold 2.1.5, an
Arial-compatible OFL font. Its license accompanies the source and prototype.
See the [approved font comparison](../../docs/images/s4-screen-font-comparison.png).

Keep labels and SELECT at 16 pixels, and select the largest 32/40/48-pixel
numeric font whose measured advance fits the 136-pixel value area.
These are font sizes, not enlarged pixel blocks. Center text using measured
proportional advances, not character count. The plus/minus sign is one glyph.
Glyphs share a cap-height baseline, so numbers do not move when their value
changes. Six fixed sizes (8 through 48 pixels) cover the existing text API.
The 8-pixel size is reserved for the diagnostic pattern's small annotations.
Sixteen coverage levels blend text into the actual underlying RGB565 fill,
including black on green/amber and dark text on active STEMS tiles.

## 4. Geometry

STEMS uses 8-pixel outside margins, 4-pixel gutters, and four 73-pixel columns.
Shared instrument names start at y=12. The two pad rows start at y=40 and
y=138 and are 94 pixels high. Labels and status occupy separate lines.
There is no scrolling, responsive breakpoint, or touch target.
Both physical screens use the same geometry.

The approved info page divides 320 x 240 into four equal 160 x 120 quadrants:
JUMP upper left, LOOP upper right, KEY lower left, RANGE lower right.
Use 12-pixel label insets, values centered horizontally at y+44, and one-pixel
dividers at x=160 and y=120. Without status headers, there are no surrounding headers or footers.
While `BACKSPIN` or `FX SYNC` is present, each active header reserves 24 pixels
at the top (24 pixels when one is active, 48 pixels when both are active, with
`BACKSPIN` on the top row and `FX SYNC` below it). Info quadrants reflow into
the remaining height (labels at y+8, values at y+34, SELECT at y+88 for one
header; labels at y+6, values at y+28, SELECT at y+76 for two headers). STEMS
names and tiles move down by the reserved header height and each tile row loses
half of that height. The amber `BACKSPIN` header names the zero-based owning
deck (`A`-`D`) on that physical wheel. The amber `FX SYNC` header
names the affected deck, strip L/R, parameters 1-3 and W for dry/wet; `?` marks
missing feedback. Removing active headers restores the original geometry.
The user-approved mockup is
`generated-images/ig_0c2d859c8ec5ece2016ad955282cc087d1a22fb13f87b3fc35.png`,
with the user's subsequent active-loop green/black amendment.

## 5. Existing primitives and states

Use the existing Frame rectangle, RGB565 serialization, and offline PPM export
primitives. Shared measured text primitives composite static coverage masks;
there is no runtime font parser, font installation, glyph allocation, or
font-rendering dependency. Rendering remains on the screen worker.

- Stem tile: confirmed on uses PRIMARY fill; confirmed off uses PANEL fill;
  unknown uses PANEL with ?. A physical hold adds a 3-pixel outline and dark
  inner separation. Default boundary thickness is 1 pixel.
- Stem page: one name per column; mute above solo. Shift changes only the
  upper action to ONLY. Its active state requires all four solo states known
  with exactly that one stem soloed; there is no invented exclusive latch.
- Info page: every non-STEMS mode shows the same four fields. Values come from
  confirmed feedback; unknown is ?, not a remembered length or invented default.
  Inactive LOOP says OFF. Active LOOP uses LOOP_ACTIVE fill and BLACK text;
  an active loop whose size is unknown keeps green fill and shows ?.
  JUMP selection follows the bridge's per-deck MOVE selection state and shows
  steady SELECT. Its quadrant alternates amber/black text and dark/white text
  every 500 ms; the number never disappears. Only the screen worker owns this
  timing, and only selecting info pages are retransmitted for flash phases.
  STEMS remains steady even if jump selection is latched in the background.
- Quiet page: retained as a raster primitive, not an ordinary pad-mode page.
- Stopped page: centered BRIDGE / STOPPED / START TO / RESUME lines.

## 6. Feedback

Persistent state comes from normal-channel Djay mute/solo feedback. A press
outline follows its actual held MIDI owner, not a new deck or Shift route.
Feedback-only changes refresh screens without starting a USB worker before
initialized HID input. Unchanged pages remain coalesced off the control thread.

## 7. Accessibility and verification

ON/OFF/? and fill inversion distinguish state without color alone. Inspect
exact 320 x 240 exports for both Shift states, all pad states, held pads,
deck changes, quiet pages, and stopped output. Check gutters and label bounds.
Physical readability must be judged on the S4 at normal playing distance;
an offline raster or completed USB write cannot establish it.

## 8. Accepted limits

This pass has one fixed dark palette and no text-size setting. Native panel
resolution remains 320 x 240; smoothing does not add panel pixels. Neural Mix
availability and numeric loop/jump/tempo-range values are not inferred from
button presses. Inactive selected loop length is not available through the
tested routes; OFF is not a length. Key uses calibrated numerical labels,
not an unverified conversion from MIDI note names. The macOS settings app
has its own design contract.
