#Requires -Version 5.1
[CmdletBinding()]
param([Parameter(Mandatory=$true)][ValidateSet('A','B','C','D','N','N-off')][string]$Case)
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
$app=Join-Path $PSScriptRoot 'App'
$identity=Join-Path $PSScriptRoot 'BUILD-IDENTITY.json'
$mapping=Join-Path $env:USERPROFILE 'Desktop\S4 MK3 Bridge BLOOP\S4 MK3 Bridge BLOOP.djayMidiMapping'
$preferences=Join-Path $env:LOCALAPPDATA 'S4 MK3 Bridge'
$run=Join-Path (Join-Path $PSScriptRoot 'captures') ((Get-Date -Format 'yyyyMMdd-HHmmss')+'-'+$Case+'-'+[guid]::NewGuid().ToString('N').Substring(0,8))
& (Join-Path $PSScriptRoot 'Prepare.ps1') -AppDirectory $app -PreferenceDirectory $preferences `
    -InstalledMapping $mapping -SourceIdentity $identity -EvidenceDirectory $run -Case $Case
& (Join-Path $PSScriptRoot 'Run-Case.ps1') -Prepared $run -Case $Case
