#Requires -Version 5.1
[CmdletBinding()]
param()
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
Set-StrictMode -Version Latest

$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
$principal=[Security.Principal.WindowsPrincipal]::new($identity)
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    $process=Start-Process -FilePath "$PSHOME\powershell.exe" -Verb RunAs -ArgumentList @(
        '-NoProfile','-ExecutionPolicy','Bypass','-File',"`"$PSCommandPath`""
    ) -Wait -PassThru
    exit $process.ExitCode
}

$identityPath=Join-Path $PSScriptRoot 'USB-LAUNCHER-IDENTITY.json'
$launcher=Get-Content -LiteralPath $identityPath -Raw | ConvertFrom-Json
foreach ($file in $launcher.files) {
    if ((Get-FileHash -LiteralPath (Join-Path $PSScriptRoot $file.path) -Algorithm SHA256).Hash.ToLowerInvariant() -cne $file.sha256) {
        throw "Launcher file changed: $($file.path)"
    }
}
. (Join-Path $PSScriptRoot 'Usb-Capture.ps1')
$captureExe='C:\Program Files\USBPcap\USBPcapCMD.exe'
if ((Get-FileHash -LiteralPath $captureExe -Algorithm SHA256).Hash.ToLowerInvariant() -cne $launcher.usbpcapSha256) {
    throw 'The inspected USBPcap executable changed; check it before recording.'
}
$captureInterface='\\.\USBPcap1'
$deviceAddress=3
$topology=Confirm-UsbCaptureTarget $captureExe $captureInterface $deviceAddress
$assembly=Join-Path $PSScriptRoot 'UsbCaptureRuntime.dll'
Add-Type -Path $assembly
$source=Join-Path $PSScriptRoot 'BUILD-IDENTITY.json'
$app=Join-Path $PSScriptRoot 'App'
$engine=Join-Path $app 'Resources\s4mk3-hid-midi-bridge.exe'
$baseIdentity=Get-Content -LiteralPath $source -Raw | ConvertFrom-Json
if ((Get-FileHash -LiteralPath $engine -Algorithm SHA256).Hash.ToLowerInvariant() -cne $baseIdentity.engineSha256) {
    throw 'Diagnostic engine does not match the preserved build identity.'
}
$pairRoot=Join-Path (Join-Path $PSScriptRoot 'usb-captures') ((Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
$null=New-Item -ItemType Directory -Path $pairRoot
[IO.File]::WriteAllText((Join-Path $pairRoot 'usb-topology.txt'),$topology)
Copy-Item -LiteralPath $identityPath -Destination $pairRoot
$prepareCase={
        param($case,$run)
        $currentTopology=Confirm-UsbCaptureTarget $captureExe $captureInterface $deviceAddress
        [IO.File]::WriteAllText((Join-Path $pairRoot "usb-topology-$case.txt"),$currentTopology)
        & (Join-Path $PSScriptRoot 'Prepare.ps1') -AppDirectory $app `
            -PreferenceDirectory (Join-Path $env:LOCALAPPDATA 'S4 MK3 Bridge') `
            -InstalledMapping (Join-Path $env:USERPROFILE 'Desktop\S4 MK3 Bridge BLOOP\S4 MK3 Bridge BLOOP.djayMidiMapping') `
            -SourceIdentity $source -EvidenceDirectory $run -Case $case
}
$runCase={
        param($case,$run)
        $box=@{session=$null;result=$null}
        $arm={
            $box.session=Open-UsbCaptureSession -Directory (Join-Path $run 'usb') `
                -Executable $captureExe -CaptureInterface $captureInterface `
                -DeviceAddress $deviceAddress -Assembly $assembly
            Write-Host 'USB stream is armed and its PCAP header is saved. The stopped bridge GUI will now open.'
        }
        $finish={
            if ($null -ne $box.session) {
                $current=$box.session
                $box.session=$null
                $box.result=Close-UsbCaptureSession $current
            }
        }
        try {
            & (Join-Path $PSScriptRoot 'Run-Case.ps1') -Prepared $run -Case $case -HandsOff -BeforeGuiStart $arm -AfterGuiExit $finish
        } finally {
            & $finish
        }
        return $box.result
}
Write-Host 'This records two short runs: A screens off, then B screen connection without images.'
Write-Host 'Use the SAME right Deck B track, TT playback and saved settings. Keep BOTH wheels untouched.'
Write-Host 'After the observation, STOP and QUIT before typing your motor/ring notes.'
Write-Host "Everything will be saved in $pairRoot"
$pair=Invoke-UsbCapturePair -PairRoot $pairRoot -DeviceAddress $deviceAddress -PrepareCase $prepareCase -RunCase $runCase `
    -ReadBaselineAnswer { Read-Host 'Type NEXT only if Deck B was playing in TT and A motor/ring behavior was normal. Otherwise stop here' }
Write-Host "Saved: $pairRoot"
if (-not $pair.complete) { exit 1 }
