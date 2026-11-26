# Windows native utility design

Reference: actual macos/DESIGN.md, Bridge/Screen/Jog/TT/LED/Palette/Diagnostics
views, NumberRow/DecimalRow primitives and BridgeStatusMenu, not a web dashboard.
Use ordinary Avalonia Fluent 11.3.8 controls and system theme, no paid components.

## Tokens

`Theme` is the only owner of geometry: 4/8/12/20/32 spacing, 14 body,
12 caption, 18 section, 24 page, 30 display typography; 4 corner radius;
120 numeric input, 180 action width, 760x540 minimum, 860x720 initial window.
Colors use Fluent dynamic resources: SystemControlForegroundBaseHighBrush,
SystemControlForegroundBaseMediumBrush, SystemControlErrorTextForegroundBrush,
SystemControlHighlightAccentBrush, SystemControlBackgroundAltHighBrush.
System font; Cascadia Mono/Consolas only for logs/command lines.
Native templates own hover/pressed/focus/disabled appearance and transitions.

## Brief

DJ preparing or ending a set: actual bridge status first, Start/Stop second,
editor/saved/launched distinction third. Save is the sole accent action.
No marketing graphics, invented connection metrics, web fonts or decoration.
Six tabs preserve every H-01..H-80 workflow; unavailable screen access and service
policy are stated, never disguised as a working handover button.

Layout: bounded scroll-body-shell; fixed status/actions, fixed six-tab strip,
one ScrollViewer per tab body, fixed Save/reset footer. Diagnostics log has its
own bounded TextBox viewport for selectable text. No outer window scroller.
Number, choice, checkbox, slider, section, hint and action primitives are reused.
All numeric fields have visible labels and accessible names. Optional values
use an explicit inherit/disabled checkbox. Changes do not resize action slots.

## QA boundary

Root owns real Windows UI QA. No Windows/SSH/MCP/hardware contact is permitted
in this worker. Compile/model evidence is not visual acceptance.
Open: all six tabs at initial/minimum sizes, light/dark, bottom scroll, keyboard
order/focus, high DPI, tray states, long errors/logs, dialogs, system shutdown.
WCAG contrast/focus/target/reflow assessments remain UNVERIFIED until rendered
native evidence. No mobile/web surface exists.
