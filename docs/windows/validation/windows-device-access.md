# Windows device access: S4 MK3 inventory and selection limits

The named `verify.ps1 -Scenario device-inventory` run on Kensuke identifies
one S4 enclosure, its MI_03 HID collection and its MI_04 display candidate.
The current source, transferred files, native process records and raw results
agree. This is scoped Native QA confirmation of inventory and metadata,
including B1 scenario execution and B2 receipt coverage. Full task 3 acceptance
remains with the parent reviewer. HID data access, individual report framing,
controls, display access, motors and physical audio coexistence remain unproven.

Evidence paths below are relative to

    .omo/evidence/windows-bridge/20261104-01a1a4a4/

The root is represented below by E. R means E/task-3/provenance-repair.
The current raw inventory is R/native/live/device-inventory; unqualified
raw/ paths below refer to that directory. R/coverage.json indexes the native
receipts. E/task-3/scenario-wire/documentation-reconciliation.json records
this documentation review's field comparisons, exact hashes and nine
applicability classes. E/task-3/current-native/coverage.md remains historical.

## Current evidence and source identity

The connected inventory is R/native/live/device-inventory/inventory.json,
collected at 2026-11-04T07:13:03.7183956Z on KENSUKE Windows_NT. It records
READY_CANDIDATE and exactly one candidate. R/native/windows-version.stdout.txt
retains both the formatted and structured PSVersionTable: Windows PowerShell
5.1.26100.9444, Desktop edition, Microsoft Windows NT 10.0.26200.0. The
version process was PID 17068, exit 0, from 07:09:41.9631579Z to
07:09:42.8155015Z on November 4, 2026. Its stderr contains module-preparation
progress XML, not an empty stream.

The source handoff is E/task-3/scenario-wire/source-handoff.json. The
completed native handoff is E/task-3/scenario-wire/native-proof-complete.json,
SHA-256 `7a19c31237efe8d7deb88f4398f20fb4bfadabe53dc589a60b5290d5fbde9bcf`.
Its verdict was checked against source, JSON fields and raw receipts rather
than accepted on its own. These exact source hashes match the retained
source copies and native Get-FileHash output:

| Current file | SHA-256 |
| --- | --- |
| windows/verify.ps1 | `34ee3970e10b064888d36c160b3c2853fd44cbc621a40ac8af76dffe40e72283` |
| windows/qa/device-inventory.ps1 | `19a02baf4a155ba36c9bc89ec4d151ce4afc2fecda028f2b374b140b79e7ebaa` |
| windows/test.ps1 | `638e7509c9c581335ad0596e54750dc1e3668fc66e58bf230c23f2c34f459f1b` |
| windows/build-smoke/HidMetadata.cs | `f7e5fb1cd9a2a2c7358ee31c01363c17a9c618dc1fa2380a36b4acd060520b01` |

R/runtime-manifest.json contains 287 entries, SHA-256
`db0918dc3a34512c431c30e62ae219f20cb555d587096d00817364ce92358196`.
R/source-inputs.json has SHA-256
`9ff0693d930fb79530a0db7d5d64203c217951b8159331f7926e7f30fed47eb7`.
The fresh stage included the current dispatcher, inventory script and all
five fixture inputs. R/native/source-stage-filehash.record.json and its
stdout retain the literal native hash command and absolute transferred
paths. R/host/native-validate-stage.record.json records exit 0; its stdout
reports all 287 manifest entries verified. The retained task-2
transfer-ready-v3 was the staging base, not an updated runnable copy of this
scenario. No rebuild is implied by this transfer.

The paired capability result is
R/native/live/device-inventory-hid-capabilities-result.json. The actual
metadata apphost SHA-256 is
`60d86cc829f7815bc125d708891dd7f8850c5d581da2e2f9012610010dbf6232`;
the managed S4BuildSmoke.dll hash is
`ebb3b69da43b1c1c497ced9b686f8a40ae6a84a02070e06f2ed43f94366fee94`.
Native hash stdout and the runtime manifest agree on both.

The following process records retain full commands, UTC start/end, machine,
PID, exit and stdout/stderr hashes. All times are November 4, 2026 UTC.

| Process | Raw record under R | PID | Start to end | Exit |
| --- | --- | --- | --- | --- |
| Named scenario | native/live-device-inventory.record.json | 5360 | 07:13:00.9464602Z to 07:13:43.0570576Z | 0 |
| Inventory child | native/live/device-inventory-native.record.json | 13140 | 07:13:03.2740497Z to 07:13:42.5562993Z | 0 |
| Metadata child | native/live/device-inventory-metadata.record.json | 16560 | 07:13:42.5953267Z to 07:13:42.9329434Z | 0 |

