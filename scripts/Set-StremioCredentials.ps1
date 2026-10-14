# Local PC-only encrypted credential intake. Neither value is echoed or accepts a CLI argument.
$ErrorActionPreference='Stop'
$TogetherUser=Read-Host 'Stremio login (hidden)' -AsSecureString
$TogetherPassword=Read-Host 'Stremio password (hidden)' -AsSecureString
$TogetherPayload=$null
try {
    if($TogetherUser.Length -eq 0 -or $TogetherPassword.Length -eq 0) {throw 'Both login and password are required'}
    $TogetherCredential=[pscredential]::new('local',$TogetherUser)
    $TogetherLogin=$TogetherCredential.GetNetworkCredential().Password
    $TogetherCredential=[pscredential]::new('local',$TogetherPassword)
    $TogetherJson=@{email=$TogetherLogin;password=$TogetherCredential.GetNetworkCredential().Password} | ConvertTo-Json -Compress
    $TogetherPayload=ConvertTo-SecureString -String $TogetherJson -AsPlainText -Force
    $TogetherDirectory=Join-Path (Split-Path -Parent $PSScriptRoot) '.secrets'
    New-Item -ItemType Directory -Path $TogetherDirectory -Force | Out-Null
    ConvertFrom-SecureString -SecureString $TogetherPayload | Set-Content -LiteralPath (Join-Path $TogetherDirectory 'stremio.dpapi') -NoNewline -Encoding UTF8
    Write-Output 'Stremio credentials stored encrypted for this Windows user. Integration is deferred until the playback gate.'
} finally {
    $TogetherUser.Dispose();$TogetherPassword.Dispose()
    if($TogetherPayload) {$TogetherPayload.Dispose()}
    $TogetherJson=$null;$TogetherLogin=$null;$TogetherCredential=$null
}
