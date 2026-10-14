param([string]$EmulatorSerial='emulator-5580',[switch]$SkipBuild)
$ErrorActionPreference='Stop'
if($EmulatorSerial -notmatch '^emulator-\d+$'){throw 'The isolated HTTPS proof accepts an emulator only'}
. (Join-Path $PSScriptRoot 'Use-LocalToolchain.ps1')
New-Item -ItemType Directory -Path (Join-Path $TogetherRoot 'work') -Force | Out-Null
. (Join-Path $PSScriptRoot 'NativeProofLock.ps1')
. (Join-Path $PSScriptRoot 'AndroidProofBuild.ps1')
. (Join-Path $PSScriptRoot 'AndroidProofReady.ps1')
$TogetherOwnershipLock=Enter-NativeProofLock -Path (Join-Path $TogetherRoot 'work/native-proof-owner.lock')
$TogetherHttpsReceipt=$null
$TogetherHttpsReceiptPath=Join-Path $TogetherRoot 'work/https-proof-last-run.json'
try {
# Enter-NativeProofLock verifies exact runner identity and uncertain metadata
# before acquiring its exclusive handle; do not use a start-time tolerance here.
if($SkipBuild){$null=Assert-AndroidProofBuild -Root $TogetherRoot}
$TogetherHttpsReceipt=[ordered]@{run_id=[guid]::NewGuid().ToString();status='running';phase='windows_native';started_utc=[datetime]::UtcNow.ToString('o');completed_utc=$null;validation_scope='pc_emulator_public_https_fixture';physical_release_gate_satisfied=$false;instrumentation_verification=$null}
$TogetherHttpsReceipt | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $TogetherHttpsReceiptPath
Push-Location $TogetherRoot
try {
    & cargo run -p together-player-mpv --example https-proof -- (Join-Path $TogetherRoot 'native\vendor\windows\libmpv-2.dll') (Join-Path $TogetherRoot 'work\media\clock-test.mp4')
    if($LASTEXITCODE -ne 0){throw 'Windows native HTTPS proof failed'}
} finally {Pop-Location}
$TogetherHttpsReceipt.phase='android_setup'
$TogetherHttpsReceipt | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $TogetherHttpsReceiptPath
if(!$SkipBuild) {
    $TogetherBeforeAndroid=Get-AndroidProofSources -Root $TogetherRoot
    Push-Location (Join-Path $TogetherRoot 'apps\android')
    try {& .\gradlew.bat --no-daemon :app:assembleDebug :app:assembleDebugAndroidTest;if($LASTEXITCODE -ne 0){throw 'Android HTTPS proof build failed'}} finally {Pop-Location}
    Write-AndroidProofBuild -Root $TogetherRoot -Before $TogetherBeforeAndroid
}
$TogetherHttpsReceipt.android_build=Assert-AndroidProofBuild -Root $TogetherRoot
$TogetherAdb=Join-Path $env:ANDROID_HOME 'platform-tools\adb.exe'
Wait-AndroidProofReady -Adb $TogetherAdb -Serial $EmulatorSerial
& $TogetherAdb -s $EmulatorSerial install -r (Join-Path $TogetherRoot 'apps\android\app\build\outputs\apk\debug\app-debug.apk')
if($LASTEXITCODE -ne 0){throw 'HTTPS proof APK install failed'}
& $TogetherAdb -s $EmulatorSerial install -r (Join-Path $TogetherRoot 'apps\android\app\build\outputs\apk\androidTest\debug\app-debug-androidTest.apk')
if($LASTEXITCODE -ne 0){throw 'HTTPS instrumentation install failed'}
$TogetherHttpsReceipt.phase='android_native'
$TogetherHttpsReceipt | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $TogetherHttpsReceiptPath
$TogetherResult=& $TogetherAdb -s $EmulatorSerial shell am instrument -r -w -e class app.together.v2.NativeHttpsTest app.together.v2.test/app.together.v2.MutedTestRunner
$TogetherInstrumentationExit=$LASTEXITCODE
$TogetherResult | Set-Content (Join-Path $TogetherRoot 'work\https-proof-result.txt')
if($TogetherInstrumentationExit -ne 0){throw 'HTTPS instrumentation command failed'}
$TogetherVerification=& python -B (Join-Path $PSScriptRoot 'verify_instrumentation.py') (Join-Path $TogetherRoot 'work/https-proof-result.txt') --expected-tests 2 --expected-identities (Join-Path $PSScriptRoot 'https-proof-tests.json')
if($LASTEXITCODE -ne 0){throw 'HTTPS instrumentation status verification failed'}
$TogetherVerification | Set-Content (Join-Path $TogetherRoot 'work/https-proof-verification.json')
$TogetherHttpsReceipt.instrumentation_verification=$TogetherVerification | ConvertFrom-Json
$TogetherHttpsReceipt.status='passed'
$TogetherHttpsReceipt.phase='complete'
Write-Output 'Both intended Android HTTPS/source replacement tests executed successfully; no ignored or assumption-skipped tests.'
} catch {
    if ($TogetherHttpsReceipt) {$TogetherHttpsReceipt.status='failed'}
    throw
} finally {
    try {
        if ($TogetherHttpsReceipt) {
            $TogetherHttpsReceipt.completed_utc=[datetime]::UtcNow.ToString('o')
            $TogetherHttpsReceipt | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $TogetherHttpsReceiptPath
        }
    } finally {$TogetherOwnershipLock.Dispose()}
}
