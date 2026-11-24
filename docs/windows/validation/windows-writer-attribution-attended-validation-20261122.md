# NI output writer attribution: attended run

## Finding

`NIHardwareService.exe` directly submitted the exact extra zero-force motor
report and additional ring reports while the bridge held its acquired NI
session. This is payload-to-process/thread/caller evidence, not a CPU-activity
correlation.

The motor report was:

```text
31 01 e0 fe 00 00 01 e0 fe 00 00
```

The target was PID **5588**, thread **15320**, handle **0x670**, using
`ntdll!NtWriteFile`. The service executable was:

```text
C:\Program Files\Common Files\Native Instruments\Hardware\NIHardwareService.exe
SHA-256 88f90c4b378f547508b09f190b4bcb30a35cbc5f5d16a7a0d533267a2c1956d9
```

The motor stream's retained stack sample includes
`KERNELBASE.dll+0xb204a`, followed by
`NIHardwareService.exe+0x238a13`, `+0x25d6ff`, `+0x25add9`,
`+0x248cc4`, `+0x237194` and `+0x228d1f`. No private function names
are inferred from these offsets.

## Scope and outcome

This was one B/acquired run using the corrected r2 writer-trace package and
frozen kit 3. The bridge acquired NI without publishing screen images.
The user gave the watching confirmation through the console boundary and
performed Start, normal Stop and Quit. No A/C/D or automatic repeat followed.

The user reported:

- Motors: "Struggled or chattered".
- Rings: "Flickered".
- GUI exit: "Yes, fully quit".
- Playback deviation: "I missed the instruction not to play the track (I did)".
- Stop confirmation: "I did type \"Stopped\". It seemed to not do anything
  for a while."

Playback means this is not the planned paused comparison. It does not remove
the direct writer attribution. The original `result.json` remains
`evidence_complete: false`; it has not been rewritten to call the test clean.

## Observed submissions

| Producer | Report | Count |
| --- | --- | ---: |
| NI service | Exact zero-force `0x31` signature | 32,245 |
| NI service | `0x32` ring reports | 6,101 |
| Bridge | All native motor report writes | 4,184 |
| Bridge | Motor writes with nonzero force magnitude | 2,429 |
| Bridge | Native ring report writes | 1,238 |

The NI zero-force submissions span 64.477558 seconds, approximately
**500.081 Hz** using the intervals between the first and last requests.
Every one uses the same thread, handle and retained motor stack sample.
The original API buffer length is 95 bytes; the canonical motor report is
eleven bytes. All NI ring submissions also use thread 15320 and handle 0x670.

Every zero-force native return is **259 / STATUS_PENDING**. These are observed
submissions, not proven USB completions. No final I/O status-block completion
or controller-applied torque is inferred.

Each of the bridge's 2,429 positive-force submissions has a following NI
zero-force submission in the API timestamp record. The delay is a median
1,123 microseconds, 95th percentile 2,197 microseconds and maximum
3,630 microseconds. These are software submission timings, not bus arrival
or applied-torque measurements.

The evidence supports a competing-output explanation for the reported
chatter and flicker. It does not establish that NI deliberately detects and
opposes bridge commands, or prove the controller's arbitration/last-command
semantics.

## Acquisition and release

All timestamps are UTC on November 22, 2026:

| Event | Time |
| --- | --- |
| NI collector attached | 23:19:43.4254188 |
| ETW armed | 23:19:43.6543322 |
| GUI launched | 23:19:43.7331642 |
| Bridge NI connect began | 23:20:22.844025 |
| First NI zero-force request | 23:20:23.472983 |
| Bridge NI ready | 23:20:23.625863 |
| Acquired marker | 23:20:23.6564632 |
| Twenty-second hold ended | 23:20:43.7643090 |
| Bridge release began | 23:21:27.922878 |
| Last NI zero-force request | 23:21:27.950541 |
| Bridge release completed successfully | 23:21:27.961084 |
| Operator stop/release confirmation accepted | 23:21:44.6735103 |
| Overall deadline marker | 23:22:12.0761287 |
| ETW save/stop receipt finished | 23:22:20.0338667 |

