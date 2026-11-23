#Requires -Version 5.1
[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Root)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot '../Bridge-Capture.ps1')

function Assert-Equal($Actual, $Expected, [string]$Name) {
    if ($Actual -cne $Expected) {
        throw "$Name : expected [$Expected], got [$Actual]"
    }
}

function New-Capture([string]$Case, [string]$Fault) {
    $caseName = @{ A='off'; B='acquired'; C='static'; D='updates'; N='normal' }[$Case]
    $header = @{ kind='header'; case=$caseName; utc_unix_us=123456 }
    $footer = @{
        kind='footer'; incomplete=$false; runtime_ok=$true; first_stop_us=123
        roles=@(
            @{ role='input'; losses=0 }
            @{ role='output'; losses=0 }
            @{ role='screen'; losses=0 }
        )
    }
    $events = [Collections.Generic.List[object]]::new()
    $events.Add(@{ kind='screen_case_ready' })
    if ($Case -in @('B','C','D')) { $events.Add(@{ kind='screen_connect_begin' }) }
    $publications = @{ A=0; B=0; C=1; D=2; N=0 }[$Case]
    for ($i = 0; $i -lt $publications; $i++) {
        $events.Add(@{ kind='screen_publish_end'; values=@(0,0,1) })
    }
    switch ($Fault) {
        'case' { $header.case = 'wrong' }
        'anchor' { $header.utc_unix_us = 0 }
        'incomplete' { $footer.incomplete = $true }
        'runtime' { $footer.runtime_ok = $false }
        'stop' { $footer.first_stop_us = $null }
        'loss' { $footer.roles[1].losses = 1 }
        'roles' { $footer.roles = @($footer.roles[0], $footer.roles[1]) }
        'ready' { $events.RemoveAt(0) }
        'traffic' { $events.Add(@{ kind='screen_connect_begin' }) }
        'missing-connect' { $events.RemoveAt(1) }
        'missing-publication' { $events.RemoveAt($events.Count - 1) }
        'publish' { $events[2].values[2] = 0 }
        'release' { $events.Add(@{ kind='screen_release_end'; values=@(0,0,0) }) }
        'lifecycle' { $events.Add(@{ kind='lifecycle'; values=@(100,0,0) }) }
    }
    $rows = [Collections.Generic.List[object]]::new()
    if ($Fault -ne 'header') { $rows.Add($header) }
    foreach ($event in $events) { $rows.Add($event) }
    if ($Fault -ne 'footer') { $rows.Add($footer) }
    return ,$rows.ToArray()
}

function Test-Fixture([string]$Case, [string]$Fault) {
    $directory = Join-Path $script:workspace ([guid]::NewGuid().ToString())
    $null = New-Item -ItemType Directory -Path $directory
    $path = Join-Path $directory 'capture.jsonl'
    $rows = New-Capture $Case $Fault
    $lines = foreach ($row in $rows) { ConvertTo-Json -InputObject $row -Depth 8 -Compress }
    [IO.File]::WriteAllLines($path, [string[]]$lines, [Text.UTF8Encoding]::new($false))
    $resultPath = Join-Path $directory 'result.json'
    [IO.File]::WriteAllBytes($resultPath, [byte[]](123,34,107,34,58,49,125,10))
    $resultBefore = [IO.File]::ReadAllBytes($resultPath)
    $before = [IO.File]::ReadAllBytes($path)
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
    $result = Test-BridgeCapture -Prepared $directory -Case $Case
    Assert-Equal $result.complete ($Fault -eq '') "$Case/$Fault complete"
    Assert-Equal @($result.incompleteReasons).Count ([int]($Fault -ne '')) "$Case/$Fault reason count"
    Assert-Equal $result.recordCount $rows.Count "$Case/$Fault record count"
    Assert-Equal $result.traceSha256 $hash "$Case/$Fault hash"
    Assert-Equal ([Convert]::ToBase64String([IO.File]::ReadAllBytes($path))) `
        ([Convert]::ToBase64String($before)) "$Case/$Fault capture unchanged"
    Assert-Equal ([Convert]::ToBase64String([IO.File]::ReadAllBytes($resultPath))) `
        ([Convert]::ToBase64String($resultBefore)) "$Case/$Fault result unchanged"
    Assert-Equal @(Get-ChildItem -LiteralPath $directory -Force).Count 2 "$Case/$Fault files unchanged"
    return $result
}

function Start-TestChild {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = (Get-Process -Id $PID).Path
    $info.Arguments = '-NoProfile -NonInteractive -Command "[void][Console]::ReadLine()"'
    $info.UseShellExecute = $false
    $info.RedirectStandardInput = $true
    $info.CreateNoWindow = $true
    return [Diagnostics.Process]::Start($info)
}

function Stop-TestChild([Diagnostics.Process]$Child) {
    $Child.Refresh()
    if (-not $Child.HasExited) {
        $Child.StandardInput.WriteLine('')
        $Child.StandardInput.Close()
        if (-not $Child.WaitForExit(10000)) {
            $Child.Kill()
            if (-not $Child.WaitForExit(10000)) { throw 'Software child could not be terminated.' }
            throw 'Software child did not exit after stdin was closed.'
        }
    }
    $Child.Dispose()
}

