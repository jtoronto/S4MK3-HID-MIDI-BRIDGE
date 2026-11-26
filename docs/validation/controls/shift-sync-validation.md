# Shift+Sync tempo-range cycling

Implemented October 6, 2026. The user observed that changing tempo range in
one Djay deck's UI changes it on every deck.

Normal Sync remains `turntableN.bpmSync`, Note 2 on channels 1-4.
Shift+Sync uses `application.tempoSliderRangeNext`, Note 2 on channels 9-12.
Both physical sides can cycle the same shared range. This native action is
present in shipped Djay mappings and labeled "Switch Tempo Slider Range".
Djay controls the supported ranges and wrap order; no range index is kept
in the bridge.

## Software verification

All 80 Rust tests and 12 Swift tests passed. Formatting, strict Clippy,
all-target build, shell syntax, plist lint, and diff whitespace checks passed.
Regressions cover both sides and all decks, simultaneous Shift/Sync event
ordering, duplicate held presses, and release ownership after changing
Shift/deck state. The serialized mapping and actual CLI export retain all
four normal Sync actions and resolve all four shifted addresses globally.

The rebuilt app passed deep strict code-signature verification and ZIP
integrity checks. Its bundled mapping contains all four shifted bindings.
The previous bundle/archive are preserved at
`/tmp/s4mk3-pre-shift-sync.4skBwU`. Existing LED preferences and the user's
`S4 MK3 Bridge 2` mapping retained their original hashes.

## Live acceptance

The user confirmed "Works" on October 6, 2026, after testing the rebuilt
mapping. Shift+Sync tempo-range cycling is accepted and the issue is complete.
The shared scope was separately confirmed from Djay's UI: changing range on
one deck changes it on all decks.

The exact numeric choices/order and individual C/D, held-button, and
release scenarios were not separately reported. Software regressions cover
the routing and ownership cases; the bridge delegates range choices and
wrap-around to Djay rather than implementing its own list.
