#Requires -Version 5.1
param([Parameter(Mandatory=$true)][string]$Root)
$ErrorActionPreference='Stop'
$ProgressPreference='SilentlyContinue'
Set-StrictMode -Version Latest
Add-Type -Path (Join-Path $PSScriptRoot '../UsbPcapReader.cs')
. (Join-Path $PSScriptRoot '../Usb-Capture.ps1')
. (Join-Path $PSScriptRoot '../Bridge-Capture.ps1')
if (Test-Path -LiteralPath $Root) { throw 'Root must not exist.' }
$null=New-Item -ItemType Directory -Path $Root
function Assert-Equal($Actual,$Expected,[string]$Name) {
    if ($Actual -cne $Expected) { throw "$Name : expected [$Expected], got [$Actual]" }
}
function Write-UsbFixture([string]$Path,[bool]$Coverage) {
    $stream=[IO.File]::Create($Path)
    $writer=[IO.BinaryWriter]::new($stream)
    try {
        $writer.Write([uint32]2712847316);$writer.Write([uint16]2);$writer.Write([uint16]4)
        $writer.Write([uint32]0);$writer.Write([uint32]0);$writer.Write([uint32]1048576);$writer.Write([uint32]249)
        if ($Coverage) {
            foreach ($spec in @(@(0x31,11),@(0x32,41))) {
                $payload=[byte[]]::new($spec[1]);$payload[0]=[byte]$spec[0]
                $length=[uint32](27+$payload.Length)
                $writer.Write([uint32]1);$writer.Write([uint32]0);$writer.Write($length);$writer.Write($length)
                $writer.Write([uint16]27);$writer.Write([uint64]$spec[0]);$writer.Write([uint32]0)
                $writer.Write([uint16]9);$writer.Write([byte]0);$writer.Write([uint16]1)
                $writer.Write([uint16]3);$writer.Write([byte]2);$writer.Write([byte]1)
                $writer.Write([uint32]$payload.Length);$writer.Write($payload)
            }
        }
    } finally {$writer.Dispose()}
}
try {
    foreach ($mode in @('good','missing-coverage','bridge-failure','usb-failure','eof','cancel')) {
        $pairRoot=Join-Path $Root $mode
        $null=New-Item -ItemType Directory -Path $pairRoot
        $prepare={
            param($case,$run)
            $null=New-Item -ItemType Directory -Path $run
            [IO.File]::WriteAllText((Join-Path $run 'prepared.txt'),$case)
        }
        $runCase={
            param($case,$run)
            $rows=@(@{kind='header';version=1;case=(@{A='off';B='acquired'}[$case]);utc_unix_us=1000000},
                @{kind='screen_case_ready'})
            if ($case -eq 'B') {$rows+=@{kind='screen_connect_begin'}}
            if ($mode -ne 'bridge-failure') {
                $rows+=@{kind='footer';runtime_ok=$true;incomplete=$false;first_stop_us=20000000;
                    roles=@(@{role='input';losses=0},@{role='output';losses=0},@{role='screen';losses=0})}
            }
            [IO.File]::WriteAllLines((Join-Path $run 'capture.jsonl'),
                [string[]]@($rows|ForEach-Object {ConvertTo-Json -InputObject $_ -Depth 6 -Compress}))
            $validated=Test-BridgeCapture -Prepared $run -Case $case
            @{evidence_complete=$validated.complete;incomplete_reasons=$validated.incompleteReasons} |
                ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $run 'result.json') -Encoding UTF8
            $usb=Join-Path $run 'usb';$null=New-Item -ItemType Directory -Path $usb
            Write-UsbFixture (Join-Path $usb 'capture.pcap') ($mode -ne 'missing-coverage')
            return [pscustomobject]@{Success=($mode -ne 'usb-failure')}
        }
        $answer={if($mode -eq 'eof'){return $null};if($mode -eq 'cancel'){return 'CANCEL'};return 'NEXT'}
        $pair=Invoke-UsbCapturePair -PairRoot $pairRoot -DeviceAddress 3 `
            -PrepareCase $prepare -RunCase $runCase -ReadBaselineAnswer $answer
        Assert-Equal $pair.complete ($mode -eq 'good') "$mode completeness"
        Assert-Equal (Test-Path -LiteralPath (Join-Path $pairRoot 'B/prepared.txt')) ($mode -eq 'good') "$mode B creation"
        Assert-Equal (Test-Path -LiteralPath (Join-Path $pairRoot 'A/capture.jsonl')) $true "$mode trace preserved"
        Assert-Equal (Test-Path -LiteralPath (Join-Path $pairRoot 'A/usb/capture.pcap')) $true "$mode USB preserved"
        $saved=Get-Content -LiteralPath (Join-Path $pairRoot 'pair-result.json') -Raw|ConvertFrom-Json
        Assert-Equal $saved.complete $pair.complete "$mode saved result"
        if ($mode -ne 'good' -and [string]::IsNullOrEmpty($saved.error)) {throw "$mode missing failure reason"}
        Write-Output "CASE_PASSED pair-$mode"
    }
    Write-Output 'SUMMARY USB pair gates passed'
} finally {Remove-Item -LiteralPath $Root -Recurse -Force}
