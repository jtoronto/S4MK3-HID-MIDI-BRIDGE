#Requires -Version 5.1
[CmdletBinding()]
param(
    [string]$DiagnosticKit = 'C:\Users\toron\Desktop\S4 Screen Motor Diagnostics 3',
    [switch]$PrepareOnly
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$kit = (Resolve-Path -LiteralPath $DiagnosticKit).ProviderPath
$identityPath = Join-Path $PSScriptRoot 'TRACE-IDENTITY.json'
$identity = Get-Content -LiteralPath $identityPath -Raw | ConvertFrom-Json
$required = @('frida-inject.exe','writer-agent.js','WriterSession.cs',
    'WriterSession.Protocol.cs','WriterSession.Process.cs','Etw-Trace.ps1',
    'TraceDeadline.cs','Usb-WriterTrace.wprp','Start-WriterTrace.ps1','08-writer-trace.cmd')
foreach ($name in $required) {
    $entry = @($identity.files | Where-Object { $_.path -ceq $name })
    if ($entry.Count -ne 1 -or
        (Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $name) -Algorithm SHA256).Hash -ine $entry[0].sha256) {
        throw "Missing or mismatched trace identity: $name"
    }
}
foreach ($name in @('Prepare.ps1','Bridge-Capture.ps1','Motor-Readiness.ps1','UsbCaptureRuntime.dll','BUILD-IDENTITY.json')) {
    if (-not (Test-Path -LiteralPath (Join-Path $kit $name) -PathType Leaf)) { throw "Kit is missing $name" }
}
$principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Run this launcher as Administrator; no elevation or installation is performed.'
}
$injector = Join-Path $PSScriptRoot 'frida-inject.exe'
$version = & $injector --version 2>&1
if ($LASTEXITCODE -ne 0 -or ($version -join "`n") -notmatch '(^|\s)17\.27\.4(\s|$)') {
    throw 'The packaged frida-inject must report version 17.27.4.'
}
. (Join-Path $PSScriptRoot 'Etw-Trace.ps1')
. (Join-Path $kit 'Bridge-Capture.ps1')
. (Join-Path $kit 'Motor-Readiness.ps1')
$collectorSources = @('WriterSession.cs','WriterSession.Protocol.cs','WriterSession.Process.cs',
    'TraceDeadline.cs') | ForEach-Object { Join-Path $PSScriptRoot $_ }
Add-Type -Path $collectorSources -ReferencedAssemblies @(
    'System.dll','System.Core.dll','System.Xml.dll','System.Runtime.Serialization.dll')
Add-Type -Path (Join-Path $kit 'UsbCaptureRuntime.dll')
$profileCheck = Invoke-WriterEtwWpr @('-profiles',(Join-Path $PSScriptRoot 'Usb-WriterTrace.wprp'))
if ($profileCheck.ExitCode -ne 0) {
    throw "Installed WPR rejected the profile: $($profileCheck.Stdout) $($profileCheck.Stderr)"
}

$root = Join-Path (Join-Path $PSScriptRoot 'captures') (
    (Get-Date -Format 'yyyyMMdd-HHmmss') + '-B-' + [guid]::NewGuid().ToString('N'))
