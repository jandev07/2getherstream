$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'AndroidProofBuild.ps1')
$TogetherFixture=Join-Path ([IO.Path]::GetTempPath()) ('together-android-build-'+[guid]::NewGuid())
$TogetherCreated=[Collections.Generic.List[string]]::new()
function Write-Fixture([string]$Name,[string]$Text) {
    $TogetherPath=Join-Path $TogetherFixture $Name
    [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($TogetherPath)) | Out-Null
    if(!(Test-Path -LiteralPath $TogetherPath)){$TogetherCreated.Add($TogetherPath)}
    [IO.File]::WriteAllText($TogetherPath,$Text)
}
function Assert-Stale {
    $TogetherAccepted=$false;$TogetherError=$null
    try {$null=Assert-AndroidProofBuild -Root $TogetherFixture;$TogetherAccepted=$true}
    catch {$TogetherError=$_.Exception.Message}
    if($TogetherAccepted -or $TogetherError -cne 'Android proof artifacts are missing or stale. Build the current sources before using SkipBuild.') {throw 'Stale build rejection failed'}
}
try {
    foreach($TogetherName in @('build.gradle.kts','settings.gradle.kts','gradle.properties','app/build.gradle.kts','gradle/wrapper/gradle-wrapper.properties','gradle/wrapper/gradle-wrapper.jar','gradlew.bat')) {
        Write-Fixture ('apps/android/'+$TogetherName) 'synthetic configuration'
    }
    Write-Fixture 'apps/android/app/src/main/java/Fixture.kt' 'synthetic source'
    Write-Fixture 'apps/android/app/src/androidTest/java/FixtureTest.kt' 'synthetic test'
    Write-Fixture 'apps/android/app/build/outputs/apk/debug/app-debug.apk' 'not an APK; checksum fixture'
    Write-Fixture 'apps/android/app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk' 'not an APK; test checksum fixture'
    [IO.Directory]::CreateDirectory((Join-Path $TogetherFixture 'work')) | Out-Null
    Assert-Stale
    $TogetherBefore=Get-AndroidProofSources -Root $TogetherFixture
    Write-AndroidProofBuild -Root $TogetherFixture -Before $TogetherBefore
    $TogetherReceiptPath=Join-Path $TogetherFixture 'work/android-proof-build.json'
    $TogetherCreated.Add($TogetherReceiptPath)
    $TogetherOriginal=[IO.File]::ReadAllText($TogetherReceiptPath)
    $TogetherReceipt=Assert-AndroidProofBuild -Root $TogetherFixture
    if($TogetherReceipt.sources.file_count -ne 9 -or $TogetherReceipt.apks.Count -ne 2 -or $TogetherReceipt.authenticated -or $TogetherReceipt.startup_attestation){throw 'Build association failed'}
    foreach($TogetherName in @('apps/android/app/src/main/java/Fixture.kt','apps/android/app/src/androidTest/java/FixtureTest.kt','apps/android/gradle.properties',
        'apps/android/app/build/outputs/apk/debug/app-debug.apk','apps/android/app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk')) {
        $TogetherPath=Join-Path $TogetherFixture $TogetherName
        $TogetherSaved=[IO.File]::ReadAllText($TogetherPath)
        Write-Fixture $TogetherName 'changed fixture'
        Assert-Stale
        if([IO.File]::ReadAllText($TogetherReceiptPath) -cne $TogetherOriginal){throw 'Validation overwrote receipt'}
        Write-Fixture $TogetherName $TogetherSaved
    }
    Write-Fixture 'apps/android/app/src/main/java/New.kt' 'new source'
    Assert-Stale
    $TogetherRejected=$false
    try {Write-AndroidProofBuild -Root $TogetherFixture -Before $TogetherBefore} catch {$TogetherRejected=$true}
    if(!$TogetherRejected -or [IO.File]::ReadAllText($TogetherReceiptPath) -cne $TogetherOriginal){throw 'Mid-build source change replaced receipt'}
    Remove-Item -LiteralPath (Join-Path $TogetherFixture 'apps/android/app/src/main/java/New.kt')
    $TogetherSource=Join-Path $TogetherFixture 'apps/android/app/src/main/java/Fixture.kt'
    $TogetherRenamed=Join-Path $TogetherFixture 'apps/android/app/src/main/java/Renamed.kt'
    Move-Item -LiteralPath $TogetherSource -Destination $TogetherRenamed
    try {Assert-Stale} finally {Move-Item -LiteralPath $TogetherRenamed -Destination $TogetherSource}
    foreach($TogetherMalformed in @('{}','[]','{"version":true}',('x'*65537))) {
        [IO.File]::WriteAllText($TogetherReceiptPath,$TogetherMalformed)
        Assert-Stale
        if([IO.File]::ReadAllText($TogetherReceiptPath) -cne $TogetherMalformed){throw 'Malformed receipt was changed'}
    }
    [IO.File]::WriteAllText($TogetherReceiptPath,$TogetherOriginal)
    $null=Assert-AndroidProofBuild -Root $TogetherFixture
    'Android build association checks passed: source/test/config additions and changes, renamed inputs, APK changes, bounded malformed metadata and preserved receipts.'
} finally {
    # Exact fixture files, then empty directories; no recursive deletion.
    foreach($TogetherPath in $TogetherCreated){if(Test-Path -LiteralPath $TogetherPath){Remove-Item -LiteralPath $TogetherPath}}
    if(Test-Path -LiteralPath $TogetherFixture) {
        Get-ChildItem -LiteralPath $TogetherFixture -Directory -Recurse | Sort-Object FullName -Descending | ForEach-Object {Remove-Item -LiteralPath $_.FullName}
        Remove-Item -LiteralPath $TogetherFixture
    }
}
