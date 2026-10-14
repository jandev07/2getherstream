$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'WindowsProofBuild.ps1')
$TogetherFixture=Join-Path ([IO.Path]::GetTempPath()) ('together-windows-build-'+[guid]::NewGuid())
$TogetherCreated=[Collections.Generic.List[string]]::new()
function Write-Fixture([string]$Name,[string]$Text) {
    $TogetherPath=Join-Path $TogetherFixture $Name
    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($TogetherPath)) | Out-Null
    if(!(Test-Path -LiteralPath $TogetherPath)){$TogetherCreated.Add($TogetherPath)}
    [IO.File]::WriteAllText($TogetherPath,$Text)
}
function Assert-Stale {
    $TogetherAccepted=$false;$TogetherError=$null
    try {$null=Assert-WindowsProofBuild -Root $TogetherFixture;$TogetherAccepted=$true}catch{$TogetherError=$_.Exception.Message}
    if($TogetherAccepted -or $TogetherError -cne 'Windows proof artifacts are missing or stale. Build the current sources before using SkipBuild.'){throw 'Stale Windows build rejection failed'}
}
try {
    foreach($TogetherName in @('Cargo.toml','Cargo.lock','apps/windows/package.json','apps/windows/package-lock.json','apps/windows/index.html',
        'apps/windows/app-icon.svg','apps/windows/vite.config.ts','apps/windows/tsconfig.json','apps/windows/svelte.config.js',
        'apps/windows/src-tauri/Cargo.toml','apps/windows/src-tauri/build.rs','apps/windows/src-tauri/tauri.conf.json')) {Write-Fixture $TogetherName 'synthetic build configuration'}
    foreach($TogetherName in @('apps/windows/src','apps/windows/src-tauri/src','apps/windows/src-tauri/capabilities','apps/windows/src-tauri/icons')){Write-Fixture ($TogetherName+'/fixture') 'synthetic source'}
    foreach($TogetherCrate in @('sync-core','player-mpv','session-server','source-resolver')) {
        Write-Fixture ('crates/'+$TogetherCrate+'/Cargo.toml') 'synthetic manifest'
        Write-Fixture ('crates/'+$TogetherCrate+'/src/fixture.rs') 'synthetic Rust source'
    }
    foreach($TogetherName in @('apps/windows/dist/index.html','target/debug/together2-windows.exe','native/vendor/windows/libmpv-2.dll')){Write-Fixture $TogetherName 'not executable; checksum fixture'}
    [IO.Directory]::CreateDirectory((Join-Path $TogetherFixture 'work')) | Out-Null
    Assert-Stale
    $TogetherBefore=Get-WindowsProofSources -Root $TogetherFixture
    $TogetherFrontend=Get-WindowsProofFrontend -Root $TogetherFixture
    Write-WindowsProofBuild -Root $TogetherFixture -BeforeSources $TogetherBefore -BeforeFrontend $TogetherFrontend
    $TogetherReceiptPath=Join-Path $TogetherFixture 'work/windows-proof-build.json';$TogetherCreated.Add($TogetherReceiptPath)
    $TogetherOriginal=[IO.File]::ReadAllText($TogetherReceiptPath)
    $TogetherReceipt=Assert-WindowsProofBuild -Root $TogetherFixture
    if($TogetherReceipt.sources.file_count -ne 24 -or $TogetherReceipt.frontend.file_count -ne 1 -or $TogetherReceipt.artifacts.Count -ne 2 -or $TogetherReceipt.authenticated -or $TogetherReceipt.startup_attestation){throw 'Windows build association failed'}
    foreach($TogetherName in @('Cargo.lock','apps/windows/package-lock.json','apps/windows/src/fixture','apps/windows/src-tauri/tauri.conf.json',
        'crates/session-server/src/fixture.rs','crates/player-mpv/Cargo.toml','apps/windows/dist/index.html','target/debug/together2-windows.exe','native/vendor/windows/libmpv-2.dll')) {
        $TogetherPath=Join-Path $TogetherFixture $TogetherName;$TogetherSaved=[IO.File]::ReadAllText($TogetherPath)
        Write-Fixture $TogetherName 'changed fixture';Assert-Stale
        if([IO.File]::ReadAllText($TogetherReceiptPath) -cne $TogetherOriginal){throw 'Validation replaced receipt'}
        if($TogetherName -in @('crates/session-server/src/fixture.rs','apps/windows/dist/index.html')) {
            $TogetherRejected=$false
            try {Write-WindowsProofBuild -Root $TogetherFixture -BeforeSources $TogetherBefore -BeforeFrontend $TogetherFrontend}catch{$TogetherRejected=$true}
            if(!$TogetherRejected -or [IO.File]::ReadAllText($TogetherReceiptPath) -cne $TogetherOriginal){throw 'Mid-build input change replaced receipt'}
        }
        Write-Fixture $TogetherName $TogetherSaved
    }
    Write-Fixture 'apps/windows/src/new' 'new source';Assert-Stale
    Remove-Item -LiteralPath (Join-Path $TogetherFixture 'apps/windows/src/new')
    $TogetherSource=Join-Path $TogetherFixture 'apps/windows/src/fixture';$TogetherRenamed=Join-Path $TogetherFixture 'apps/windows/src/renamed'
    Move-Item -LiteralPath $TogetherSource -Destination $TogetherRenamed
    try {Assert-Stale}finally{Move-Item -LiteralPath $TogetherRenamed -Destination $TogetherSource}
    Remove-Item -LiteralPath $TogetherSource
    Assert-Stale;Write-Fixture 'apps/windows/src/fixture' 'synthetic source'
    Write-Fixture 'config/private-fixture.json' 'PRIVATE_SYNTHETIC_INPUT'
    Write-Fixture 'apps/windows/node_modules/private-fixture' 'PRIVATE_SYNTHETIC_DEPENDENCY'
    $null=Assert-WindowsProofBuild -Root $TogetherFixture
    foreach($TogetherName in @('apps/windows/src/redirected','apps/windows/dist/redirected')) {
        $TogetherLink=Join-Path $TogetherFixture $TogetherName
        $TogetherTarget=Join-Path $TogetherFixture 'config'
        New-Item -ItemType Junction -Path $TogetherLink -Target $TogetherTarget | Out-Null
        try {Assert-Stale}finally{[IO.Directory]::Delete($TogetherLink)}
        if([IO.File]::ReadAllText((Join-Path $TogetherTarget 'private-fixture.json')) -cne 'PRIVATE_SYNTHETIC_INPUT'){throw 'Redirect rejection changed its target'}
    }
    foreach($TogetherMalformed in @('{}','[]','{"version":true}',('x'*65537))) {
        [IO.File]::WriteAllText($TogetherReceiptPath,$TogetherMalformed);Assert-Stale
        if([IO.File]::ReadAllText($TogetherReceiptPath) -cne $TogetherMalformed){throw 'Malformed receipt modified'}
    }
    [IO.File]::WriteAllText($TogetherReceiptPath,$TogetherOriginal)
    $TogetherBad=$TogetherOriginal | ConvertFrom-Json;$TogetherBad.artifacts[0].file='PRIVATE_SYNTHETIC_PATH'
    [IO.File]::WriteAllText($TogetherReceiptPath,($TogetherBad|ConvertTo-Json -Depth 5));Assert-Stale
    foreach($TogetherField in @('authenticated','startup_attestation','version')) {
        $TogetherBad=$TogetherOriginal|ConvertFrom-Json
        $TogetherBad.$TogetherField=if($TogetherField -eq 'version'){'1'}else{$true}
        [IO.File]::WriteAllText($TogetherReceiptPath,($TogetherBad|ConvertTo-Json -Depth 5));Assert-Stale
    }
    foreach($TogetherKind in @('sources','frontend')) {
        $TogetherBad=$TogetherOriginal|ConvertFrom-Json;$TogetherBad.$TogetherKind.file_count=$TogetherBad.$TogetherKind.file_count.ToString()
        [IO.File]::WriteAllText($TogetherReceiptPath,($TogetherBad|ConvertTo-Json -Depth 5));Assert-Stale
    }
    $TogetherBad=$TogetherOriginal|ConvertFrom-Json;$TogetherBad.recorded_utc='PRIVATE_SYNTHETIC_DATE'
    [IO.File]::WriteAllText($TogetherReceiptPath,($TogetherBad|ConvertTo-Json -Depth 5));Assert-Stale
    $TogetherBad=$TogetherOriginal|ConvertFrom-Json
    $TogetherBad|Add-Member -NotePropertyName private_input -NotePropertyValue 'PRIVATE_SYNTHETIC_FIELD'
    $TogetherBad.artifacts[0]|Add-Member -NotePropertyName private_input -NotePropertyValue 'PRIVATE_SYNTHETIC_FIELD'
    [IO.File]::WriteAllText($TogetherReceiptPath,($TogetherBad|ConvertTo-Json -Depth 5))
    $TogetherChecked=Assert-WindowsProofBuild -Root $TogetherFixture
    if(($TogetherChecked|ConvertTo-Json -Depth 5) -match 'PRIVATE_SYNTHETIC_FIELD'){throw 'Unknown receipt fields escaped checked schema'}
    [IO.File]::WriteAllText($TogetherReceiptPath,$TogetherOriginal)
    $null=Assert-WindowsProofBuild -Root $TogetherFixture
    'Windows build association checks passed: Rust/UI/config/manifests/lockfiles/frontend/executable/DLL changes, additions/removal/rename, bounded malformed metadata, preserved receipts and excluded private/cache inputs.'
}finally {
    foreach($TogetherPath in $TogetherCreated){if(Test-Path -LiteralPath $TogetherPath){Remove-Item -LiteralPath $TogetherPath}}
    if(Test-Path -LiteralPath $TogetherFixture) {
        Get-ChildItem -LiteralPath $TogetherFixture -Directory -Recurse | Sort-Object FullName -Descending | ForEach-Object {Remove-Item -LiteralPath $_.FullName}
        Remove-Item -LiteralPath $TogetherFixture
    }
}