$app = Join-Path $kit 'App'
$source = Join-Path $kit 'BUILD-IDENTITY.json'
$kitIdentity = Get-Content -LiteralPath $source -Raw | ConvertFrom-Json
$engine = Join-Path $app 'Resources\s4mk3-hid-midi-bridge.exe'
if ((Get-FileHash -LiteralPath $engine -Algorithm SHA256).Hash -ine $kitIdentity.engineSha256) {
    throw 'Diagnostic kit engine does not match its build identity.'
}
if ((Get-FileHash -LiteralPath $engine -Algorithm SHA256).Hash -ine $identity.baseEngineSha256 -or
    (Get-FileHash -LiteralPath (Join-Path $app 'S4Bridge.dll') -Algorithm SHA256).Hash -ine $identity.baseHostSha256) {
    throw 'The base kit differs from the frozen host/engine used for A/B.'
}
& (Join-Path $kit 'Prepare.ps1') -AppDirectory $app `
    -PreferenceDirectory (Join-Path $env:LOCALAPPDATA 'S4 MK3 Bridge') `
    -InstalledMapping (Join-Path $env:USERPROFILE 'Desktop\S4 MK3 Bridge BLOOP\S4 MK3 Bridge BLOOP.djayMidiMapping') `
    -SourceIdentity $source -EvidenceDirectory $root -Case B
Copy-Item -LiteralPath $identityPath -Destination (Join-Path $root 'TRACE-IDENTITY.json')
$manifest = Get-Content -LiteralPath (Join-Path $root 'manifest.json') -Raw | ConvertFrom-Json
$config = Join-Path $root 'diagnostic-config.json'
if ($manifest.version -ne 1 -or $manifest.case -cne 'B' -or
    (Get-FileHash -LiteralPath $config -Algorithm SHA256).Hash -ine $manifest.config_sha256) {
    throw 'Prepared B identity or configuration is invalid.'
}
$settings = Get-Content -LiteralPath $config -Raw | ConvertFrom-Json
if ($settings.case -cne 'acquired' -or -not $settings.capture -or
    $settings.capture_path -cne (Join-Path $root 'capture.jsonl')) { throw 'Prepared B configuration is invalid.' }
foreach ($file in $manifest.app_files) {
    if ((Get-FileHash -LiteralPath (Join-Path (Join-Path $root 'App') $file.path) -Algorithm SHA256).Hash -ine $file.sha256) {
        throw "Prepared app changed: $($file.path)"
    }
}
if (-not $manifest.mappings_match) { Write-Warning 'Installed and packaged mappings differ; neither was changed.' }

$targets = @()
foreach ($name in @('NIHardwareService.exe','NIHostIntegrationAgent.exe')) {
    $matches = @(Get-CimInstance Win32_Process -Filter "Name='$name'")
    if ($name -ceq 'NIHardwareService.exe' -and $matches.Count -eq 0) {
        throw 'NIHardwareService is absent. No service will be started and no process was attached.'
    }
    foreach ($p in $matches) {
        if ([string]::IsNullOrWhiteSpace($p.ExecutablePath)) { throw "Cannot identify target executable for PID $($p.ProcessId)." }
        $targets += [ordered]@{ name=$name; pid=[int]$p.ProcessId; path=$p.ExecutablePath; created=$p.CreationDate
            sha256=(Get-FileHash -LiteralPath $p.ExecutablePath -Algorithm SHA256).Hash }
    }
}
$targets | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $root 'targets.json') -Encoding UTF8
Write-Host "Prepared evidence: $root"
if ($PrepareOnly) {
    Write-Host 'PREPARED_STOPPED: identities, collector compilation and WPR profile checked. No attachment, trace or GUI started.'
    return
}
Write-Host 'Pause ALL Djay decks, turn BOTH TT modes off, and leave the original driver and tuning unchanged.'
Write-Host 'Either platter can move. Keep BOTH wheels untouched throughout. No playback or track reload is needed.'
if ((Read-Host 'Type YES only when setup is complete and you are watching') -cne 'YES') {
    throw 'Watching confirmation not given; no trace started.'
}

$sessions = [Collections.Generic.List[object]]::new()
$reasons = [Collections.Generic.List[string]]::new()
$clock = [Diagnostics.Stopwatch]::StartNew()
$signal = $null; $gui = $null; $etw = $false; $complete = $false
$instance = 'S4Writer' + [guid]::NewGuid().ToString('N')
$log = Join-Path $root 'observations.jsonl'
function Mark([string]$kind, [string]$text) {
    [ordered]@{utc=[DateTime]::UtcNow.ToString('o'); kind=$kind; text=$text} |
        ConvertTo-Json -Compress | Add-Content -LiteralPath $log -Encoding UTF8
}
function Check-Trace {
    if ($clock.Elapsed.TotalSeconds -ge 150) { throw 'Trace deadline reached; stopping collectors and preserving incomplete evidence.' }
    foreach ($session in $sessions) { $session.ThrowIfFailed() }
}
function Read-Timed([string]$prompt) {
    Write-Host $prompt
    $read = [Console]::In.ReadLineAsync()
    while (-not $read.Wait(250)) { Check-Trace }
    Check-Trace
    if ($null -eq $read.Result) { throw 'Console input closed.' }
    return $read.Result
}
function Wait-Observed([int]$seconds) {
    $until = $clock.Elapsed.TotalSeconds + $seconds
    while ($clock.Elapsed.TotalSeconds -lt $until) {
        Check-Trace
        [Threading.Tasks.Task]::Delay(250).Wait()
    }
}
try {
    foreach ($target in $targets) {
        Check-Trace
        $current = Get-CimInstance Win32_Process -Filter "ProcessId=$($target.pid)"
        if ($null -eq $current -or $current.ExecutablePath -cne $target.path -or
            $current.CreationDate -ne $target.created -or
            (Get-FileHash -LiteralPath $target.path -Algorithm SHA256).Hash -ine $target.sha256) {
            throw "Target changed before attach: PID $($target.pid)"
        }
        $session = [WriterSession]::new($injector, $target.pid, (Join-Path $PSScriptRoot 'writer-agent.js'), $root)
        $sessions.Add($session)
        $session.Start([TimeSpan]::FromSeconds(10))
        Mark 'attached' "pid=$($target.pid); path=$($target.path); sha256=$($target.sha256)"
    }
    Check-Trace
    $null = Start-WriterEtw -Profile (Join-Path $PSScriptRoot 'Usb-WriterTrace.wprp') `
        -Instance $instance -Output (Join-Path $root 'usb.etl')
    $etw = $true
    Mark 'etw_armed' $instance
    Check-Trace
    foreach ($name in @('S4Bridge','s4mk3-hid-midi-bridge')) {
        if (Get-Process -Name $name -ErrorAction SilentlyContinue) { throw "Competing $name process appeared; GUI not opened." }
    }
    $signal = [S4UsbCapture.ReadinessSignal]::new($root)
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = Join-Path $root 'App\S4Bridge.exe'
    $info.WorkingDirectory = Join-Path $root 'App'
    $info.UseShellExecute = $false
    $info.EnvironmentVariables['S4_SCREEN_MOTOR_DIAGNOSTIC_CONFIG'] = $config
    $gui = [Diagnostics.Process]::Start($info)
    if ($null -eq $gui) { throw 'GUI did not start.' }
    Mark 'gui_launched' "pid=$($gui.Id)"
    $answer = Read-Timed 'Confirm GUI is stopped and you are watching; press Enter, then click Start.'
    if ($answer -cne '') { throw 'Start prompt cancelled.' }
    if ($clock.Elapsed.TotalSeconds -ge 120) { throw 'Insufficient time for bounded readiness.' }
    $ready = Wait-BridgeMotorReadiness -Prepared $root -Signal $signal
    Check-Trace
    foreach ($session in $sessions) { $session.Mark('acquired') }
    Mark 'acquired' "launch=$($ready.launch_id); worker_pid=$($ready.pid)"
    Mark 'coverage' 'Existing NI targets listed in targets.json attached; bridge worker was not attached.'
    Write-Host "Acquired current worker PID $($ready.pid). Keep both wheels untouched for 20 seconds."
    Wait-Observed 20
    Mark 'observation_end' '20-second acquired hold completed.'
    Write-Host 'Click Stop, confirm neutral/release, and quit the GUI normally. Never force-kill it.'
    while (-not $gui.HasExited) {
        $answer = Read-Timed 'Press Enter after normal Stop and Quit, or type CANCEL.'
        if ($answer -cne '') { throw 'Operator cancelled before GUI exit.' }
        $gui.Refresh()
    }
    if ((Read-Timed 'Type STOPPED only after neutral and release are confirmed') -cne 'STOPPED') {
        throw 'Stop and release were not confirmed.'
    }
    foreach ($session in $sessions) { $session.Mark('released') }
    Mark 'released' 'Operator confirmed Stop and release after GUI exit.'
    Wait-Observed 35
    Mark 'post_release_end' '35-second post-release NI trace completed.'
    $capture = Test-BridgeCapture -Prepared $root -Case B
    foreach ($reason in $capture.incompleteReasons) { $reasons.Add($reason) }
    $complete = $reasons.Count -eq 0
} catch {
    $reasons.Add($_.Exception.Message)
    Mark 'error' $_.Exception.Message
} finally {
    if ($null -ne $gui) {
        $gui.Refresh()
        if (-not $gui.HasExited) {
            Write-Warning 'The GUI is still running. Stop and quit it normally; this launcher will not terminate it.'
        }
    }
    if ($null -ne $signal) {
        try { $signal.Dispose() }
        catch { $reasons.Add("Readiness cleanup failed: $($_.Exception.Message)"); $complete=$false }
    }
    if ($etw) {
        try { Stop-WriterEtw -Instance $instance -Output (Join-Path $root 'usb.etl') }
        catch { $reasons.Add("ETW stop failed: $($_.Exception.Message)"); $complete = $false }
    }
    foreach ($session in $sessions) {
        try { $session.Stop([TimeSpan]::FromSeconds(5)) }
        catch { $reasons.Add("Injector stop failed: $($_.Exception.Message)"); $complete = $false }
        finally {
            try { $session.Dispose() }
            catch { $reasons.Add("Injector disposal failed: $($_.Exception.Message)"); $complete=$false }
        }
    }
    if ($clock.Elapsed.TotalSeconds -gt 180) { $reasons.Add('Overall 180-second trace deadline exceeded.'); $complete = $false }
    [ordered]@{case='B'; utc=[DateTime]::UtcNow.ToString('o'); evidence_complete=$complete
        incomplete_reasons=@($reasons.ToArray()); elapsed_seconds=$clock.Elapsed.TotalSeconds
        bridge_worker_traced=$false; etw_pid_attribution='corroborating logging context, not guaranteed origin'} |
        ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $root 'result.json') -Encoding UTF8
    Write-Host "Evidence: $root"
    if (-not $complete) { Write-Warning ("INCOMPLETE: " + ($reasons -join '; ')) }
}
if (-not $complete) { exit 1 }
