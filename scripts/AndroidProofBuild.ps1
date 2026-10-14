# Local build association, not authentication or bytecode/startup attestation.
function Assert-AndroidProofPath {
    param([string]$Root,[string]$Path)
    $TogetherBase=[IO.Path]::GetFullPath($Root).TrimEnd([IO.Path]::DirectorySeparatorChar)
    $TogetherCurrent=[IO.Path]::GetFullPath($Path)
    if(!$TogetherCurrent.StartsWith($TogetherBase+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){throw 'Invalid Android build input path'}
    while($TogetherCurrent -cne $TogetherBase) {
        if(([IO.File]::GetAttributes($TogetherCurrent) -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw 'Redirected Android build input'}
        $TogetherCurrent=[IO.Path]::GetDirectoryName($TogetherCurrent)
    }
}
function Get-AndroidProofSources {
    param([Parameter(Mandatory)][string]$Root,[switch]$IncludeFiles)
    $TogetherAndroid=Join-Path $Root 'apps/android'
    $TogetherPaths=[Collections.Generic.List[string]]::new()
    foreach($TogetherName in @('build.gradle.kts','settings.gradle.kts','gradle.properties','app/build.gradle.kts','gradle/wrapper/gradle-wrapper.properties','gradle/wrapper/gradle-wrapper.jar','gradlew.bat')) {
        $TogetherPath=Join-Path $TogetherAndroid $TogetherName
        if(!(Test-Path -LiteralPath $TogetherPath -PathType Leaf)){throw 'Android build input missing'}
        Assert-AndroidProofPath -Root $Root -Path $TogetherPath
        $TogetherPaths.Add($TogetherPath)
    }
    $TogetherPending=[Collections.Generic.Stack[string]]::new()
    Assert-AndroidProofPath -Root $Root -Path (Join-Path $TogetherAndroid 'app/src')
    $TogetherPending.Push((Join-Path $TogetherAndroid 'app/src'))
    while($TogetherPending.Count) {
        $TogetherDirectory=$TogetherPending.Pop()
        if(([IO.File]::GetAttributes($TogetherDirectory) -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw 'Redirected Android build input'}
        foreach($TogetherPath in [IO.Directory]::EnumerateFileSystemEntries($TogetherDirectory)) {
            $TogetherAttributes=[IO.File]::GetAttributes($TogetherPath)
            if(($TogetherAttributes -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw 'Redirected Android build input'}
            if(($TogetherAttributes -band [IO.FileAttributes]::Directory) -ne 0){$TogetherPending.Push($TogetherPath)} else {$TogetherPaths.Add($TogetherPath)}
        }
        if($TogetherPaths.Count+$TogetherPending.Count -gt 10000){throw 'Android build input inventory too large'}
    }
    $TogetherRows=@($TogetherPaths | Sort-Object -CaseSensitive | ForEach-Object {
        if(([IO.File]::GetAttributes($_) -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw 'Redirected Android build input'}
        [ordered]@{file=[IO.Path]::GetRelativePath($Root,$_).Replace('\','/');sha256=(Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash.ToLowerInvariant()}
    })
    $TogetherBytes=[Text.Encoding]::UTF8.GetBytes((ConvertTo-Json -InputObject $TogetherRows -Depth 4 -Compress))
    $TogetherHasher=[Security.Cryptography.SHA256]::Create()
    try {$TogetherDigest=[Convert]::ToHexString($TogetherHasher.ComputeHash($TogetherBytes)).ToLowerInvariant()} finally {$TogetherHasher.Dispose()}
    $TogetherResult=[ordered]@{sha256=$TogetherDigest;file_count=$TogetherRows.Count}
    if($IncludeFiles){$TogetherResult.files=$TogetherRows}
    $TogetherResult
}
function Get-AndroidProofApks {
    param([Parameter(Mandatory)][string]$Root)
    @(foreach($TogetherName in @('apps/android/app/build/outputs/apk/debug/app-debug.apk','apps/android/app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk')) {
        $TogetherPath=Join-Path $Root $TogetherName
        if(!(Test-Path -LiteralPath $TogetherPath -PathType Leaf) -or ([IO.File]::GetAttributes($TogetherPath) -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw 'Android proof APK unavailable'}
        Assert-AndroidProofPath -Root $Root -Path $TogetherPath
        [ordered]@{file=$TogetherName;sha256=(Get-FileHash -LiteralPath $TogetherPath -Algorithm SHA256).Hash.ToLowerInvariant()}
    })
}
function Write-AndroidProofBuild {
    param([Parameter(Mandatory)][string]$Root,[Parameter(Mandatory)]$Before)
    $TogetherAfter=Get-AndroidProofSources -Root $Root
    if($Before.sha256 -cne $TogetherAfter.sha256){throw 'Android inputs changed during build; build again before testing'}
    $TogetherReceipt=[ordered]@{version=1;sources=$TogetherAfter;apks=@(Get-AndroidProofApks -Root $Root);recorded_utc=[datetime]::UtcNow.ToString('o');authenticated=$false;startup_attestation=$false}
    $TogetherReceiptPath=Join-Path $Root 'work/android-proof-build.json'
    if(Test-Path -LiteralPath $TogetherReceiptPath) {
        if(([IO.File]::GetAttributes($TogetherReceiptPath) -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw 'Android proof build receipt unavailable'}
    }
    $TogetherTemporary=Join-Path $Root ('work/android-build-'+[guid]::NewGuid().ToString()+'.tmp')
    try {
        [IO.File]::WriteAllText($TogetherTemporary,(ConvertTo-Json -InputObject $TogetherReceipt -Depth 5),[Text.UTF8Encoding]::new($false))
        [IO.File]::Move($TogetherTemporary,$TogetherReceiptPath,$true)
    } finally {if(Test-Path -LiteralPath $TogetherTemporary){Remove-Item -LiteralPath $TogetherTemporary}}
}
function Assert-AndroidProofBuild {
    param([Parameter(Mandatory)][string]$Root)
    try {
        $TogetherReceiptPath=Join-Path $Root 'work/android-proof-build.json'
        if(([IO.File]::GetAttributes($TogetherReceiptPath) -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw 'Invalid receipt'}
        $TogetherFile=[IO.File]::OpenRead($TogetherReceiptPath)
        try {
            $TogetherBytes=[byte[]]::new(65537);$TogetherCount=0
            while($TogetherCount -lt $TogetherBytes.Length) {
                $TogetherRead=$TogetherFile.Read($TogetherBytes,$TogetherCount,$TogetherBytes.Length-$TogetherCount)
                if(!$TogetherRead){break};$TogetherCount+=$TogetherRead
            }
        } finally {$TogetherFile.Dispose()}
        if($TogetherCount -lt 1 -or $TogetherCount -gt 65536){throw 'Invalid receipt'}
        $TogetherReceipt=[Text.Encoding]::UTF8.GetString($TogetherBytes,0,$TogetherCount) | ConvertFrom-Json
        if($TogetherReceipt.version -is [bool] -or $TogetherReceipt.version -ne 1 -or $TogetherReceipt.sources.sha256 -isnot [string] -or $TogetherReceipt.sources.sha256 -cnotmatch '^[0-9a-f]{64}$'){throw 'Invalid receipt'}
        $TogetherSources=Get-AndroidProofSources -Root $Root
        if($TogetherReceipt.sources.sha256 -cne $TogetherSources.sha256 -or $TogetherReceipt.sources.file_count -ne $TogetherSources.file_count){throw 'Stale source receipt'}
        $TogetherApks=@(Get-AndroidProofApks -Root $Root)
        if(@($TogetherReceipt.apks).Count -ne $TogetherApks.Count){throw 'Invalid receipt'}
        for($TogetherIndex=0;$TogetherIndex -lt $TogetherApks.Count;$TogetherIndex++) {
            if($TogetherReceipt.apks[$TogetherIndex].file -cne $TogetherApks[$TogetherIndex].file -or $TogetherReceipt.apks[$TogetherIndex].sha256 -cne $TogetherApks[$TogetherIndex].sha256){throw 'Stale APK receipt'}
        }
        return $TogetherReceipt
    } catch {throw 'Android proof artifacts are missing or stale. Build the current sources before using SkipBuild.'}
}
