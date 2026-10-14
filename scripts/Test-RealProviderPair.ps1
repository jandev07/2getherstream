param([string]$CandidateId='20261012-184349-353ad1a9')
$ErrorActionPreference='Stop'
. "$PSScriptRoot/Use-LocalToolchain.ps1"
. "$PSScriptRoot/NativeProofLock.ps1"
. "$PSScriptRoot/AndroidProofReady.ps1"
. "$PSScriptRoot/WindowsProofBuild.ps1"
if($CandidateId -notmatch '^\d{8}-\d{6}-[a-f0-9]{8}$'){throw 'Invalid candidate'}
$candidate=Join-Path $TogetherRoot "work/private-alpha-candidates/$CandidateId"
Assert-WindowsProofPath -Root "$TogetherRoot/work/private-alpha-candidates" -Path $candidate
$build=Get-Content "$candidate/candidate-build.json" -Raw | ConvertFrom-Json
$apk="$candidate/package/2gether-PrivateAlpha.apk"
$testApk="$candidate/android-build/app/outputs/apk/androidTest/debug/app-debug-androidTest.apk"
if((Get-FileHash $apk).Hash.ToLowerInvariant() -cne $build.files.Where({$_.file -ceq '2gether-PrivateAlpha.apk'})[0].sha256){throw 'Candidate APK integrity mismatch'}
if(!(Test-Path $testApk)){throw 'Archived instrumentation missing; this runner does not build APKs'}
$adb=Join-Path $env:ANDROID_HOME 'platform-tools/adb.exe';$serial='emulator-5580'
$lease=Enter-NativeProofLock -Path "$TogetherRoot/work/native-proof-owner.lock"
$run=[guid]::NewGuid().ToString();$marker="$TogetherRoot/work/native-proof-running.json"
$output=Join-Path $TogetherRoot "work/real-provider-pair/$run";$runtime="$output/runtime"
New-Item -ItemType Directory "$runtime/work" -Force | Out-Null
$receipt=[ordered]@{run_id=$run;kind='emulator_real_torbox_private_alpha';candidate_id=$CandidateId;runner_pid=$PID;runner_started_utc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o');status='running';phase='runtime_preparation';android_apk_sha256=(Get-FileHash $apk).Hash.ToLowerInvariant();archived_test_apk_sha256=(Get-FileHash $testApk).Hash.ToLowerInvariant();account_bearing_private_exception=$true;android_handoff_verified=$false;physical_gate_passed=$false;new_apk_built=$false}
$hostProcess=$null;$emulatorProcess=$null;$reverseOwned=$false
$saved=@{};foreach($name in @('ANDROID_AVD_HOME','TOGETHER_SESSION_PROOF','TOGETHER_PROOF_RUN_ID','TOGETHER_PHONE_TOKEN')){$saved[$name]=[Environment]::GetEnvironmentVariable($name,'Process')}
try {
 $receipt | ConvertTo-Json | Set-Content $marker
 if((& $adb devices) -match '^emulator-5580\s'){throw 'Existing emulator preserved'}
 if(Get-NetTCPConnection -LocalPort 47822 -State Listen -ErrorAction SilentlyContinue){throw 'Isolated control port occupied'}
 foreach($payload in $build.files.Where({$_.file.StartsWith('Windows/native/')})){
  $source=Join-Path "$candidate/package" $payload.file;Assert-WindowsProofPath -Root $candidate -Path $source
  if((Get-FileHash $source).Hash.ToLowerInvariant() -cne $payload.sha256){throw 'Native payload mismatch'}
  $target=Join-Path $runtime ($payload.file.Substring(8));New-Item -ItemType Directory (Split-Path -Parent $target) -Force | Out-Null;Copy-Item -LiteralPath $source -Destination $target
 }
 $selected=@(Get-Content "$PSScriptRoot/native-proof-tests.json" -Raw | ConvertFrom-Json | Where-Object {$_[0] -eq 'app.together.v2.SessionPlaybackTest'})
 if($selected.Count -ne 1){throw 'Expected playback identity missing'}
 ConvertTo-Json -InputObject $selected | Set-Content "$output/expected-tests.json"
 $env:ANDROID_AVD_HOME='C:\Users\astro\Documents\Codex\2026-10-06\new-chat\Sidecar\tools\avd'
 $emulatorProcess=Start-Process "$env:ANDROID_HOME/emulator/emulator.exe" -ArgumentList '-avd','SidecarTest','-port','5580','-no-window','-no-audio','-no-snapshot-save','-cores','2','-gpu','swiftshader_indirect' -WindowStyle Hidden -PassThru -RedirectStandardOutput "$output/emulator.stdout.log" -RedirectStandardError "$output/emulator.stderr.log"
 $receipt.phase='emulator_boot';$receipt | ConvertTo-Json | Set-Content $marker
 Wait-AndroidProofReady -Adb $adb -Serial $serial -TimeoutSeconds 180
 & $adb -s $serial install -r $apk *> "$output/install-app.log";if($LASTEXITCODE){throw 'Existing Alpha install failed'}
 & $adb -s $serial install -r $testApk *> "$output/install-tests.log";if($LASTEXITCODE){throw 'Archived tests install failed'}
 $env:TOGETHER_SESSION_PROOF='1';$env:TOGETHER_PROOF_RUN_ID=$run;$env:TOGETHER_PHONE_TOKEN=[guid]::NewGuid().ToString()+[guid]::NewGuid().ToString()
 $receipt.phase='real_provider_host';$receipt | ConvertTo-Json | Set-Content $marker
 $hostProcess=Start-Process "$TogetherRoot/work/private-alpha-candidates/20261012-135012-d336360a/rust-target/debug/examples/torbox-paired-proof.exe" -ArgumentList ('"'+$runtime+'"') -WindowStyle Hidden -PassThru -RedirectStandardOutput "$output/host.stdout.log" -RedirectStandardError "$output/host.stderr.log"
 $timer=[Diagnostics.Stopwatch]::StartNew()
 do {if($hostProcess.HasExited){throw 'Provider host failed'};if((Get-Content "$output/host.stdout.log" -Raw) -match 'REAL_PROVIDER_HOST_READY'){break};Start-Sleep -Milliseconds 250}while($timer.Elapsed.TotalSeconds -lt 90)
 if($timer.Elapsed.TotalSeconds -ge 90){throw 'Provider preparation deadline'}
 & $adb -s $serial reverse tcp:47820 tcp:47822 | Out-Null;if($LASTEXITCODE){throw 'Local control reverse failed'};$reverseOwned=$true
 $config=@{url='ws://127.0.0.1:47820/session';token=$env:TOGETHER_PHONE_TOKEN;code=(Get-Content "$runtime/work/pairing-code-proof.txt" -Raw)} | ConvertTo-Json -Compress
 & $adb -s $serial shell run-as app.together.v2 mkdir -p files
 $config | & $adb -s $serial shell run-as app.together.v2 sh -c '"cat > files/proof-pairing.json"';if($LASTEXITCODE){throw 'Private proof configuration failed'}
 $receipt.phase='real_provider_native_playback';$receipt | ConvertTo-Json | Set-Content $marker
 & $adb -s $serial shell am instrument -r -w -e steadySeconds 60 -e proofRunId $run -e class app.together.v2.SessionPlaybackTest app.together.v2.test/app.together.v2.MutedTestRunner *> "$output/instrumentation.txt"
 if($LASTEXITCODE){throw 'Native command failed'}
 $verification=& python -B "$PSScriptRoot/verify_instrumentation.py" "$output/instrumentation.txt" --expected-tests 1 --expected-identities "$output/expected-tests.json"
 if($LASTEXITCODE){throw 'Real provider playback assertions failed'}
 $receipt.instrumentation=$verification | ConvertFrom-Json
 & $adb -s $serial exec-out run-as app.together.v2 cat files/phone-session-proof.jsonl | Set-Content "$output/phone-session-proof.jsonl"
 $receipt.android_handoff_verified=$true;$receipt.status='passed';$receipt.phase='complete'
}catch {$receipt.status='failed';$receipt.failure=$_.Exception.Message}
finally {
 if($reverseOwned){
  # Preserve failed native observations too, before another test replaces them.
  $phone=@(& $adb -s $serial exec-out run-as app.together.v2 cat files/phone-session-proof.jsonl 2>$null)
  if($LASTEXITCODE -eq 0 -and $phone.Count){try {if(($phone[0] | ConvertFrom-Json).run_id -ceq $run){$phone | Set-Content "$output/phone-session-proof.jsonl"}}catch {}}
 }
 if($hostProcess -and !$hostProcess.HasExited){New-Item -ItemType File "$runtime/work/startup-proof-stop.txt" -Force | Out-Null;if(!$hostProcess.WaitForExit(20000)){$hostProcess.Kill();$hostProcess.WaitForExit();$receipt.forced_owned_host_cleanup=$true}}
 if($reverseOwned){& $adb -s $serial shell run-as app.together.v2 rm -f files/proof-pairing.json 2>$null;& $adb -s $serial reverse --remove tcp:47820 2>$null}
 if($emulatorProcess -and !$emulatorProcess.HasExited){& $adb -s $serial emu kill *> "$output/emulator-stop.log";$null=$emulatorProcess.WaitForExit(15000)}
 $receipt.completed_utc=[datetime]::UtcNow.ToString('o');$receipt | ConvertTo-Json -Depth 6 | Set-Content "$output/receipt.json";$receipt | ConvertTo-Json -Depth 6 | Set-Content "$TogetherRoot/work/real-provider-pair-last-run.json"
 if(Test-Path $marker){$owner=Get-Content $marker -Raw | ConvertFrom-Json;if($owner.run_id -ceq $run){Remove-Item $marker}}
 foreach($name in $saved.Keys){[Environment]::SetEnvironmentVariable($name,$saved[$name],'Process')};$lease.Dispose()
}
$receipt | ConvertTo-Json -Depth 6