The NI motor stream starts 628.958 ms after connect begins and 152.880 ms
before bridge readiness. Its last request is 10.543 ms before successful
release returns. No further matched NI motor or ring request appears after
that release in the retained call trace.

This run's output stopped with release. That does not erase the earlier
mistimed B evidence where output continued after a release acknowledgement;
the difference in NI lifecycle/ownership remains unresolved.

The operator's stop confirmation was accepted. The following quiet console
period was the post-release wait. The 150-second overall budget cut that
formal 35-second wait short after **27.403 seconds**. The deadline occurred
44.115 seconds after the bridge's recorded successful release, because
operator confirmation followed that release by 16.712 seconds.

## Integrity and shutdown

The NI trace contains 38,346 entries and 38,346 matching returns, with unique,
contiguous sequence IDs and no missing entry/return IDs. It reports zero
collector API errors and zero outstanding calls.

The bounded stack sample table reached 256 entries. Exactly 1,319 entries
have no stack sample, accounting for the reported `dropped: 1319` counter.
The motor stream retains stack ID 3 throughout. No missing call/return
sequence was observed, but full stack coverage is not claimed.

That nonzero counter caused the collector's final completeness validation
to reject Stop. The actual stop receipt nevertheless reports
`reason: operator`, `hook_count: 0` and `outstanding_calls: 0`.
No owned bridge worker, GUI or injector process remained in the post-run
process check.

The bridge footer reports `runtime_ok: true`, `incomplete: false` and zero
losses across output (62,896 records), input (37,494) and screen (5).
All five screen records are lifecycle events; no screen image publication
appears.

ETW start and stop both returned zero. Its scoped receipt reports complete
save, no deadline activation, no failure and no cancel. `tracerpt` processed
73,563 events and reported zero events lost; the decoded header also reports
zero buffers lost. It warned that some events do not match the schema.
The original ETL is preserved. No USB payload/IRP/completion join or kernel
writer attribution is claimed from that ETL.

## Preservation and retained evidence

All 515 frozen source-app files still match the pre-run manifest. The installed
mapping is unchanged. Of 24 snapshotted preference/log files, only
`Logs\last-session.log` changed during normal app use; its original is retained
in the run's `preferences-before` snapshot. Saved tuning/settings files are
unchanged. Previous apps, mappings and captures were not removed or rewritten.
No service, driver or registry configuration was changed.

Original run:

```text
C:\Users\toron\Desktop\S4 Output Writer Trace-20261122-9fbe057e-ready\captures\
20261122-181126-B-2a6051e2e8654f9b86a80f13549d990d
```

Compact local evidence:
`.omo/evidence/s4-writer-attended-20261122-7bb73579/`.
The exact 17-file recording snapshot is retained as `recording.zip`,
3,562,989 bytes, SHA-256:

```text
f348f13d6c9917a1c66a3c29166a761490c02cdd989d69de3d5ff21c6f0fe9e8
```

`recording-file-hashes.json` binds its individual files.
`writer-attribution.json`, `lifecycle-and-interleaving.json`,
`user-observations.json`, `usb-summary.txt` and preservation/cleanup receipts
provide the focused evidence without retaining temporary decoding trees.

The next implementation investigation is NI output ownership during
screen acquisition. No suppression patch, service stop, private protocol
change or further hardware run was performed in this attended task.

## References

- `docs/windows-usb-output-ownership-investigation-20261122.md`
- `docs/windows-writer-trace-preparation-validation-20261122.md`
- `windows/qa/writer-trace/README.md`
- `https://learn.microsoft.com/windows-server/administration/windows-commands/tracerpt`
- `https://learn.microsoft.com/windows-hardware/drivers/ddi/wdm/nf-wdm-zwwritefile`
