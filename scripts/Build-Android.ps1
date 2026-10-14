$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Use-LocalToolchain.ps1')
. (Join-Path $PSScriptRoot 'NativeProofLock.ps1')
. (Join-Path $PSScriptRoot 'AndroidProofBuild.ps1')
New-Item -ItemType Directory -Path (Join-Path $TogetherRoot 'work') -Force | Out-Null
$TogetherOwnershipLock=Enter-NativeProofLock -Path (Join-Path $TogetherRoot 'work/native-proof-owner.lock')
try {
$TogetherBeforeAndroid=Get-AndroidProofSources -Root $TogetherRoot
Push-Location (Join-Path $TogetherRoot 'apps\android')
try {
    & .\gradlew.bat --no-daemon :app:testDebugUnitTest :app:assembleDebug :app:assembleDebugAndroidTest
    if ($LASTEXITCODE -ne 0) { throw 'Android build failed' }
    Write-AndroidProofBuild -Root $TogetherRoot -Before $TogetherBeforeAndroid
    $TogetherReleaseDirectory = Join-Path $TogetherRoot 'releases'
    New-Item -ItemType Directory -Path $TogetherReleaseDirectory -Force | Out-Null
    Copy-Item -LiteralPath 'app\build\outputs\apk\debug\app-debug.apk' -Destination (Join-Path $TogetherReleaseDirectory 'Together2-Dev.apk')
} finally { Pop-Location }
} finally {$TogetherOwnershipLock.Dispose()}
