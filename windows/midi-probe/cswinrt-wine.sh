#!/bin/bash
set -euo pipefail

: "${CSWINRT_PACKAGE:?Set CSWINRT_PACKAGE to the restored CsWinRT package directory}"
: "${WINEPREFIX:?Set WINEPREFIX to an isolated Wine prefix}"
: "${CSWINRT_METADATA:?Set CSWINRT_METADATA to the restored Windows SDK metadata directory}"
: "${CSWINRT_GENERATED:?Set CSWINRT_GENERATED to the projection output directory}"
: "${MIDI_API_PACKAGE:?Set MIDI_API_PACKAGE to the restored MIDI API package directory}"
export WINEDEBUG=-all

# cswinrt treats Unix absolute paths as command options. Wine maps / to Z:/.
exec wine "$CSWINRT_PACKAGE/cswinrt.exe" \
    -target net8.0 \
    -input "Z:$CSWINRT_METADATA" \
    -input "Z:$MIDI_API_PACKAGE/ref/net10.0/Windows.Devices.Midi2.winmd" \
    -output "Z:$CSWINRT_GENERATED" \
    -include Windows.Devices.Midi2 -exclude Windows -exclude Microsoft
