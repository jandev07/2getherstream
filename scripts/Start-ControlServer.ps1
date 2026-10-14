$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'Use-LocalToolchain.ps1')
if (!$env:TOGETHER_PC_TOKEN -or !$env:TOGETHER_PHONE_TOKEN) {
    throw 'Set two separate random TOGETHER_PC_TOKEN and TOGETHER_PHONE_TOKEN environment variables (32+ characters). They will not be printed.'
}
Push-Location $TogetherRoot
try {
    & cargo run -p together-session-server
    if ($LASTEXITCODE -ne 0) { throw 'Control server failed' }
} finally { Pop-Location }
