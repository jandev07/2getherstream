$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'NativeProofLock.ps1')
$lockPath=Join-Path ([IO.Path]::GetTempPath()) ('together-proof-lock-'+[guid]::NewGuid().ToString()+'.lock')
$owner=$null
$job=$null
try {
    $owner=Enter-NativeProofLock -Path $lockPath
    $helper=Join-Path $PSScriptRoot 'NativeProofLock.ps1'
    $job=Start-Job -ArgumentList $helper,$lockPath -ScriptBlock {
        param($helper,$lockPath)
        . $helper
        try {
            $other=Enter-NativeProofLock -Path $lockPath
            $other.Dispose()
            'unexpected_acquisition'
        } catch {'ownership_rejected'}
    }
    if (!(Wait-Job $job -Timeout 20)) {throw 'Lock contender did not finish'}
    $result=@(Receive-Job $job -ErrorAction Stop)
    if ($result.Count -ne 1 -or $result[0] -ne 'ownership_rejected') {throw 'Concurrent runner was not excluded'}
    $owner.Dispose();$owner=$null
    # A stale empty file is harmless; OS ownership ends when its handle closes.
    $owner=Enter-NativeProofLock -Path $lockPath
    $owner.Dispose();$owner=$null
    # The runner's outer finally also covers preflight/build failures.
    try {$owner=Enter-NativeProofLock -Path $lockPath;throw 'synthetic_build_failure'}
    catch {} finally {if ($owner) {$owner.Dispose();$owner=$null}}
    $owner=Enter-NativeProofLock -Path $lockPath
    'Native proof lock checks passed: concurrent runner excluded and failure ownership released.'
} finally {
    if ($owner) {$owner.Dispose()}
    if ($job) {Remove-Job $job -Force}
    if (Test-Path -LiteralPath $lockPath) {Remove-Item -LiteralPath $lockPath}
}
