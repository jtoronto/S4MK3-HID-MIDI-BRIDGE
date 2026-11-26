#Requires -Version 5.1
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$AppDirectory,
    [Parameter(Mandatory=$true)][string]$PreferenceDirectory,
    [Parameter(Mandatory=$true)][string]$InstalledMapping,
    [Parameter(Mandatory=$true)][string]$SourceIdentity,
    [Parameter(Mandatory=$true)][string]$EvidenceDirectory,
    [Parameter(Mandatory=$true)][ValidateSet('A','B','C','D','N','N-off')][string]$Case
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

# Never launch the app, change preferences, install a mapping, or touch services.
foreach ($name in @('S4Bridge','s4mk3-hid-midi-bridge')) {
    if (Get-Process -Name $name -ErrorAction SilentlyContinue) {
        throw "Stop all $name processes before preparing a diagnostic copy."
    }
}
$app = (Resolve-Path -LiteralPath $AppDirectory).ProviderPath
$prefs = (Resolve-Path -LiteralPath $PreferenceDirectory).ProviderPath
$mapping = (Resolve-Path -LiteralPath $InstalledMapping).ProviderPath
$source = (Resolve-Path -LiteralPath $SourceIdentity).ProviderPath
foreach ($p in @($mapping,$source)) {
    if (-not (Test-Path -LiteralPath $p -PathType Leaf)) { throw "Expected file: $p" }
}
foreach ($p in @($app,$prefs)) {
    if (-not (Test-Path -LiteralPath $p -PathType Container)) { throw "Expected directory: $p" }
}
$destination = [IO.Path]::GetFullPath($EvidenceDirectory)
if (Test-Path -LiteralPath $destination) { throw "Evidence directory already exists: $destination" }
foreach ($protected in @($app,$prefs)) {
    $base = $protected.TrimEnd('\') + '\'
    if (($destination + '\').StartsWith($base, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Evidence directory must not be inside the app or preferences.'
    }
}
$required = @('S4Bridge.exe','Resources\s4mk3-hid-midi-bridge.exe',
    'Resources\s4-screen-supervisor.exe','Resources\s4-encdr-screen-worker.exe',
    'Resources\S4 MK3 Bridge Windows.djayMidiMapping','Resources\screen-access.json')
foreach ($relative in $required) {
    if (-not (Test-Path -LiteralPath (Join-Path $app $relative) -PathType Leaf)) {
        throw "Diagnostic app is missing $relative"
    }
}
$packagedMapping = Join-Path $app 'Resources\S4 MK3 Bridge Windows.djayMidiMapping'
$installedMappingHash = (Get-FileHash -LiteralPath $mapping -Algorithm SHA256).Hash
$packagedMappingHash = (Get-FileHash -LiteralPath $packagedMapping -Algorithm SHA256).Hash
if ($installedMappingHash -ne $packagedMappingHash) {
    Write-Warning 'Installed and packaged mapping hashes differ. Both are preserved; mapping equivalence is unverified.'
}
$null = New-Item -ItemType Directory -Path $destination
$staged = Join-Path $destination 'App'
$snapshot = Join-Path $destination 'preferences-before'
Copy-Item -LiteralPath $app -Destination $staged -Recurse
Copy-Item -LiteralPath $prefs -Destination $snapshot -Recurse
Copy-Item -LiteralPath $mapping -Destination (Join-Path $destination 'installed-mapping.djayMidiMapping')
Copy-Item -LiteralPath $source -Destination (Join-Path $destination 'source-identity')
$appFiles = @(Get-ChildItem -LiteralPath $app -File -Recurse)
$appHashes = @()
foreach ($file in $appFiles) {
    $relative = $file.FullName.Substring($app.Length).TrimStart('\')
    $copy = Join-Path $staged $relative
    $hash = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
    if ($hash -ne (Get-FileHash -LiteralPath $copy -Algorithm SHA256).Hash) { throw "App copy mismatch: $relative" }
    $appHashes += [ordered]@{ path = $relative; sha256 = $hash }
}
$preferenceHashes = @()
foreach ($file in @(Get-ChildItem -LiteralPath $prefs -File -Recurse)) {
    $relative = $file.FullName.Substring($prefs.Length).TrimStart('\')
    $hash = (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash
    if ($hash -ne (Get-FileHash -LiteralPath (Join-Path $snapshot $relative) -Algorithm SHA256).Hash) {
        throw "Preference snapshot mismatch: $relative"
    }
    $preferenceHashes += [ordered]@{ path = $relative; sha256 = $hash }
}
$caseName = @{ A='off'; B='acquired'; C='static'; D='updates'; N='normal'; 'N-off'='normal' }[$Case]
$capture = $Case -ne 'N-off'
$configPath = Join-Path $destination 'diagnostic-config.json'
$capturePath = Join-Path $destination 'capture.jsonl'
$configJson = [ordered]@{ version=1; case=$caseName; capture_path=$capturePath; capture=$capture } | ConvertTo-Json
[IO.File]::WriteAllText($configPath, $configJson, [Text.UTF8Encoding]::new($false))
$manifest = [ordered]@{
    version=1; prepared_utc=[DateTime]::UtcNow.ToString('o'); case=$Case
    app_source=$app; preference_source=$prefs; mapping_source=$mapping; source_identity=$source
    mapping_sha256=$installedMappingHash; packaged_mapping_sha256=$packagedMappingHash
    mappings_match=($installedMappingHash -eq $packagedMappingHash)
    source_sha256=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
    config_sha256=(Get-FileHash -LiteralPath $configPath -Algorithm SHA256).Hash
    app_files=$appHashes; preference_files=$preferenceHashes
}
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $destination 'manifest.json') -Encoding UTF8
Write-Host "Prepared stopped app: $staged"
Write-Host "Start command: powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$PSScriptRoot\Run-Case.ps1`" -Prepared `"$destination`" -Case `"$Case`""
Write-Host 'No app was started; preferences and services were not changed.'
