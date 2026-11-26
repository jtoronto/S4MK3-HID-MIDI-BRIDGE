param(
    [Parameter(Mandatory = $true)]
    [string]$Executable,
    [ValidatePattern('^[A-Za-z0-9_:]*$')]
    [string]$Filter = "drop_tests",
    [ValidateRange(1, 300)]
    [int]$TimeoutSeconds = 30
)

$ErrorActionPreference = "Stop"
$path = (Resolve-Path -LiteralPath $Executable).Path
$arguments = @("--test-threads=1", "--nocapture")
if (-not [string]::IsNullOrWhiteSpace($Filter)) {
    $arguments = @($Filter) + $arguments
}

# This watchdog owns only the test process it starts. Compilation is separate,
# so a stalled synchronous native test cannot hide inside a Cargo build wait.
$start = New-Object System.Diagnostics.ProcessStartInfo
$start.FileName = $path
$start.Arguments = $arguments -join ' '
$start.UseShellExecute = $false
$process = New-Object System.Diagnostics.Process
$process.StartInfo = $start
try {
    if (-not $process.Start()) { throw "Native test process did not start." }
    if (-not $process.WaitForExit($TimeoutSeconds * 1000)) {
        & taskkill.exe /PID $process.Id /T /F
        throw "Native test process exceeded $TimeoutSeconds seconds; its process tree was terminated."
    }
    $process.WaitForExit()
    $code = $process.ExitCode
    Write-Output "NATIVE_TEST_EXIT=$code"
}
finally {
    $process.Dispose()
}
exit $code
