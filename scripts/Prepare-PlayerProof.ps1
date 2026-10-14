param([ValidateRange(60,7500)][int]$DurationSeconds=180,[switch]$CompactSoak)
$ErrorActionPreference = 'Stop'
$TogetherRepository = Split-Path -Parent $PSScriptRoot
$TogetherWork = Join-Path $TogetherRepository 'work'
New-Item -ItemType Directory -Path $TogetherWork -Force | Out-Null
. (Join-Path $PSScriptRoot 'NativeProofLock.ps1')
$TogetherOwnershipLock=Enter-NativeProofLock -Path (Join-Path $TogetherWork 'native-proof-owner.lock')
try {
# Shared entry checks exact identity and uncertain metadata, including legacy
# runners without an OS-held lock, before any fixture or DLL changes.
$TogetherMediaDirectory = Join-Path $TogetherWork 'media'
$TogetherAssets = Join-Path $TogetherRepository 'apps\android\app\src\main\assets'
New-Item -ItemType Directory -Path $TogetherMediaDirectory,$TogetherAssets -Force | Out-Null
if (!(Get-Command ffmpeg -ErrorAction SilentlyContinue)) { throw 'FFmpeg is required to generate the local synthetic proof video.' }
$TogetherVideoSource=if($CompactSoak) {'testsrc2=size=640x360:rate=10'} else {'testsrc2=size=640x360:rate=30'}
$TogetherCrf=if($CompactSoak) {'42'} else {'23'}
& ffmpeg -hide_banner -loglevel error -f lavfi -i $TogetherVideoSource -f lavfi -i 'sine=frequency=440:sample_rate=48000' -t $DurationSeconds -c:v libx264 -preset veryfast -crf $TogetherCrf -pix_fmt yuv420p -c:a aac -b:a 32k -movflags +faststart -y (Join-Path $TogetherMediaDirectory 'clock-test.mp4')
if ($LASTEXITCODE -ne 0) { throw 'Fixture generation failed' }
Copy-Item -LiteralPath (Join-Path $TogetherMediaDirectory 'clock-test.mp4') -Destination (Join-Path $TogetherAssets 'clock-test.mp4')
$TogetherArchive = Join-Path $TogetherWork 'mpv-dev-x86_64-20261007.7z'
if (!(Test-Path -LiteralPath $TogetherArchive)) {
    Invoke-WebRequest -Uri 'https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20261007/mpv-dev-x86_64-20261007-git-eb0ee10315.7z' -OutFile $TogetherArchive
}
if ((Get-FileHash -LiteralPath $TogetherArchive -Algorithm SHA256).Hash.ToLowerInvariant() -ne '3fd93055d437310ad094d3f80c9b04f9a56e8b138bcb7d9c5cf01870abbfe5a5') { throw 'mpv checksum mismatch' }
if (!(Get-Command 7z -ErrorAction SilentlyContinue)) { throw '7-Zip or NanaZip is required to extract the verified libmpv archive.' }
$TogetherDestination = Join-Path $TogetherRepository 'native\vendor\windows'
& 7z x $TogetherArchive "-o$TogetherDestination" -y
if ($LASTEXITCODE -ne 0) { throw 'Native archive extraction failed' }
Write-Output 'Synthetic local video and verified Windows libmpv are ready.'
} finally {
    $TogetherOwnershipLock.Dispose()
}
