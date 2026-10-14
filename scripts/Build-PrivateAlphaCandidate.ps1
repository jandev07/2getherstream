# Isolated private candidate build. Does not install, launch, overwrite offered
# artifacts, or publish standard native-proof build receipts.
param([string]$RustTargetDirectory='')
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Use-LocalToolchain.ps1')
. (Join-Path $PSScriptRoot 'NativeProofLock.ps1')
. (Join-Path $PSScriptRoot 'WindowsProofBuild.ps1')
. (Join-Path $PSScriptRoot 'AndroidProofBuild.ps1')
$TogetherCandidateId=[datetime]::UtcNow.ToString('yyyyMMdd-HHmmss')+'-'+[guid]::NewGuid().ToString('N').Substring(0,8)
$TogetherCandidate=Join-Path $TogetherRoot ('work/private-alpha-candidates/'+$TogetherCandidateId)
if($RustTargetDirectory) {
    $TogetherRustTarget=[IO.Path]::GetFullPath((Join-Path $TogetherRoot $RustTargetDirectory))
    $TogetherCacheRoot=[IO.Path]::GetFullPath((Join-Path $TogetherRoot 'work/private-alpha-candidates')).TrimEnd('\','/')
    if(!$TogetherRustTarget.StartsWith($TogetherCacheRoot+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetFileName($TogetherRustTarget) -cne 'rust-target') {throw 'Candidate compiler cache must stay within isolated candidates'}
    Assert-WindowsProofPath -Root $TogetherRoot -Path $TogetherRustTarget
}else {$TogetherRustTarget=Join-Path $TogetherCandidate 'rust-target'}
$TogetherCandidateLock=Enter-NativeProofLock -Path (Join-Path $TogetherRoot 'work/native-proof-owner.lock')
$TogetherOldConfig=$env:TAURI_CONFIG
try {
    $TogetherWindowsSources=Get-WindowsProofSources -Root $TogetherRoot
    $TogetherAndroidSources=Get-AndroidProofSources -Root $TogetherRoot
    Push-Location $TogetherRoot
    try {
        git diff --quiet HEAD -- Cargo.toml Cargo.lock crates apps/windows apps/android
        if($LASTEXITCODE -ne 0){throw 'Commit compiled source changes before building a source-associated candidate'}
        $TogetherUntracked=@(git ls-files --others --exclude-standard -- Cargo.toml Cargo.lock crates apps/windows apps/android)
        if($LASTEXITCODE -ne 0 -or $TogetherUntracked.Count -ne 0){throw 'Track and commit compiled source files before building a source-associated candidate'}
        $TogetherSourceCommit=(git rev-parse HEAD).Trim()
    }finally {Pop-Location}
    $TogetherPreserved=@('target/debug/together2-windows.exe','releases/Together2-Dev.apk','apps/android/app/build/outputs/apk/debug/app-debug.apk')
    $TogetherOriginal=@(foreach($TogetherFile in $TogetherPreserved) {
        $TogetherPath=Join-Path $TogetherRoot $TogetherFile
        if(Test-Path -LiteralPath $TogetherPath -PathType Leaf){[ordered]@{file=$TogetherFile;sha256=(Get-FileHash -LiteralPath $TogetherPath).Hash}}
    })
    New-Item -ItemType Directory -Path $TogetherCandidate -Force | Out-Null
    $TogetherFrontend=Join-Path $TogetherCandidate 'frontend'
    Push-Location (Join-Path $TogetherRoot 'apps/windows')
    try {
        npm.cmd run check;if($LASTEXITCODE -ne 0){throw 'Candidate frontend check failed'}
        npm.cmd run build -- --outDir $TogetherFrontend;if($LASTEXITCODE -ne 0){throw 'Candidate frontend build failed'}
    }finally {Pop-Location}
    $TogetherFrontendInput=Get-WindowsProofInventory -Root $TogetherRoot -Directories @([IO.Path]::GetRelativePath($TogetherRoot,$TogetherFrontend))
    # C:/... deserializes as a URL. Relative paths force directory embedding.
    $TogetherFrontendRelative=[IO.Path]::GetRelativePath((Join-Path $TogetherRoot 'apps/windows/src-tauri'),$TogetherFrontend).Replace('\','/')
    if([IO.Path]::IsPathRooted($TogetherFrontendRelative) -or $TogetherFrontendRelative.Contains(':')) {throw 'Candidate frontend must be a relative directory'}
    $env:TAURI_CONFIG=ConvertTo-Json -InputObject @{build=@{frontendDist=$TogetherFrontendRelative}} -Depth 4 -Compress
    Push-Location $TogetherRoot
    try {
        cargo build -j 2 --manifest-path apps/windows/src-tauri/Cargo.toml --features custom-protocol,private-alpha-account-links --target-dir $TogetherRustTarget
        if($LASTEXITCODE -ne 0){throw 'Private Alpha Windows build failed'}
    }finally {Pop-Location}
    # Route every Gradle project output into the same isolated candidate. No
    # SDK/device changes or installation task is used.
    $TogetherGradleRoot=(Join-Path $TogetherCandidate 'android-build').Replace('\','/')
    $TogetherInit=Join-Path $TogetherCandidate 'candidate.init.gradle'
    $TogetherInitText="allprojects { layout.buildDirectory.set(file('$TogetherGradleRoot/' + (project.path == ':' ? 'root' : project.path.substring(1).replace(':', '/')))) }"
    [IO.File]::WriteAllText($TogetherInit,$TogetherInitText,[Text.UTF8Encoding]::new($false))
    Push-Location (Join-Path $TogetherRoot 'apps/android')
    try {
        & .\gradlew.bat --no-daemon --max-workers=1 --init-script $TogetherInit :app:testDebugUnitTest :app:assembleDebug :app:compileDebugAndroidTestKotlin
        if($LASTEXITCODE -ne 0){throw 'Private Alpha Android build failed'}
    }finally {Pop-Location}
    if($TogetherWindowsSources.sha256 -cne (Get-WindowsProofSources -Root $TogetherRoot).sha256){throw 'Windows source changed during candidate build'}
    if($TogetherAndroidSources.sha256 -cne (Get-AndroidProofSources -Root $TogetherRoot).sha256){throw 'Android source changed during candidate build'}
    if($TogetherFrontendInput.sha256 -cne (Get-WindowsProofInventory -Root $TogetherRoot -Directories @([IO.Path]::GetRelativePath($TogetherRoot,$TogetherFrontend))).sha256){throw 'Frontend changed during candidate build'}
    foreach($TogetherArtifact in $TogetherOriginal) {
        if($TogetherArtifact.sha256 -cne (Get-FileHash -LiteralPath (Join-Path $TogetherRoot $TogetherArtifact.file)).Hash){throw 'Offered test artifact changed during isolated build'}
    }
    $TogetherPackage=Join-Path $TogetherCandidate 'package'
    $TogetherWindows=Join-Path $TogetherPackage 'Windows'
    $TogetherNative=Join-Path $TogetherWindows 'native/vendor/windows'
    New-Item -ItemType Directory -Path $TogetherNative -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $TogetherRustTarget 'debug/together2-windows.exe') -Destination (Join-Path $TogetherWindows '2gether.exe')
    $TogetherProbe=Start-Process -FilePath (Join-Path $TogetherWindows '2gether.exe') -ArgumentList '--verify-alpha-assets' -WindowStyle Hidden -PassThru -Wait -RedirectStandardOutput (Join-Path $TogetherCandidate 'frontend-proof.stdout') -RedirectStandardError (Join-Path $TogetherCandidate 'frontend-proof.stderr')
    if($TogetherProbe.ExitCode -ne 0 -or (Get-Content -LiteralPath (Join-Path $TogetherCandidate 'frontend-proof.stdout') -Raw).Trim() -cne 'ALPHA_EMBEDDED_FRONTEND_VERIFIED') {throw 'Packaged EXE embedded frontend verification failed'}
    Copy-Item -LiteralPath (Join-Path $TogetherRoot 'native/vendor/windows/libmpv-2.dll') -Destination (Join-Path $TogetherNative 'libmpv-2.dll')
    $TogetherConnector=Join-Path $TogetherRoot 'native/vendor/windows/cloudflared.exe'
    if((Get-FileHash -LiteralPath $TogetherConnector).Hash.ToLowerInvariant() -cne 'f096265ec2fcbe9bb6e2d64268db167ced3fcbb83d894bdb9e2fcdb26f2ea7e2'){throw 'Private Alpha connector integrity mismatch'}
    Copy-Item -LiteralPath $TogetherConnector -Destination (Join-Path $TogetherNative 'cloudflared.exe')
    Copy-Item -LiteralPath (Join-Path $TogetherGradleRoot 'app/outputs/apk/debug/app-debug.apk') -Destination (Join-Path $TogetherPackage '2gether-PrivateAlpha.apk')
    $TogetherAapt=Join-Path $env:ANDROID_HOME 'build-tools/35.0.0/aapt.exe'
    $TogetherApkHeader=@(& $TogetherAapt dump badging (Join-Path $TogetherPackage '2gether-PrivateAlpha.apk'))[0]
    if($LASTEXITCODE -ne 0 -or $TogetherApkHeader -notmatch "^package: name='(app\.together\.v2)' versionCode='([0-9]+)' versionName='([a-zA-Z0-9.\-]+)' "){throw 'Candidate APK identity could not be verified'}
    $TogetherAndroidIdentity=[ordered]@{application_id=$Matches[1];version_code=[int]$Matches[2];version_name=$Matches[3]}
    Copy-Item -LiteralPath (Join-Path $TogetherRoot 'THIRD-PARTY-NOTICES.md') -Destination $TogetherPackage
    Copy-Item -LiteralPath (Join-Path $TogetherRoot 'docs/ANDROID-UPDATES-AND-LOGS.md') -Destination $TogetherPackage
    Copy-Item -LiteralPath (Join-Path $TogetherRoot 'third-party') -Destination (Join-Path $TogetherPackage 'third-party') -Recurse
    $TogetherSample=Join-Path $TogetherRoot 'work/media/clock-test.mp4'
    if(Test-Path -LiteralPath $TogetherSample -PathType Leaf) {
        $TogetherMedia=Join-Path $TogetherWindows 'work/media'
        New-Item -ItemType Directory -Path $TogetherMedia -Force | Out-Null
        Copy-Item -LiteralPath $TogetherSample -Destination (Join-Path $TogetherMedia 'clock-test.mp4')
    }
    Push-Location $TogetherRoot
    try {
        git archive --format=zip --output (Join-Path $TogetherPackage '2gether-source.zip') $TogetherSourceCommit
        if($LASTEXITCODE -ne 0){throw 'Candidate source archive failed'}
    }finally {Pop-Location}
    $TogetherReadme=@'
2gether private Alpha candidate — not a fully accepted beta.

Open Windows/2gether.exe with its native folder kept beside it. Install the APK
manually on your phone if you choose to test this candidate. No app was launched
or installed by the builder. The build receipt records the actual APK version
and application ID; signing compatibility and pairing retention must be verified
before an update. Keep pairing/settings when updating; do not uninstall first.

Select on PC: ready TorBox videos, Stremio movie/series catalogs with episode lists, or a recognized copied
Torrentio/TorBox source after Stremio starts a movie/episode. Prepare in TorBox
adds only the explicitly chosen cached torrent. Pending/account-limit conditions
do not delete other items or automatically retry uncertain creation requests.

This Windows candidate explicitly enables account-bearing TorBox playback links
for the user-authorized PRIVATE Alpha fallback. The paired phone receives that
account credential in its direct CDN link. Keep the session private. Credentials
are not included in these build files; the PC uses its existing encrypted vaults.
Ordinary builds retain the strict file-scoped policy.

Use the Together cloudflared control tunnel for remote-phone testing. Movie bytes
must go directly from the CDN, never through the control tunnel. No VPN adapter or
system network changes are required. This candidate has compilation/unit checks,
not proof of real Stremio preparation, native paired playback, WAN recovery or a
physical-phone two-hour soak. The stable download page and updater metadata use the separate release host.
Hosting must be verified separately before delivery; a built QR does not prove it is live.
The source archive and notices accompany this private candidate. Native dependency
binary-to-source correspondence remains an outstanding public-release check.

Phone: Send logs to PC. Normal PC log folder:
C:/Users/astro/AppData/Local/app.together.v2.windows/diagnostics
'@
    [IO.File]::WriteAllText((Join-Path $TogetherPackage 'READ-ME.txt'),$TogetherReadme,[Text.UTF8Encoding]::new($false))
    $TogetherHashes=@(Get-ChildItem -LiteralPath $TogetherPackage -File -Recurse | Sort-Object FullName | ForEach-Object {
        [ordered]@{file=[IO.Path]::GetRelativePath($TogetherPackage,$_.FullName).Replace('\','/');sha256=(Get-FileHash -LiteralPath $_.FullName).Hash.ToLowerInvariant();bytes=$_.Length}
    })
    $TogetherReceipt=[ordered]@{version=1;candidate_id=$TogetherCandidateId;source_commit=$TogetherSourceCommit;recorded_utc=[datetime]::UtcNow.ToString('o');windows_sources=$TogetherWindowsSources;android_sources=$TogetherAndroidSources;android_identity=$TogetherAndroidIdentity;frontend=$TogetherFrontendInput;windows_features=@('custom-protocol','private-alpha-account-links');tauri_config=(ConvertFrom-Json $env:TAURI_CONFIG);gradle_init_sha256=(Get-FileHash -LiteralPath $TogetherInit).Hash.ToLowerInvariant();files=$TogetherHashes;installed=$false;native_playback_verified=$false;physical_gate_passed=$false;standard_artifacts_preserved=$true}
    [IO.File]::WriteAllText((Join-Path $TogetherCandidate 'candidate-build.json'),(ConvertTo-Json -InputObject $TogetherReceipt -Depth 8),[Text.UTF8Encoding]::new($false))
    Write-Output "Private Alpha candidate package: $TogetherPackage"
}finally {$env:TAURI_CONFIG=$TogetherOldConfig;$TogetherCandidateLock.Dispose()}
