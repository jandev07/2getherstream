param([string]$EmulatorSerial='emulator-5580')
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Use-LocalToolchain.ps1')
. (Join-Path $PSScriptRoot 'NativeProofLock.ps1')
. (Join-Path $PSScriptRoot 'AndroidProofBuild.ps1')
. (Join-Path $PSScriptRoot 'AndroidProofReady.ps1')
$TogetherLock=Enter-NativeProofLock -Path (Join-Path $TogetherRoot 'work/native-proof-owner.lock')
$TogetherReceipt=$null
$TogetherReceiptPath=Join-Path $TogetherRoot 'work/update-and-logs-last-run.json'
try {
    $TogetherBuild=Assert-AndroidProofBuild -Root $TogetherRoot
    $TogetherAdb=Join-Path $env:ANDROID_HOME 'platform-tools/adb.exe'
    Wait-AndroidProofReady -Adb $TogetherAdb -Serial $EmulatorSerial
    $TogetherReceipt=[ordered]@{run_id=[guid]::NewGuid().ToString();status='running';phase='install';started_utc=[datetime]::UtcNow.ToString('o');completed_utc=$null;android_build=$TogetherBuild;validation_scope='emulator_update_rejection_and_log_privacy';torbox_delivery_validated=$false;physical_release_gate_satisfied=$false}
    $TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath
    foreach($TogetherApk in @('apps/android/app/build/outputs/apk/debug/app-debug.apk','apps/android/app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk')) {
        & $TogetherAdb -s $EmulatorSerial install -r (Join-Path $TogetherRoot $TogetherApk)
        if($LASTEXITCODE -ne 0){throw 'Update/log test installation failed'}
    }
    $TogetherReceipt.phase='metadata_tests'
    $TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath
    $TogetherResult=& $TogetherAdb -s $EmulatorSerial shell am instrument -r -w -e class app.together.v2.UpdateAndLogsTest app.together.v2.test/app.together.v2.MutedTestRunner
    $TogetherExit=$LASTEXITCODE
    $TogetherResult | Set-Content -LiteralPath (Join-Path $TogetherRoot 'work/update-and-logs-result.txt')
    if($TogetherExit -ne 0){throw 'Update/log instrumentation failed'}
    & python -B (Join-Path $PSScriptRoot 'verify_instrumentation.py') (Join-Path $TogetherRoot 'work/update-and-logs-result.txt') --expected-tests (@(Get-Content (Join-Path $PSScriptRoot 'update-and-logs-tests.json') -Raw | ConvertFrom-Json).Count) --expected-identities (Join-Path $PSScriptRoot 'update-and-logs-tests.json')
    if($LASTEXITCODE -ne 0){throw 'Update/log exact test verification failed'}
    $null=Assert-AndroidProofBuild -Root $TogetherRoot
    $TogetherReceipt.status='passed';$TogetherReceipt.phase='complete'
}catch {
    if($null -ne $TogetherReceipt){$TogetherReceipt.status='failed'}
    throw
}finally {
    if($null -ne $TogetherReceipt){$TogetherReceipt.completed_utc=[datetime]::UtcNow.ToString('o');$TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath}
    $TogetherLock.Dispose()
}
