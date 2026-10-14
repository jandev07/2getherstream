# Store the key locally using Windows DPAPI for the current Windows user.
# No argument accepts a plaintext key, and no key is printed or committed.
$ErrorActionPreference = 'Stop'
$TogetherRepository = Split-Path -Parent $PSScriptRoot
$TogetherSecretDirectory = Join-Path $TogetherRepository '.secrets'
$TogetherKey = Read-Host 'Paste your TorBox API key (hidden)' -AsSecureString
try {
    if ($TogetherKey.Length -eq 0) { throw 'No key entered' }
    New-Item -ItemType Directory -Path $TogetherSecretDirectory -Force | Out-Null
    $TogetherEncrypted = ConvertFrom-SecureString -SecureString $TogetherKey
    Set-Content -LiteralPath (Join-Path $TogetherSecretDirectory 'torbox.dpapi') -Value $TogetherEncrypted -NoNewline -Encoding UTF8
    Write-Output 'TorBox key stored for this Windows user. The resolver is not connected yet.'
} finally { $TogetherKey.Dispose() }