Both children ran within the outer process interval. The aggregate result,
R/native/live/device-inventory-result.json, records
inventory-and-metadata-observed and hardware_acceptance false; its SHA-256
is `2a83dc1ed82fa553d4c1c133fb0021a48e506e39040e055eab153fe225a3774e`.
The inventory hash is
`11ac21c30987e6c1e4d7e59b773226d9d31c795f186cbe7572e49bd704399ff2`.
All seven inventory/raw files match the live sha256sums.txt manifest.

## Observed identity and bindings

The live raw/s4-match-detail.json contains six matching devnodes, all with
Status OK and container {6528CA11-A6A1-53FE-B1A5-38FE48FC1149}. The live
raw/pnp-devices.json explicitly projects Present true for each. The
composite is USB\VID_17CC&PID_1720\18F73BA2, at Port_#0001.Hub_#0006.
18F73BA2 is the instance-ID serial component; the separately queried
DEVPKEY_Device_SerialNumber is null. Its parent is
USB\VID_17CC&PID_1723\MSFT2018F73BA2, excluded by the exact PID matcher.

| Devnode | Class and service | Provider, version, INF |
| --- | --- | --- |
| Composite | USB, usbccgp | Microsoft, 10.0.26100.9444, usb.inf |
| MI_00 audio | MEDIA, usbaudio2 | Microsoft, 10.0.26100.9457, usbaudio2.inf |
| MI_03 USB HID interface | HIDClass, HidUsb | Microsoft, 10.0.26100.9444, input.inf |
| MI_03 HID collection | HIDClass, service property null | Microsoft, 10.0.26100.9444, input.inf |
| MI_04 Traktor Kontrol S4 MK3 BD | USBDevice, niks4m3bd | Native Instruments, 6.0.0.24, oem39.inf |
| MI_05 Traktor Kontrol S4 MK3 DFU | USBDevice, niks4m3dfu | Native Instruments, 6.0.0.24, oem53.inf |

The USB interface IDs are
USB\VID_17CC&PID_1720&MI_00\8&248F7348&1&0000,
USB\VID_17CC&PID_1720&MI_03\8&248F7348&1&0003,
USB\VID_17CC&PID_1720&MI_04\8&248F7348&1&0004, and
USB\VID_17CC&PID_1720&MI_05\8&248F7348&1&0005. Their location paths
contain USBMI(0), USBMI(3), USBMI(4), and USBMI(5), respectively. MI_04's
compatible IDs declare class/subclass ff/bd; MI_05 declares fe/01/01.

The collection is HID\VID_17CC&PID_1720&MI_03\9&21C48119&0&0000.
Its parent is the MI_03 USB interface above. The hardware IDs include
HID\VID_17CC&UP:FF01_U:0000 and HID_DEVICE_UP:FF01_U:0000. The paired
capability result independently records the same collection, interface,
composite and container, with interface_number 3, usage_page 65281
(0xFF01), usage 0 and link_collection_nodes 18. Its actual enumerated path is:

    \\?\hid#vid_17cc&pid_1720&mi_03#9&21c48119&0&0000#{4d1e55b2-f16f-11cf-88cb-001111000030}

This is an observed path on this installation, not a path to synthesize or
assume stable on another machine. HidMetadata.IsVerified in
windows/build-smoke/HidMetadata.cs checks the recorded ancestry, container
and usage hardware ID before opening exactly one verified collection.
The probe is pinned to this observed device; it isn't a general production
selector for other S4 serials.

## Report capabilities and handle boundary

| Report kind | Maximum bytes including ID | Observed report IDs, decimal |
| --- | --- | --- |
| Input | 79 | 1, 2, 3, 4 |
| Output | 95 | 48, 49, 50, 128, 129, 130, 142, 244 |
| Feature | 33 | 216, 217, 242, 248, 249 |

These are the result's input/output/feature.maximum_report_bytes_including_id
and report_ids fields. HidP button/value capabilities supply the ID sets;
constant-only IDs may be absent. The arrays retain usage, link-collection,
bit-size and count metadata. They aren't raw descriptor bytes or proof of
individual Windows read lengths, padding or write framing.

