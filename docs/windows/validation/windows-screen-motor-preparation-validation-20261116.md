# Diagnostic preparation startup validation

## Scope

The first diagnostic A/off run held motor force at neutral because Loaded and
Internal feedback stayed unknown. Endpoint-before-receiver ordering already
existed in the previous working build. The effect of the new capture setup on
that timing was not measured, so it is not an established physical root cause.
The user requested removal of that added startup risk regardless.

## Change

Diagnostic startup now has a preparation barrier:

1. The host prepares its owned BLOOP lease without publishing the endpoint.
2. It starts the diagnostic worker with `--prepare-screen-motor-diagnostics`.
3. The worker completes capture configuration, allocation and metadata output,
   then flushes `S4_SCREEN_DIAG_PREPARED 1`.
4. The host creates the endpoint and records the worker identity.
5. It flushes `S4_SCREEN_DIAG_BEGIN 1`. Only then does the worker enter its
   existing device, MIDI and screen runtime.

The preparation wait is bounded and belongs to the current child launch.
Early failure or Stop during preparation prevents endpoint publication; failure during creation
keeps cleanup behind completion of the owned SDK operation. STOP or stdin EOF
while the worker awaits permission exports the aborted capture and exits
without entering the device runtime.

Ordinary startup without the diagnostic flag is unchanged. This change does
not fabricate playback state, alter force laws, weaken authorization or
change the mapping. A preparation receipt is not bridge/screen readiness.

## Verification

- Strict Rust formatting and all-target/all-feature Clippy passed.
- Full source Rust suite: 617 tests passed.
- Swift suite: 80 tests passed. Existing unhandled resource/script warnings
  remain.
- Full host suite on macOS: 125/125 passed.
- Shell syntax, Info.plist lint and diff whitespace checks passed.
- Native Windows Rust suites: 128 shared, 279 worker and 2 CLI tests passed.
- Native Windows host suite: 125/125 passed.
- Native preparation-only runs of the exact packaged worker passed with both
  managed STOP and stdin EOF. Metadata/capture reservation existed before the
  prepared receipt; both exited zero. Their lifecycle sequence was 1,4,2:
  preparation completed and aborted without the runtime grant. No input,
  native HID write, screen acquisition or frame publication events occurred.
- The startup-grant test failed for the intended reason when the production
  accepted version was temporarily changed to 2. The source was restored
  byte-for-byte, and the affected test passed again.
- Real supervised child pipes and a fake endpoint verified that preparation
  gates publication and runtime grant, and that failure, EOF, endpoint
  creation failure and Stop preserve cleanup.

C# LSP was unavailable; the warning-as-error compiler and both host suites
passed. Rust diagnostics for the new barrier and CLI were clean.

## Package identity and preservation

Prepared worker SHA-256:

`75bf26f55c1cfbe8bde7e25bf7e8e325922f6d044e8905235b4a5bdf8747618c`

App build ZIP SHA-256:

`33bb50a0c2deff0be3323ab4dbc6329c23aff9ffc643ccc673cfb235b3e57333`

The generated mapping retains SHA-256:

`b7834821c10c2814a46c15273ae2bb50b132a0dc386319004f9cb513f41fb94e`

The package was delivered stopped in **S4 Screen Motor Diagnostics 2** on
the Windows desktop. All 514 runtime payloads and 529 portable ZIP entries
were verified. The portable ZIP SHA-256 is:

`0ab4ada90fd402803585d2f49c8691544bf87dde1111b20aba8fb69def167631`

The first diagnostic folder and its A capture, accepted BLOOP package and
saved settings are preserved. The original checkout's 12 protected platter
files matched their retained hashes. Build/source identities, native results,
preparation traces and final delivery/cleanup receipts are retained under:

`.omo/evidence/s4-preparation-fix-20261116-bB5QZQ/`

The source snapshot records the compiled dirty source against
`60df4353335f597d644ccb2aba82e2f743407c3c`; this closeout record follows that
snapshot. Delivery uses a create-only destination, verifies runtime files and
portable ZIP entries, and compares the preserved files/settings with their
baseline. The delivery receipt records completion and the portable ZIP hash.

## Limits

No diagnostic GUI or controller run was started. The native preparation
checks intentionally never sent BEGIN, and software endpoint tests used
fakes. Actual Loaded/Internal startup delivery and physical motor recovery
remain for the next attended A run. Passing these checks proves the removal
of diagnostic preparation from the published-endpoint interval, not that the
reported physical problem has been fixed.

The permanent replacement package is retained for that attended use.
Task-owned temporary build and Windows staging output are removed after
preserving their useful evidence; cleanup receipts record the result.