$rootPath = [IO.Path]::GetFullPath($Root)
$null = [IO.Directory]::CreateDirectory($rootPath)
$script:workspace = Join-Path $rootPath ("bridge-capture-tests-" + [guid]::NewGuid())
$null = New-Item -ItemType Directory -Path $script:workspace
try {
    foreach ($case in @('A','B','C','D','N')) {
        $result = Test-Fixture $case ''
        $expectedConnect = [int]($case -in @('B','C','D'))
        $expectedPublish = @{ A=0; B=0; C=1; D=2; N=0 }[$case]
        Assert-Equal $result.header.case (@{ A='off'; B='acquired'; C='static'; D='updates'; N='normal' }[$case]) "$case header"
        Assert-Equal $result.footer.runtime_ok $true "$case footer"
        Assert-Equal $result.readyCount 1 "$case ready"
        Assert-Equal $result.connectCount $expectedConnect "$case connect"
        Assert-Equal $result.publishCount $expectedPublish "$case publish"
        Assert-Equal $result.failedPublishCount 0 "$case failed publish"
        Assert-Equal $result.failedReleaseCount 0 "$case failed release"
    }
    foreach ($fault in @('header','footer','case','anchor','incomplete','runtime','stop','loss','roles','ready','traffic','release','lifecycle')) {
        $result = Test-Fixture 'A' $fault
        if ($fault -eq 'release') { Assert-Equal $result.failedReleaseCount 1 'failed release count' }
    }
    $null = Test-Fixture 'B' 'missing-connect'
    $null = Test-Fixture 'C' 'missing-publication'
    $null = Test-Fixture 'D' 'missing-publication'
    $result = Test-Fixture 'C' 'publish'
    Assert-Equal $result.failedPublishCount 1 'failed publication count'
    Assert-Equal $result.publishCount 1 'failed publication included in count'

    $missing = Join-Path $script:workspace 'missing'
    $null = New-Item -ItemType Directory -Path $missing
    $result = Test-BridgeCapture -Prepared $missing -Case 'A'
    Assert-Equal $result.complete $false 'missing capture incomplete'
    Assert-Equal @($result.incompleteReasons).Count 1 'missing capture reason'
    Assert-Equal $result.recordCount 0 'missing capture record count'
    Assert-Equal $result.traceSha256 $null 'missing capture hash'

    $disabledPath = Join-Path $missing 'capture.jsonl'
    [IO.File]::WriteAllText($disabledPath, 'capture remains untouched')
    $disabledBefore = [IO.File]::ReadAllBytes($disabledPath)
    $result = Test-BridgeCapture -Prepared $missing -Case 'N-off'
    Assert-Equal $result.complete $false 'disabled capture incomplete'
    Assert-Equal @($result.incompleteReasons).Count 1 'disabled capture reason'
    Assert-Equal $result.recordCount 0 'disabled capture record count'
    Assert-Equal $result.traceSha256 $null 'disabled capture hash'
    Assert-Equal ([Convert]::ToBase64String([IO.File]::ReadAllBytes($disabledPath))) `
        ([Convert]::ToBase64String($disabledBefore)) 'disabled capture unchanged'

    $child = Start-TestChild
    try {
        $state = @{ calls=0; child=$child }
        $answer = {
            $state.calls++
            $state.child.Refresh()
            if ($state.child.HasExited) { throw 'Child exited before retry.' }
            if ($state.calls -eq 1) { return '' }
            if ($state.calls -ne 2) { throw 'Unexpected additional retry.' }
            $state.child.StandardInput.WriteLine('')
            $state.child.StandardInput.Close()
            if (-not $state.child.WaitForExit(10000)) { throw 'Child exit timed out.' }
            return ''
        }.GetNewClosure()
        Assert-Equal (Wait-BridgeGuiExit -Process $child -ReadAnswer $answer) $true 'retry then exit'
        Assert-Equal $state.calls 2 'retry callback count'
    } finally { Stop-TestChild $child }

    $child = Start-TestChild
    try {
        $state = @{ calls=0; child=$child }
        $answer = {
            $state.calls++
            $state.child.Refresh()
            if ($state.child.HasExited) { throw 'Child exited before cancellation.' }
            return 'CANCEL'
        }.GetNewClosure()
        Assert-Equal (Wait-BridgeGuiExit -Process $child -ReadAnswer $answer) $false 'cancel'
        Assert-Equal $state.calls 1 'cancel callback count'
        $child.Refresh()
        Assert-Equal $child.HasExited $false 'cancel leaves child running'
    } finally { Stop-TestChild $child }
    $child = Start-TestChild
    try {
        $eofState=@{calls=0}
        $answer = {
            $eofState.calls++
            if($eofState.calls -gt 1){throw 'Closed input was unexpectedly retried'}
            return $null
        }.GetNewClosure()
        Assert-Equal (Wait-BridgeGuiExit -Process $child -ReadAnswer $answer) $false 'EOF cancels wait'
        $child.Refresh()
        Assert-Equal $child.HasExited $false 'EOF leaves child untouched'
    } finally { Stop-TestChild $child }
    Write-Host 'Bridge capture tests passed.'
} finally {
    Remove-Item -LiteralPath $script:workspace -Recurse -Force
}
