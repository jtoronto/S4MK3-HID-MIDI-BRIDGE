#Requires -Version 5.1

function Open-UsbCaptureSession {
    param(
        [Parameter(Mandatory=$true)][string]$Directory,
        [Parameter(Mandatory=$true)][string]$Executable,
        [Parameter(Mandatory=$true)][string]$CaptureInterface,
        [Parameter(Mandatory=$true)][int]$DeviceAddress,
        [Parameter(Mandatory=$true)][string]$Assembly
    )
    if (Test-Path -LiteralPath $Directory) { throw 'USB capture directory already exists.' }
    $null = New-Item -ItemType Directory -Path $Directory
    $id = [guid]::NewGuid().ToString('N')
    $readyName = "Local\S4UsbReady-$id"
    $stopName = "Local\S4UsbStop-$id"
    $ready = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::ManualReset, $readyName)
    $stop = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::ManualReset, $stopName)
    $session = [pscustomobject]@{ Directory=$Directory; Ready=$ready; Stop=$stop; Process=$null }
    try {
        $configuration = [ordered]@{
            executable=$Executable; captureInterface=$CaptureInterface; deviceAddress=$DeviceAddress
            directory=$Directory; assembly=$Assembly; readyEvent=$readyName; stopEvent=$stopName
            parentPid=$PID; maximumSeconds=180
        }
        $config = Join-Path $Directory 'usb-session.json'
        [IO.File]::WriteAllText($config, ($configuration | ConvertTo-Json), [Text.UTF8Encoding]::new($false))
        # The host needs its own real console: USBPcap reads console KEY_EVENTs,
        # not text sent through a redirected stdin pipe.
        $hostScript = Join-Path $PSScriptRoot 'Usb-CaptureHost.ps1'
        $session.Process = Start-Process -FilePath "$PSHOME\powershell.exe" -ArgumentList @(
            '-NoProfile','-ExecutionPolicy','Bypass','-File',"`"$hostScript`"",
            '-Configuration',"`"$config`""
        ) -WindowStyle Minimized -PassThru
        if (-not [S4UsbCapture.CaptureHost]::WaitForReady($ready, $session.Process, 15000)) {
            throw "USB capture did not become ready. See $Directory"
        }
        return $session
    } catch {
        if ($null -ne $session.Process) { $null = Close-UsbCaptureSession $session }
        else { $ready.Dispose(); $stop.Dispose() }
        throw
    }
}

function Close-UsbCaptureSession {
    param([Parameter(Mandatory=$true)]$Session)
    try {
        $null = $Session.Stop.Set()
        if (-not $Session.Process.WaitForExit(20000)) {
            throw "USB capture host did not stop; preserve $($Session.Directory). Host PID $($Session.Process.Id)."
        }
        $receipt = Join-Path $Session.Directory 'usb-result.json'
        if (-not (Test-Path -LiteralPath $receipt -PathType Leaf)) {
            throw "USB capture host exited without a completion receipt: $($Session.Directory)"
        }
        $result = Get-Content -LiteralPath $receipt -Raw | ConvertFrom-Json
        if (-not $result.Success) { throw "USB recording incomplete: $($result.Error)" }
        return $result
    } finally {
        $Session.Ready.Dispose()
        $Session.Stop.Dispose()
        $Session.Process.Dispose()
    }
}

function Confirm-UsbCaptureTarget {
    param(
        [Parameter(Mandatory=$true)][string]$Executable,
        [Parameter(Mandatory=$true)][string]$CaptureInterface,
        [Parameter(Mandatory=$true)][int]$DeviceAddress
    )
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $Executable
    $info.Arguments = "--extcap-config --extcap-interface $CaptureInterface"
    $info.UseShellExecute = $false
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.CreateNoWindow = $true
    $process = [Diagnostics.Process]::Start($info)
    try {
        $output = $process.StandardOutput.ReadToEndAsync()
        $errors = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit(10000)) {
            $process.Kill()
            $process.WaitForExit()
            throw 'USB topology inspection timed out.'
        }
        $text = $output.GetAwaiter().GetResult()
        $stderr = $errors.GetAwaiter().GetResult()
        if ($process.ExitCode -ne 0 -or $stderr) { throw "USB topology inspection failed: $stderr" }
        $parent = [regex]::Escape("$DeviceAddress")
        $s4 = "(?m)^value .*?\{display=Traktor Kontrol S4 MK3\}.*?\{parent=$parent\}\r?$"
        if ($text -notmatch $s4) {
            throw 'The S4 USB address changed. Stop and have the capture configuration checked; do not select another device by guess.'
        }
        return $text
    } finally { $process.Dispose() }
}

function Invoke-UsbCapturePair {
    param(
        [Parameter(Mandatory=$true)][string]$PairRoot,
        [Parameter(Mandatory=$true)][int]$DeviceAddress,
        [Parameter(Mandatory=$true)][scriptblock]$PrepareCase,
        [Parameter(Mandatory=$true)][scriptblock]$RunCase,
        [Parameter(Mandatory=$true)][scriptblock]$ReadBaselineAnswer
    )
    $pair=[ordered]@{version=1;startedUtc=[DateTime]::UtcNow.ToString('o');complete=$false;baselineConfirmed=$false;cases=@();error=$null}
    try {
        foreach ($case in @('A','B')) {
            $run=Join-Path $PairRoot $case
            & $PrepareCase $case $run
            $usbResult=& $RunCase $case $run
            $bridge=Get-Content -LiteralPath (Join-Path $run 'result.json') -Raw|ConvertFrom-Json
            $pcap=Join-Path (Join-Path $run 'usb') 'capture.pcap'
            $coverage=[S4UsbCapture.UsbPcapReader]::Read($pcap,$DeviceAddress)
            $record=[ordered]@{
                case=$case;bridge=$bridge;usb=$usbResult;coverage=$coverage
                pcapSha256=(Get-FileHash -LiteralPath $pcap -Algorithm SHA256).Hash.ToLowerInvariant()
            }
            $record|ConvertTo-Json -Depth 10|Set-Content -LiteralPath (Join-Path $run 'combined-result.json') -Encoding UTF8
            $pair.cases+=@($record)
            if (-not $bridge.evidence_complete -or $null -eq $usbResult -or -not $usbResult.Success -or
                -not $coverage.FileComplete -or -not $coverage.HasMotorAndRingCoverage) {
                throw "Case $case is incomplete or expected S4 motor/ring traffic is missing. Preserve the folder; no next case will start."
            }
            Write-Host "Case $case saved; S4 motor and ring USB requests are present."
            if ($case -eq 'A') {
                $answer=& $ReadBaselineAnswer
                if ($answer -cne 'NEXT') { throw 'Stopped after A at operator request or input closure.' }
                $pair.baselineConfirmed=$true
            }
        }
        $pair.complete=$true
    } catch {
        $pair.error=$_.Exception.ToString()
        Write-Warning "Stopped: $($_.Exception.Message)"
    } finally {
        $pair.endedUtc=[DateTime]::UtcNow.ToString('o')
        $pair|ConvertTo-Json -Depth 12|Set-Content -LiteralPath (Join-Path $PairRoot 'pair-result.json') -Encoding UTF8
    }
    return [pscustomobject]$pair
}
