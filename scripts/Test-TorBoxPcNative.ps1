param([Parameter(Mandatory)][string]$CandidateDirectory,[string]$RustTargetDirectory)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Use-LocalToolchain.ps1')
. (Join-Path $PSScriptRoot 'NativeProofLock.ps1')
. (Join-Path $PSScriptRoot 'WindowsProofBuild.ps1')
$TogetherCandidate=[IO.Path]::GetFullPath($CandidateDirectory)
Assert-WindowsProofPath -Root (Join-Path $TogetherRoot 'work/private-alpha-candidates') -Path $TogetherCandidate
$TogetherTarget=Join-Path $TogetherCandidate 'rust-target'
if($RustTargetDirectory){
    $TogetherTarget=[IO.Path]::GetFullPath($RustTargetDirectory,$TogetherRoot)
    Assert-WindowsProofPath -Root (Join-Path $TogetherRoot 'work/private-alpha-candidates') -Path $TogetherTarget
    if((Split-Path -Leaf $TogetherTarget) -cne 'rust-target'){throw 'Invalid isolated compiler cache'}
}
$TogetherDll=Join-Path $TogetherCandidate 'package/Windows/native/vendor/windows/libmpv-2.dll'
Assert-WindowsProofPath -Root $TogetherCandidate -Path $TogetherDll
$TogetherBuild=Get-Content -LiteralPath (Join-Path $TogetherCandidate 'candidate-build.json') -Raw | ConvertFrom-Json
$TogetherDllHash=(Get-FileHash -LiteralPath $TogetherDll).Hash.ToLowerInvariant()
if($TogetherBuild.files.Where({$_.file -ceq 'Windows/native/vendor/windows/libmpv-2.dll'}).Count -ne 1 -or
    $TogetherBuild.files.Where({$_.file -ceq 'Windows/native/vendor/windows/libmpv-2.dll'})[0].sha256 -cne $TogetherDllHash){throw 'Candidate runtime hash mismatch'}
$TogetherPcLock=Enter-NativeProofLock -Path (Join-Path $TogetherRoot 'work/native-proof-owner.lock')
$TogetherRunId=[guid]::NewGuid().ToString()
$TogetherRunDirectory=Join-Path $TogetherRoot ('work/pc-provider-native/'+$TogetherRunId)
$TogetherMarker=Join-Path $TogetherRoot 'work/native-proof-running.json'
$TogetherRun=[ordered]@{run_id=$TogetherRunId;kind='pc_only_torbox_native';runner_pid=$PID;runner_started_utc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o');candidate_id=$TogetherBuild.candidate_id;dll_sha256=$TogetherDllHash;status='running';android_handoff=$false;physical_gate_passed=$false}
try {
    New-Item -ItemType Directory -Path $TogetherRunDirectory -Force | Out-Null
    [IO.File]::WriteAllText($TogetherMarker,(ConvertTo-Json -InputObject $TogetherRun),[Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $TogetherRunDirectory 'receipt.json'),(ConvertTo-Json -InputObject $TogetherRun),[Text.UTF8Encoding]::new($false))
    Push-Location $TogetherRoot
    try {
        cargo build --manifest-path apps/windows/src-tauri/Cargo.toml --example torbox-native-proof --features custom-protocol,private-alpha-account-links --target-dir $TogetherTarget *> (Join-Path $TogetherRunDirectory 'build.log')
        if($LASTEXITCODE -ne 0){throw 'PC native provider proof build failed'}
        & (Join-Path $TogetherTarget 'debug/examples/torbox-native-proof.exe') $TogetherDll *> (Join-Path $TogetherRunDirectory 'result.json')
        $TogetherNativeExit=$LASTEXITCODE
    }finally {Pop-Location}
    $TogetherNative=Get-Content -LiteralPath (Join-Path $TogetherRunDirectory 'result.json') -Raw | ConvertFrom-Json
    $TogetherRun.status=$TogetherNative.status;$TogetherRun.completed_utc=[datetime]::UtcNow.ToString('o')
    $TogetherRun.native=$TogetherNative
    [IO.File]::WriteAllText((Join-Path $TogetherRunDirectory 'receipt.json'),(ConvertTo-Json -InputObject $TogetherRun -Depth 6),[Text.UTF8Encoding]::new($false))
    if($TogetherNativeExit -ne 0 -or $TogetherNative.status -cne 'passed'){throw 'PC native provider check failed; inspect its fixed-code receipt'}
    Write-Output "PC-only native provider receipt: $TogetherRunDirectory"
}catch {
    $TogetherRun.status='failed';$TogetherRun.completed_utc=[datetime]::UtcNow.ToString('o')
    [IO.File]::WriteAllText((Join-Path $TogetherRunDirectory 'receipt.json'),(ConvertTo-Json -InputObject $TogetherRun -Depth 6),[Text.UTF8Encoding]::new($false))
    throw 'PC-only native provider proof failed. Standard native-pair receipts and user installs are unchanged.'
}finally {
    if(Test-Path -LiteralPath $TogetherMarker) {
        $TogetherOwner=Get-Content -LiteralPath $TogetherMarker -Raw | ConvertFrom-Json
        if($TogetherOwner.run_id -ceq $TogetherRunId){Remove-Item -LiteralPath $TogetherMarker}
    }
    $TogetherPcLock.Dispose()
}
