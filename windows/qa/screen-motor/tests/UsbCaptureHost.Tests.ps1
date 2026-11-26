#Requires -Version 5.1
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$Root,
    [string]$Source
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ([string]::IsNullOrEmpty($Source)) { $Source = Join-Path $PSScriptRoot '../UsbCaptureHost.cs' }

# The host gets a new console. The GUI-subsystem capturer must attach like
# stock USBPcapCMD rather than inheriting a console as a ConsoleApplication.
$helperSource = [IO.File]::ReadAllText((Join-Path $PSScriptRoot 'CaptureHostFixture.cs'))
$consoleSource = [IO.File]::ReadAllText((Join-Path $PSScriptRoot 'UsbPcapConsoleFixture.cs'))

$launcherSource = [IO.File]::ReadAllText((Join-Path $PSScriptRoot 'CaptureFixtureLauncher.cs'))

function Assert-Equal($Actual, $Expected, [string]$Name) {
    if ($Actual -cne $Expected) { throw "$Name : expected [$Expected], got [$Actual]" }
}

function Invoke-Case([string]$Mode) {
    $id = [guid]::NewGuid().ToString('N')
    $directory = Join-Path $script:workspace $id
    $null = [IO.Directory]::CreateDirectory($directory)
    $names = @("Local\S4Ready-$id", "Local\S4Stop-$id", "Local\S4Partial-$id", "Local\S4Release-$id", "Local\S4Exit-$id")
    $events = @()
    $process = $null
    try {
        foreach ($name in $names) {
            $events += [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::ManualReset, $name)
        }
        $scenario = "$Mode|$($names[2])|$($names[3])|$($names[4])"
        $arguments = 'host "{0}" "{1}" "{2}" "{3}" {4}' -f $scenario, $directory, $names[0], $names[1], $PID
        $process = [CaptureFixtureLauncher]::Start($script:helper, $arguments)
        if (-not $events[2].WaitOne(10000)) {
            $errorPath=Join-Path $directory 'result-error.txt'
            if(Test-Path -LiteralPath $errorPath){Write-Output ([IO.File]::ReadAllText($errorPath))}
            throw "$Mode partial header timed out"
        }
        Assert-Equal ([S4UsbCapture.CaptureHost]::WaitForReady($events[0], $process, 0)) $false "$Mode partial header not ready"
        $null = $events[3].Set()
        if ($Mode -eq 'good') {
            Assert-Equal ([S4UsbCapture.CaptureHost]::WaitForReady($events[0], $process, 10000)) $true 'complete header ready'
            $null = $events[1].Set()
        } elseif ($Mode -eq 'exit') {
            Assert-Equal ([S4UsbCapture.CaptureHost]::WaitForReady($events[0], $process, 10000)) $true 'exit case ready before child exit'
            $null = $events[4].Set()
        } else {
            Assert-Equal ([S4UsbCapture.CaptureHost]::WaitForReady($events[0], $process, 10000)) $false "$Mode invalid header not ready"
        }
        if (-not $process.WaitForExit(15000)) { throw "$Mode host exit timed out" }
        Assert-Equal $process.ExitCode 0 "$Mode helper exit"
        $result = ConvertFrom-StringData ([IO.File]::ReadAllText((Join-Path $directory 'result.txt')))
        Assert-Equal $result.ParentLost 'False' "$Mode parent alive"
        Assert-Equal $result.UsbPacketLossKnown 'False' "$Mode loss unknown"
        if ($Mode -eq 'good') {
            if ($result.Success -cne 'True') {
                Write-Output ([IO.File]::ReadAllText((Join-Path $directory 'usb-stderr.txt')))
            }
            Assert-Equal $result.Success 'True' 'success'
            Assert-Equal $result.HeaderReady 'True' 'header ready'
            Assert-Equal $result.ForcedTermination 'False' 'graceful stop'
            Assert-Equal $result.UnexpectedExit 'False' 'expected exit'
            Assert-Equal $result.ExitCode '0' 'capture exit'
            Assert-Equal $result.BytesWritten '27' 'persisted byte count'
            Assert-Equal $result.HasError 'False' 'no error'
            $expected = [byte[]](0xd4,0xc3,0xb2,0xa1,2,0,4,0,0,0,0,0,0,0,0,0,0,0,16,0,249,0,0,0,0x51,0x52,0x53)
            Assert-Equal ([Convert]::ToBase64String([IO.File]::ReadAllBytes((Join-Path $directory 'capture.pcap')))) ([Convert]::ToBase64String($expected)) 'header and post-q bytes'
            Assert-Equal ([IO.File]::ReadAllText((Join-Path $directory 'usb-stderr.txt'))) 'fake-stderr' 'stderr drained'
        } else {
            Assert-Equal $result.Success 'False' "$Mode failure"
            Assert-Equal $result.HasError 'True' "$Mode error reported"
            Assert-Equal ([S4UsbCapture.CaptureHost]::WaitForReady($events[0], $process, 0)) $false "$Mode exited host not ready"
            if ($Mode -in @('bad','bad-snap')) {
                Assert-Equal $result.HeaderReady 'False' 'invalid header rejected'
                Assert-Equal $result.ForcedTermination 'True' 'invalid stream terminated'
            } else {
                Assert-Equal $result.UnexpectedExit 'True' 'unexpected child exit'
                Assert-Equal $result.ExitCode '19' 'unexpected child exit code'
            }
        }
        Write-Output "CASE_PASSED $Mode"
    } finally {
        if ($null -ne $process) {
            $process.Refresh()
            if (-not $process.HasExited) {
                if ($events.Count -gt 3) { $null = $events[3].Set() }
                if ($events.Count -gt 1) { $null = $events[1].Set() }
                if ($events.Count -gt 4) { $null = $events[4].Set() }
                if (-not $process.WaitForExit(35000)) {
                    $process.Kill()
                    if (-not $process.WaitForExit(10000)) { throw 'Fixture host cleanup timed out' }
                }
            }
            $process.Dispose()
        }
        foreach ($event in $events) { $event.Dispose() }
    }
}