The PnP inventory script opens no HID collection. The separate capability
probe did open a verified metadata-only handle with desired_access 0,
share_mode 3, creation_disposition 3 and inherited false. It queried cached
preparsed metadata, freed that data and closed the handle before output.
The receipt records closed_before_output true, report_reads 0,
report_writes 0 and hardware_acceptance false. Calling the combined evidence
"no handles opened" would be incorrect. Neither that handle nor exit 0
establishes GENERIC_READ/GENERIC_WRITE access or report delivery.

## Services, processes and audio discovery

At the inventory observation, raw/services.json records NIHardwareService
Running, StartMode Auto, ProcessId 4772 and PathName:

    "C:\Program Files\Common Files\Native Instruments\Hardware\NIHardwareService.exe"

The exact-name record matters: the script's broad name filter also returns
unrelated services. raw/processes.json contains NIHardwareService PID 4772,
djayApp PID 15608 at
C:\Program Files\WindowsApps\59BEBC1A.djay_5.6.7091.0_x64__e3tqh12mt5rj6\djayApp.exe,
and loopBeMon PID 10076 at
C:\Program Files (x86)\nerds.de\LoopBe1\loopBeMon.exe.
No Traktor, Mixxx, NIHardwareAgent or matching bridge/probe process appears
in that filtered snapshot. It doesn't prove exclusive device ownership,
service dependencies, binary signature, or the state now.

raw/audio-interfaces.json contains only the MI_00 MEDIA record. Despite its
comment, the source filters MEDIA/AudioEndpoint records by VID/PID in the
instance ID, not by container membership. It therefore doesn't enumerate
all same-container audio endpoints. raw/usb-children.json separately lists
Microphone and Speakers (Traktor Kontrol S4 MK3) AudioEndpoint records in
the MI_00 Related array. Their IDs are
`SWD\MMDEVAPI\{0.0.1.00000000}.{853DE6B3-3B09-4DD6-9968-2571CE349EB2}`
and `SWD\MMDEVAPI\{0.0.0.00000000}.{32573682-A36B-435C-A6D6-1F1FB4492729}`,
respectively. Related includes siblings as well as children.
This is discovery evidence only; no audio stream, routing or audible
master/headphone/input behavior was tested here.

## Authoritative presence and selection

The native authority is Get-PnpDevice -PresentOnly, followed by the explicit
Present = true projection. Select-S4Candidates requires a Boolean Present
on every input record, including unrelated records, and filters false
records before both S4 and unrelated HID/USB classification. Status is a
health field and never substitutes for presence. Attached Unknown-health
fixtures remain eligible; absent ghosts don't.

Test-S4Match requires VID_17CC&PID_1720 followed by a backslash or ampersand,
excluding PID 1723 and 17200. Matching records group by container. One
container must have exactly one top node whose parent lies outside the
matched set before READY_CANDIDATE is returned. The source uses
case-insensitive identity comparisons. It doesn't select the first match.

| Status | Exact selection meaning | Exit |
| --- | --- | --- |
| READY_CANDIDATE | One container and one top candidate | 0 |
| ABSENT | No present match and no present unrelated HID/USB record | 2 |
| UNRELATED_ONLY | No present match, but present unrelated HID/USB records | 2 |
| MULTIPLE | Multiple container keys, or top-node count differs from one | 2 |

MULTIPLE candidates can be container keys or NOPARENT: instance keys in
the multiple-container branch, and top IDs in the other ambiguity branch.
They aren't always openable paths or composite IDs. An unplugged controller
on a machine with other HID/USB devices can yield UNRELATED_ONLY, not ABSENT.
The recorded absent fixture is an empty list; no refreshed physical unplug
run is present.

## Commands and observed checks

The current verify.ps1 allowlist includes device-inventory alongside
build-smoke and hid-capabilities. The exact successful invocation retained
in R/native/live-device-inventory.record.json is:

```powershell
C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File "C:\Users\toron\AppData\Local\Temp\s4inv-receipts-01a1a55c\runtime\verify.ps1" -Scenario device-inventory -RuntimeRoot "C:\Users\toron\AppData\Local\Temp\s4inv-receipts-01a1a55c\runtime" -EvidenceRoot "C:\Users\toron\AppData\Local\Temp\s4inv-receipts-01a1a55c\receipts\live"
```

