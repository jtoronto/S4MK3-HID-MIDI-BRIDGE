param(
    [Parameter(Mandatory = $true)][string]$RuntimeRoot,
    [Parameter(Mandatory = $true)][string]$EvidenceRoot
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$runtime = (Resolve-Path -LiteralPath $RuntimeRoot).Path
# Before this runner, set S4_SCREEN_SUPERVISOR_FIXTURE to the absolute path of
# the staged s4-screen-supervisor.exe; its explicit self-test modes never use NI.
$screenSupervisorFixture = [Environment]::GetEnvironmentVariable('S4_SCREEN_SUPERVISOR_FIXTURE')
if ([string]::IsNullOrWhiteSpace($screenSupervisorFixture)) { throw 'Required environment missing: S4_SCREEN_SUPERVISOR_FIXTURE' }
if (-not (Test-Path -LiteralPath $screenSupervisorFixture -PathType Leaf)) { throw 'Required staged supervisor fixture executable is missing' }
$screenSupervisorFixture = (Resolve-Path -LiteralPath $screenSupervisorFixture).Path
if (Test-Path -LiteralPath $EvidenceRoot) { throw "Evidence root already exists: $EvidenceRoot" }
$null = New-Item -ItemType Directory -Path $EvidenceRoot
$evidence = (Resolve-Path -LiteralPath $EvidenceRoot).Path
# Validate every shipped payload before running any test executable.
$ipcFixtureVerified = $false
foreach ($line in [IO.File]::ReadAllLines((Join-Path $runtime 'SHA256SUMS'))) {
    if ($line -notmatch '^([a-f0-9]{64})  (.+)$') { throw 'Invalid SHA256SUMS record' }
    $relative = $Matches[2]
    if ([IO.Path]::IsPathRooted($relative) -or $relative.Split([char[]]@('/', '\')) -contains '..') { throw 'Unsafe manifest path' }
    $path = Join-Path $runtime $relative
    if ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant() -cne $Matches[1]) {
        throw "Hash mismatch: $relative"
    }
    if ($relative -ceq 'tests/s4-screen-ipc-fixture.exe') { $ipcFixtureVerified = $true }
}
if (-not $ipcFixtureVerified) { throw 'Required hashed IPC fixture missing: tests/s4-screen-ipc-fixture.exe' }
foreach ($test in @('shared-unit', 'engine-unit', 'engine-cli')) {
    $executable = Join-Path $runtime "tests\$test.exe"
    $start = [DateTime]::UtcNow
    $info = New-Object Diagnostics.ProcessStartInfo
    $info.FileName = $executable
    $info.Arguments = '--test-threads=1'
    $info.WorkingDirectory = $runtime
    $info.UseShellExecute = $false
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.EnvironmentVariables['S4_BRIDGE_TEST_EXECUTABLE'] = Join-Path $runtime 's4mk3-hid-midi-bridge.exe'
    $info.EnvironmentVariables['S4_SCREEN_SUPERVISOR_FIXTURE'] = $screenSupervisorFixture
    $process = New-Object Diagnostics.Process
    $process.StartInfo = $info
    if (-not $process.Start()) { throw "Could not start $test" }
    $pidValue = $process.Id
    $stdout = $process.StandardOutput.ReadToEndAsync()
    $stderr = $process.StandardError.ReadToEndAsync()
    if (-not $process.WaitForExit(120000)) {
        $process.Kill()
        $process.Dispose()
        throw "Native test timeout: $test"
    }
    $exit = $process.ExitCode
    $text = $stdout.GetAwaiter().GetResult()
    $errors = $stderr.GetAwaiter().GetResult()
    $process.Dispose()
    [IO.File]::WriteAllText((Join-Path $evidence "$test.stdout.txt"), $text)
    [IO.File]::WriteAllText((Join-Path $evidence "$test.stderr.txt"), $errors)
    @{
        executable = $executable; arguments = '--test-threads=1'; pid = $pidValue
        started_utc = $start.ToString('o'); ended_utc = [DateTime]::UtcNow.ToString('o')
        exit_code = $exit; hardware_acceptance = $false
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $evidence "$test.record.json") -Encoding UTF8
    if ($exit -ne 0 -or $text -notmatch 'test result: ok\. [1-9][0-9]* passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;') {
        throw "Native suite failed or incomplete: $test (exit $exit)"
    }
}
Write-Output 'WINDOWS_ENGINE_NATIVE_TESTS_PASSED'
