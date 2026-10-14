param([string]$PhoneLog,[string]$PcLog,[string]$OutputPath,[string]$ExpectedRunId,[switch]$AllowLegacyUnbound,[switch]$SteadyOnly,[switch]$IgnoreUnterminatedPcTail)
$ErrorActionPreference='Stop'
$TogetherRoot=Split-Path -Parent $PSScriptRoot
if(!$PhoneLog) {$PhoneLog=Join-Path $TogetherRoot 'work\phone-session-proof.jsonl'}
if(!$PcLog) {$PcLog=Join-Path $TogetherRoot 'work\native-session-proof.jsonl'}
if(!$OutputPath) {$OutputPath=Join-Path $TogetherRoot 'work\native-proof-summary.json'}
$TogetherPhone=Get-Content -LiteralPath $PhoneLog | ForEach-Object {ConvertFrom-Json $_}
$TogetherIgnoredPcTail=$false
if($IgnoreUnterminatedPcTail) {
    # Opt-in post-exit recovery only. Preserve raw bytes; never ignore a broken
    # complete record or suppress identity validation on a valid final record.
    $TogetherPcText=[IO.File]::ReadAllText($PcLog)
    if($TogetherPcText.Length -gt 0 -and !$TogetherPcText.EndsWith("`n")) {
        $TogetherLastNewline=$TogetherPcText.LastIndexOf("`n")
        try {$null=ConvertFrom-Json -InputObject $TogetherPcText.Substring($TogetherLastNewline+1)}
        catch {
            if($TogetherLastNewline -lt 0){throw 'No complete PC proof records'}
            $TogetherPcText=$TogetherPcText.Substring(0,$TogetherLastNewline+1)
            $TogetherIgnoredPcTail=$true
        }
    }
    $TogetherReader=[IO.StringReader]::new($TogetherPcText)
    try {$TogetherPc=@(while($null -ne ($TogetherLine=$TogetherReader.ReadLine())) {ConvertFrom-Json -InputObject $TogetherLine})}
    finally {$TogetherReader.Dispose()}
} else {$TogetherPc=Get-Content -LiteralPath $PcLog | ForEach-Object {ConvertFrom-Json $_}}
$TogetherExpected=[guid]::Empty
if(![guid]::TryParse($ExpectedRunId,[ref]$TogetherExpected) -or $TogetherExpected -eq [guid]::Empty){throw 'Expected proof run ID required'}
$TogetherIdentityVerified=$true
foreach($row in @($TogetherPhone)+@($TogetherPc)) {
    if($null -eq $row.run_id -and $AllowLegacyUnbound){$TogetherIdentityVerified=$false;continue}
    if($row.run_id -isnot [string] -or $row.run_id -cne $TogetherExpected.ToString()){throw 'Proof observation run ID mismatch'}
}
# Index steady PC samples by generation. Binary search keeps a two-hour run from
# performing billions of repeated comparisons on the user's PC.
$TogetherIndex=@{}
foreach($pcRow in $TogetherPc) {
    if($pcRow.PSObject.Properties.Name -contains 'position_read_ms') {
        $value=$pcRow.position_read_ms
        if($null -eq $value -or $value -is [string] -or $value -is [bool] -or $value -isnot [ValueType] -or [double]::IsNaN([double]$value) -or [double]::IsInfinity([double]$value) -or $value -lt 0){throw 'Invalid PC position read duration'}
    }
    if($pcRow.paused -or $pcRow.preparing -or $pcRow.buffering -or $pcRow.seeking) {continue}
    $key=[string]$pcRow.generation
    if(!$TogetherIndex.ContainsKey($key)) {$TogetherIndex[$key]=[System.Collections.Generic.List[object]]::new()}
    $TogetherIndex[$key].Add($pcRow)
}
$TogetherReadDurations=[System.Collections.Generic.List[double]]::new()
$TogetherSignedDifferences=[System.Collections.Generic.List[double]]::new()
$TogetherDifferences=@(foreach($row in $TogetherPhone) {
    if($SteadyOnly -and $row.phase -ne 'steady') {continue}
    if($row.state -ne 'Playing' -or $row.preparing -or $row.buffering -or $row.seeking -or $null -eq $row.offset_ms) {continue}
    $observed=if($row.PSObject.Properties.Name -contains 'sampled_at_ms') {$row.sampled_at_ms} else {$row.client_ms}
    $serverTime=$observed+$row.offset_ms
    $samples=$TogetherIndex[[string]$row.generation]
    if(!$samples -or $samples.Count -eq 0) {continue}
    $low=0;$high=$samples.Count
    while($low -lt $high) {$middle=[int][math]::Floor(($low+$high)/2);if($samples[$middle].server_ms -lt $serverTime){$low=$middle+1}else{$high=$middle}}
    $near=$null;$distance=120.0
    foreach($candidate in @(($low-1),$low)) {
        if($candidate -ge 0 -and $candidate -lt $samples.Count) {
            $delta=[math]::Abs($samples[$candidate].server_ms-$serverTime)
            if($delta -lt $distance) {$near=$samples[$candidate];$distance=$delta}
        }
    }
    if($near) {
        if($near.PSObject.Properties.Name -contains 'position_read_ms') {$TogetherReadDurations.Add([double]$near.position_read_ms)}
        $signedDifference=$row.position_ms-($near.position_ms+($serverTime-$near.server_ms)*$near.rate)
        $TogetherSignedDifferences.Add($signedDifference)
        [math]::Abs($signedDifference)
    }
})
$TogetherSorted=@($TogetherDifferences | Sort-Object)
$TogetherReadSorted=@($TogetherReadDurations | Sort-Object)
$TogetherSignedSorted=@($TogetherSignedDifferences | Sort-Object)
$TogetherRtts=@($TogetherPhone | Where-Object {$_.connected -and $null -ne $_.rtt_ms -and (!$SteadyOnly -or $_.phase -eq 'steady')} | ForEach-Object {$_.rtt_ms} | Sort-Object)
if($TogetherSorted.Count -lt 20) {throw 'Insufficient matching real-player samples for drift comparison'}
$TogetherSummary=[pscustomobject]@{
    run_id=$TogetherExpected.ToString()
    log_identity_verified=$TogetherIdentityVerified
    ignored_unterminated_pc_tail=$TogetherIgnoredPcTail
    sample_scope=if($SteadyOnly){'explicit_steady_phase'}else{'all_eligible_playing_samples'}
    matched_samples=$TogetherSorted.Count
    maximum_ms=($TogetherSorted | Measure-Object -Maximum).Maximum
    median_ms=$TogetherSorted[[int]($TogetherSorted.Count*.5)]
    p95_ms=$TogetherSorted[[int]($TogetherSorted.Count*.95)]
    provisional_cross_device_p95_goal_ms=150
    provisional_cross_device_p95_goal_met=($TogetherSorted[[int]($TogetherSorted.Count*.95)] -le 150)
    signed_difference_definition='Phone position minus interpolated PC position; positive means phone ahead. Clock-offset error remains.'
    signed_median_ms=$TogetherSignedSorted[[int]($TogetherSignedSorted.Count*.5)]
    phone_ahead_samples=@($TogetherSignedDifferences | Where-Object {$_ -gt 0}).Count
    phone_behind_samples=@($TogetherSignedDifferences | Where-Object {$_ -lt 0}).Count
    equal_position_samples=@($TogetherSignedDifferences | Where-Object {$_ -eq 0}).Count
    matched_pc_position_read_samples=$TogetherReadSorted.Count
    matched_pc_position_read_p95_ms=if($TogetherReadSorted.Count){$TogetherReadSorted[[int]($TogetherReadSorted.Count*.95)]}else{$null}
    matched_pc_position_read_max_ms=if($TogetherReadSorted.Count){$TogetherReadSorted[-1]}else{$null}
    measured_clock_rtt_median_ms=if($TogetherRtts.Count){$TogetherRtts[[int]($TogetherRtts.Count*.5)]}else{$null}
    measured_clock_rtt_maximum_ms=if($TogetherRtts.Count){$TogetherRtts[-1]}else{$null}
    method='Same generation; nearest PC sample within 120 ms, interpolated with observed rate. Offset and property-sampling error remain.'
}
$TogetherSummary | ConvertTo-Json | Set-Content -LiteralPath $OutputPath
$TogetherSummary | Format-List
