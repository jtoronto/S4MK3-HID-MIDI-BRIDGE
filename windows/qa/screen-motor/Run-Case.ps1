#Requires -Version 5.1
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$Prepared,
    [Parameter(Mandatory=$true)][ValidateSet('A','B','C','D','N','N-off')][string]$Case,
    [switch]$HandsOff,
    [scriptblock]$BeforeGuiStart,
    [scriptblock]$AfterGuiExit
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot 'Bridge-Capture.ps1')
. (Join-Path $PSScriptRoot 'Motor-Readiness.ps1')
$root = (Resolve-Path -LiteralPath $Prepared).ProviderPath
$manifest = Get-Content -LiteralPath (Join-Path $root 'manifest.json') -Raw | ConvertFrom-Json
if ($manifest.version -ne 1 -or $manifest.case -cne $Case) { throw 'Prepared case does not match this launcher.' }
$config = Join-Path $root 'diagnostic-config.json'
if ((Get-FileHash -LiteralPath $config -Algorithm SHA256).Hash -ne $manifest.config_sha256) { throw 'Diagnostic config changed.' }
$settings = Get-Content -LiteralPath $config -Raw | ConvertFrom-Json
$expected = @{ A='off'; B='acquired'; C='static'; D='updates'; N='normal'; 'N-off'='normal' }[$Case]
if ($settings.case -cne $expected -or $settings.capture -ne ($Case -ne 'N-off') -or
    $settings.capture_path -cne (Join-Path $root 'capture.jsonl')) { throw 'Invalid prepared diagnostic config.' }
