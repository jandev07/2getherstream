$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Use-LocalToolchain.ps1')
Push-Location (Join-Path $TogetherRoot 'apps\windows')
try {
    if (!(Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'Rust is missing from PATH. See README toolchain setup.' }
    if (!(Test-Path -LiteralPath 'node_modules')) { & npm.cmd ci; if ($LASTEXITCODE -ne 0) { throw 'Frontend install failed' } }
    & npm.cmd run tauri dev
    if ($LASTEXITCODE -ne 0) { throw 'Windows development launch failed' }
} finally { Pop-Location }
