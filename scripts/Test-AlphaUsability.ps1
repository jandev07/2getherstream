param([Parameter(Mandatory)][string]$CandidateId)
$ErrorActionPreference='Stop'
$proofEnvironment=@{}
foreach($name in @('ANDROID_AVD_HOME','TOGETHER_SESSION_PROOF','TOGETHER_HIDDEN_ALPHA_PROOF','TOGETHER_PROOF_RUN_ID','TOGETHER_PHONE_TOKEN')) {
 $proofEnvironment[$name]=[Environment]::GetEnvironmentVariable($name,'Process')
}
. "$PSScriptRoot/Use-LocalToolchain.ps1"
. "$PSScriptRoot/NativeProofLock.ps1"
. "$PSScriptRoot/AndroidProofReady.ps1"
. "$PSScriptRoot/WindowsProofBuild.ps1"
. "$PSScriptRoot/AndroidProofBuild.ps1"
if($CandidateId -notmatch '^\d{8}-\d{6}-[a-f0-9]{8}$'){throw 'Invalid candidate'}
$candidate=Join-Path $TogetherRoot "work/private-alpha-candidates/$CandidateId"
$package=Join-Path $candidate 'package'
$appRoot=Join-Path $package 'Windows'
Assert-WindowsProofPath -Root (Join-Path $TogetherRoot 'work/private-alpha-candidates') -Path $candidate
$build=Get-Content "$candidate/candidate-build.json" -Raw | ConvertFrom-Json
if($build.candidate_id -cne $CandidateId -or $build.windows_sources.sha256 -cne (Get-WindowsProofSources -Root $TogetherRoot).sha256 -or $build.android_sources.sha256 -cne (Get-AndroidProofSources -Root $TogetherRoot).sha256){throw 'Candidate sources differ; refuse a mismatched instrumentation build'}
foreach($payload in $build.files){
 $file=Join-Path $package $payload.file
 Assert-WindowsProofPath -Root $package -Path $file
 if((Get-FileHash $file).Hash.ToLowerInvariant() -cne $payload.sha256){throw 'Candidate payload integrity mismatch'}
}
$adb=Join-Path $env:ANDROID_HOME 'platform-tools/adb.exe'
$emulator=Join-Path $env:ANDROID_HOME 'emulator/emulator.exe'
$serial='emulator-5580'
$lease=Enter-NativeProofLock -Path "$TogetherRoot/work/native-proof-owner.lock"
$run=[guid]::NewGuid().ToString()
$output=Join-Path $candidate "usability-$run"
New-Item -ItemType Directory -Path $output | Out-Null
$marker="$TogetherRoot/work/native-proof-running.json"
$receipt=[ordered]@{run_id=$run;candidate_id=$CandidateId;status='running';phase='instrumentation_build';rendered_frontend_verified=$false;physical_release_gate_satisfied=$false;scope='hidden_windows_and_android_emulator_fixture';runner_pid=$PID;runner_started_utc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o')}
$hostProcess=$null;$emulatorProcess=$null;$reverseOwned=$false
try {
 $receipt | ConvertTo-Json | Set-Content $marker
 if(Get-NetTCPConnection -LocalPort 47822 -State Listen -ErrorAction SilentlyContinue){throw 'Isolated host port occupied'}
 if((& $adb devices) -match '^emulator-5580\s'){throw 'Emulator already present; preserve its owner'}
 Push-Location "$TogetherRoot/apps/android"
 try {& ./gradlew.bat --no-daemon --max-workers=1 --init-script "$candidate/candidate.init.gradle" :app:assembleDebugAndroidTest *> "$output/instrumentation-build.log";if($LASTEXITCODE){throw 'Instrumentation build failed'}} finally {Pop-Location}
 $env:ANDROID_AVD_HOME='C:\Users\astro\Documents\Codex\2026-10-06\new-chat\Sidecar\tools\avd'
 $receipt.phase='emulator_boot';$receipt | ConvertTo-Json | Set-Content $marker
 $emulatorProcess=Start-Process $emulator -ArgumentList '-avd','SidecarTest','-port','5580','-no-window','-no-audio','-no-snapshot-save','-cores','2','-gpu','swiftshader_indirect' -WindowStyle Hidden -PassThru -RedirectStandardOutput "$output/emulator.stdout.log" -RedirectStandardError "$output/emulator.stderr.log"
 Wait-AndroidProofReady -Adb $adb -Serial $serial -TimeoutSeconds 180
 & $adb -s $serial install -r "$package/2gether-PrivateAlpha.apk" *> "$output/install-app.log";if($LASTEXITCODE){throw 'App install failed'}
 & $adb -s $serial install -r "$candidate/android-build/app/outputs/apk/androidTest/debug/app-debug-androidTest.apk" *> "$output/install-tests.log";if($LASTEXITCODE){throw 'Tests install failed'}
 $env:TOGETHER_SESSION_PROOF='1';$env:TOGETHER_HIDDEN_ALPHA_PROOF='1';$env:TOGETHER_PROOF_RUN_ID=$run
 $env:TOGETHER_PHONE_TOKEN=[guid]::NewGuid().ToString()+[guid]::NewGuid().ToString()
 $receipt.phase='hidden_application_startup';$receipt | ConvertTo-Json | Set-Content $marker
 $hostProcess=Start-Process "$appRoot/2gether.exe" -WorkingDirectory $appRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput "$output/app.stdout.log" -RedirectStandardError "$output/app.stderr.log"
 $timer=[Diagnostics.Stopwatch]::StartNew()
 do {
  if($hostProcess.HasExited){throw 'Hidden app exited'}
  if((Get-Content "$output/app.stdout.log" -Raw -ErrorAction SilentlyContinue) -match 'ALPHA_RENDERED_UI_AND_NATIVE_STARTUP_VERIFIED'){$receipt.rendered_frontend_verified=$true;break}
  Start-Sleep -Milliseconds 250
 }while($timer.Elapsed.TotalSeconds -lt 60)
 if(!$receipt.rendered_frontend_verified){throw 'Actual frontend/native startup proof missing'}
 $codePath="$appRoot/work/pairing-code-proof.txt"
 if(!(Test-Path $codePath)){throw 'Pairing nonce missing'}
 & $adb -s $serial reverse tcp:47820 tcp:47822 | Out-Null;if($LASTEXITCODE){throw 'Reverse failed'};$reverseOwned=$true
 $pairing=@{url='ws://127.0.0.1:47820/session';token=$env:TOGETHER_PHONE_TOKEN;code=(Get-Content $codePath -Raw)} | ConvertTo-Json -Compress
 & $adb -s $serial shell run-as app.together.v2 mkdir -p files
 $pairing | & $adb -s $serial shell run-as app.together.v2 sh -c '"cat > files/proof-pairing.json"'
 if($LASTEXITCODE){throw 'Private pairing write failed'}
 $receipt.phase='native_instrumentation';$receipt | ConvertTo-Json | Set-Content $marker
 & $adb -s $serial shell am instrument -r -w -e steadySeconds 60 -e proofRunId $run -e class app.together.v2.PairingStoreTest,app.together.v2.PairingConnectionTest,app.together.v2.NativeTlsTest,app.together.v2.SessionIdentityTest,app.together.v2.NativePlayerTest,app.together.v2.SessionPlaybackTest app.together.v2.test/app.together.v2.MutedTestRunner *> "$output/instrumentation.txt"
 if($LASTEXITCODE){throw 'Instrumentation failed'}
 $count=@(Get-Content "$PSScriptRoot/native-proof-tests.json" -Raw | ConvertFrom-Json).Count
 $verification=& python -B "$PSScriptRoot/verify_instrumentation.py" "$output/instrumentation.txt" --expected-tests $count --expected-identities "$PSScriptRoot/native-proof-tests.json"
 if($LASTEXITCODE){throw 'Executed native cases failed verification'}
 $receipt.instrumentation=$verification | ConvertFrom-Json
 & $adb -s $serial exec-out run-as app.together.v2 cat files/phone-session-proof.jsonl | Set-Content "$output/phone-session-proof.jsonl"
 # A second normal-mode host uses real DPAPI persistence, not synthetic credentials.
 # Pair through actual Android fields and restore through a new normal Activity.
 $receipt.phase='normal_pairing_ui';$receipt | ConvertTo-Json | Set-Content $marker
 New-Item -ItemType File "$appRoot/work/startup-proof-stop.txt" -Force | Out-Null
 if(!$hostProcess.WaitForExit(15000)){throw 'Synthetic host did not stop gracefully'}
 Remove-Item -LiteralPath "$appRoot/work/startup-proof-stop.txt"
 Remove-Item -LiteralPath "$appRoot/work/pairing-code-proof.txt"
 & $adb -s $serial shell run-as app.together.v2 rm -f files/proof-pairing.json
 if($LASTEXITCODE){throw 'Injected Android pairing cleanup failed'}
 $env:TOGETHER_SESSION_PROOF=$null
 $hostProcess=Start-Process "$appRoot/2gether.exe" -WorkingDirectory $appRoot -WindowStyle Hidden -PassThru -RedirectStandardOutput "$output/normal-app.stdout.log" -RedirectStandardError "$output/normal-app.stderr.log"
 $timer.Restart()
 do {
  if($hostProcess.HasExited){throw 'Normal hidden app exited'}
  if((Get-Content "$output/normal-app.stdout.log" -Raw -ErrorAction SilentlyContinue) -match 'ALPHA_RENDERED_UI_AND_NATIVE_STARTUP_VERIFIED' -and (Test-Path $codePath)){break}
  Start-Sleep -Milliseconds 250
 }while($timer.Elapsed.TotalSeconds -lt 60)
 if(!(Test-Path $codePath)){throw 'Normal application pairing nonce missing'}
 $normalCode=(Get-Content $codePath -Raw).Trim()
 & $adb -s $serial shell am instrument -r -w -e pairingCode $normalCode -e class app.together.v2.PairingScreenTest app.together.v2.test/app.together.v2.MutedTestRunner *> "$output/normal-pairing-instrumentation.txt"
 if($LASTEXITCODE){throw 'Normal pairing screen instrumentation failed'}
 $normalVerification=& python -B "$PSScriptRoot/verify_instrumentation.py" "$output/normal-pairing-instrumentation.txt" --expected-tests 1 --expected-identities "$PSScriptRoot/pairing-screen-tests.json"
 if($LASTEXITCODE){throw 'Normal pairing screen and persistence failed'}
 $receipt.normal_pairing_instrumentation=$normalVerification | ConvertFrom-Json
 $receipt.normal_pairing_ui_verified=$true
 $receipt.normal_dpapi_host_verified=$true
 if($build.android_sources.sha256 -cne (Get-AndroidProofSources -Root $TogetherRoot).sha256 -or $build.windows_sources.sha256 -cne (Get-WindowsProofSources -Root $TogetherRoot).sha256){throw 'Application sources changed during usability verification'}
 $receipt.status='passed';$receipt.phase='complete'
}catch {$receipt.status='failed';$receipt.failure=$_.Exception.Message}
finally {
 if($reverseOwned){
  $phone=@(& $adb -s $serial exec-out run-as app.together.v2 cat files/phone-session-proof.jsonl 2>$null)
  if($LASTEXITCODE -eq 0 -and $phone.Count){try {if(($phone[0] | ConvertFrom-Json).run_id -ceq $run){$phone | Set-Content "$output/phone-session-proof.jsonl"}}catch {}}
 }
 if($hostProcess -and !$hostProcess.HasExited){New-Item -ItemType File "$appRoot/work/startup-proof-stop.txt" -Force | Out-Null;if(!$hostProcess.WaitForExit(15000)){$hostProcess.Kill();$hostProcess.WaitForExit();$receipt.forced_owned_host_cleanup=$true}}
 if($reverseOwned){& $adb -s $serial shell run-as app.together.v2 rm -f files/proof-pairing.json 2>$null;& $adb -s $serial reverse --remove tcp:47820 2>$null}
 if($emulatorProcess -and !$emulatorProcess.HasExited){& $adb -s $serial emu kill *> "$output/emulator-stop.log";$null=$emulatorProcess.WaitForExit(15000)}
 $receipt.completed_utc=[datetime]::UtcNow.ToString('o')
 $receipt | ConvertTo-Json -Depth 8 | Set-Content "$output/receipt.json"
 $receipt | ConvertTo-Json -Depth 8 | Set-Content "$TogetherRoot/work/alpha-usability-last-run.json"
 if(Test-Path $marker){$owner=Get-Content $marker -Raw | ConvertFrom-Json;if($owner.run_id -ceq $run){Remove-Item $marker}}
 foreach($name in $proofEnvironment.Keys){[Environment]::SetEnvironmentVariable($name,$proofEnvironment[$name],'Process')}
 $lease.Dispose()
}
$receipt | ConvertTo-Json -Depth 8
