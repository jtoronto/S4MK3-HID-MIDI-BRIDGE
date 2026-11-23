# Windows USB capture launcher preparation - November 16, 2026

The stopped launcher was delivered to
`C:\Users\toron\Desktop\S4 Screen Motor Diagnostics 2\07-usb-pair.cmd`.
Its ordered instructions are `USB CAPTURE START HERE.txt` in the same folder.
This preparation did not start the product bridge, a real USB capture, HID,
MIDI, motors or screens. It did not install or restart a driver or NI service.
The existing diagnostic app was reused without rebuilding it.

The launcher prepares A with screens off, then requires an explicit normal
baseline confirmation before preparing B with the screen connection acquired
and no image frames. Both runs use the same right Deck B track and TT playback,
untouched wheels and at least 20 seconds of observation. Each run requires the
operator to confirm watching before arming USB capture and opening the stopped
GUI. The GUI must be stopped and quit before USB shutdown and observation notes.

USB capture uses the inspected USBPcap 1.5.4.0 executable, pinned by SHA-256
`8b13173e9453fda9914d707c732880de44081e20e1113b3a2c827abd7a28b15d`.
Preflight enumerates topology using `--extcap-config`, checks the S4 address
and does not start recording. The selected interface/address are USBPcap1/device 3;
a changed address stops preparation rather than guessing.

The helper preserves binary stdout, validates and flushes a complete PCAP
header before authorizing GUI launch, and uses an isolated console key event
for USBPcap's `q` shutdown. Redirected stdin text is not used. Failure cleanup
targets only the owned capture process. A normal process exit, a structurally
valid file and motor/ring request coverage are all required before proceeding.
Input closure cancels the GUI-exit wait without killing the GUI.

Native Windows PowerShell 5.1 verification passed:

- 17 PCAP parser fixture cases covering valid coverage, wrong target,
  malformed/truncated records and USBPcap header variants.
- Eight capture-host cases covering partial-header gating, valid and invalid
  headers, unexpected exit, the actual DLL-loader/console-stop wrapper, failed
  arm preventing GUI launch, real Run-Case callback ordering, and real Run-Case
  input closure with the fake GUI left alive.
- Six pair sequencing cases through the shared production sequencing owner,
  real bridge validator and PCAP reader: success, missing coverage, incomplete
  bridge evidence, USB failure, input closure and cancellation. Failed A cases
  preserved their files and did not create B.
- The shared bridge validation and GUI-exit retry/cancellation suite.
- Five isolated regression mutations rejected for their intended behavioral
  failures: omitted header rejection, omitted pre-GUI arm, omitted motor/ring
  coverage gate, omitted EOF cancellation and an incorrect endpoint filter.
- All 11 staged PowerShell scripts parsed; the combined .NET Framework helper
  compiled with warnings treated as errors and its exported types loaded.

The final helper SHA-256 is
`ea052b5ceb270701a1b269de75eff9cac3d0a73d51130f119f75cbfb053ba68c`.
The delivered identity SHA-256 is
`89d1a829a88f501de88dcbed11c3c9e05084c39f7fe67d1c201ef2a1ccf36fe6`.
The identity covers the launcher, helper, sources and existing preparation/build
identity files. No controller or USBPcap capture process was running at delivery.

The two replaced scripts, Run-Case.ps1 and README.md, were checked against their
previous delivered hashes and backed up under
`Previous launcher scripts\s4-usb-launcher-20261116-gQ2fg6`.
The post-delivery checks preserved 148 other files, including the existing
build/capture evidence, all 24 files in the preference tree and all 13 installed
mapping files. Existing A/B recordings and their operator correction were retained.

Evidence is under
`.omo/evidence/s4-usb-launcher-preparation-20261116-3badf02f/`: native logs,
specific mutation failures, compiler and delivery receipts, baseline and launcher
identity, and a verified small source/runtime ZIP. The ZIP is an evidence
snapshot, not a standalone app distribution.

The task-owned Windows staging directory and `/tmp` workspace were removed
after evidence retrieval. The permanent desktop launcher, its two script backups
and the small evidence archive are intentionally retained for the later attended
test. No SDK or temporary build tree was retained.

Git's scoped diff check hung, including with filesystem monitoring disabled and
the system Git binary. It could not be completed. An independent scan of all QA
text files and the user guide found no trailing whitespace. Rust, Swift and
Avalonia product code was not changed or rebuilt for this launcher.

No attended USBPcap/S4 run occurred. The synthetic fixtures establish software
orchestration and byte handling, not actual USB traffic coverage or physical
motor behavior. Normal USBPcap shutdown does not prove a complete kernel tail or
zero USB packet loss. USBPcap requests do not identify their originating user
process; bridge-versus-NI attribution and the screen/motor cause remain unproven.
Fresh confirmation that the operator is watching is still required before a
real diagnostic.
