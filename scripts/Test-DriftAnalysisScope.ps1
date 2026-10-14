$ErrorActionPreference='Stop'
$TogetherRoot=Split-Path -Parent $PSScriptRoot
$TogetherTestDir=Join-Path $TogetherRoot ('work/drift-analysis-scope-'+[guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TogetherTestDir -Force | Out-Null
$TogetherRunId=[guid]::NewGuid().ToString()
$TogetherPc=@();$TogetherPhone=@()
for($TogetherIndex=0;$TogetherIndex -lt 30;$TogetherIndex++) {
    $TogetherTime=$TogetherIndex*200
    $TogetherPhase=if($TogetherIndex -lt 20){'steady'}else{'recovery'}
    $TogetherDifference=if($TogetherIndex -lt 20){10}else{300}
    $TogetherPc+=([ordered]@{run_id=$TogetherRunId;generation=1;server_ms=$TogetherTime;position_ms=$TogetherTime;rate=1.0;paused=$false;preparing=$false;buffering=$false;seeking=$false}|ConvertTo-Json -Compress)
    $TogetherPhone+=([ordered]@{run_id=$TogetherRunId;generation=1;sampled_at_ms=$TogetherTime;client_ms=$TogetherTime;offset_ms=0;position_ms=$TogetherTime+$TogetherDifference;phase=$TogetherPhase;state='Playing';preparing=$false;buffering=$false;seeking=$false;connected=$true;rtt_ms=80}|ConvertTo-Json -Compress)
}
$TogetherPcPath=Join-Path $TogetherTestDir 'pc.jsonl';$TogetherPhonePath=Join-Path $TogetherTestDir 'phone.jsonl'
$TogetherPc|Set-Content -LiteralPath $TogetherPcPath
$TogetherPhone|Set-Content -LiteralPath $TogetherPhonePath
# Raw phone position can be higher while trailing the time-interpolated PC.
$TogetherRatePc=$TogetherPc.Clone();$TogetherRatePhone=$TogetherPhone.Clone()
for($TogetherIndex=0;$TogetherIndex -lt 30;$TogetherIndex++) {
    $TogetherPcRow=$TogetherRatePc[$TogetherIndex]|ConvertFrom-Json
    $TogetherPhoneRow=$TogetherRatePhone[$TogetherIndex]|ConvertFrom-Json
    $TogetherPcRow.rate=2.0
    $TogetherPhoneRow.sampled_at_ms=$TogetherPcRow.server_ms+50
    $TogetherPhoneRow.position_ms=$TogetherPcRow.position_ms+50
    $TogetherRatePc[$TogetherIndex]=$TogetherPcRow|ConvertTo-Json -Compress
    $TogetherRatePhone[$TogetherIndex]=$TogetherPhoneRow|ConvertTo-Json -Compress
}
$TogetherRatePc|Set-Content -LiteralPath $TogetherPcPath
$TogetherRatePhone|Set-Content -LiteralPath $TogetherPhonePath
$TogetherRatePath=Join-Path $TogetherTestDir 'rate-direction.json'
$null=& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $TogetherPhonePath -PcLog $TogetherPcPath -OutputPath $TogetherRatePath -ExpectedRunId $TogetherRunId
$TogetherRate=Get-Content $TogetherRatePath -Raw|ConvertFrom-Json
if($TogetherRate.signed_median_ms -ne -50 -or $TogetherRate.phone_behind_samples -ne 30 -or $TogetherRate.p95_ms -ne 50){throw 'Direction must use rate-interpolated PC position'}
$TogetherPc|Set-Content -LiteralPath $TogetherPcPath
$TogetherPhone|Set-Content -LiteralPath $TogetherPhonePath
$TogetherAllPath=Join-Path $TogetherTestDir 'all.json';$TogetherSteadyPath=Join-Path $TogetherTestDir 'steady.json'
$null=& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $TogetherPhonePath -PcLog $TogetherPcPath -OutputPath $TogetherAllPath -ExpectedRunId $TogetherRunId
$null=& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $TogetherPhonePath -PcLog $TogetherPcPath -OutputPath $TogetherSteadyPath -ExpectedRunId $TogetherRunId -SteadyOnly
$TogetherAll=Get-Content $TogetherAllPath -Raw|ConvertFrom-Json
$TogetherSteady=Get-Content $TogetherSteadyPath -Raw|ConvertFrom-Json
if($TogetherAll.matched_samples -ne 30 -or $TogetherAll.p95_ms -ne 300 -or $TogetherAll.provisional_cross_device_p95_goal_met -or $TogetherAll.sample_scope -ne 'all_eligible_playing_samples'){throw 'Default analysis scope changed'}
if($TogetherSteady.matched_samples -ne 20 -or $TogetherSteady.p95_ms -ne 10 -or !$TogetherSteady.provisional_cross_device_p95_goal_met -or $TogetherSteady.sample_scope -ne 'explicit_steady_phase'){throw 'Steady scope analysis incorrect'}
if($TogetherAll.matched_pc_position_read_samples -ne 0 -or $null -ne $TogetherAll.matched_pc_position_read_p95_ms -or $null -ne $TogetherAll.matched_pc_position_read_max_ms){throw 'Missing historical latency invented'}
if($TogetherAll.signed_median_ms -ne 10 -or $TogetherAll.phone_ahead_samples -ne 30 -or $TogetherAll.phone_behind_samples -ne 0 -or $TogetherAll.equal_position_samples -ne 0){throw 'Positive timing direction incorrect'}
$TogetherBehindPhone=$TogetherPhone.Clone()
for($TogetherIndex=0;$TogetherIndex -lt 30;$TogetherIndex++) {
    $TogetherRow=$TogetherBehindPhone[$TogetherIndex]|ConvertFrom-Json
    $TogetherRow.position_ms=$TogetherRow.sampled_at_ms-$(if($TogetherIndex -lt 20){10}else{300})
    $TogetherBehindPhone[$TogetherIndex]=$TogetherRow|ConvertTo-Json -Compress
}
$TogetherBehindPhone|Set-Content -LiteralPath $TogetherPhonePath
$TogetherBehindPath=Join-Path $TogetherTestDir 'behind.json'
$null=& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $TogetherPhonePath -PcLog $TogetherPcPath -OutputPath $TogetherBehindPath -ExpectedRunId $TogetherRunId
$TogetherBehind=Get-Content $TogetherBehindPath -Raw|ConvertFrom-Json
if($TogetherBehind.signed_median_ms -ne -10 -or $TogetherBehind.phone_behind_samples -ne 30 -or $TogetherBehind.phone_ahead_samples -ne 0 -or $TogetherBehind.p95_ms -ne 300 -or $TogetherBehind.provisional_cross_device_p95_goal_met){throw 'Negative direction changed absolute timing acceptance'}
$TogetherPhone|Set-Content -LiteralPath $TogetherPhonePath
# Optional latency explains only the native property call boundary, and must not
# change which position comparisons pass the timing target.
$TogetherOriginalPc=$TogetherPc.Clone()
for($TogetherIndex=0;$TogetherIndex -lt 30;$TogetherIndex++) {
    $TogetherPc[$TogetherIndex]=$TogetherPc[$TogetherIndex].Replace('"rate":1.0',('"position_read_ms":'+$TogetherIndex+',"rate":1.0'))
    # ConvertTo-Json may emit integral-valued doubles without a decimal point.
    if($TogetherPc[$TogetherIndex] -notmatch 'position_read_ms') {
        $TogetherPc[$TogetherIndex]=$TogetherPc[$TogetherIndex].Replace('"rate":1',('"position_read_ms":'+$TogetherIndex+',"rate":1'))
    }
}
$TogetherPc|Set-Content -LiteralPath $TogetherPcPath
$TogetherLatencyPath=Join-Path $TogetherTestDir 'latency.json'
$null=& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $TogetherPhonePath -PcLog $TogetherPcPath -OutputPath $TogetherLatencyPath -ExpectedRunId $TogetherRunId
$TogetherLatency=Get-Content $TogetherLatencyPath -Raw|ConvertFrom-Json
if($TogetherLatency.matched_pc_position_read_samples -ne 30 -or $TogetherLatency.matched_pc_position_read_p95_ms -ne 28 -or $TogetherLatency.matched_pc_position_read_max_ms -ne 29 -or $TogetherLatency.p95_ms -ne 300 -or $TogetherLatency.provisional_cross_device_p95_goal_met){throw 'Latency diagnostic changed timing acceptance or used incorrect population'}
$TogetherSteadyLatencyPath=Join-Path $TogetherTestDir 'steady-latency.json'
$null=& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $TogetherPhonePath -PcLog $TogetherPcPath -OutputPath $TogetherSteadyLatencyPath -ExpectedRunId $TogetherRunId -SteadyOnly
$TogetherSteadyLatency=Get-Content $TogetherSteadyLatencyPath -Raw|ConvertFrom-Json
if($TogetherSteadyLatency.matched_pc_position_read_samples -ne 20 -or $TogetherSteadyLatency.matched_pc_position_read_p95_ms -ne 19 -or $TogetherSteadyLatency.matched_pc_position_read_max_ms -ne 19){throw 'Latency diagnostic ignored selected sample scope'}
$TogetherLatencyPc=$TogetherPc.Clone()
foreach($TogetherInvalid in @('-1','"private-value"','null','true')) {
    $TogetherPc=$TogetherLatencyPc.Clone()
    $TogetherPc[29]=$TogetherPc[29].Replace('"position_read_ms":29',('"position_read_ms":'+$TogetherInvalid)).Replace('"paused":false','"paused":true')
    $TogetherPc|Set-Content -LiteralPath $TogetherPcPath
    $TogetherRejected=$false
    try {$null=& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $TogetherPhonePath -PcLog $TogetherPcPath -OutputPath (Join-Path $TogetherTestDir 'invalid-latency.json') -ExpectedRunId $TogetherRunId} catch {$TogetherRejected=$true}
    if(!$TogetherRejected){throw 'Invalid excluded PC latency accepted'}
}
$TogetherPc=$TogetherOriginalPc
$TogetherPc|Set-Content -LiteralPath $TogetherPcPath
# Even excluded recovery rows must have the right run identity; scope cannot hide
# stale/cross-run observations or upgrade an undersampled segment into acceptance.
$TogetherOriginal=$TogetherPhone.Clone()
$TogetherPhone[29]=$TogetherPhone[29].Replace($TogetherRunId,[guid]::NewGuid().ToString())
$TogetherPhone|Set-Content -LiteralPath $TogetherPhonePath
$TogetherRejected=$false
try {$null=& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $TogetherPhonePath -PcLog $TogetherPcPath -OutputPath (Join-Path $TogetherTestDir 'mixed.json') -ExpectedRunId $TogetherRunId -SteadyOnly} catch {$TogetherRejected=$true}
if(!$TogetherRejected){throw 'Excluded cross-run rows accepted'}
$TogetherPhone=$TogetherOriginal
$TogetherPhone[19]=$TogetherPhone[19].Replace('"phase":"steady"','"phase":"recovery"')
$TogetherPhone|Set-Content -LiteralPath $TogetherPhonePath
$TogetherRejected=$false
try {$null=& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $TogetherPhonePath -PcLog $TogetherPcPath -OutputPath (Join-Path $TogetherTestDir 'undersampled.json') -ExpectedRunId $TogetherRunId -SteadyOnly} catch {$TogetherRejected=$true}
if(!$TogetherRejected){throw 'Undersampled steady phase accepted'}
Write-Output 'Drift scope checks passed: default preserved, steady separated, signed/interpolated direction and optional latency validated, excluded invalid latency/cross-run identity and undersampling rejected.'
