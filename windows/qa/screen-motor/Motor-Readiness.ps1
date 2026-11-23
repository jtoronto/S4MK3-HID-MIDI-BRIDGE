#Requires -Version 5.1

function Read-BridgeMotorReadiness {
    param([Parameter(Mandatory=$true)][string]$Prepared)
    $path=Join-Path $Prepared 'capture.readiness.json'
    if(-not (Test-Path -LiteralPath $path -PathType Leaf)){return $null}
    $sharing=[IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete
    $reader=[IO.StreamReader]::new([IO.File]::Open($path,[IO.FileMode]::Open,[IO.FileAccess]::Read,$sharing))
    try{$status=$reader.ReadToEnd()|ConvertFrom-Json}finally{$reader.Dispose()}
    $config=Get-Content -LiteralPath (Join-Path $Prepared 'diagnostic-config.json') -Raw|ConvertFrom-Json
    if($status.version -ne 1 -or $status.case -cne $config.case -or
        [string]::IsNullOrWhiteSpace($status.launch_id)){
        throw 'Invalid diagnostic readiness identity.'
    }
    if($status.state -in @('stopped','failed')){throw "Bridge became $($status.state) before the observation."}
    if($status.receiver_ready){
        $metadata=Get-Content -LiteralPath (Join-Path $Prepared 'capture.metadata.json') -Raw|ConvertFrom-Json
        if($status.pid -le 0 -or $status.pid -ne $metadata.pid){throw 'Readiness belongs to a different worker.'}
        $null=Get-Process -Id $status.pid -ErrorAction Stop
    }
    if($status.ready){
        $right=$status.right
        if(-not $status.receiver_ready -or -not $status.case_ready -or $status.state -cne 'ready' -or
            $null -eq $right -or $right.deck -ne 2 -or -not $right.tt_enabled -or
            -not $right.authorized -or $right.loaded -ne $true -or $right.playing -ne $true -or
            $right.internal -ne $true -or $right.thru -eq $true){
            throw 'Motor readiness lacks confirmed Deck B authorization.'
        }
    }
    return $status
}

function Wait-BridgeMotorReadiness {
    param(
        [Parameter(Mandatory=$true)][string]$Prepared,
        [Parameter(Mandatory=$true)]$Signal,
        [switch]$Authorization
    )
    $deadline=[Diagnostics.Stopwatch]::StartNew()
    $last='No current worker readiness receipt.'
    while($true){
        $status=Read-BridgeMotorReadiness $Prepared
        if($null -ne $status){
            $connected=$status.receiver_ready -and $status.case_ready -and $status.state -ceq 'ready'
            if(-not $Authorization -and $connected){return $status}
            $right=$status.right
            if($Authorization -and $connected -and $status.ready){return $status}
            $last=if($null -eq $right){'Waiting for actual right Deck B motor-gate feedback.'}else{
                "Deck=$($right.deck), TT=$($right.tt_enabled), Loaded=$($right.loaded), Playing=$($right.playing), Internal=$($right.internal), authorized=$($right.authorized)."
            }
        }
        $remaining=30000-[int]$deadline.ElapsedMilliseconds
        if($remaining -le 0 -or -not $Signal.Wait($remaining)){throw "Motor test readiness timed out. $last"}
    }
}