This is the recorded command, not a still-existing installation: cleanup
removed that temporary root. A future run needs a newly staged, hash-verified
runtime and a new owned EvidenceRoot. The dispatcher takes -RuntimeRoot and
-EvidenceRoot, not -Evidence. It rejects existing evidence roots and checks
the manifest and required dependencies before starting the inventory child.
That child takes -Evidence and has a 120-second bound; the separate metadata
child has a 20-second bound. Both use Invoke-S4Process from windows/test.ps1.
The inventory script itself uses New-Item -Force, so direct invocation has
no existing-directory refusal. Direct fixture mode adds -FixtureJson and
writes fixture-selection.json. Native mode writes inventory.json, six raw
JSON files and sha256sums.txt.

Five fixtures ran as separate native Windows PowerShell 5.1.26100.9444
processes. Each R/native/fixture-NAME.record.json has the literal command,
actual times, PID, exit and stream hashes. Its semantic output is
R/native/fixture-NAME/fixture-selection.json; source inputs remain at
E/task-3/raw/fixture-NAME.json and their transferred hashes are in the native
hash stdout and runtime manifest.

| NAME | Actual status | Candidates | PID | Exit |
| --- | --- | --- | --- | --- |
| absent | ABSENT | none | 8340 | 2 |
| unrelated | UNRELATED_ONLY | none | 15368 | 2 |
| multiple | MULTIPLE | NOPARENT:USB\VID_17CC&PID_1720\S4A and NOPARENT:USB\VID_17CC&PID_1720\S4B | 15284 | 2 |
| ghost | ABSENT | none | 10104 | 2 |
| attached-unknown-health | READY_CANDIDATE | USB\VID_17CC&PID_1720\18F73BA2 | 13292 | 0 |

Expected exit 2 remains a recorded nonzero result. These checks establish
software selection on Windows, not physical unplug or two-controller trials.
The native invalid-runtime-root and missing-inventory-script records each
show exit 1 with the corresponding raw diagnostic. The held script was
restored and its hash checked before the live run, ending at
07:13:00.9138706Z. Malformed Present and broken JSON were not newly exercised
natively; their retained failures are local PowerShell evidence.

The prior source-repair run records 18/18 expected results on macOS
PowerShell 7.5.4, including malformed presence/JSON, ghost filtering, exact
PID matching, literal hostile display text, native-shaped projection replay
and historical same-container grouping. See current-native/coverage.md for command times,
exact artifacts and the distinctions between native observations and replay.
No tests were added or rerun for this documentation-only change.

## Receipt reconciliation and cleanup

The documentation comparison found matching bytes for all 148 distinct
files referenced by R/coverage.json and all 74 recorded stdout/stderr
streams from 37 process receipts. Its 62 coverage rows include version,
transfer, five fixtures, failure boundaries, live inventory/metadata and
cleanup. R/native-complete-receipts.json has SHA-256
`88f3798a65a483be9e7b941719f4f336a3688ef72f9f020a1fd8f57802e130d1`;
R/coverage.json has SHA-256
`74d83cb22d616fa1271b4d81c26c3d76c8db0bdee4ae4fd47c59bf5274e0b80e`.
Receipt completeness here applies to the recorded inventory scope.

| Changed claim | Actual comparison and limit |
| --- | --- |
| Named route exists and ran | Current verify.ps1 allowlist/branch, native/live-device-inventory.record.json command/exit, nested child records and aggregate JSON agree. |
| Source reached Windows unchanged | Current file hashes, source-inputs.json, runtime-manifest.json and native/source-stage-filehash.stdout.txt agree on scripts, fixtures and metadata binaries. |
| One present enclosure and the same MI_03/MI_04 bindings | inventory.json candidates/compositeId, raw/pnp-devices.json Present, raw/s4-match-detail.json ancestry/container/driver keys and metadata identity agree. All six serial-number properties are null; 18F73BA2 is the composite instance-ID component. |
| Metadata scope remains bounded | Capability maxima and ID arrays agree with the result; desired_access 0, closed_before_output true, zero report reads/writes and hardware_acceptance false agree with HidMetadata.cs. |
| Native fixture receipts are now complete | Each fixture's input hash, raw process exit and selection JSON agree. Missing-Present/broken-JSON cases remain local-only. |
| Cleanup has actual post-removal evidence | host/cleanup-native-file.stdout.bin records timestamped path/PID checks; its host process record records exit 0. Local stat and process-query records retain their expected exit 1 and raw absence output. |

The earlier evidence-driver variable collision and oversized SSH cleanup
command remain recorded as exit 1 in R/host/native-environment.record.json
and R/host/cleanup-native.record.json. The corrected manifest validation and
file-based cleanup completed separately. Successful version/hash subcommands
weren't relabeled as a successful outer environment run.

