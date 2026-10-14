$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Use-LocalToolchain.ps1')
Push-Location $TogetherRoot
try {
    & cargo test -p together-sync-core -p together-session-server -p together-player-mpv
    if ($LASTEXITCODE -ne 0) { throw 'Core tests failed' }
} finally { Pop-Location }
