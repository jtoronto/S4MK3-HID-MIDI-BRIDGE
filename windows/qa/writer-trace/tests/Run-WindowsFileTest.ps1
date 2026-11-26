#Requires -Version 5.1
param(
    [Parameter(Mandatory=$true)][string]$PackageDirectory,
    [Parameter(Mandatory=$true)][string]$EvidenceDirectory
)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
if (Test-Path -LiteralPath $EvidenceDirectory) { throw 'Use a new fixture evidence directory.' }
$null=New-Item -ItemType Directory -Path $EvidenceDirectory
$identity=Get-Content -LiteralPath (Join-Path $PackageDirectory 'TRACE-IDENTITY.json') -Raw|ConvertFrom-Json
foreach($entry in $identity.files){
    if((Get-FileHash -LiteralPath (Join-Path $PackageDirectory $entry.path) -Algorithm SHA256).Hash -ine $entry.sha256){
        throw "Fixture package identity mismatch: $($entry.path)"
    }
}
$refs=@('System.dll','System.Core.dll','System.Xml.dll','System.Runtime.Serialization.dll')
$sources=@('WriterSession.cs','WriterSession.Process.cs','WriterSession.Protocol.cs')|
    ForEach-Object{Join-Path $PackageDirectory $_}
Add-Type -Path $sources -ReferencedAssemblies $refs
$exe=Join-Path $EvidenceDirectory 'OrdinaryFileFixture.exe'
Add-Type -Path (Join-Path $PSScriptRoot 'WindowsFileFixture.cs') -OutputAssembly $exe `
    -OutputType ConsoleApplication -ReferencedAssemblies $refs
$data=Join-Path $EvidenceDirectory 'ordinary-file.bin'
$info=[Diagnostics.ProcessStartInfo]::new()
$info.FileName=$exe
$info.Arguments='"'+$data+'"'
$info.UseShellExecute=$false
$info.CreateNoWindow=$true
$info.RedirectStandardInput=$true
$info.RedirectStandardOutput=$true
$info.RedirectStandardError=$true
$process=$null
$session=$null
$passed=$false
$cleanupError=$null
function Save-Phase([string]$phase){
    [IO.File]::WriteAllText((Join-Path $EvidenceDirectory 'phase.txt'),$phase)
}
try{
    Save-Phase 'starting-fixture'
    $process=[Diagnostics.Process]::Start($info)
    $ready=$process.StandardOutput.ReadLineAsync()
    if(-not $ready.Wait(5000) -or $ready.Result -cne "READY $($process.Id)"){throw 'File fixture failed readiness.'}
    $session=[WriterSession]::new((Join-Path $PackageDirectory 'frida-inject.exe'),$process.Id,
        (Join-Path $PackageDirectory 'writer-agent.js'),$EvidenceDirectory)
    $session.Start([TimeSpan]::FromSeconds(10))
    Save-Phase 'attached-owned-file-fixture'
    $process.StandardInput.WriteLine('WRITE')
    $process.StandardInput.Flush()
    $written=$process.StandardOutput.ReadLineAsync()
    if(-not $written.Wait(5000) -or $written.Result -cne 'WRITTEN'){throw 'Ordinary file write did not finish.'}
    $session.Stop([TimeSpan]::FromSeconds(10))
    Save-Phase 'collector-stopped'
    $rows=@(Get-Content -LiteralPath $session.StdoutPath|ForEach-Object{$_|ConvertFrom-Json})
    $batches=@($rows|Where-Object{$_.kind -ceq 'batch'})
    $entries=@($batches|ForEach-Object{$_.events}|Where-Object{$_.kind -ceq 'entry'})
    if($entries.Count -ne 302){throw "Expected 302 real NtWriteFile entries; found $($entries.Count)."}
    if($entries[0].api -cne 'NtWriteFile' -or $entries[0].reports[0].hex -cne '3101e0fe000001e0fe0000' -or
        $entries[1].reports[0].hex -cne '3101200100000120018629' -or
        $entries[1].reports[0].buffer_length -ne 95){throw 'Native Windows payload decoding differs.'}
    $varying=@($entries | Select-Object -Skip 2)
    $stackIds=@($varying.stack_id | Sort-Object -Unique)
    if($stackIds.Count -ne 1 -or $null -eq $stackIds[0] -or
        @($varying | Where-Object {$null -eq $_.stack_id}).Count -ne 0 -or
        $entries[1].stack_id -eq $stackIds[0]){
        throw 'Varying payloads must share a stack sample within their own write stream.'
    }
    for($i=0; $i -lt 300; $i++){
        $expected='3101{0:x2}{1:x2}000001e0fe0000' -f ($i -band 255),($i -shr 8)
        if($varying[$i].reports[0].hex -cne $expected){throw "Native payload differs at write $i."}
    }
    if(@($rows|Where-Object{$_.pid -ne $process.Id}).Count -ne 0){throw 'Writer PID attribution differs.'}
    if(@($entries|Where-Object{$_.tid -le 0 -or $null -eq $_.caller_address}).Count -ne 0){throw 'Caller identity missing.'}
    $returns=@($batches|ForEach-Object{$_.events}|Where-Object{$_.kind -ceq 'return'})
    if($returns.Count -ne 302 -or @($returns|Where-Object{$_.status -ne 0 -or -not $_.accepted}).Count -ne 0){
        throw 'Synchronous ordinary-file returns differ.'
    }
    $process.StandardInput.WriteLine('EXIT')
    $process.StandardInput.Flush()
    if(-not $process.WaitForExit(5000) -or $process.ExitCode -ne 0){throw 'File fixture did not exit normally.'}
    Save-Phase 'fixture-exited'
    $bytes=[IO.File]::ReadAllBytes($data)
    if($bytes.Length -ne 3406 -or $bytes[0] -ne 0x31 -or $bytes[11] -ne 0x31){throw 'File writes were altered.'}
    $passed=$true
    Save-Phase 'passed'
    Write-Output 'WINDOWS_FILE_HOOK_TEST_PASS: exact bytes, PID/TID, caller and STOP acknowledgement; ordinary file only.'
}catch{
    [IO.File]::WriteAllText((Join-Path $EvidenceDirectory 'failure.txt'),$_.Exception.ToString())
    throw
}finally{
    if($null -ne $session){
        try{$session.Dispose()}catch{$cleanupError=$_.Exception.Message}
    }
    if($null -ne $process -and -not $process.HasExited){
        try{
            $process.StandardInput.WriteLine('EXIT')
            $process.StandardInput.Flush()
        }catch{
            $cleanupError=$_.Exception.Message
        }finally{
            if(-not $process.WaitForExit(1000)){$process.Kill()}
        }
    }
    [ordered]@{passed=$passed;controller_opened=$false;hid_api_called=$false;usb_trace_started=$false;
        ni_process_attached=$false; evidence=$EvidenceDirectory;cleanup_error=$cleanupError}|
        ConvertTo-Json|Set-Content -LiteralPath (Join-Path $EvidenceDirectory 'result.json') -Encoding UTF8
    if($null -ne $cleanupError){throw $cleanupError}
}
