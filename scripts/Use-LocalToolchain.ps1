# Dot-source this file. Uses normal PATH unless a local toolchain configuration exists.
$TogetherRoot = Split-Path -Parent $PSScriptRoot
$TogetherConfigPath = Join-Path $TogetherRoot 'config\toolchains.local.json'
$TogetherTools = $null
if (Test-Path -LiteralPath $TogetherConfigPath) {
    $TogetherTools = Get-Content -LiteralPath $TogetherConfigPath -Raw | ConvertFrom-Json
    if ($TogetherTools.rustBin) { $env:PATH = $TogetherTools.rustBin + ';' + $env:PATH }
    if ($TogetherTools.javaHome) { $env:JAVA_HOME = $TogetherTools.javaHome }
    if ($TogetherTools.androidHome) { $env:ANDROID_HOME = $TogetherTools.androidHome }
}
$env:CARGO_HOME = if($TogetherTools.cargoHome) { $TogetherTools.cargoHome } else { Join-Path $TogetherRoot '.dev-cache\cargo' }
$env:GRADLE_USER_HOME = if($TogetherTools.gradleUserHome) { $TogetherTools.gradleUserHome } else { Join-Path $TogetherRoot '.dev-cache\gradle' }