if (Test-Path -LiteralPath $Root) { throw 'Root must not already exist.' }
$script:workspace = [IO.Path]::GetFullPath($Root)
$null = [IO.Directory]::CreateDirectory($script:workspace)
try {
    $script:helper = Join-Path $script:workspace 'CaptureFixture.exe'
    $consolePath=Join-Path ([IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($Source))) 'CaptureConsole.cs'
    $signalPath=Join-Path ([IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($Source))) 'ReadinessSignal.cs'
    $resultPath=Join-Path ([IO.Path]::GetDirectoryName([IO.Path]::GetFullPath($Source))) 'CaptureResult.cs'
    $hostSource = [IO.File]::ReadAllText([IO.Path]::GetFullPath($Source)) +
        [Environment]::NewLine + [IO.File]::ReadAllText($consolePath) +
        [Environment]::NewLine + [IO.File]::ReadAllText($signalPath) +
        [Environment]::NewLine + [IO.File]::ReadAllText($resultPath)
    $script:runtimeLibrary=Join-Path $script:workspace 'UsbCaptureRuntime.dll'
    $compiler=[Microsoft.CSharp.CSharpCodeProvider]::new()
    try {
        $options=[CodeDom.Compiler.CompilerParameters]::new()
        $options.GenerateInMemory=$false
        $options.OutputAssembly=$script:runtimeLibrary
        $options.TreatWarningsAsErrors=$true
        $null=$options.ReferencedAssemblies.Add('System.dll')
        $compiled=$compiler.CompileAssemblyFromFile($options,[string[]]@([IO.Path]::GetFullPath($Source),$consolePath,$signalPath,$resultPath))
        if ($compiled.Errors.HasErrors) { throw ($compiled.Errors|Out-String) }
    } finally { $compiler.Dispose() }
    $definition=$hostSource + [Environment]::NewLine + $helperSource + [Environment]::NewLine + $consoleSource
    Add-Type -TypeDefinition $definition -OutputAssembly $script:helper -OutputType ConsoleApplication
    $script:captureFixture=Join-Path $script:workspace 'UsbPcapFixture.exe'
    $compiler=[Microsoft.CSharp.CSharpCodeProvider]::new()
    try {
        $options=[CodeDom.Compiler.CompilerParameters]::new()
        $options.GenerateExecutable=$true
        $options.GenerateInMemory=$false
        $options.OutputAssembly=$script:captureFixture
        $options.CompilerOptions='/target:winexe'
        $options.TreatWarningsAsErrors=$true
        $null=$options.ReferencedAssemblies.Add('System.dll')
        $compiled=$compiler.CompileAssemblyFromSource($options,$definition)
        if($compiled.Errors.HasErrors){throw ($compiled.Errors|Out-String)}
    } finally {$compiler.Dispose()}
    $null = [Reflection.Assembly]::Load([IO.File]::ReadAllBytes($script:helper))
    Add-Type -TypeDefinition $launcherSource
    foreach ($mode in @('good', 'bad', 'bad-snap', 'exit')) { Invoke-Case $mode }
    . (Join-Path $PSScriptRoot '../Usb-Capture.ps1')
    $moduleId=[guid]::NewGuid().ToString('N')
    $partialName="Local\S4ModulePartial-$moduleId"
    $releaseName="Local\S4ModuleRelease-$moduleId"
    $unusedName="Local\S4ModuleExit-$moduleId"
    $partial=[Threading.EventWaitHandle]::new($false,[Threading.EventResetMode]::ManualReset,$partialName)
    $release=[Threading.EventWaitHandle]::new($true,[Threading.EventResetMode]::ManualReset,$releaseName)
    $unused=[Threading.EventWaitHandle]::new($false,[Threading.EventResetMode]::ManualReset,$unusedName)
    try {
        $box=@{session=$null;result=$null}
        $open={
            $box.session=Open-UsbCaptureSession -Directory (Join-Path $script:workspace 'module') `
                -Executable $script:captureFixture -CaptureInterface "good|$partialName|$releaseName|$unusedName" `
                -DeviceAddress 3 -Assembly $script:runtimeLibrary
        }
        $close={
            if ($null -ne $box.session) {
                $current=$box.session
                $box.session=$null
                $box.result=Close-UsbCaptureSession $current
            }
        }
        & $open
        & $close
        $outcome=$box.result
        Assert-Equal $outcome.Success $true 'PowerShell host wrapper success'
        Assert-Equal $outcome.BytesWritten 27 'PowerShell host wrapper stream bytes'
        Write-Output 'CASE_PASSED powershell-wrapper'
    } finally { $partial.Dispose();$release.Dispose();$unused.Dispose() }

    . (Join-Path $PSScriptRoot 'Run-Case.Wiring.Tests.ps1')
    Write-Output 'SUMMARY capture host fixture cases passed'
} finally {
    Remove-Item -LiteralPath $script:workspace -Recurse -Force
}
