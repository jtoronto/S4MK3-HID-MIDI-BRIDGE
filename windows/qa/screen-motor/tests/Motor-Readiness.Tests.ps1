#Requires -Version 5.1
param([Parameter(Mandatory=$true)][string]$Root)
$ErrorActionPreference='Stop'
Set-StrictMode -Version Latest
. (Join-Path $PSScriptRoot '../Motor-Readiness.ps1')
if(Test-Path -LiteralPath $Root){throw 'Fixture root already exists.'}
$null=[IO.Directory]::CreateDirectory($Root)
function Check($Condition,[string]$Message){if(-not $Condition){throw $Message}}
function Reject([scriptblock]$Action,[string]$Message){
    $rejected=$false
    try{& $Action}catch{$rejected=$true}
    Check $rejected $Message
}
try{
    Add-Type -Path (Join-Path $PSScriptRoot '../ReadinessSignal.cs')
    @{version=1;case='off';capture=$true;capture_path=(Join-Path $Root 'capture.jsonl')}|
        ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Root 'diagnostic-config.json') -Encoding UTF8
    @{pid=$PID}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $Root 'capture.metadata.json') -Encoding UTF8
    $signal=[S4UsbCapture.ReadinessSignal]::new($Root)
    try{
        $record=@{
            version=1;launch_id=[guid]::NewGuid().ToString();pid=$PID;case='off';state='ready'
            receiver_ready=$true;case_ready=$true;ready=$true;error=$null
            right=@{deck=2;tt_enabled=$true;loaded=$true;playing=$true;internal=$true;thru=$false;authorized=$true}
        }
        $statusPath=Join-Path $Root 'capture.readiness.json'
        $record|ConvertTo-Json -Depth 4|Set-Content -LiteralPath $statusPath -Encoding UTF8
        Check ($signal.Wait(10000)) 'Created receipt did not wake the already registered reader.'
        $accepted=Wait-BridgeMotorReadiness -Prepared $Root -Signal $signal -Authorization
        Check ($accepted.launch_id -ceq $record.launch_id) 'Fresh authorized receipt was not accepted.'
        Write-Output 'CASE_PASSED authorized-current-worker'
        foreach($field in @('loaded','playing','internal','tt_enabled','authorized')){
            $saved=$record.right[$field]
            $record.right[$field]=$null
            $record|ConvertTo-Json -Depth 4|Set-Content -LiteralPath $statusPath -Encoding UTF8
            Reject {Read-BridgeMotorReadiness $Root} "Unknown $field was accepted as motor-ready."
            $record.right[$field]=$saved
        }
        $record.right.thru=$true
        $record|ConvertTo-Json -Depth 4|Set-Content -LiteralPath $statusPath -Encoding UTF8
        Reject {Read-BridgeMotorReadiness $Root} 'THRU was accepted as motor-ready.'
        $record.right.thru=$false
        $record.pid=$PID+100000
        $record|ConvertTo-Json -Depth 4|Set-Content -LiteralPath $statusPath -Encoding UTF8
        Reject {Read-BridgeMotorReadiness $Root} 'Another worker receipt was accepted.'
        $record.pid=$PID
        $record.state='stopped'
        $record.ready=$false
        $record|ConvertTo-Json -Depth 4|Set-Content -LiteralPath $statusPath -Encoding UTF8
        Reject {Read-BridgeMotorReadiness $Root} 'Stopped receipt was accepted.'
        Write-Output 'CASE_PASSED unknown-mode-wrong-worker-stopped-rejected'
    }finally{$signal.Dispose()}
    Write-Output 'SUMMARY motor readiness file contract passed'
}finally{Remove-Item -LiteralPath $Root -Recurse -Force}
