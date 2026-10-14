$ErrorActionPreference='Stop'
$TogetherRunId=[guid]::NewGuid().ToString()
$TogetherAnalysisTest=Join-Path ([System.IO.Path]::GetTempPath()) ('together-analysis-'+[guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TogetherAnalysisTest | Out-Null
try {
    $phonePath=Join-Path $TogetherAnalysisTest 'phone.jsonl'
    $pcPath=Join-Path $TogetherAnalysisTest 'pc.jsonl'
    $outputPath=Join-Path $TogetherAnalysisTest 'summary.json'
    $pc=@(for($i=0;$i -lt 20;$i++) {
        @{run_id=$TogetherRunId;server_ms=1000+$i*100;position_ms=1000+$i*100;generation=1;rate=1.0;paused=$false;preparing=$false;buffering=$false;seeking=$false} | ConvertTo-Json -Compress
    })
    $pc | Set-Content -LiteralPath $pcPath
    foreach($highDrift in @($false,$true)) {
        @(for($i=0;$i -lt 20;$i++) {
            $position=1000+$i*100+$(if($highDrift){200}else{$i})
            # Receipt is delayed by 50 ms; comparison must use observation time.
            @{run_id=$TogetherRunId;client_ms=1050+$i*100;sampled_at_ms=1000+$i*100;position_ms=$position;generation=1;offset_ms=0;connected=$true;rtt_ms=40;state='Playing';preparing=$false;buffering=$false;seeking=$false} | ConvertTo-Json -Compress
        }) | Set-Content -LiteralPath $phonePath
        & (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $phonePath -PcLog $pcPath -OutputPath $outputPath -ExpectedRunId $TogetherRunId | Out-Null
        $summary=Get-Content -LiteralPath $outputPath -Raw | ConvertFrom-Json
        $expectedMax=if($highDrift){200}else{19}
        $expectedMedian=if($highDrift){200}else{10}
        if($summary.matched_samples -ne 20 -or $summary.maximum_ms -ne $expectedMax -or $summary.median_ms -ne $expectedMedian -or $summary.p95_ms -ne $expectedMax) {throw 'Incorrect native sample comparison statistics'}
        if($summary.provisional_cross_device_p95_goal_ms -ne 150 -or $summary.provisional_cross_device_p95_goal_met -ne (!$highDrift)) {throw 'Incorrect provisional quality classification'}
    }
    # Historical logs without observation timestamps retain the receipt fallback.
    @([System.IO.File]::ReadAllLines($phonePath) | ForEach-Object {
        $row=$_ | ConvertFrom-Json;$row.client_ms=$row.sampled_at_ms;$row.PSObject.Properties.Remove('sampled_at_ms');$row | ConvertTo-Json -Compress
    }) | Set-Content -LiteralPath $phonePath
    & (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $phonePath -PcLog $pcPath -OutputPath $outputPath -ExpectedRunId $TogetherRunId | Out-Null
    if((Get-Content -LiteralPath $outputPath -Raw | ConvertFrom-Json).maximum_ms -ne 200) {throw 'Historical receipt fallback changed'}
    # Mismatched identity in a normally skipped/paused sample must still reject.
    $saved=[IO.File]::ReadAllLines($pcPath)
    $bad=@{run_id=[guid]::NewGuid().ToString();paused=$true}|ConvertTo-Json -Compress
    Add-Content -LiteralPath $pcPath -Value $bad
    $rejected=$false
    try {& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $phonePath -PcLog $pcPath -OutputPath $outputPath -ExpectedRunId $TogetherRunId|Out-Null}catch{$rejected=$true}
    if(!$rejected){throw 'Mixed PC run identity accepted'}
    $saved|Set-Content -LiteralPath $pcPath
    $TogetherComplete=[IO.File]::ReadAllText($pcPath)
    [IO.File]::AppendAllText($pcPath,'{"device":"pc","position_ms":')
    $rejected=$false
    try {& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $phonePath -PcLog $pcPath -OutputPath $outputPath -ExpectedRunId $TogetherRunId|Out-Null}catch{$rejected=$true}
    if(!$rejected){throw 'Malformed PC tail accepted without explicit recovery'}
    & (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $phonePath -PcLog $pcPath -OutputPath $outputPath -ExpectedRunId $TogetherRunId -IgnoreUnterminatedPcTail|Out-Null
    $summary=Get-Content $outputPath -Raw|ConvertFrom-Json
    if(!$summary.ignored_unterminated_pc_tail -or $summary.matched_samples -ne 20 -or $summary.maximum_ms -ne 200){throw 'Incorrect explicit tail recovery'}
    if([IO.File]::ReadAllText($pcPath) -cne ($TogetherComplete+'{"device":"pc","position_ms":')){throw 'Tail recovery modified raw evidence'}
    foreach($TogetherTail in @(('broken record'+"`n"),$bad)) {
        [IO.File]::WriteAllText($pcPath,$TogetherComplete+$TogetherTail)
        $rejected=$false
        try {& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $phonePath -PcLog $pcPath -OutputPath $outputPath -ExpectedRunId $TogetherRunId -IgnoreUnterminatedPcTail|Out-Null}catch{$rejected=$true}
        if(!$rejected){throw 'Complete malformed or valid mismatched final record was suppressed'}
    }
    $TogetherValidTail=@{run_id=$TogetherRunId;paused=$true}|ConvertTo-Json -Compress
    [IO.File]::WriteAllText($pcPath,$TogetherComplete+$TogetherValidTail)
    & (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $phonePath -PcLog $pcPath -OutputPath $outputPath -ExpectedRunId $TogetherRunId -IgnoreUnterminatedPcTail|Out-Null
    if((Get-Content $outputPath -Raw|ConvertFrom-Json).ignored_unterminated_pc_tail){throw 'Valid final record was discarded'}
    [IO.File]::WriteAllText($pcPath,$TogetherComplete)
    & (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $phonePath -PcLog $pcPath -OutputPath $outputPath -ExpectedRunId $TogetherRunId -IgnoreUnterminatedPcTail|Out-Null
    if((Get-Content $outputPath -Raw|ConvertFrom-Json).ignored_unterminated_pc_tail){throw 'Absent partial tail claimed ignored'}
    foreach($path in @($pcPath,$phonePath)) {
        @([IO.File]::ReadAllLines($path)|ForEach-Object {$r=$_|ConvertFrom-Json;$r.PSObject.Properties.Remove('run_id');$r|ConvertTo-Json -Compress})|Set-Content -LiteralPath $path
    }
    $rejected=$false
    try {& (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $phonePath -PcLog $pcPath -OutputPath $outputPath -ExpectedRunId $TogetherRunId|Out-Null}catch{$rejected=$true}
    if(!$rejected){throw 'Unbound historical logs accepted implicitly'}
    & (Join-Path $PSScriptRoot 'Analyze-SessionProof.ps1') -PhoneLog $phonePath -PcLog $pcPath -OutputPath $outputPath -ExpectedRunId $TogetherRunId -AllowLegacyUnbound|Out-Null
    if((Get-Content $outputPath -Raw|ConvertFrom-Json).log_identity_verified){throw 'Unbound historical identity claimed verified'}
    'Native analysis checks passed: observation timestamps, exact statistics, provisional goal, historical fallback and explicit partial-tail recovery preserving raw bytes and complete-record identity checks'
} finally {
    # Only remove this explicitly created unique temporary test directory.
    $resolved=[System.IO.Path]::GetFullPath($TogetherAnalysisTest)
    $temporaryRoot=[System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
    if(!$resolved.StartsWith($temporaryRoot,[System.StringComparison]::OrdinalIgnoreCase) -or !(Split-Path $resolved -Leaf).StartsWith('together-analysis-')) {throw 'Unsafe analysis-test cleanup path'}
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
