# Local operator console for one trial host. Never opens a device or NI pipe.
$ErrorActionPreference = 'Stop'
$exe = Join-Path $PSScriptRoot 'app\S4Bridge.exe'
if (!(Test-Path -LiteralPath $exe -PathType Leaf)) { throw 'Extract the complete trial kit first.' }
Write-Host 'Paused JOG only. Close Traktor and other bridge hosts. Read ATTENDED-RUN.md.'
if ((Read-Host 'Operator present, watching, both wheels stopped? Type WATCHING') -cne 'WATCHING') { return }

$run = Join-Path $PSScriptRoot ('runs\' + [guid]::NewGuid().ToString('N'))
[void][IO.Directory]::CreateDirectory($run)
$control = Join-Path $run 'command.txt'
$journal = Join-Path $run 'operator.log'
function Record([string]$entry) {
    [IO.File]::AppendAllText($journal, ([DateTime]::UtcNow.ToString('o') + ' ' + $entry + "`r`n"))
}
[IO.File]::WriteAllText($control, 'idle', [Text.Encoding]::ASCII)
$env:S4_BRIDGE_NI_JOGT_TRIAL_FILE = $control
$env:S4_BRIDGE_EXPERIMENTAL_NI_DECKS = '1'
Remove-Item Env:S4_SCREEN_MOTOR_DIAGNOSTIC_CONFIG -ErrorAction SilentlyContinue
Record 'launch idle'
$trialHost = Start-Process -FilePath $exe -WorkingDirectory (Split-Path $exe) -PassThru
Write-Host "Notes: $journal"
Write-Host 'Click Start bridge in the trial host. Wait for both screens and trial idle in Diagnostics.'
Write-Host 'A = 25; B = 244; R = restore 107. Between values use I = idle and wait for Diagnostics.'
Write-Host 'Q exits this console AFTER normal Stop bridge and Quit. It does not stop the bridge.'
while (!$trialHost.HasExited) {
    $choice = Read-Host 'A / B / R / I / Q'
    if ($choice -eq 'Q') {
        if (!$trialHost.HasExited) { Write-Host 'Stop bridge and Quit the trial host first.'; continue }
        break
    }
    if ($trialHost.HasExited) { break }
    $value = switch ($choice) { A {'25'} B {'244'} R {'107'} I {'idle'} default {''} }
    if (!$value) { Write-Host 'No change.'; continue }
    $next = Join-Path $run 'next.tmp'
    [IO.File]::WriteAllText($next, $value, [Text.Encoding]::ASCII)
    [IO.File]::Replace($next, $control, (Join-Path $run 'previous.txt'))
    Record "requested $value"
    Write-Host "Queued $value. Check the worker's Diagnostics before any nudge; this console records intent only."
}
Record 'host exited'
