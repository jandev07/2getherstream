# Removes reproducible compiler data only. Reports, receipts, APK/EXE outputs,
# source archives, media, emulator disks and credentials are never selected.
param([switch]$Apply)
$ErrorActionPreference='Stop'
$TogetherRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..')).TrimEnd('\','/')
. (Join-Path $PSScriptRoot 'NativeProofLock.ps1')
$TogetherLease=Enter-NativeProofLock -Path (Join-Path $TogetherRoot 'work/native-proof-owner.lock')
$TogetherControlLease=$null
try {
    $TogetherControlMarker=Join-Path $TogetherRoot 'work/control-soak-running.json'
    if(Test-Path -LiteralPath $TogetherControlMarker){throw 'Control marker exists; inspect its PID/start before cleanup.'}
    $TogetherControlLease=[IO.FileStream]::new((Join-Path $TogetherRoot 'work/control-soak-owner.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
    if(Get-Process cargo,rustc,java,gradle -ErrorAction SilentlyContinue){throw 'Compiler process present; cleanup deferred.'}
    $TogetherTargets=[Collections.Generic.List[string]]::new()
    $TogetherTargets.Add('target/debug/incremental')
    $TogetherTargets.Add('work/independent-source-validation/debug/incremental')
    $TogetherTargets.Add('work/header-validation-target/debug/incremental')
    $TogetherCandidates=Join-Path $TogetherRoot 'work/private-alpha-candidates'
    foreach($TogetherCandidate in Get-ChildItem -LiteralPath $TogetherCandidates -Directory){
        if($TogetherCandidate.Name -notmatch '^\d{8}-\d{6}-[a-f0-9]{8}$'){continue}
        # Retain the shared recent Rust cache used by current Alpha development.
        if($TogetherCandidate.Name -ne '20261012-135012-d336360a'){
            $TogetherTargets.Add('work/private-alpha-candidates/'+$TogetherCandidate.Name+'/rust-target')
            $TogetherTargets.Add('work/private-alpha-candidates/'+$TogetherCandidate.Name+'/android-build/app/intermediates')
            $TogetherTargets.Add('work/private-alpha-candidates/'+$TogetherCandidate.Name+'/android-build/app/tmp')
        }
    }
    $TogetherRows=@()
    foreach($TogetherRelative in $TogetherTargets){
        $TogetherPath=[IO.Path]::GetFullPath((Join-Path $TogetherRoot $TogetherRelative))
        if(!$TogetherPath.StartsWith($TogetherRoot+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){throw 'Cleanup target escaped repository'}
        if(!(Test-Path -LiteralPath $TogetherPath)){continue}
        # Validate all ancestors and descendants before recursive removal. Do not
        # follow a junction into another checkout, toolchain or user directory.
        $TogetherAncestor=Get-Item -LiteralPath $TogetherPath
        while($TogetherAncestor.FullName -ne $TogetherRoot){
            if($TogetherAncestor.Attributes -band [IO.FileAttributes]::ReparsePoint){throw 'Reparse point in cleanup target'}
            $TogetherAncestor=$TogetherAncestor.Parent
        }
        $TogetherItems=@(Get-ChildItem -LiteralPath $TogetherPath -Force -Recurse)
        if($TogetherItems | Where-Object {$_.Attributes -band [IO.FileAttributes]::ReparsePoint}){throw 'Reparse point inside cleanup target'}
        $TogetherBytes=($TogetherItems | Where-Object {!$_.PSIsContainer} | Measure-Object Length -Sum).Sum
        $TogetherRows+= [pscustomobject]@{path=$TogetherRelative;bytes=[long]$TogetherBytes;removed=[bool]$Apply}
        if($Apply){Remove-Item -LiteralPath $TogetherPath -Recurse -Force}
    }
    $TogetherReceipt=[pscustomobject]@{utc=[datetime]::UtcNow.ToString('o');applied=[bool]$Apply;bytes=($TogetherRows|Measure-Object bytes -Sum).Sum;targets=$TogetherRows;shared_rust_cache_preserved=$true;packages_and_evidence_preserved=$true}
    $TogetherReceipt | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $TogetherRoot ('work/storage-cleanup-'+[datetime]::UtcNow.ToString('yyyyMMdd-HHmmss')+'.json')) -Encoding utf8
    $TogetherReceipt | Select-Object utc,applied,bytes,shared_rust_cache_preserved,packages_and_evidence_preserved
} finally {
    if($TogetherControlLease){$TogetherControlLease.Dispose()}
    $TogetherLease.Dispose()
}