if (Test-Path -LiteralPath $settings.capture_path) { throw 'Capture path already exists; prepare a new run.' }
if (Test-Path -LiteralPath (Join-Path $root 'observations.jsonl')) { throw 'Run already used; prepare a new run.' }
foreach ($name in @('S4Bridge','s4mk3-hid-midi-bridge')) {
    if (Get-Process -Name $name -ErrorAction SilentlyContinue) { throw "Stop all $name processes first." }
}
foreach ($file in $manifest.app_files) {
    $path = Join-Path (Join-Path $root 'App') $file.path
    if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or
        (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $file.sha256) {
        throw "Staged app changed: $($file.path)"
    }
}
$reasons = [Collections.Generic.List[string]]::new()
if (-not $manifest.mappings_match) { $reasons.Add('Installed and packaged mappings differ; mapping equivalence is unverified.') }
$log = Join-Path $root 'observations.jsonl'
function Mark([string]$Kind, [string]$Text) {
    [ordered]@{ utc=[DateTime]::UtcNow.ToString('o'); kind=$Kind; text=$Text } |
        ConvertTo-Json -Compress | Add-Content -LiteralPath $log -Encoding UTF8
}
function Require-Yes([string]$Prompt) {
    $answer = Read-Host "$Prompt (type YES to confirm)"
    if ($answer -cne 'YES') { throw 'Operator did not confirm; run is incomplete.' }
}
Mark 'case' $Case
if ($HandsOff) {
    Write-Host 'Use the right wheel, Deck B, and TT playback. Use the same saved settings for both paired runs.'
    Write-Host 'Keep Deck B paused and right TT off until the new feedback receiver is confirmed.'
    Write-Host 'Keep both wheels untouched throughout; do not perform Assist or Brake gestures.'
    Mark 'setup' 'Hands-off paired run: right wheel, Deck B, TT playback, same saved settings, both wheels untouched.'
} else {
    Mark 'setup' (Read-Host 'Record track, physical side/deck, other wheel, playback, touch/Vinyl, tension and relevant settings')
    Mark 'motor_mode' (Read-Host 'Record exact motor mode and authorization')
}
Write-Host 'The diagnostic GUI opens stopped. Starting can move either platter and activate screens.'
Require-Yes 'Setup complete, competing bridge stopped, and operator is watching'
$process = $null
$readinessSignal = $null
try {
    if ($null -ne $BeforeGuiStart) { & $BeforeGuiStart }
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = Join-Path $root 'App\S4Bridge.exe'
    $info.WorkingDirectory = Join-Path $root 'App'
    $info.UseShellExecute = $false
    $info.EnvironmentVariables['S4_SCREEN_MOTOR_DIAGNOSTIC_CONFIG'] = $config
    $process = [Diagnostics.Process]::Start($info)
    if ($null -eq $process) { throw 'GUI did not start.' }
    Mark 'gui_launched' "pid=$($process.Id)"
    Write-Host 'Confirm the GUI is stopped. Keep this console visible.'
    Require-Yes 'GUI is stopped and you are still watching'
    if($HandsOff){
        if(-not ('S4UsbCapture.ReadinessSignal' -as [type])){
            Add-Type -Path (Join-Path $PSScriptRoot 'UsbCaptureRuntime.dll')
        }
        $readinessSignal=[S4UsbCapture.ReadinessSignal]::new($root)
    }
    Write-Host 'Click Start in the GUI immediately after pressing Enter here.'
    $null = Read-Host 'Press Enter, then click Start'
    $start = [Diagnostics.Stopwatch]::StartNew()
    Mark 'manual_start_prompt' 'Operator instructed to click Start immediately.'
    Write-Host 'Wait for actual readiness in the host UI or an observable receipt. Do not guess.'
    $ready='FAIL'
    if ($HandsOff) {
        try {
            $connected=Wait-BridgeMotorReadiness -Prepared $root -Signal $readinessSignal
            Write-Host "The current feedback receiver and S4_SCREEN_DIAG_READY $expected are confirmed."
            Write-Host 'Keep Deck B paused. Use its software input-mode selector in Djay.'
            if($null -eq $connected.right -or $connected.right.internal -ne $true){
                Write-Host 'Internal is not confirmed. Select Thru and then Internal while paused to send fresh mode feedback.'
            }else{Write-Host 'Internal feedback is confirmed; leave that mode selected.'}
            Write-Host 'Now eject the track from Deck B in Djay, then load the agreed track again AFTER this connection.'
            Write-Host 'Start Deck B playback and select right-side TT. Keep both wheels untouched.'
            $answer=Read-Host 'Type CHECK after these steps, or FAIL to stop'
            if($answer -ceq 'CHECK'){
                $proof=Wait-BridgeMotorReadiness -Prepared $root -Signal $readinessSignal -Authorization
                Mark 'motor_authorization' "launch=$($proof.launch_id); pid=$($proof.pid); deck=2; actual feedback authorized"
                $ready='READY'
                Write-Host 'Motor authorization is confirmed from this worker. The observation can begin.'
            }
        } catch {
            $reasons.Add("Motor readiness failed: $($_.Exception.Message)")
            Mark 'motor_readiness_error' $_.Exception.Message
            Write-Warning $_.Exception.Message
        }
    } else {
        $ready=Read-Host 'Type READY when established, or FAIL if not established'
    }
    $startupSeconds = $start.Elapsed.TotalSeconds
    Mark 'manual_readiness' "$ready; elapsed_from_prompt_seconds=$startupSeconds"
    if ($ready -cne 'READY') { $reasons.Add('Readiness not established.') }
    $startupLimit=if($HandsOff){90}else{60}
    if ($startupSeconds -gt $startupLimit) {
        $reasons.Add("Readiness took over $startupLimit seconds from the manual Start prompt.")
        $ready='FAIL'
    }
    if ($ready -ceq 'READY') {
        Mark 'observation_start' 'Manual 20-second observation begins.'
        $window = [Diagnostics.Stopwatch]::StartNew()
        if ($HandsOff) {
            Write-Host 'Observe for at least 20 seconds without touching either wheel, then press Enter and immediately Stop and quit the GUI.'
            $null = Read-Host 'Press Enter after at least 20 seconds'
        } else {
            Write-Host 'Observe at least 20 seconds. TT: hands off. Assist/Brake: repeat the same gesture.'
            Write-Host 'For each gesture or symptom, enter a note and press Enter; enter END after 20 seconds.'
            do {
                $note = Read-Host 'Gesture/symptom note, or END'
                if ($note -cne 'END') { Mark 'gesture_or_symptom' $note }
            } while ($note -cne 'END')
        }
        $duration = $window.Elapsed.TotalSeconds
        Mark 'observation_end' "elapsed_seconds=$duration"
        if ($duration -lt 20) { $reasons.Add('Observation was shorter than 20 seconds.') }
    }
    Write-Host 'Click Stop in the GUI; wait for stopped/neutral and screen release. Then quit the GUI normally.'
    $exited = Wait-BridgeGuiExit -Process $process -ReadAnswer { param($prompt) Read-Host $prompt }
    $stop = if ($exited) { Read-Host 'Type STOPPED only after Stop, neutral and release confirmation; otherwise type FAIL' } else { 'CANCEL' }
    Mark 'manual_stop' $stop
    if ($null -ne $AfterGuiExit) {
        try { & $AfterGuiExit }
        catch { $reasons.Add("USB stop failed: $($_.Exception.Message)"); Mark 'usb_stop_error' $_.Exception.Message }
    }
    if ($stop -cne 'STOPPED') { $reasons.Add('Stop, neutral, release or GUI exit not confirmed.') }
    if ($stop -ceq 'STOPPED') {
        Mark 'motor_observation' (Read-Host 'Describe motor response independently')
        Mark 'ring_observation' (Read-Host 'Describe ring behavior independently')
    }
    if (Get-Process -Name 's4mk3-hid-midi-bridge' -ErrorAction SilentlyContinue) {
        $reasons.Add('Bridge worker is still running; capture export is not confirmed.')
    }
    if ($settings.capture -and $exited -and
        -not (Get-Process -Name 's4mk3-hid-midi-bridge' -ErrorAction SilentlyContinue)) {
        $capture = Test-BridgeCapture -Prepared $root -Case $Case
        foreach ($reason in $capture.incompleteReasons) { $reasons.Add($reason) }
        if ($null -ne $capture.header -and $null -ne $capture.header.utc_unix_us -and $capture.header.utc_unix_us -gt 0) {
            Mark 'capture_utc_anchor_us' "$($capture.header.utc_unix_us)"
        }
        if ($null -ne $capture.traceSha256) {
            Mark 'capture_check' "ready=$($capture.readyCount) connect=$($capture.connectCount) publish=$($capture.publishCount) failed_publish=$($capture.failedPublishCount)"
            Mark 'capture_sha256' $capture.traceSha256
        }
    } elseif (-not $settings.capture) {
        $reasons.Add('Capture disabled: readiness, loss and traffic cannot be validated after Stop.')
    } else {
        $reasons.Add('Capture was not inspected because the GUI or worker is still running.')
    }
} catch {
    $reasons.Add("Operator or validation error: $($_.Exception.Message)")
    Mark 'error' $_.Exception.Message
} finally {
    if($null -ne $readinessSignal){$readinessSignal.Dispose()}
    $result = [ordered]@{ case=$Case; utc=[DateTime]::UtcNow.ToString('o')
        evidence_complete=($reasons.Count -eq 0); incomplete_reasons=@($reasons) }
    $result | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $root 'result.json') -Encoding UTF8
    Write-Host "Result: $(Join-Path $root 'result.json')"
    if ($reasons.Count -gt 0) { Write-Warning ("INCOMPLETE: " + ($reasons -join '; ')) }
}
