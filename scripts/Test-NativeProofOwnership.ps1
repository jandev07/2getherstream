$ErrorActionPreference='Stop'
$TogetherTestDirectory=Join-Path ([IO.Path]::GetTempPath()) ('together-ownership-'+[guid]::NewGuid())
New-Item -ItemType Directory -Path $TogetherTestDirectory | Out-Null
$TogetherScript=Join-Path $PSScriptRoot 'Get-NativeProofOwnership.ps1'
$TogetherMarkerPath=Join-Path $TogetherTestDirectory 'native-proof-running.json'
$TogetherLockPath=Join-Path $TogetherTestDirectory 'native-proof-owner.lock'
$TogetherActualStart=(Get-Process -Id $PID).StartTime.ToUniversalTime()
function Assert-Ownership($identity,$lock,$preserve) {
    $TogetherObservation=& $TogetherScript -WorkDirectory $TogetherTestDirectory
    if($TogetherObservation.runner_identity -ne $identity -or $TogetherObservation.owner_lock -ne $lock -or
        $TogetherObservation.resources_must_be_preserved -ne $preserve -or $TogetherObservation.duplicate_run_authorized) {
        throw 'Native ownership classification failed'
    }
}
function Write-Ownership($runner,$start) {
    @{runner_pid=$runner;runner_started_utc=$start} | ConvertTo-Json | Set-Content -LiteralPath $TogetherMarkerPath
}
try {
    Assert-Ownership 'absent' 'absent' $false
    if(Test-Path -LiteralPath $TogetherLockPath){throw 'Read-only inventory created a lock'}
    # The build can own resources before it publishes its running marker.
    $TogetherOwner=[IO.FileStream]::new($TogetherLockPath,[IO.FileMode]::CreateNew,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
    try {Assert-Ownership 'absent' 'unavailable' $true} finally {$TogetherOwner.Dispose()}
    Assert-Ownership 'absent' 'free' $false
    foreach($start in @($TogetherActualStart.ToString('o'),([datetimeoffset]$TogetherActualStart).ToOffset([timespan]::FromHours(5.5)).ToString('o'))) {
        Write-Ownership $PID $start
        Assert-Ownership 'matching' 'free' $true
    }
    # A reused PID with a different start instant is not the saved owner, even
    # within the former two-second tolerance. Inventory never stops this process.
    Write-Ownership $PID $TogetherActualStart.AddMilliseconds(1).ToString('o')
    Assert-Ownership 'different_process' 'free' $false
    foreach($invalid in @('[]','{"runner_pid":true,"runner_started_utc":"2026-10-10T00:00:00Z"}',
        '{"runner_pid":1.5,"runner_started_utc":"2026-10-10T00:00:00Z"}',
        '{"runner_pid":"PRIVATE_INPUT","runner_started_utc":"PRIVATE_INPUT"}',('x'*4097))) {
        [IO.File]::WriteAllText($TogetherMarkerPath,$invalid)
        Assert-Ownership 'unverified' 'free' $true
        if([IO.File]::ReadAllText($TogetherMarkerPath) -cne $invalid){throw 'Inventory modified ownership metadata'}
    }
    Write-Ownership $PID 'PRIVATE_INPUT'
    Assert-Ownership 'unverified' 'free' $true
    Write-Ownership $PID $null
    Assert-Ownership 'unverified' 'free' $true
    Write-Ownership $PID $TogetherActualStart.ToString('s')
    Assert-Ownership 'unverified' 'free' $true
    Write-Ownership ([int]::MaxValue) $TogetherActualStart.ToString('o')
    Assert-Ownership 'exited' 'free' $false
    . (Join-Path $PSScriptRoot 'NativeProofLock.ps1')
    foreach($TogetherEntryCase in @(
        @{runner=$PID;start=$TogetherActualStart.ToString('o')},
        @{runner=$PID;start='PRIVATE_INPUT'},
        @{runner=$PID;start=$null}
    )) {
        Write-Ownership $TogetherEntryCase.runner $TogetherEntryCase.start
        $TogetherOriginalMarker=[IO.File]::ReadAllText($TogetherMarkerPath)
        $TogetherUnexpected=$null
        $TogetherRejected=$false
        try {$TogetherUnexpected=Enter-NativeProofLock -Path $TogetherLockPath}
        catch {
            $TogetherRejected=$true
            if($_.Exception.Message -cne 'Native proof ownership unavailable; another runner may be building or testing. No resources were changed.') {throw 'Entry error was not fixed'}
        } finally {if($TogetherUnexpected){$TogetherUnexpected.Dispose()}}
        if(!$TogetherRejected -or [IO.File]::ReadAllText($TogetherMarkerPath) -cne $TogetherOriginalMarker) {throw 'Runner entry did not preserve protected metadata'}
    }
    foreach($TogetherEntryCase in @(
        @{runner=$PID;start=$TogetherActualStart.AddMilliseconds(1).ToString('o')},
        @{runner=[int]::MaxValue;start=$TogetherActualStart.ToString('o')}
    )) {
        Write-Ownership $TogetherEntryCase.runner $TogetherEntryCase.start
        $TogetherEntry=Enter-NativeProofLock -Path $TogetherLockPath
        try {if(!(& $TogetherScript -WorkDirectory $TogetherTestDirectory).resources_must_be_preserved){throw 'Entered runner did not own lock'}}
        finally {$TogetherEntry.Dispose()}
    }
    Write-Output 'Native ownership checks passed: exact UTC identity, reused PID, build lock, malformed bounded metadata, read-only inventory.'
} finally {
    # Exact individually created files only; no recursive cleanup.
    if(Test-Path -LiteralPath $TogetherMarkerPath){Remove-Item -LiteralPath $TogetherMarkerPath}
    if(Test-Path -LiteralPath $TogetherLockPath){Remove-Item -LiteralPath $TogetherLockPath}
    Remove-Item -LiteralPath $TogetherTestDirectory
}
