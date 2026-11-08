#Requires -Version 5.1
<#
.SYNOPSIS
  Read-only inventory of the Traktor Kontrol S4 MK3 on Windows (task 3).

.DESCRIPTION
  Enumerates VID/PID 17cc:1720 PnP devices, container/serial identity, USB
  children, HID sub-device paths, audio interfaces in the same container,
  installed driver/service identities, and NI/Traktor/Mixxx/bridge/Djay/LoopBe
  processes. Writes inventory.json plus raw native evidence into -Evidence.

  READ-ONLY: uses Get-PnpDevice, Get-PnpDeviceProperty, Get-CimInstance and
  Get-Process only. It never opens HID collections, sends USB/MIDI traffic,
  or stops/starts services, changes drivers, or writes audio state.

  Selection never picks the first match: zero matches report ABSENT,
  unrelated-only matches report UNRELATED_ONLY, and multiple matches report
  MULTIPLE with every candidate listed. A single unambiguous composite
  reports READY_CANDIDATE, which is collection evidence only, not proof of
  read/write access or audio coexistence.

.PARAMETER Evidence
  Owned output directory receiving inventory.json and raw/ evidence.

.PARAMETER FixtureJson
  Optional canned device list (as produced by the raw device dump) used to
  exercise absent/unrelated/multiple selection logic without hardware.
  Production runs omit this flag.

.EXAMPLE
  powershell.exe -NoProfile -File windows\qa\device-inventory.ps1 -Evidence C:\S4Evidence\task-3
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$Evidence,
  [string]$FixtureJson = ""
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$VidPattern = "VID_17CC&PID_1720"
$VidPidRegex = "VID_17CC&PID_1720[\\&]"
function Test-S4Match([string]$InstanceId) { return $InstanceId -match $VidPidRegex }
$StatusValues = @("READY_CANDIDATE", "ABSENT", "UNRELATED_ONLY", "MULTIPLE")

function Get-ParentOf([string]$InstanceId) {
  try {
    return (Get-PnpDeviceProperty -InstanceId $InstanceId -KeyName "DEVPKEY_Device_Parent" -ErrorAction Stop | Select-Object -ExpandProperty Data -ErrorAction Stop)
  } catch { return $null }
}

function Write-Utf8Json($Value, [string]$Path, [int]$Depth) {
  $Value | ConvertTo-Json -Depth $Depth | Set-Content -Encoding utf8 -Path $Path
}

function Get-DeviceProperties([string]$InstanceId) {
  $keys = @(
    "DEVPKEY_Device_FriendlyName", "DEVPKEY_Device_DeviceDesc",
    "DEVPKEY_Device_Class", "DEVPKEY_Device_Service",
    "DEVPKEY_Device_ContainerId", "DEVPKEY_Device_Parent",
    "DEVPKEY_Device_LocationInfo", "DEVPKEY_Device_LocationPaths",
    "DEVPKEY_Device_BusReportedDeviceDesc", "DEVPKEY_Device_SerialNumber",
    "DEVPKEY_Device_UINumber", "DEVPKEY_Device_Capabilities",
    "DEVPKEY_Device_Driver", "DEVPKEY_Device_DriverVersion",
    "DEVPKEY_Device_DriverProvider", "DEVPKEY_Device_DriverInfPath",
    "DEVPKEY_Device_BusTypeGuid", "DEVPKEY_Device_HardwareIds",
    "DEVPKEY_Device_CompatibleIds", "DEVPKEY_Device_PDOName"
  )
  $props = @{}
  foreach ($key in $keys) {
    try {
      $value = Get-PnpDeviceProperty -InstanceId $InstanceId -KeyName $key -ErrorAction Stop |
        Select-Object -ExpandProperty Data -ErrorAction Stop
      $props[$key] = "$value"
    } catch {
      $props[$key] = $null
    }
  }
  return $props
}

function Select-S4Candidates([array]$Devices) {
  # Native -PresentOnly records and fixtures share an explicit Boolean
  # Present field. Validate before any classification; Status is health only.
  foreach ($device in $Devices) {
    $presence = $device.PSObject.Properties["Present"]
    if ($null -eq $presence -or $presence.Value -isnot [bool]) {
      throw "Invalid device record: Present must be an explicit Boolean."
    }
  }
  $presentDevices = @($Devices | Where-Object { $_.Present })
  $matched = @($presentDevices | Where-Object { Test-S4Match $_.InstanceId })
  $unrelatedHid = @($presentDevices | Where-Object {
    (Test-S4Match $_.InstanceId) -eq $false -and ($_.Class -eq "HIDClass" -or $_.Class -eq "USB")
  })
  if ($matched.Count -eq 0) {
    if ($unrelatedHid.Count -gt 0) { return @{ status = "UNRELATED_ONLY"; candidates = @() } }
    return @{ status = "ABSENT"; candidates = @() }
  }
  # Group by device container: one physical S4 enclosure is one candidate.
  # Group names/keys are never reported as candidates; only the top
  # composite (the matched device whose parent is outside the match set).
  $containers = @($matched | ForEach-Object {
    $c = (Get-DeviceProperties $_.InstanceId)["DEVPKEY_Device_ContainerId"]
    if ([string]::IsNullOrEmpty($c)) { "NOPARENT:" + $_.InstanceId.ToUpperInvariant() } else { $c.ToUpperInvariant() }
  } | Sort-Object -Unique)
  if ($containers.Count -gt 1) {
    return @{ status = "MULTIPLE"; candidates = @($containers) }
  }
  $ids = @($matched | ForEach-Object { $_.InstanceId.ToUpperInvariant() })
  $tops = @($matched | Where-Object {
    $p = (Get-DeviceProperties $_.InstanceId)["DEVPKEY_Device_Parent"]
    [string]::IsNullOrEmpty($p) -or ($ids -notcontains $p.ToUpperInvariant())
  } | Select-Object -ExpandProperty InstanceId)
  if ($tops.Count -ne 1) {
    return @{ status = "MULTIPLE"; candidates = @($tops) }
  }
  return @{ status = "READY_CANDIDATE"; candidates = @($tops[0]) }
}

