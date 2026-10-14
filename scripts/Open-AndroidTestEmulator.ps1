# This interactive launcher runs only when the user opens the Desktop shortcut.
$ErrorActionPreference='Stop'
. "$PSScriptRoot/Use-LocalToolchain.ps1"
. "$PSScriptRoot/AndroidProofReady.ps1"
$ownership=& "$PSScriptRoot/Get-NativeProofOwnership.ps1"
if($ownership.resources_must_be_preserved){throw 'Android validation is running. Try the launcher again after it finishes.'}
$desktop=[Environment]::GetFolderPath('Desktop')
$offered=@(Get-ChildItem -LiteralPath $desktop -Directory -Filter '2gether Alpha *' | Where-Object {
 $deliveryPath=Join-Path $_.FullName 'desktop-delivery.json'
 if(Test-Path $deliveryPath){(Get-Content $deliveryPath -Raw | ConvertFrom-Json).application_usability_verified -eq $true}
} | Sort-Object LastWriteTime -Descending)
if(!$offered.Count){throw 'The usable Alpha is still being prepared. Wait for its delivery message before opening the emulator.'}
$apk=Join-Path $offered[0].FullName '2gether-PrivateAlpha.apk'
$delivery=Get-Content (Join-Path $offered[0].FullName 'desktop-delivery.json') -Raw|ConvertFrom-Json
$payload=@($delivery.files | Where-Object {$_.file -ceq '2gether-PrivateAlpha.apk'})
if($payload.Count -ne 1 -or (Get-FileHash -LiteralPath $apk).Hash.ToLowerInvariant() -cne $payload[0].sha256){throw 'Delivered Android app integrity check failed'}
$adb=Join-Path $env:ANDROID_HOME 'platform-tools/adb.exe'
$emulator=Join-Path $env:ANDROID_HOME 'emulator/emulator.exe'
$serial='emulator-5580'
$env:ANDROID_AVD_HOME='C:\Users\astro\Documents\Codex\2026-10-06\new-chat\Sidecar\tools\avd'
if((& $adb devices) -match '^emulator-5580\s'){throw 'An emulator is already running. Use its existing window; this launcher will not restart it.'}
$leasePath=Join-Path $TogetherRoot 'work/native-proof-owner.lock'
$lease=[IO.FileStream]::new($leasePath,[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
$marker=Join-Path $TogetherRoot 'work/native-proof-running.json'
$runId=[guid]::NewGuid().ToString()
$emulatorProcess=$null
try {
 $latest=& "$PSScriptRoot/Get-NativeProofOwnership.ps1"
 if($latest.runner_identity -notin @('absent','exited','different_process')){throw 'A validation runner owns the emulator.'}
 $emulatorProcess=Start-Process $emulator -ArgumentList '-avd','SidecarTest','-port','5580','-no-audio','-cores','2','-gpu','swiftshader_indirect' -PassThru
 @{run_id=$runId;kind='user_interactive_emulator';runner_pid=$emulatorProcess.Id;runner_started_utc=$emulatorProcess.StartTime.ToUniversalTime().ToString('o')} | ConvertTo-Json | Set-Content $marker
 Wait-AndroidProofReady -Adb $adb -Serial $serial -TimeoutSeconds 180
 & $adb -s $serial install -r $apk | Out-Null;if($LASTEXITCODE){throw 'Android app installation failed'}
 # Test-only injected configuration must never override the user's normal UI
 # or encrypted pairing after a crashed/aborted earlier validation.
 & $adb -s $serial shell run-as app.together.v2 rm -f files/proof-pairing.json | Out-Null
 if($LASTEXITCODE){throw 'Could not clear obsolete validation setup; app was not opened'}
 & $adb -s $serial reverse tcp:47820 tcp:47820 | Out-Null;if($LASTEXITCODE){throw 'Local PC connection could not be prepared'}
 & $adb -s $serial shell am start -n app.together.v2/.MainActivity | Out-Null
 if($LASTEXITCODE){throw 'Android app could not open'}
 Write-Host 'Use the mouse to tap and scroll. Open the matching Windows Alpha, then pair using http://127.0.0.1:47820 and its pairing code. Emulator sound is disabled. Close the emulator window when finished.'
 # Reserve the AVD and adb reverse for the entire user session. Development can
 # continue around this handle but must not install/restart the user's emulator.
 while((& $adb devices) -match '^emulator-5580\s'){Start-Sleep -Seconds 5}
}finally {
 # Preserve the live emulator marker even if the user closes this terminal;
 # subsequent development must not mistake a released handle for permission.
 if($emulatorProcess -and $emulatorProcess.HasExited -and (Test-Path $marker)){
  $owner=Get-Content $marker -Raw | ConvertFrom-Json
  if($owner.run_id -ceq $runId){Remove-Item $marker}
 }
 $lease.Dispose()
}
