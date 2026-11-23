#Requires -Version 5.1
param(
    [Parameter(Mandatory=$true)][string]$Root,
    [string]$SourceDirectory
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ([string]::IsNullOrEmpty($SourceDirectory)) { $SourceDirectory = Split-Path -Parent $PSScriptRoot }
Add-Type -Path (Join-Path $SourceDirectory 'UsbPcapReader.cs')
if (Test-Path -LiteralPath $Root) { throw 'Root must not already exist.' }
$null = New-Item -ItemType Directory -Path $Root

function Packet([int]$Device, [int]$Endpoint, [int]$Info, [int]$Transfer, [int]$Header, [byte[]]$Data) {
    $s = New-Object IO.MemoryStream
    $w = New-Object IO.BinaryWriter($s)
    $w.Write([uint16]$Header); $w.Write([uint64]1); $w.Write([uint32]0)
    $w.Write([uint16]0); $w.Write([byte]$Info); $w.Write([uint16]1)
    $w.Write([uint16]$Device); $w.Write([byte]$Endpoint); $w.Write([byte]$Transfer)
    $w.Write([uint32]$Data.Length)
    if ($Header -gt 27) { $w.Write([byte[]]::new($Header - 27)) }
    $w.Write($Data)
    $bytes = $s.ToArray(); $w.Dispose()
    return ,$bytes
}
function Report([int]$Id, [int]$Length) {
    $data = [byte[]]::new($Length)
    if ($Length -gt 0) { $data[0] = [byte]$Id }
    return ,$data
}
function Capture([string]$Name, [object[]]$Packets, [int]$Snap = 65535, [int]$Link = 249) {
    $path = Join-Path $Root "$Name.pcap"
    $s = [IO.File]::Create($path); $w = New-Object IO.BinaryWriter($s)
    $w.Write([uint32]2712847316); $w.Write([uint16]2); $w.Write([uint16]4)
    $w.Write([uint32]0); $w.Write([uint32]0); $w.Write([uint32]$Snap); $w.Write([uint32]$Link)
    foreach ($p in $Packets) {
        $w.Write([uint32]1); $w.Write([uint32]0)
        $w.Write([uint32]$p.Length); $w.Write([uint32]$p.Length); $w.Write([byte[]]$p)
    }
    $w.Dispose()
    return $path
}
function Equal($Actual, $Expected, [string]$Name) {
    if ($Actual -ne $Expected) { throw "$Name : expected $Expected, got $Actual" }
}
function Reject([string]$Name, [byte[]]$Bytes) {
    $path = Join-Path $Root "$Name.pcap"
    [IO.File]::WriteAllBytes($path, $Bytes)
    $rejected = $false
    try { $null = [S4UsbCapture.UsbPcapReader]::Read($path, 3) }
    catch { $rejected = $true }
    if (-not $rejected) { throw "$Name was accepted" }
    Write-Output "CASE_PASSED $Name"
}

$motor = Packet 3 2 0 1 27 (Report 0x31 11)
$ring = Packet 3 2 0 3 27 (Report 0x32 95)
$packets = New-Object 'System.Collections.Generic.List[object]'
$packets.Add($motor)
$packets.Add($ring)
$packets.Add((Packet 4 2 0 1 27 (Report 0x31 11)))
$packets.Add((Packet 3 2 1 1 27 (Report 0x31 11)))
$packets.Add((Packet 3 130 0 1 27 (Report 0x32 41)))
$packets.Add((Packet 3 2 0 1 27 (Report 0x31 10)))
$packets.Add((Packet 3 2 0 1 27 (Report 0x32 40)))
$packets.Add((Packet 3 2 0 1 27 (Report 0x31 12)))
$packets.Add((Packet 3 2 0 1 27 (Report 0x32 42)))
$packets.Add((Packet 3 2 0 2 28 (Report 0x31 11)))
$packets.Add((Packet 3 2 0 1 31 (Report 0x31 95)))
$packets.Add((Packet 3 2 0 0 39 (Report 0x32 41)))
$path = Capture 'coverage' ($packets.ToArray())
$result = [S4UsbCapture.UsbPcapReader]::Read($path, 3)
Equal $result.FileComplete $true 'complete'
Equal $result.PacketCount 12 'packets'
Equal $result.TargetPackets 11 'target packets'
Equal $result.TargetOutRequests 9 'target requests'
Equal $result.MotorRequests 2 'motor reports'
Equal $result.RingRequests 1 'ring reports'
Equal $result.HasMotorAndRingCoverage $true 'coverage'
Equal $result.UsbPacketLossKnown $false 'loss unknown'
Equal $result.Snaplen 65535 'snaplen'
Equal $result.Bytes ([IO.File]::ReadAllBytes($path).Length) 'bytes'
Write-Output 'CASE_PASSED coverage'

$empty = Capture 'empty' @()
$result = [S4UsbCapture.UsbPcapReader]::Read($empty, 3)
Equal $result.PacketCount 0 'empty packets'
Equal $result.HasMotorAndRingCoverage $false 'empty coverage'
Write-Output 'CASE_PASSED empty'
$wrong = [S4UsbCapture.UsbPcapReader]::Read($path, 99)
Equal $wrong.TargetPackets 0 'wrong device packets'
Equal $wrong.HasMotorAndRingCoverage $false 'wrong device coverage'
Write-Output 'CASE_PASSED wrong-device'

$valid = [IO.File]::ReadAllBytes((Capture 'base' (, $motor)))
Reject 'global-cut' $valid[0..22]
Reject 'record-header-cut' $valid[0..30]
Reject 'packet-cut' $valid[0..($valid.Length - 2)]
$b = [byte[]]$valid.Clone(); $b[0] = 0; Reject 'magic' $b
$b = [byte[]]$valid.Clone(); $b[4] = 3; Reject 'version' $b
$b = [byte[]]$valid.Clone(); $b[20] = 1; Reject 'linktype' $b
$b = [byte[]]$valid.Clone(); $b[16] = 1; $b[17] = 0; Reject 'snaplen' $b
$b = [byte[]]$valid.Clone(); $b[32] = 100; Reject 'captured-exceeds-original' $b
$b = [byte[]]$valid.Clone(); $b[36]++; Reject 'truncated-packet' $b
$b = [byte[]]$valid.Clone(); $b[40] = 26; Reject 'header-short' $b
$b = [byte[]]$valid.Clone(); $b[40] = 100; Reject 'header-long' $b
$b = [byte[]]$valid.Clone(); $b[63]++; Reject 'data-length' $b
$control = Packet 3 2 0 2 27 (Report 0x31 11)
$b = [IO.File]::ReadAllBytes((Capture 'control-base' (, $control)))
Reject 'control-header' $b
$iso = Packet 3 2 0 0 39 (Report 0x31 11)
$b = [IO.File]::ReadAllBytes((Capture 'iso-base' (, $iso)))
$b[71] = 1; Reject 'iso-descriptor' $b
Write-Output 'SUMMARY UsbPcapReader fixture cases passed'