R/host/cleanup-native-file.stdout.bin records post-removal checks at
2026-11-04T07:18:08.4773691Z: the owned s4inv-receipts-01a1a55c directory
and archive, prior s4inv-proof-01a1a4a4 root and obsolete s4debug01a1a4a4
root were absent. All paths are under
C:/Users/toron/AppData/Local/Temp/. No matches remained for the 19 recorded
native PIDs or owned executable paths. The obsolete caps root was removed
after comparison with 283 retained local v2 files and a native ownership
check. Local v2/v3 evidence remains retained.

R/host/cleanup-local-stat.record.json and its stderr record the owned work
directory and /tmp/s4-task3-proof-1uHGda absent. The local process query
returned no rows for its 19 prior external PIDs. R/operator-release.json
records release of the native operator. These are observations at their
recorded times, not a new remote state check by this documentation executor.
No scheduled task was created by that run; no global scheduled-task absence
query is claimed. This documentation pass created no temporary roots,
child processes, subscriptions, transfers or hardware resources.

## Preserved history and downstream gaps

The direct inventory at E/task-3/current-native/live/inventory.json remains
the observation from 2026-11-04T06:20:14.0796438Z. Its paired task-2
native/task2-v3-hid-caps process ran from 06:19:57.3199990Z to
06:19:57.6350672Z, PID 14928, exit 0. The current-native summary and task-2
harness-repair/console-proof.json labels of 06:35:00Z are retained summary
times, not execution timestamps. That direct-run evidence lacked separate
inventory and per-fixture process receipts. The first named-route proof at
E/task-3/scenario-wire/native-proof.json and task-3/native-proof/ also
remains history; its abbreviated commands and approximate outer times
aren't substituted for the fresh complete receipts.

The old missing-route assertion and receipt gaps are closed for the new
run by the exact current source and R evidence above. Historical records
weren't rewritten. The old summary's purported missing-Present
fixture-absent case remains uncorroborated: the retained absent input is
an empty array. Likewise, fixture IDs S4A/S4B aren't proof of physical
ghost devices. Neither attribution is repaired by a later metadata pass.

The old native inventory at E/task-3/inventory.json is dated
2026-11-04T04:19:58.5456672Z and used script SHA-256
5eb76510f0e3b0ab3bf733f5af77bbe9c5da8757bd27e06975c2343ab1110f8f
from run-record.json. It remains historical. The intermediate source
12429bad33d67ab8e842cb4e07ee67e510edfb9d7344c6b5694d479758f27412
omitted Present in its native projection and misclassified unrelated ghosts;
its failing executions are preserved in presence-repair-attempt3. The
initial independent-audit.md records an even earlier source identity,
8ac09da15369ff127b0edd362a11345127449941ca7b20063a7a060fe92a29d6,
and a then-missing native observation. Its no-channel/no-device finding
is superseded by the refreshed artifacts, not a current blocker.

The earlier capability report remains at
E/task-2/native-caps/task2-hid-caps/hid-capabilities-result.json. Its bytes
match the new result, SHA-256
aacd5c214564de3cf07532f24476eed8f16947dd4061951cb0476604274ca938.
The new process receipt establishes the new execution, rather than a
changed JSON payload establishing freshness.

Task 9 still needs writer and snapshot data handles, actual Windows raw
and normalized framing, report reads/writes and failure behavior. The
existing src/input.rs contract requires reports 1 = 23, 3 = 60 and 2 = 79
bytes including ID, with touch/buttons before movement and stale-interrupt
rejection. The collection maxima don't prove those individual lengths.

Task 12 still needs an actual safe MI_04 open/claim/write/release path,
both displays visibly showing a bounded frame, and concurrent HID/audio
checks. src/screens.rs names interface 4 and bulk OUT endpoint 0x03;
these are source requirements, not observed Windows endpoint access.
The observed niks4m3bd binding doesn't establish WinUSB compatibility.
Service signature/dependencies, access privileges, ownership effects and
restoration remain untested. A service stop alone wouldn't prove access.
No service or driver change is authorized by this evidence.

Physical audio acceptance remains separate. These records don't prove
playback, cue/headphones, inputs, or coexistence while controlling HID and
screens. The documentation reconciles the named-scenario identity record
with these limits; it doesn't certify full task 3 acceptance, tasks 9 or 12,
GUI acceptance, or runtime parity. The existing task-2 GUI authentication
blocker and pending visual acceptance are outside this inventory proof.
