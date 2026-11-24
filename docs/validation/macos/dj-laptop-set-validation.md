# DJ-laptop set: follow-up usage validation

October 24, 2026, with an October 25 acceptance clarification.
Internal acceptance record, not a setup guide.

## User report

The user reported playing an hour-long set with the latest build on the actual
DJ laptop and described the result as working fantastically. No failure was
reported in that follow-up.

This followed the rebuilt musical-key fix and its focused physical check:
Jacker `11A`, direct replacement with Drop Ha `2A`, then eject `--`.
Build and package evidence for that increment is in
[key feedback validation](key-feedback-order-validation.md).

## October 25 acceptance clarification

The user explicitly confirmed that held Preview scrubbing, Shift+CUE, library
controls and screen states across decks all worked in this test set. These
normal-use acceptance items are complete. The user also confirmed working
LED/FX behavior and reported discovering no LED/FX edge-case problems.

The older LED/FX checklists describe optional exhaustive regression scenarios,
not additional observed failures or required work before accepting these
features. The separately documented leftmost manual-FX button duplication of
slot 1 remains an unresolved behavior.

## What this establishes

The report adds successful ordinary DJ use on the intended laptop to the
earlier software, native-MIDI and focused hardware checks. It supports the
current app workflow rather than only a short diagnostic run.

The clarification establishes acceptance of the named controls and cross-deck
screen behavior. It does not claim an instrumented pass of every simultaneous
button combination, mapping-reconnection case, driver-restoration path or
deliberately induced 50 ms input gap. Those evidence boundaries remain in their
existing validation records; they do not make the accepted controls unfinished.

The prior diagnostic run's `wheel 0 sensor time reversed` exit was not changed
by the key fix. A successful set does not establish that the condition was
triggered, explained or fixed.

## Documentation follow-up

The user requested a maintained, accessible guide explaining workflows and
the chosen mappings/functions. [user-guide.md](user-guide.md) is the canonical
user-facing guide. Dated acceptance reports, test counts, local artifacts and
investigation history stay in validation records rather than that guide or
the public README.