$evidenceDir = New-Item -ItemType Directory -Force -Path $Evidence
$rawDir = New-Item -ItemType Directory -Force -Path (Join-Path $Evidence "raw")
$utc = (Get-Date).ToUniversalTime().ToString("o")
$machine = "$env:COMPUTERNAME $($env:OS)"

if ($FixtureJson -ne "") {
  $devices = Get-Content -Raw -Path $FixtureJson | ConvertFrom-Json
  $selection = Select-S4Candidates @($devices)
  $selection | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 -Path (Join-Path $Evidence "fixture-selection.json")
  if ($selection.status -eq "READY_CANDIDATE") { exit 0 }
  exit 2
}

# 1. PnP enumeration (native, read-only).
$allPnp = Get-PnpDevice -PresentOnly |
  Select-Object InstanceId, FriendlyName, Class, Status, @{Name = "Present"; Expression = { $true }}
Write-Utf8Json $allPnp (Join-Path $rawDir "pnp-devices.json") 4

$selection = Select-S4Candidates @($allPnp)
$compositeId = if ($selection.candidates.Count -eq 1) { $selection.candidates[0] } else { $null }

# 2. Per-device detail for every VID/PID match (no device opens).
$matchedDetail = @()
foreach ($dev in @($allPnp | Where-Object { Test-S4Match $_.InstanceId })) {
  $matchedDetail += @{
    InstanceId   = $dev.InstanceId
    FriendlyName = $dev.FriendlyName
    Class        = $dev.Class
    Status       = "$($dev.Status)"
    Properties   = Get-DeviceProperties $dev.InstanceId
  }
}
Write-Utf8Json $matchedDetail (Join-Path $rawDir "s4-match-detail.json") 6

# 3. USB children + HID sub-device paths under each match.
# Single bulk pass: one single-key property query per device (no nesting).
$parentById = @{}
foreach ($dev in @($allPnp)) { $parentById[$dev.InstanceId] = Get-ParentOf $dev.InstanceId }
$children = @()
foreach ($dev in @($allPnp | Where-Object { Test-S4Match $_.InstanceId })) {
  $parent = $parentById[$dev.InstanceId]
  $siblings = @($allPnp | Where-Object {
    $k = $parentById[$_.InstanceId]; $dk = $dev.InstanceId
    ($null -ne $k -and $k.ToUpperInvariant() -eq $dk.ToUpperInvariant()) -or
    ($null -ne $parent -and $null -ne $k -and $k.ToUpperInvariant() -eq $parent.ToUpperInvariant())
  } | Select-Object InstanceId, FriendlyName, Class, Status)
  $children += @{ Composite = $dev.InstanceId; Related = $siblings }
}
Write-Utf8Json $children (Join-Path $rawDir "usb-children.json") 6

# 4. Audio interfaces sharing a container with any match.
$audio = Get-PnpDevice -PresentOnly | Where-Object {
  ($_.Class -eq "MEDIA" -or $_.Class -eq "AudioEndpoint") -and (Test-S4Match $_.InstanceId)
} | Select-Object InstanceId, FriendlyName, Class, Status
Write-Utf8Json $audio (Join-Path $rawDir "audio-interfaces.json") 4

# 5. Services: exact NI identities, startup types, prior state (read-only query).
$services = Get-CimInstance Win32_Service | Where-Object {
  $_.Name -like "*NI*" -or $_.Name -like "*Traktor*" -or $_.Name -like "*LoopBe*"
} | Select-Object Name, DisplayName, State, StartMode, PathName, ProcessId
Write-Utf8Json $services (Join-Path $rawDir "services.json") 4

# 6. Competing/related processes: identity only, no termination.
$processNames = @("NIHardwareService", "NIHardwareAgent", "Traktor", "Mixxx",
  "S4MK3*", "S4*Bridge*", "S4MidiProbe", "S4LoopBeProbe", "djay*", "LoopBeMon*")
$processes = Get-Process | Where-Object {
  $n = $_.ProcessName; @($processNames | Where-Object { $n -like $_ }).Count -gt 0
} | Select-Object ProcessName, Id, Path
Write-Utf8Json $processes (Join-Path $rawDir "processes.json") 3

$inventory = [ordered]@{
  schema      = "s4-windows-device-inventory/v1"
  utc         = $utc
  machine     = $machine
  vidPid      = "17cc:1720"
  status      = $selection.status
  compositeId = $compositeId
  candidates  = $selection.candidates
  note        = "Collection evidence only; not proof of HID read/write, screen, motor, or audio access."
}
$inventory | ConvertTo-Json -Depth 4 | Set-Content -Encoding utf8 -Path (Join-Path $Evidence "inventory.json")

$manifest = Get-ChildItem $rawDir, (Join-Path $Evidence "inventory.json") -File |
  ForEach-Object {
    $h = (Get-FileHash -Algorithm SHA256 $_.FullName).Hash
    "$($h.ToLower())  $($_.Name)"
  }
$manifest | Set-Content (Join-Path $Evidence "sha256sums.txt")

if ($selection.status -ne "READY_CANDIDATE") { exit 2 }
exit 0
