#Requires -Version 5.1
$ErrorActionPreference = 'Stop'

$devices = @((Get-PnpDevice -PresentOnly).InstanceId |
    Where-Object { $_ -match '^USB\\VID_17CC&PID_1720\\' })
if ($devices.Count -ne 1) {
    throw "Expected one connected S4 MK3, found $($devices.Count)."
}
if ($devices[0] -notmatch '^USB\\VID_17CC&PID_1720\\([a-zA-Z0-9]{1,128})\z') {
    throw 'Windows did not report a usable serial for the connected S4 MK3.'
}
$serial = $Matches[1]

$principal = [Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Open Windows PowerShell as administrator to run the NI output observer.'
}
if (@(Get-Process | Where-Object ProcessName -Match '^(Traktor.*|djay.*|S4Bridge.*|s4mk3-hid-midi-bridge.*|s4-encdr-screen-worker|s4-ni-left-jog)$').Count -ne 0) {
    throw 'Close the other S4 clients and try again.'
}
$service = @(Get-Process -Name NIHardwareService -ErrorAction Stop)
if ($service.Count -ne 1) { throw 'Expected one running NIHardwareService process.' }

$sources = @('WriterSession.cs', 'WriterSession.Protocol.cs', 'WriterSession.Process.cs') |
    ForEach-Object { Join-Path $PSScriptRoot $_ }
Add-Type -Path $sources -ReferencedAssemblies @(
    'System.dll', 'System.Core.dll', 'System.Xml.dll', 'System.Runtime.Serialization.dll')

$id = [Guid]::NewGuid().ToString('N')
$run = Join-Path (Join-Path $PSScriptRoot 'captures') $id
[void](New-Item -ItemType Directory -Path $run)
$engine = Join-Path $PSScriptRoot 's4-ni-left-jog.exe'
$target = $service[0]
[ordered]@{
    utc = [DateTime]::UtcNow.ToString('o')
    servicePid = $target.Id
    serviceSha256 = (Get-FileHash -LiteralPath $target.Path).Hash
    diagnosticSha256 = (Get-FileHash -LiteralPath $engine).Hash
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $run 'identity.json') -Encoding UTF8

if ((Read-Host 'Keep both platters clear. Type WATCHING to start the observer') -cne 'WATCHING') {
    throw 'Operator did not start the observer.'
}
$observer = [WriterSession]::new((Join-Path $PSScriptRoot 'frida-inject.exe'),
    $target.Id, (Join-Path $PSScriptRoot 'writer-agent.js'), $run)
$clientExit = $null
$observerStopped = $false
Start-Transcript -LiteralPath (Join-Path $run 'console.txt') | Out-Null
try {
    $observer.Start([TimeSpan]::FromSeconds(10))
    Write-Host "NI observer ready. Capture folder $run"
    $observer.Mark('left jog diagnostic launched')
    & $engine --serial $serial
    $clientExit = $LASTEXITCODE
    $observer.Mark("left jog diagnostic exit $clientExit")
    $observer.Stop([TimeSpan]::FromSeconds(5))
    $observerStopped = $true
} finally {
    try { $observer.Dispose() }
    finally {
        [ordered]@{
            utc = [DateTime]::UtcNow.ToString('o')
            clientExit = $clientExit
            observerStopped = $observerStopped
        } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $run 'result.json') -Encoding UTF8
        Write-Host "Capture folder $run"
        Stop-Transcript | Out-Null
    }
}
exit $clientExit
