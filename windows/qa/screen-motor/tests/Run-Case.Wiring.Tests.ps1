$gate=Join-Path $script:workspace 'gate'
$null=New-Item -ItemType Directory -Path (Join-Path $gate 'App') -Force
Copy-Item -LiteralPath $script:helper -Destination (Join-Path $gate 'App\S4Bridge.exe')
$configPath=Join-Path $gate 'diagnostic-config.json'
@{version=1;case='off';capture=$true;capture_path=(Join-Path $gate 'capture.jsonl')} |
    ConvertTo-Json|Set-Content -LiteralPath $configPath -Encoding UTF8
@{version=1;case='A';config_sha256=(Get-FileHash $configPath -Algorithm SHA256).Hash;
    mappings_match=$true;app_files=@(@{path='S4Bridge.exe';sha256=(Get-FileHash (Join-Path $gate 'App\S4Bridge.exe') -Algorithm SHA256).Hash})} |
    ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $gate 'manifest.json') -Encoding UTF8
function Read-Host { param($Prompt) 'YES' }
try {
    & (Join-Path $PSScriptRoot '../Run-Case.ps1') -Prepared $gate -Case A -HandsOff `
        -BeforeGuiStart { throw 'CONTROLLED_USB_NOT_READY' }
} finally { Remove-Item Function:\Read-Host }
$gateResult=Get-Content -LiteralPath (Join-Path $gate 'result.json') -Raw|ConvertFrom-Json
Assert-Equal $gateResult.evidence_complete $false 'failed USB gate incomplete'
Assert-Equal (Test-Path -LiteralPath (Join-Path $gate 'fake-gui-started')) $false 'failed USB gate never starts GUI'
Assert-Equal (@($gateResult.incomplete_reasons|Where-Object {$_ -match 'CONTROLLED_USB_NOT_READY'}).Count) 1 'USB gate failure retained'
Write-Output 'CASE_PASSED usb-gates-gui'
# Drive the real script through CHECK with unknown motor feedback. It must
# skip observation and stop USB before notes, using only the fixture GUI.
$wire=Join-Path $script:workspace 'wire'
$null=New-Item -ItemType Directory -Path (Join-Path $wire 'App') -Force
Copy-Item -LiteralPath $script:helper -Destination (Join-Path $wire 'App\S4Bridge.exe')
$wireConfig=Join-Path $wire 'diagnostic-config.json'
@{version=1;case='off';capture=$true;capture_path=(Join-Path $wire 'capture.jsonl')} |
    ConvertTo-Json|Set-Content -LiteralPath $wireConfig -Encoding UTF8
@{version=1;case='A';config_sha256=(Get-FileHash $wireConfig -Algorithm SHA256).Hash;
    mappings_match=$true;app_files=@(@{path='S4Bridge.exe';sha256=(Get-FileHash (Join-Path $wire 'App\S4Bridge.exe') -Algorithm SHA256).Hash})} |
    ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $wire 'manifest.json') -Encoding UTF8
$wireId=[guid]::NewGuid().ToString('N')
$wirePartial="Local\S4WirePartial-$wireId"
$wireRelease="Local\S4WireRelease-$wireId"
$wireUnused="Local\S4WireExit-$wireId"
$wp=[Threading.EventWaitHandle]::new($false,[Threading.EventResetMode]::ManualReset,$wirePartial)
$wr=[Threading.EventWaitHandle]::new($true,[Threading.EventResetMode]::ManualReset,$wireRelease)
$wu=[Threading.EventWaitHandle]::new($false,[Threading.EventResetMode]::ManualReset,$wireUnused)
$wireBox=@{session=$null;result=$null;answers=0;notes=0}
$wireGuiName='Local\S4WireGui-'+[guid]::NewGuid().ToString('N')
$wireGui=[Threading.EventWaitHandle]::new($false,[Threading.EventResetMode]::ManualReset,$wireGuiName)
[IO.File]::WriteAllText((Join-Path $wire 'hold-gui-event.txt'),$wireGuiName)
$wireExecutable=$script:captureFixture
$wireAssembly=$script:runtimeLibrary
$wireOpen={
    $wireBox.session=Open-UsbCaptureSession -Directory (Join-Path $wire 'usb') `
        -Executable $wireExecutable -CaptureInterface "good|$wirePartial|$wireRelease|$wireUnused" `
        -DeviceAddress 3 -Assembly $wireAssembly
}
$wireClose={
    if ($null -ne $wireBox.session) {
        $owned=$wireBox.session;$wireBox.session=$null
        $wireBox.result=Close-UsbCaptureSession $owned
    }
}
function Read-Host {
    param($Prompt)
    $wireBox.answers++
    switch ($wireBox.answers) {
        1 { return 'YES' }
        2 { return 'YES' }
        3 { return '' }
        4 { return 'CHECK' }
        5 {
            $null=$wireGui.Set()
            foreach ($owned in @(Get-Process -Name S4Bridge -ErrorAction SilentlyContinue |
                Where-Object {$_.Path -eq (Join-Path $wire 'App\S4Bridge.exe')})) {
                if (-not $owned.WaitForExit(10000)) { throw 'Fixture GUI did not exit.' }
            }
            return ''
        }
        6 { return 'STOPPED' }
        default {
            if ($wireBox.answers -gt 8) { throw 'Unexpected operator request.' }
            if ($null -eq $wireBox.result -or -not $wireBox.result.Success) {
                throw 'USB recording was not stopped before observation notes.'
            }
            $wireBox.notes++
            return 'Software fixture; no physical observation.'
        }
    }
}
try {
    & (Join-Path $PSScriptRoot '../Run-Case.ps1') -Prepared $wire -Case A -HandsOff `
        -BeforeGuiStart $wireOpen -AfterGuiExit $wireClose
    Assert-Equal (Test-Path (Join-Path $wire 'fake-gui-started')) $true 'armed callback allows fixture GUI'
    Assert-Equal $wireBox.result.Success $true 'actual script stops USB helper'
    Assert-Equal $wireBox.notes 2 'both observation notes follow USB stop'
    $wireMarks=@(Get-Content -LiteralPath (Join-Path $wire 'observations.jsonl')|ForEach-Object{$_|ConvertFrom-Json})
    Assert-Equal @($wireMarks|Where-Object{$_.kind -eq 'observation_start'}).Count 0 'CHECK bypassed missing motor authorization'
    $wireOutcome=Get-Content -LiteralPath (Join-Path $wire 'result.json') -Raw|ConvertFrom-Json
    Assert-Equal (($wireOutcome.incomplete_reasons -join ';') -match 'Motor readiness failed') $true 'Missing motor readiness was not reported'
    Write-Output 'CASE_PASSED script-callback-lifecycle'
} finally {
    Remove-Item Function:\Read-Host
    & $wireClose
    $null=$wireGui.Set()
    $wireGui.Dispose()
    $wp.Dispose();$wr.Dispose();$wu.Dispose()
}
$eofRun=Join-Path $script:workspace 'gui-eof'
$null=New-Item -ItemType Directory -Path (Join-Path $eofRun 'App') -Force
Copy-Item -LiteralPath $script:helper -Destination (Join-Path $eofRun 'App\S4Bridge.exe')
$eofConfig=Join-Path $eofRun 'diagnostic-config.json'
@{version=1;case='off';capture=$true;capture_path=(Join-Path $eofRun 'capture.jsonl')} |
    ConvertTo-Json|Set-Content -LiteralPath $eofConfig -Encoding UTF8
