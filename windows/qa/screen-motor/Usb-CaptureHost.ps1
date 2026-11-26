#Requires -Version 5.1
[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Configuration)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
$config=Get-Content -LiteralPath $Configuration -Raw | ConvertFrom-Json
$receipt=Join-Path $config.directory 'usb-result.json'
try {
    Add-Type -Path $config.assembly
    $result=[S4UsbCapture.CaptureHost]::Run(
        $config.executable, $config.captureInterface, $config.deviceAddress,
        $config.directory, $config.readyEvent, $config.stopEvent,
        $config.parentPid, $config.maximumSeconds)
} catch {
    $result=[pscustomobject]@{ Success=$false; Error=$_.Exception.ToString() }
} finally {
    $temporary=$receipt+'.tmp'
    [IO.File]::WriteAllText($temporary, ($result | ConvertTo-Json -Depth 6), [Text.UTF8Encoding]::new($false))
    [IO.File]::Move($temporary,$receipt)
}
if (-not $result.Success) { exit 1 }
