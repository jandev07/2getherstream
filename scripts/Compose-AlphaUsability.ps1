# Combine independently executed native and normal-UI components. Preserve failures.
param([Parameter(Mandatory)][string]$CandidateId,[Parameter(Mandatory)][string]$CoreRunId,[Parameter(Mandatory)][string]$NormalRunId)
$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
. "$PSScriptRoot/WindowsProofBuild.ps1"
. "$PSScriptRoot/AndroidProofBuild.ps1"
if($CandidateId -notmatch '^\d{8}-\d{6}-[a-f0-9]{8}$' -or $CoreRunId -notmatch '^[a-f0-9-]{36}$' -or $NormalRunId -notmatch '^[a-f0-9-]{36}$'){throw 'Invalid component identities'}
$candidate=Join-Path $root "work/private-alpha-candidates/$CandidateId"
Assert-WindowsProofPath -Root $root -Path $candidate
$build=Get-Content "$candidate/candidate-build.json" -Raw|ConvertFrom-Json
$core=Get-Content "$candidate/usability-$CoreRunId/receipt.json" -Raw|ConvertFrom-Json
$normal=Get-Content "$candidate/normal-pairing-$NormalRunId/receipt.json" -Raw|ConvertFrom-Json
if($build.candidate_id -cne $CandidateId -or $core.candidate_id -cne $CandidateId -or $normal.candidate_id -cne $CandidateId -or $normal.status -cne 'passed' -or !$normal.normal_pairing_ui_verified -or !$normal.normal_dpapi_host_verified -or !$core.rendered_frontend_verified -or $core.forced_owned_host_cleanup -or $normal.forced_cleanup){throw 'Executed component acceptance missing'}
# The earlier overall run failed only in the later normal-UI component; native
# evidence is independently reparsed below, never relabelled by changing its receipt.
if($core.phase -cne 'normal_pairing_ui'){throw 'Native component did not finish before UI'}
$expected=@(Get-Content "$PSScriptRoot/native-proof-tests.json" -Raw|ConvertFrom-Json).Count
$nativeVerification=& python -B "$PSScriptRoot/verify_instrumentation.py" "$candidate/usability-$CoreRunId/instrumentation.txt" --expected-tests $expected --expected-identities "$PSScriptRoot/native-proof-tests.json"
if($LASTEXITCODE){throw 'Native component not independently verified'}
$normalVerification=& python -B "$PSScriptRoot/verify_instrumentation.py" "$candidate/normal-pairing-$NormalRunId/instrumentation.txt" --expected-tests 1 --expected-identities "$PSScriptRoot/pairing-screen-tests.json"
if($LASTEXITCODE){throw 'Normal component not independently verified'}
$equivalence=$normal.harness_equivalence
if($equivalence.candidate_id -cne $CandidateId -or $equivalence.before.sha256 -cne $build.android_sources.sha256 -or $equivalence.after.sha256 -cne (Get-AndroidProofSources -Root $root).sha256 -or $equivalence.before.file_count -ne $equivalence.after.file_count){throw 'Runtime-equivalent source association missing'}
$before=@{};foreach($row in $equivalence.before.files){if($before.ContainsKey($row.file)){throw 'Duplicate source input'};$before[$row.file]=$row.sha256}
$changes=@($equivalence.after.files |Where-Object {$before[$_.file] -cne $_.sha256}|ForEach-Object {$_.file})
if($changes.Count -ne 1 -or $changes[0] -cne 'apps/android/app/src/androidTest/java/app/together/v2/PairingScreenTest.kt'){throw 'Production source changed between components'}
if($build.windows_sources.sha256 -cne (Get-WindowsProofSources -Root $root).sha256){throw 'Windows source changed between components'}
foreach($item in $build.files){if((Get-FileHash "$candidate/package/$($item.file)").Hash.ToLowerInvariant() -cne $item.sha256){throw 'Delivered payload changed between components'}}
$combined=[ordered]@{run_id=[guid]::NewGuid().ToString();candidate_id=$CandidateId;status='passed';phase='complete';scope='same_packaged_apps_native_fixture_and_normal_ui_components';rendered_frontend_verified=$true;normal_pairing_ui_verified=$true;normal_dpapi_host_verified=$true;instrumentation=($nativeVerification|ConvertFrom-Json);normal_pairing_instrumentation=($normalVerification|ConvertFrom-Json);native_component="usability-$CoreRunId/receipt.json";normal_component="normal-pairing-$NormalRunId/receipt.json";runtime_inputs_unchanged=$true;harness_only_changed_file=$changes[0];physical_release_gate_satisfied=$false;public_control_tunnel_verified=$false;recorded_utc=[datetime]::UtcNow.ToString('o')}
$destination="$candidate/usability-combined-$($combined.run_id)"
New-Item -ItemType Directory $destination | Out-Null
$combined |ConvertTo-Json -Depth 6|Set-Content "$destination/receipt.json"
Copy-Item -LiteralPath "$root/apps/android/app/src/androidTest/java/app/together/v2/PairingScreenTest.kt" -Destination "$destination/PairingScreenTest.kt"
$validation=Get-Content "$candidate/replacement-validation.json" -Raw|ConvertFrom-Json
$validation.usability_receipt="usability-combined-$($combined.run_id)/receipt.json"
$validation.status='native_and_normal_ui_passed'
$validation |ConvertTo-Json -Depth 6|Set-Content "$candidate/replacement-validation.json"
$combined |ConvertTo-Json -Depth 6