@{version=1;case='A';config_sha256=(Get-FileHash $eofConfig -Algorithm SHA256).Hash;
    mappings_match=$true;app_files=@(@{path='S4Bridge.exe';sha256=(Get-FileHash (Join-Path $eofRun 'App\S4Bridge.exe') -Algorithm SHA256).Hash})} |
    ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $eofRun 'manifest.json') -Encoding UTF8
$holdName='Local\S4GuiHold-'+[guid]::NewGuid().ToString('N')
$hold=[Threading.EventWaitHandle]::new($false,[Threading.EventResetMode]::ManualReset,$holdName)
[IO.File]::WriteAllText((Join-Path $eofRun 'hold-gui-event.txt'),$holdName)
$eofState=@{answers=0}
function Read-Host {
    param($Prompt)
    $eofState.answers++
    if($eofState.answers -le 2){return 'YES'}
    if($eofState.answers -eq 3){return ''}
    if($eofState.answers -eq 4){return 'FAIL'}
    return $null
}
try {
    & (Join-Path $PSScriptRoot '../Run-Case.ps1') -Prepared $eofRun -Case A -HandsOff
    $eofResult=Get-Content -LiteralPath (Join-Path $eofRun 'result.json') -Raw|ConvertFrom-Json
    Assert-Equal $eofResult.evidence_complete $false 'EOF real script incomplete'
    $live=@(Get-Process -Name S4Bridge -ErrorAction SilentlyContinue |
        Where-Object {$_.Path -eq (Join-Path $eofRun 'App\S4Bridge.exe')})
    Assert-Equal $live.Count 1 'EOF leaves fixture GUI alive'
    Write-Output 'CASE_PASSED script-gui-eof'
} finally {
    Remove-Item Function:\Read-Host
    $null=$hold.Set()
    foreach($owned in @(Get-Process -Name S4Bridge -ErrorAction SilentlyContinue |
        Where-Object {$_.Path -eq (Join-Path $eofRun 'App\S4Bridge.exe')})) {
        if(-not $owned.WaitForExit(10000)){throw 'Fixture GUI cleanup failed'}
    }
    $hold.Dispose()
}
