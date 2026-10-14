$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'AndroidProofReady.ps1')
$TogetherOriginal=(Get-Item Function:Invoke-AndroidProofReadyCommand).ScriptBlock
try {
    $script:TogetherCalls=[Collections.Generic.List[string]]::new()
    $script:TogetherBoots=0
    function Invoke-AndroidProofReadyCommand {
        param($Adb,$Serial,$Arguments)
        $script:TogetherCalls.Add(($Arguments -join ' '))
        if($Arguments[0] -eq 'getprop'){
            $script:TogetherBoots++
            if($script:TogetherBoots -eq 1){return $null}
            return '1'
        }
        if($script:TogetherBoots -eq 2){return 'Service package: not found'}
        return 'Service package: found'
    }
    Wait-AndroidProofReady -Adb 'unused' -Serial emulator-5580 -TimeoutSeconds 5
    if(($script:TogetherCalls -join ',') -cne 'getprop sys.boot_completed,getprop sys.boot_completed,service check package,getprop sys.boot_completed,service check package'){
        throw 'Readiness did not wait for both boot and package service'
    }
    function Invoke-AndroidProofReadyCommand {param($Adb,$Serial,$Arguments);return 'Service package: not found'}
    try {Wait-AndroidProofReady -Adb 'unused' -Serial emulator-5580 -TimeoutSeconds 1;throw 'Unexpected timeout success'}
    catch {if($_.Exception.Message -cne 'Emulator is not ready for proof installation. No APK was installed.'){throw}}
    try {Wait-AndroidProofReady -Adb 'unused' -Serial 'physical-device';throw 'Unexpected physical device probe'}
    catch {if($_.Exception.Message -cne 'The isolated readiness check accepts an emulator only'){throw}}
    'Android readiness checks passed: offline/cold boot, late package service, fixed timeout and emulator-only boundary.'
}finally{Set-Item Function:Invoke-AndroidProofReadyCommand $TogetherOriginal}

# Exercise the real owned-process deadline with a harmless Python stand-in.
# Python's -s accepts our serial position as its script, ignoring trailing args.
$TogetherFixture=Join-Path ([IO.Path]::GetTempPath()) ('together-readiness-'+[guid]::NewGuid())
$TogetherPidFile=Join-Path $TogetherFixture 'owned-pid.txt'
$TogetherFake=Join-Path $TogetherFixture 'fake-adb.py'
try {
    [IO.Directory]::CreateDirectory($TogetherFixture) | Out-Null
    $TogetherQuotedPid=ConvertTo-Json -InputObject $TogetherPidFile -Compress
    [IO.File]::WriteAllText($TogetherFake,"import os,time,pathlib`npathlib.Path($TogetherQuotedPid).write_text(str(os.getpid()))`ntime.sleep(30)`n")
    $TogetherElapsed=[Diagnostics.Stopwatch]::StartNew()
    $TogetherResult=Invoke-AndroidProofReadyCommand -Adb (Get-Command python -CommandType Application | Select-Object -First 1).Source -Serial $TogetherFake -Arguments @('getprop','sys.boot_completed')
    if($null -ne $TogetherResult -or $TogetherElapsed.Elapsed.TotalSeconds -gt 8 -or !(Test-Path -LiteralPath $TogetherPidFile)){
        throw 'Owned probe timeout was not bounded'
    }
    $TogetherOwnedPid=[int][IO.File]::ReadAllText($TogetherPidFile)
    if(Get-Process -Id $TogetherOwnedPid -ErrorAction SilentlyContinue){throw 'Timed-out owned probe was left running'}
    if($null -ne (Invoke-AndroidProofReadyCommand -Adb (Join-Path $TogetherFixture 'PRIVATE_MISSING_BINARY') -Serial emulator-5580 -Arguments @('getprop','sys.boot_completed'))){
        throw 'Failed probe spawn returned readiness'
    }
    'Actual probe process checks passed: timeout reaped only its child and missing executable returned no private error.'
}finally{
    foreach($TogetherFile in @($TogetherFake,$TogetherPidFile)){if(Test-Path -LiteralPath $TogetherFile){Remove-Item -LiteralPath $TogetherFile}}
    if(Test-Path -LiteralPath $TogetherFixture){Remove-Item -LiteralPath $TogetherFixture}
}
