param([string]$EmulatorSerial='emulator-5580',[switch]$SkipBuild,[ValidateRange(0,1600)][int]$ControlRttMs=0,[ValidateRange(0,400)][int]$JitterMs=0,[ValidateRange(60,7200)][int]$SteadySeconds=60)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Assert-NativeProofResult.ps1')
if($EmulatorSerial -notmatch '^emulator-\d+$') {throw 'This isolated proof runner accepts an emulator only.'}
. (Join-Path $PSScriptRoot 'Use-LocalToolchain.ps1')
$TogetherAdb=Join-Path $env:ANDROID_HOME 'platform-tools\adb.exe'
$TogetherExe=Join-Path $TogetherRoot 'target\debug\together2-windows.exe'
$TogetherWork=Join-Path $TogetherRoot 'work'
New-Item -ItemType Directory -Force -Path $TogetherWork | Out-Null
. (Join-Path $PSScriptRoot 'NativeProofLock.ps1')
. (Join-Path $PSScriptRoot 'AndroidProofBuild.ps1')
. (Join-Path $PSScriptRoot 'WindowsProofBuild.ps1')
. (Join-Path $PSScriptRoot 'AndroidProofReady.ps1')
$TogetherOwnershipLock=Enter-NativeProofLock -Path (Join-Path $TogetherWork 'native-proof-owner.lock')
try {
$TogetherRunningPath=Join-Path $TogetherWork 'native-proof-running.json'
# Entry already checks bounded metadata and exact UTC process identity before
# and after exclusive lock acquisition. Do not apply a start-time tolerance.
if(!(Test-Path -LiteralPath (Join-Path $TogetherRoot 'apps\android\app\src\main\assets\clock-test.mp4'))) {
    throw 'Run Prepare-PlayerProof.ps1 first to generate the synthetic video.'
}
if(Get-NetTCPConnection -LocalPort 47820 -State Listen -ErrorAction SilentlyContinue) {
    throw 'Port 47820 is occupied. Close the previous 2.0 proof host yourself; no existing application will be stopped.'
}
if(!$SkipBuild) {
    $TogetherBeforeWindows=Get-WindowsProofSources -Root $TogetherRoot
    $TogetherBeforeAndroid=Get-AndroidProofSources -Root $TogetherRoot
    Push-Location (Join-Path $TogetherRoot 'apps\windows')
    try {
        & npm.cmd ci; if($LASTEXITCODE -ne 0) {throw 'Frontend install failed'}
        & npm.cmd run check; if($LASTEXITCODE -ne 0) {throw 'Frontend check failed'}
        & npm.cmd run build; if($LASTEXITCODE -ne 0) {throw 'Frontend build failed'}
    } finally {Pop-Location}
    $TogetherBeforeFrontend=Get-WindowsProofFrontend -Root $TogetherRoot
    Push-Location $TogetherRoot
    try {
        & cargo build --manifest-path apps/windows/src-tauri/Cargo.toml --features custom-protocol
        if($LASTEXITCODE -ne 0) {throw 'Windows proof build failed'}
    } finally {Pop-Location}
    Write-WindowsProofBuild -Root $TogetherRoot -BeforeSources $TogetherBeforeWindows -BeforeFrontend $TogetherBeforeFrontend
    Push-Location (Join-Path $TogetherRoot 'apps\android')
    try {
        & .\gradlew.bat --no-daemon :app:testDebugUnitTest :app:assembleDebug :app:assembleDebugAndroidTest
        if($LASTEXITCODE -ne 0) {throw 'Android proof build failed'}
    } finally {Pop-Location}
    Write-AndroidProofBuild -Root $TogetherRoot -Before $TogetherBeforeAndroid
}
$TogetherWindowsBuild=Assert-WindowsProofBuild -Root $TogetherRoot
$TogetherAndroidBuild=Assert-AndroidProofBuild -Root $TogetherRoot
Wait-AndroidProofReady -Adb $TogetherAdb -Serial $EmulatorSerial
$TogetherPreviousToken=$env:TOGETHER_PHONE_TOKEN
$TogetherPreviousProof=$env:TOGETHER_SESSION_PROOF
$TogetherPreviousRunId=$env:TOGETHER_PROOF_RUN_ID
$TogetherProcess=$null
$TogetherProxy=$null
$TogetherReceiptPath=Join-Path $TogetherWork 'native-proof-last-run.json'
$TogetherReceipt=[ordered]@{run_id=[guid]::NewGuid().ToString();status='running';phase='host_startup';started_utc=[datetime]::UtcNow.ToString('o');completed_utc=$null;requested_uninterrupted_seconds=$SteadySeconds;completed_uninterrupted_seconds=$null;configured_rtt_ms=$ControlRttMs;jitter_ms=$JitterMs;validation_scope='emulator_synthetic_fixture';metrics_file=$null;proof_log_schema=2;provisional_timing_target_met=$null;physical_release_gate_satisfied=$false}
$TogetherReceipt.android_build=$TogetherAndroidBuild
$TogetherReceipt.windows_build=$TogetherWindowsBuild
if(!('TogetherProofPower' -as [type])) {
    Add-Type -TypeDefinition @'
using System.Runtime.InteropServices;
public static class TogetherProofPower {
    [DllImport("kernel32.dll")] public static extern uint SetThreadExecutionState(uint state);
}
'@
}
try {
    $TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath
    [TogetherProofPower]::SetThreadExecutionState([uint32]2147483649) | Out-Null
    @{runner_pid=$PID;runner_started_utc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o');started_utc=[datetime]::UtcNow.ToString('o');steady_seconds=$SteadySeconds;configured_rtt_ms=$ControlRttMs;jitter_ms=$JitterMs} | ConvertTo-Json | Set-Content -LiteralPath $TogetherRunningPath
    $env:TOGETHER_PHONE_TOKEN=[guid]::NewGuid().ToString()+[guid]::NewGuid().ToString()
    $env:TOGETHER_SESSION_PROOF='1'
    $env:TOGETHER_PROOF_RUN_ID=$TogetherReceipt.run_id
    $TogetherCodePath=Join-Path $TogetherWork 'pairing-code-proof.txt'
    if(Test-Path -LiteralPath $TogetherCodePath){Remove-Item -LiteralPath $TogetherCodePath}
    # This runner owns this process only. Credentials are never command-line arguments.
    $TogetherProcess=Start-Process -FilePath $TogetherExe -WorkingDirectory $TogetherRoot -WindowStyle Hidden -PassThru
    $TogetherReady=$false
    for($i=0;$i -lt 30;$i++) {
        if($TogetherProcess.HasExited) {throw 'Proof host exited during startup'}
        try {if((Invoke-WebRequest -Uri 'http://127.0.0.1:47820/health' -TimeoutSec 1).Content -eq 'together-control-v1') {$TogetherReady=$true;break}} catch{}
        Start-Sleep -Milliseconds 200
    }
    if(!$TogetherReady) {throw 'Proof host did not start'}
    for($i=0;$i -lt 30 -and !(Test-Path -LiteralPath $TogetherCodePath);$i++){Start-Sleep -Milliseconds 100}
    if(!(Test-Path -LiteralPath $TogetherCodePath)){throw 'Initial pairing proof nonce was not prepared'}
    $TogetherPairing=@{url='ws://127.0.0.1:47820/session';token=$env:TOGETHER_PHONE_TOKEN;code=(Get-Content -LiteralPath $TogetherCodePath -Raw)} | ConvertTo-Json -Compress
    $TogetherControlPort=47820
    if($ControlRttMs -gt 0 -or $JitterMs -gt 0) {
        if(Get-NetTCPConnection -LocalPort 47821 -State Listen -ErrorAction SilentlyContinue) {throw 'Test proxy port is occupied'}
        Push-Location $TogetherRoot
        try {& cargo build -p together-session-server --example delayed-proxy;if($LASTEXITCODE -ne 0){throw 'Control test proxy build failed'}} finally {Pop-Location}
        $TogetherProxy=Start-Process -FilePath (Join-Path $TogetherRoot 'target\debug\examples\delayed-proxy.exe') -ArgumentList @($ControlRttMs,$JitterMs) -WindowStyle Hidden -PassThru
        for($i=0;$i -lt 30;$i++) {
            if($TogetherProxy.HasExited){throw 'Control test proxy exited'}
            if(Get-NetTCPConnection -LocalPort 47821 -State Listen -ErrorAction SilentlyContinue){break}
            Start-Sleep -Milliseconds 100
        }
        if(!(Get-NetTCPConnection -LocalPort 47821 -State Listen -ErrorAction SilentlyContinue)){throw 'Control test proxy did not start'}
        $TogetherControlPort=47821
    }
    $TogetherReceipt.phase='emulator_setup';$TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath
    & $TogetherAdb -s $EmulatorSerial reverse tcp:47820 "tcp:$TogetherControlPort" | Out-Null
    if($LASTEXITCODE -ne 0) {throw 'Emulator reverse connection failed'}
    & $TogetherAdb -s $EmulatorSerial install -r (Join-Path $TogetherRoot 'apps\android\app\build\outputs\apk\debug\app-debug.apk')
    if($LASTEXITCODE -ne 0) {throw 'Proof APK installation failed'}
    & $TogetherAdb -s $EmulatorSerial install -r (Join-Path $TogetherRoot 'apps\android\app\build\outputs\apk\androidTest\debug\app-debug-androidTest.apk')
    if($LASTEXITCODE -ne 0) {throw 'Instrumentation APK installation failed'}
    & $TogetherAdb -s $EmulatorSerial shell run-as app.together.v2 mkdir -p files
    $TogetherPairing | & $TogetherAdb -s $EmulatorSerial shell run-as app.together.v2 sh -c '"cat > files/proof-pairing.json"'
    if($LASTEXITCODE -ne 0) {throw 'Private debug pairing write failed'}
    & $TogetherAdb -s $EmulatorSerial shell run-as app.together.v2 test -s files/proof-pairing.json
    if($LASTEXITCODE -ne 0) {throw 'Private pairing file missing; refuse to run a skipped proof'}
    $TogetherReceipt.expected_native_tests=@(Get-Content (Join-Path $PSScriptRoot 'native-proof-tests.json') -Raw | ConvertFrom-Json).Count
    $TogetherReceipt.phase='native_tests';$TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath
    $TogetherResult=& $TogetherAdb -s $EmulatorSerial shell am instrument -r -w -e steadySeconds $SteadySeconds -e proofRunId $TogetherReceipt.run_id -e class app.together.v2.PairingStoreTest,app.together.v2.PairingConnectionTest,app.together.v2.NativeTlsTest,app.together.v2.SessionIdentityTest,app.together.v2.NativePlayerTest,app.together.v2.SessionPlaybackTest app.together.v2.test/app.together.v2.MutedTestRunner
    $TogetherInstrumentationExit=$LASTEXITCODE
    $TogetherResult | Set-Content -LiteralPath (Join-Path $TogetherWork 'native-proof-result.txt')
    if($TogetherInstrumentationExit -ne 0) {throw 'Native instrumentation command failed'}
    # Raw status packets distinguish executed tests from ignored/assumption skips.
    # The terse JUnit OK(count) summary alone cannot establish this.
    $TogetherVerification=& python -B (Join-Path $PSScriptRoot 'verify_instrumentation.py') (Join-Path $TogetherWork 'native-proof-result.txt') --expected-tests $TogetherReceipt.expected_native_tests --expected-identities (Join-Path $PSScriptRoot 'native-proof-tests.json')
    if($LASTEXITCODE -ne 0) {throw 'Native instrumentation status verification failed'}
    $TogetherReceipt.instrumentation_verification=$TogetherVerification | ConvertFrom-Json
    $TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath
    Write-Output 'All selected native tests executed successfully; no ignored or assumption-skipped tests.'
    & $TogetherAdb -s $EmulatorSerial exec-out run-as app.together.v2 cat files/phone-session-proof.jsonl | Set-Content -LiteralPath (Join-Path $TogetherWork 'phone-session-proof.jsonl')
    $TogetherReceipt.phase='journal_validation';$TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath
    # The isolated production journal must contain actual host and native records.
    # Never fall back to an older journal or treat compilation as runtime evidence.
    $TogetherJournalPath=Join-Path $TogetherWork 'native-journal-proof/session-0.jsonl'
    if(!(Test-Path -LiteralPath $TogetherJournalPath)){throw 'Native diagnostic journal missing'}
    $TogetherJournalStream=[IO.FileStream]::new($TogetherJournalPath,[IO.FileMode]::Open,[IO.FileAccess]::Read,([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
    $TogetherJournalReader=[IO.StreamReader]::new($TogetherJournalStream)
    try {$TogetherJournalText=$TogetherJournalReader.ReadToEnd()} finally {$TogetherJournalReader.Dispose()}
    $TogetherJournalEnd=$TogetherJournalText.LastIndexOf("`n")
    if($TogetherJournalEnd -lt 0){throw 'Native diagnostic journal has no complete records'}
    # The writer remains active; ignore only an unfinished final line.
    $TogetherJournal=@($TogetherJournalText.Substring(0,$TogetherJournalEnd).Split("`n",[StringSplitOptions]::RemoveEmptyEntries) | ForEach-Object {$_ | ConvertFrom-Json})
    $TogetherJournalStarted=[datetimeoffset]::Parse($TogetherReceipt.started_utc).ToUnixTimeMilliseconds()
    if(!$TogetherJournal.Count -or ($TogetherJournal | Where-Object {$_.recorded_unix_ms -lt $TogetherJournalStarted})){throw 'Native diagnostic journal is stale'}
    if(!($TogetherJournal | Where-Object {$_.record.event -eq 'session'}) -or !($TogetherJournal | Where-Object {$_.record.event -eq 'native_pc'})){throw 'Native diagnostic journal lacks authority/player observations'}
    if($TogetherJournal | Where-Object {$_.dropped_records -ne 0 -or $_.io_errors -ne 0}){throw 'Native diagnostic journal reports lost records or storage errors'}
    $TogetherReceipt.phase='drift_analysis';$TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath
    & (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -ExpectedRunId $TogetherReceipt.run_id
    $TogetherSummary=Get-Content (Join-Path $TogetherWork 'native-proof-summary.json') -Raw | ConvertFrom-Json
    # Native assertions can pass while the provisional timing target is missed.
    # Keep that result explicit in the receipt; do not reinterpret it as a gate.
    $TogetherReceipt.provisional_timing_target_met=[bool]$TogetherSummary.provisional_cross_device_p95_goal_met
    $TogetherSummary | Add-Member -NotePropertyName configured_control_rtt_ms -NotePropertyValue $ControlRttMs
    $TogetherSummary | Add-Member -NotePropertyName configured_one_way_jitter_ms -NotePropertyValue $JitterMs
    $TogetherSummary | Add-Member -NotePropertyName uninterrupted_play_seconds -NotePropertyValue $SteadySeconds
    $TogetherSummary | Add-Member -NotePropertyName started_utc -NotePropertyValue $TogetherReceipt.started_utc
    $TogetherSummary | Add-Member -NotePropertyName completed_utc -NotePropertyValue ([datetime]::UtcNow.ToString('o'))
    $TogetherSummary | Add-Member -NotePropertyName validation_scope -NotePropertyValue $TogetherReceipt.validation_scope
    $TogetherMetricsFile="native-proof-rtt-$ControlRttMs-jitter-$JitterMs.json"
    $TogetherSummary | ConvertTo-Json | Set-Content (Join-Path $TogetherWork $TogetherMetricsFile)
    $TogetherReceipt.status='passed'
    $TogetherReceipt.phase='complete'
    $TogetherReceipt.completed_uninterrupted_seconds=$SteadySeconds
    $TogetherReceipt.metrics_file=$TogetherMetricsFile
} catch {
    # No exception text: native/provider errors must not enter diagnostic receipts.
    $TogetherReceipt.status='failed'
    throw
} finally {
    [TogetherProofPower]::SetThreadExecutionState([uint32]2147483648) | Out-Null
    & $TogetherAdb -s $EmulatorSerial shell run-as app.together.v2 rm -f files/proof-pairing.json 2>$null
    & $TogetherAdb -s $EmulatorSerial reverse --remove tcp:47820 2>$null
    if($TogetherProxy -and !$TogetherProxy.HasExited) {$TogetherProxy.Kill();$TogetherProxy.WaitForExit()}
    if($TogetherProcess -and !$TogetherProcess.HasExited) {$TogetherProcess.Kill();$TogetherProcess.WaitForExit()}
    if($TogetherCodePath -and (Test-Path -LiteralPath $TogetherCodePath)){Remove-Item -LiteralPath $TogetherCodePath}
    if(Test-Path -LiteralPath $TogetherRunningPath){Remove-Item -LiteralPath $TogetherRunningPath}
    $env:TOGETHER_PHONE_TOKEN=$TogetherPreviousToken
    $env:TOGETHER_SESSION_PROOF=$TogetherPreviousProof
    $env:TOGETHER_PROOF_RUN_ID=$TogetherPreviousRunId
    $TogetherReceipt.completed_utc=[datetime]::UtcNow.ToString('o')
    $TogetherReceipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $TogetherReceiptPath
}
} finally {
    $TogetherOwnershipLock.Dispose()
}
