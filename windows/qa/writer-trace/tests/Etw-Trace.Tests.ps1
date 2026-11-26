#Requires -Version 5.1
# Ordinary-file CLI fixture only. No WPR session, NI attachment or device I/O.
param([string]$Directory,[string]$FixtureExecutable)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Add-Type -Path (Join-Path $root 'TraceDeadline.cs')
$env:SystemRoot = $Directory
. (Join-Path $root 'Etw-Trace.ps1')
$script:WriterEtwExecutable = $FixtureExecutable
$profile = Join-Path $root 'Usb-WriterTrace.wprp'
$instance = 'S4Writer' + [guid]::NewGuid().ToString('N')
$output = Join-Path $Directory 'owned trace.etl'
$null = Start-WriterEtw -Profile $profile -Instance $instance -Output $output
$result = Stop-WriterEtw -Instance $instance -Output $output
if (-not $result.Complete -or (Get-Content -LiteralPath $output -Raw).Trim() -cne 'fixture-etl') { throw 'Owned output was not preserved.' }
$record = Get-Content -LiteralPath ($output+'.json') -Raw | ConvertFrom-Json
if ($record.instance -cne $instance -or -not $record.complete) { throw 'Owned completion receipt differs.' }
$foreign = Join-Path $Directory 'foreign.etl'
$rejected=$false
try { Stop-WriterEtw -Instance ('S4Writer'+[guid]::NewGuid().ToString('N')) -Output $foreign } catch { $rejected=$true }
if (-not $rejected -or (Test-Path -LiteralPath $foreign)) { throw 'Foreign session was acted on.' }
$env:S4_WPR_FIXTURE_START_FAIL = '1'
$failedOutput = Join-Path $Directory 'failed trace.etl'
$failedInstance = 'S4Writer'+[guid]::NewGuid().ToString('N')
$rejected=$false
try { Start-WriterEtw -Profile $profile -Instance $failedInstance -Output $failedOutput } catch { $rejected=$true }
if (-not $rejected -or $script:WriterEtwOwned.ContainsKey($failedInstance)) { throw 'Failed start was not cleaned.' }
if (-not (Test-Path -LiteralPath ($failedOutput+'.json'))) { throw 'Failed start evidence missing.' }
Remove-Item Env:S4_WPR_FIXTURE_START_FAIL
Write-Output 'ETW wrapper tests passed: owned save, foreign refusal, failed-start cleanup.'
