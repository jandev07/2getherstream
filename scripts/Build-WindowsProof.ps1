$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Use-LocalToolchain.ps1')
. (Join-Path $PSScriptRoot 'NativeProofLock.ps1')
. (Join-Path $PSScriptRoot 'WindowsProofBuild.ps1')
New-Item -ItemType Directory -Force -Path (Join-Path $TogetherRoot 'work') | Out-Null
$TogetherLock=Enter-NativeProofLock -Path (Join-Path $TogetherRoot 'work/native-proof-owner.lock')
try {
    $TogetherBeforeSources=Get-WindowsProofSources -Root $TogetherRoot
    Push-Location (Join-Path $TogetherRoot 'apps/windows')
    try {
        npm.cmd ci;if($LASTEXITCODE -ne 0){throw 'Frontend install failed'}
        npm.cmd run check;if($LASTEXITCODE -ne 0){throw 'Frontend check failed'}
        npm.cmd run build;if($LASTEXITCODE -ne 0){throw 'Frontend build failed'}
    }finally{Pop-Location}
    $TogetherBeforeFrontend=Get-WindowsProofFrontend -Root $TogetherRoot
    Push-Location $TogetherRoot
    try {
        cargo build --manifest-path apps/windows/src-tauri/Cargo.toml --features custom-protocol
        if($LASTEXITCODE -ne 0){throw 'Windows proof build failed'}
    }finally{Pop-Location}
    Write-WindowsProofBuild -Root $TogetherRoot -BeforeSources $TogetherBeforeSources -BeforeFrontend $TogetherBeforeFrontend
}finally{$TogetherLock.Dispose()}
