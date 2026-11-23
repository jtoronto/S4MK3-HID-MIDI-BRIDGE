#Requires -Version 5.1

function Wait-BridgeGuiExit {
    param(
        [Parameter(Mandatory=$true)][Diagnostics.Process]$Process,
        [Parameter(Mandatory=$true)][scriptblock]$ReadAnswer
    )
    while ($true) {
        $Process.Refresh()
        if ($Process.HasExited) { return $true }
        $answer = & $ReadAnswer 'GUI is still running. Stop and quit it normally, then press Enter to retry, or type CANCEL'
        if ($null -eq $answer -or $answer -ceq 'CANCEL') { return $false }
    }
}

function Test-BridgeCapture {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory=$true)][string]$Prepared,
        [Parameter(Mandatory=$true)][ValidateSet('A','B','C','D','N','N-off')][string]$Case
    )
    $expected = @{ A='off'; B='acquired'; C='static'; D='updates'; N='normal'; 'N-off'='normal' }[$Case]
    $path = Join-Path $Prepared 'capture.jsonl'
    $reasons = [Collections.Generic.List[string]]::new()
    $header = $null; $footer = $null
    $recordCount = 0; $readyCount = 0; $connectCount = 0; $publishCount = 0
    $failedPublishCount = 0; $failedReleaseCount = 0; $traceSha256 = $null
    if ($Case -eq 'N-off') {
        $reasons.Add('Capture disabled: readiness, loss and traffic cannot be validated after Stop.')
    } elseif (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        $reasons.Add('Capture missing after Stop.')
    } else {
        $traceSha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash
        try {
            foreach ($line in [IO.File]::ReadLines($path)) {
                $row = $line | ConvertFrom-Json
                $recordCount++
                switch ($row.kind) {
                    'header' { $header = $row }
                    'footer' { $footer = $row }
                    'screen_case_ready' { $readyCount++ }
                    'screen_connect_begin' { $connectCount++ }
                    'screen_publish_end' { $publishCount++; if ($row.values[2] -ne 1) { $failedPublishCount++ } }
                    'screen_release_end' { if ($row.values[2] -ne 1) { $failedReleaseCount++ } }
                    'lifecycle' { if ($row.values[0] -eq 100) { $reasons.Add('Captured screen generation failed.') } }
                }
            }
            if ($null -eq $header -or $null -eq $footer) { $reasons.Add('Capture header or footer missing.') }
            else {
                if ($header.case -cne $expected -or $footer.incomplete -or -not $footer.runtime_ok -or
                    $null -eq $footer.first_stop_us) { $reasons.Add('Capture case, completion, runtime or Stop receipt invalid.') }
                if ($null -eq $header.utc_unix_us -or $header.utc_unix_us -le 0) { $reasons.Add('Capture UTC anchor missing.') }
                if (@($footer.roles).Count -ne 3) { $reasons.Add('Capture producer summaries missing.') }
                foreach ($role in $footer.roles) {
                    if ($null -eq $role.losses -or $role.losses -ne 0) { $reasons.Add("Capture losses or missing producer: $($role.role)") }
                }
            }
            if ($readyCount -lt 1) { $reasons.Add('No captured case-ready receipt.') }
            if ($failedPublishCount -gt 0) { $reasons.Add('A captured frame publication failed.') }
            if ($failedReleaseCount -gt 0) { $reasons.Add('Screen release was unconfirmed.') }
            if ($Case -eq 'A' -and ($connectCount -ne 0 -or $publishCount -ne 0)) { $reasons.Add('Off case acquired or published.') }
            if ($Case -eq 'B' -and ($connectCount -lt 1 -or $publishCount -ne 0)) { $reasons.Add('Acquired case did not match its expected traffic.') }
            if ($Case -eq 'C' -and $publishCount -ne 1) { $reasons.Add('Static case did not publish exactly one pair.') }
            if ($Case -eq 'D' -and $publishCount -lt 2) { $reasons.Add('Updates case did not publish multiple pairs.') }
        } catch {
            $reasons.Add("Capture validation error: $($_.Exception.Message)")
        }
    }
    [pscustomobject]@{
        complete=($reasons.Count -eq 0); incompleteReasons=@($reasons.ToArray())
        header=$header; footer=$footer; recordCount=$recordCount; readyCount=$readyCount
        connectCount=$connectCount; publishCount=$publishCount
        failedPublishCount=$failedPublishCount; failedReleaseCount=$failedReleaseCount
        traceSha256=$traceSha256
    }
}
