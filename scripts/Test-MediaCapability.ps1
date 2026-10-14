param([string]$EmulatorSerial='emulator-5580')
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Use-LocalToolchain.ps1')
. (Join-Path $PSScriptRoot 'NativeProofLock.ps1')
. (Join-Path $PSScriptRoot 'AndroidProofBuild.ps1')
. (Join-Path $PSScriptRoot 'AndroidProofReady.ps1')
$TogetherLock=Enter-NativeProofLock -Path (Join-Path $TogetherRoot 'work/native-proof-owner.lock')
$TogetherReceipt=$null
$TogetherReceiptPath=Join-Path $TogetherRoot 'work/media-capability-last-run.json'
try {
    $TogetherBuild=Assert-AndroidProofBuild -Root $TogetherRoot
    $TogetherAdb=Join-Path $env:ANDROID_HOME 'platform-tools/adb.exe'
    Wait-AndroidProofReady -Adb $TogetherAdb -Serial $EmulatorSerial
    $TogetherReceipt=[ordered]@{run_id=[guid]::NewGuid().ToString();status='running';phase='install';started_utc=[datetime]::UtcNow.ToString('o');completed_utc=$null;android_build=$TogetherBuild;validation_scope='emulator_synthetic_control_metadata';torbox_delivery_validated=$false;physical_release_gate_satisfied=$false}
    $TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath
    foreach($TogetherApk in @('apps/android/app/build/outputs/apk/debug/app-debug.apk','apps/android/app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk')) {
        & $TogetherAdb -s $EmulatorSerial install -r (Join-Path $TogetherRoot $TogetherApk)
        if($LASTEXITCODE -ne 0){throw 'Media metadata test installation failed'}
    }
    $TogetherReceipt.phase='metadata_tests'
    $TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath
    $TogetherResult=& $TogetherAdb -s $EmulatorSerial shell am instrument -r -w -e class app.together.v2.MediaCapabilityTest app.together.v2.test/app.together.v2.MutedTestRunner
    $TogetherExit=$LASTEXITCODE
    $TogetherResult | Set-Content -LiteralPath (Join-Path $TogetherRoot 'work/media-capability-result.txt')
    if($TogetherExit -ne 0){throw 'Media metadata instrumentation failed'}
    & python -B (Join-Path $PSScriptRoot 'verify_instrumentation.py') (Join-Path $TogetherRoot 'work/media-capability-result.txt') --expected-tests 4 --expected-identities (Join-Path $PSScriptRoot 'media-capability-tests.json')
    if($LASTEXITCODE -ne 0){throw 'Media metadata exact test verification failed'}
    $null=Assert-AndroidProofBuild -Root $TogetherRoot
    $TogetherReceipt.status='passed';$TogetherReceipt.phase='complete'
}catch {
    if($null -ne $TogetherReceipt){$TogetherReceipt.status='failed'}
    throw
}finally {
    if($null -ne $TogetherReceipt){$TogetherReceipt.completed_utc=[datetime]::UtcNow.ToString('o');$TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath}
    $TogetherLock.Dispose()
}
