#Requires -Version 5.1
# All native commands and the independent deadline retain one unique instance.
$script:WriterEtwOwned = @{}
$script:WriterEtwExecutable = Join-Path $env:SystemRoot 'System32\wpr.exe'
function New-WriterEtwCommand([string[]]$Arguments) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $script:WriterEtwExecutable
    $info.Arguments = ($Arguments | ForEach-Object { '"' + $_ + '"' }) -join ' '
    return $info
}
function Invoke-WriterEtwWpr([string[]]$Arguments) {
    return [WriterTraceDeadline]::Run((New-WriterEtwCommand $Arguments), 5000)
}
function Start-WriterEtw {
    param([string]$Profile,
        [ValidatePattern('^S4Writer[a-f0-9]{32}$')][string]$Instance,
        [string]$Output)
    if ($script:WriterEtwOwned.ContainsKey($Instance)) { throw 'Instance already owned.' }
    $profilePath = (Resolve-Path -LiteralPath $Profile).ProviderPath
    $etl = [IO.Path]::GetFullPath($Output)
    if (-not (Test-Path -LiteralPath (Split-Path -Parent $etl) -PathType Container)) { throw 'ETL parent must exist.' }
    if ((Test-Path -LiteralPath $etl) -or (Test-Path -LiteralPath ($etl+'.json'))) { throw 'ETW output already exists.' }
    $commands = [WriterTraceCommands]::new()
    $commands.Stop = New-WriterEtwCommand @('-stop',$etl,'-skipPdbGen','-instancename',$Instance)
    $commands.Cancel = New-WriterEtwCommand @('-cancel','-instancename',$Instance)
    $deadline = [WriterTraceDeadline]::new($commands,[TimeSpan]::FromSeconds(180))
    # Reserve our freshly generated name before start: a timed-out start may
    # still have created this instance, and must receive scoped cleanup.
    $script:WriterEtwOwned[$Instance] = [pscustomobject]@{
        Deadline=$deadline; Output=$etl; Profile=$profilePath; Started=[DateTime]::UtcNow.ToString('o')
        StartAccepted=$false; StartResult=$null
    }
    try {
        $result = Invoke-WriterEtwWpr @('-start',($profilePath+'!UsbWriter'),'-instancename',$Instance)
        $script:WriterEtwOwned[$Instance].StartResult=$result
        if ($result.ExitCode -ne 0) { throw "WPR start failed: $($result.ExitCode) $($result.Stderr)" }
        $script:WriterEtwOwned[$Instance].StartAccepted=$true
        return [pscustomobject]@{InstanceName=$Instance; Deadline=$deadline}
    } catch {
        $original = $_.Exception.Message
        try { Stop-WriterEtw -Instance $Instance -Output $etl | Out-Null }
        catch { throw "$original; cleanup: $($_.Exception.Message)" }
        throw $original
    }
}
function Stop-WriterEtw {
    param([string]$Instance,[string]$Output)
    if (-not $script:WriterEtwOwned.ContainsKey($Instance)) { throw 'Refusing foreign ETW instance.' }
    $owned = $script:WriterEtwOwned[$Instance]
    if ([IO.Path]::GetFullPath($Output) -cne $owned.Output) { throw 'Owned ETW output differs.' }
    $failure = $null
    try { $owned.Deadline.Stop() } catch { $failure = $_.Exception.Message }
    $complete = $owned.StartAccepted -and $null -eq $failure -and -not $owned.Deadline.DeadlineReached -and
        (Test-Path -LiteralPath $owned.Output -PathType Leaf) -and (Get-Item -LiteralPath $owned.Output).Length -gt 0
    $record = [ordered]@{instance=$Instance; output=$owned.Output; profile=$owned.Profile
        started_utc=$owned.Started; ended_utc=[DateTime]::UtcNow.ToString('o'); complete=$complete
        deadline_reached=$owned.Deadline.DeadlineReached; failure=$failure
        start=$owned.StartResult; stop=$owned.Deadline.Result; cancel=$owned.Deadline.CancelResult}
    try {
        [IO.File]::WriteAllText(($owned.Output+'.json'),($record|ConvertTo-Json -Depth 6),[Text.UTF8Encoding]::new($false))
    } finally {
        try { $owned.Deadline.Dispose() } catch { if ($null -eq $failure) { $failure=$_.Exception.Message } }
        $script:WriterEtwOwned.Remove($Instance)
    }
    if (-not $complete -or $null -ne $failure) { throw "ETW incomplete; receipt: $($owned.Output).json; $failure" }
    return [pscustomobject]@{Complete=$true; EtlPath=$owned.Output}
}
