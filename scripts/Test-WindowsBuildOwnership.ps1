$ErrorActionPreference='Stop'
$TogetherFixture=Join-Path ([IO.Path]::GetTempPath()) ('together-windows-build-entry-'+[guid]::NewGuid())
$TogetherScriptDirectory=Join-Path $TogetherFixture 'scripts'
$TogetherWorkDirectory=Join-Path $TogetherFixture 'work'
$TogetherFiles=[Collections.Generic.List[string]]::new()
try {
    [IO.Directory]::CreateDirectory($TogetherScriptDirectory) | Out-Null
    [IO.Directory]::CreateDirectory($TogetherWorkDirectory) | Out-Null
    foreach($TogetherName in @('Build-WindowsProof.ps1','Use-LocalToolchain.ps1','NativeProofLock.ps1','Get-NativeProofOwnership.ps1','WindowsProofBuild.ps1')) {
        $TogetherTarget=Join-Path $TogetherScriptDirectory $TogetherName
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot $TogetherName) -Destination $TogetherTarget
        $TogetherFiles.Add($TogetherTarget)
    }
    $TogetherInvoke=Join-Path $TogetherScriptDirectory 'invoke-build.ps1'
    [IO.File]::WriteAllText($TogetherInvoke, '$ErrorActionPreference="Stop";try {& (Join-Path $PSScriptRoot "Build-WindowsProof.ps1")} catch {Write-Output $_.Exception.Message;exit 7}')
    $TogetherFiles.Add($TogetherInvoke)
    $TogetherMarker=Join-Path $TogetherWorkDirectory 'native-proof-running.json'
    $TogetherFiles.Add($TogetherMarker)
    $TogetherStart=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o')
    foreach($TogetherMetadata in @((@{runner_pid=$PID;runner_started_utc=$TogetherStart} | ConvertTo-Json),'{"runner_pid":true,"runner_started_utc":"PRIVATE_INPUT"}')) {
        [IO.File]::WriteAllText($TogetherMarker,$TogetherMetadata)
        $TogetherOutput=@(& (Join-Path $PSHOME 'pwsh.exe') -NoProfile -NonInteractive -File $TogetherInvoke)
        if($LASTEXITCODE -ne 7 -or $TogetherOutput.Count -ne 1 -or $TogetherOutput[0] -cne 'Native proof ownership unavailable; another runner may be building or testing. No resources were changed.') {throw 'Build entry failed to reject protected resources'}
        if([IO.File]::ReadAllText($TogetherMarker) -cne $TogetherMetadata -or (Test-Path -LiteralPath (Join-Path $TogetherWorkDirectory 'native-proof-owner.lock')) -or (Test-Path -LiteralPath (Join-Path $TogetherFixture 'apps'))) {throw 'Blocked build changed protected resources'}
    }
    'Standalone Windows proof build checks passed: live and uncertain marker rejected before compilation, no lock/artifact/marker changes.'
} finally {
    foreach($TogetherPath in $TogetherFiles){if(Test-Path -LiteralPath $TogetherPath){Remove-Item -LiteralPath $TogetherPath}}
    foreach($TogetherPath in @($TogetherScriptDirectory,$TogetherWorkDirectory,$TogetherFixture)) {if(Test-Path -LiteralPath $TogetherPath){Remove-Item -LiteralPath $TogetherPath}}
}

