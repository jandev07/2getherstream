# Internal version8 -> version9 archive/data validation. Never publishes an Alpha.
param()
$ErrorActionPreference='Stop'
. "$PSScriptRoot/Use-LocalToolchain.ps1"
. "$PSScriptRoot/NativeProofLock.ps1"
. "$PSScriptRoot/AndroidProofBuild.ps1"
. "$PSScriptRoot/AndroidProofReady.ps1"
if(Test-Path "$TogetherRoot/work/control-soak-running.json"){throw 'Review control ownership before this run'}
$control=[IO.FileStream]::new("$TogetherRoot/work/control-soak-owner.lock",[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None);$control.Dispose()
$lease=Enter-NativeProofLock -Path "$TogetherRoot/work/native-proof-owner.lock"
$run=[guid]::NewGuid().ToString()
$output=Join-Path $TogetherRoot "work/update-archive-proof/$run"
New-Item -ItemType Directory -Path $output | Out-Null
$adb=Join-Path $env:ANDROID_HOME 'platform-tools/adb.exe'
$emulator=Join-Path $env:ANDROID_HOME 'emulator/emulator.exe'
$serial='emulator-5580';$emulatorProcess=$null;$ready=$false;$testInstalled=$false;$emulatorStopped=$true
$oldAvd=$env:ANDROID_AVD_HOME
$marker="$TogetherRoot/work/native-proof-running.json"
$remote="/data/local/tmp/2gether-archive-$run.apk"
$receipt=[ordered]@{run_id=$run;status='running';phase='build';runner_pid=$PID;runner_started_utc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o');scope='real_android_package_archive_verification_and_adb_upgrade_data_retention';public_download_verified=$false;in_app_installer_confirmed=$false;physical_gate_passed=$false}
function Save-Phase([string]$phase){$receipt.phase=$phase;$receipt|ConvertTo-Json -Depth 8|Set-Content $marker}
function Verify-Cases([string]$result,[string]$identities){
 $count=@(Get-Content $identities -Raw |ConvertFrom-Json).Count
 $verified=& python -B "$PSScriptRoot/verify_instrumentation.py" $result --expected-tests $count --expected-identities $identities
 if($LASTEXITCODE){throw 'Exact instrumentation verification failed'}
 return $verified|ConvertFrom-Json
}
try {
 Save-Phase 'build'
 Push-Location $TogetherRoot
 try{git diff --quiet HEAD -- apps/android;if($LASTEXITCODE){throw 'Commit Android inputs before the owned build'};if(@(git ls-files --others --exclude-standard -- apps/android).Count){throw 'Untracked Android source'}}finally{Pop-Location}
 $source=Get-AndroidProofSources -Root $TogetherRoot
 $receipt.source_commit=(git -C $TogetherRoot rev-parse HEAD).Trim();$receipt.android_sources=$source
 foreach($code in @(8,9)) {
  $buildPath="$output/build-$code".Replace('\','/')
  $init="$output/version-$code.init.gradle"
  $text="allprojects { layout.buildDirectory.set(file('$buildPath/' + (project.path == ':' ? 'root' : project.path.substring(1).replace(':', '/')))) }`n"
  if($code -eq 8){$text+="allprojects { plugins.withId('com.android.application') { androidComponents.finalizeDsl { a -> a.defaultConfig.versionCode=8; a.defaultConfig.versionName='0.1.8-updater-validation-lower' } } }`n"}
  [IO.File]::WriteAllText($init,$text,[Text.UTF8Encoding]::new($false))
  Push-Location "$TogetherRoot/apps/android"
  try{if($code -eq 8){& ./gradlew.bat --no-daemon --max-workers=1 --init-script $init :app:assembleDebug :app:assembleDebugAndroidTest *> "$output/build-$code.log"}else{& ./gradlew.bat --no-daemon --max-workers=1 --init-script $init :app:assembleDebug *> "$output/build-$code.log"};if($LASTEXITCODE){throw 'Isolated update artifact build failed'}}finally{Pop-Location}
 }
 if($source.sha256 -cne (Get-AndroidProofSources -Root $TogetherRoot).sha256){throw 'Source changed during update build'}
 $lower="$output/build-8/app/outputs/apk/debug/app-debug.apk"
 $higher="$output/build-9/app/outputs/apk/debug/app-debug.apk"
 $tests="$output/build-8/app/outputs/apk/androidTest/debug/app-debug-androidTest.apk"
 $receipt.payloads=@(foreach($item in @($lower,$higher,$tests)){[ordered]@{file=[IO.Path]::GetRelativePath($output,$item);sha256=(Get-FileHash $item).Hash.ToLowerInvariant()}})
 $aapt=Join-Path $env:ANDROID_HOME 'build-tools/35.0.0/aapt.exe'
 foreach($pair in @(@($lower,8),@($higher,9))){$identity=@(& $aapt dump badging $pair[0])[0];if($LASTEXITCODE -or $identity -notmatch "name='app\.together\.v2' versionCode='$($pair[1])'"){throw 'Validation APK version identity mismatch'}}
 if(!(Test-AndroidProofEmulatorAbsent -Adb $adb -Serial $serial)){throw 'Device inventory occupied or uncertain; preserve ownership'}
 $env:ANDROID_AVD_HOME='C:\Users\astro\Documents\Codex\2026-10-06\new-chat\Sidecar\tools\avd'
 Save-Phase 'emulator_boot'
 $emulatorProcess=Start-Process $emulator -ArgumentList '-avd','SidecarTest','-port','5580','-no-window','-no-audio','-no-snapshot-save','-cores','2','-gpu','swiftshader_indirect' -WindowStyle Hidden -PassThru -RedirectStandardOutput "$output/emulator.stdout.log" -RedirectStandardError "$output/emulator.stderr.log"
 Wait-AndroidProofReady -Adb $adb -Serial $serial -TimeoutSeconds 180;$ready=$true
 & $adb -s $serial install -r -d $lower *> "$output/install-lower.log";if($LASTEXITCODE){throw 'Validation lower install failed'}
 & $adb -s $serial install -r $tests *> "$output/install-tests.log";if($LASTEXITCODE){throw 'Validation test install failed'};$testInstalled=$true
 & $adb -s $serial push $higher $remote *> "$output/stage.log";if($LASTEXITCODE){throw 'Owned APK staging failed'}
 & $adb -s $serial shell run-as app.together.v2 mkdir -p cache/updates
 & $adb -s $serial shell run-as app.together.v2 cp $remote "cache/updates/archive-validation-$run.apk";if($LASTEXITCODE){throw 'Private APK staging failed'}
 $hash=(Get-FileHash $higher).Hash.ToLowerInvariant()
 Save-Phase 'archive_verification'
 & $adb -s $serial shell am instrument -r -w -e archiveRun $run -e archiveHash $hash -e class 'app.together.v2.UpdateAndLogsTest,app.together.v2.UpdateArchiveTest#higherSignedArchiveAndContentProviderAreVerifiedBeforeUpgrade' app.together.v2.test/app.together.v2.MutedTestRunner *> "$output/before.txt"
 if($LASTEXITCODE){throw 'Before-upgrade instrumentation failed'}
 $receipt.before=Verify-Cases "$output/before.txt" "$PSScriptRoot/update-archive-before-tests.json"
 Save-Phase 'adb_package_upgrade'
 & $adb -s $serial install -r $higher *> "$output/install-higher.log";if($LASTEXITCODE){throw 'Owned package upgrade failed'}
 & $adb -s $serial shell am instrument -r -w -e archiveRun $run -e class 'app.together.v2.UpdateArchiveTest#packageUpgradeRetainsIsolatedEncryptedPairingAndSettings' app.together.v2.test/app.together.v2.MutedTestRunner *> "$output/after.txt"
 if($LASTEXITCODE){throw 'After-upgrade instrumentation failed'}
 $receipt.after=Verify-Cases "$output/after.txt" "$PSScriptRoot/update-archive-after-tests.json"
 if($source.sha256 -cne (Get-AndroidProofSources -Root $TogetherRoot).sha256){throw 'Source changed during update verification'}
 $receipt.status='passed';$receipt.phase='complete'
}catch{$receipt.status='failed';$receipt.failure=$_.Exception.Message}
finally {
 if($ready -and $testInstalled){
  & $adb -s $serial shell am instrument -r -w -e archiveRun $run -e class 'app.together.v2.UpdateArchiveTest#cleanupOwnedArchiveValidation' app.together.v2.test/app.together.v2.MutedTestRunner *> "$output/cleanup.txt"
  try{$receipt.cleanup=Verify-Cases "$output/cleanup.txt" "$PSScriptRoot/update-archive-cleanup-tests.json"}catch{$receipt.status='failed';$receipt.cleanup_failed=$true}
  & $adb -s $serial shell rm -f $remote *> "$output/stage-cleanup.log"
 }
 if($emulatorProcess){
  if(!$emulatorProcess.HasExited){Invoke-AndroidProofCommand -Adb $adb -Arguments @('-s',$serial,'emu','kill') | Set-Content "$output/emulator-stop.log"}
  if(!$emulatorProcess.HasExited -and !$emulatorProcess.WaitForExit(15000)){
   $emulatorProcess.Kill($true);$null=$emulatorProcess.WaitForExit(5000);$receipt.forced_owned_emulator_cleanup=$true
  }
  $emulatorStopped=$emulatorProcess.HasExited -and (Test-AndroidProofEmulatorAbsent -Adb $adb -Serial $serial)
  $receipt.emulator_stopped=$emulatorStopped
  if(!$emulatorStopped){$receipt.status='failed';$receipt.cleanup_uncertain=$true}
 }
 $env:ANDROID_AVD_HOME=$oldAvd
 $receipt.completed_utc=[datetime]::UtcNow.ToString('o')
 $receipt|ConvertTo-Json -Depth 8|Set-Content "$output/receipt.json"
 $receipt|ConvertTo-Json -Depth 8|Set-Content "$TogetherRoot/work/update-archive-last-run.json"
 if(Test-Path $marker){$owner=Get-Content $marker -Raw|ConvertFrom-Json;if($owner.run_id -ceq $run){
  if($emulatorStopped){Remove-Item -LiteralPath $marker}
  else{
   # Explicitly unknown resource owner forces the inventory's unverified path.
   # Do not claim the exited shell runner still identifies the remaining device.
   @{run_id=$run;runner_pid=$null;runner_started_utc=$null;original_runner_pid=$PID;resource_owner='unverified_after_owned_emulator_stop'}|ConvertTo-Json|Set-Content $marker
  }
 }}
 $lease.Dispose()
}
$receipt|ConvertTo-Json -Depth 8
if($receipt.status -ne 'passed'){throw 'Owned update archive proof failed; immutable history retained'}
