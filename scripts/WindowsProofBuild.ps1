# Local build association only; no compiler, dependency or startup attestation.
function Assert-WindowsProofPath {
    param([string]$Root,[string]$Path)
    $TogetherBase=[IO.Path]::GetFullPath($Root).TrimEnd([IO.Path]::DirectorySeparatorChar)
    $TogetherCurrent=[IO.Path]::GetFullPath($Path)
    if(!$TogetherCurrent.StartsWith($TogetherBase+[IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)){throw 'Invalid Windows build input path'}
    while($TogetherCurrent -cne $TogetherBase) {
        if(([IO.File]::GetAttributes($TogetherCurrent) -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw 'Redirected Windows build input'}
        $TogetherCurrent=[IO.Path]::GetDirectoryName($TogetherCurrent)
    }
}
function Get-WindowsProofInventory {
    param([string]$Root,[string[]]$Files=@(),[string[]]$Directories=@())
    $TogetherPaths=[Collections.Generic.List[string]]::new()
    foreach($TogetherName in $Files) {
        $TogetherPath=Join-Path $Root $TogetherName
        if(!(Test-Path -LiteralPath $TogetherPath -PathType Leaf)){throw 'Windows build input missing'}
        Assert-WindowsProofPath -Root $Root -Path $TogetherPath
        $TogetherPaths.Add($TogetherPath)
    }
    $TogetherPending=[Collections.Generic.Stack[string]]::new()
    foreach($TogetherName in $Directories) {
        $TogetherPath=Join-Path $Root $TogetherName
        if(!(Test-Path -LiteralPath $TogetherPath -PathType Container)){throw 'Windows build input directory missing'}
        Assert-WindowsProofPath -Root $Root -Path $TogetherPath
        $TogetherPending.Push($TogetherPath)
    }
    while($TogetherPending.Count) {
        $TogetherDirectory=$TogetherPending.Pop()
        Assert-WindowsProofPath -Root $Root -Path $TogetherDirectory
        foreach($TogetherPath in [IO.Directory]::EnumerateFileSystemEntries($TogetherDirectory)) {
            $TogetherAttributes=[IO.File]::GetAttributes($TogetherPath)
            if(($TogetherAttributes -band [IO.FileAttributes]::ReparsePoint) -ne 0){throw 'Redirected Windows build input'}
            if(($TogetherAttributes -band [IO.FileAttributes]::Directory) -ne 0){$TogetherPending.Push($TogetherPath)}else{$TogetherPaths.Add($TogetherPath)}
            if($TogetherPaths.Count+$TogetherPending.Count -gt 20000){throw 'Windows build input inventory too large'}
        }
    }
    $TogetherRows=@($TogetherPaths | Sort-Object -CaseSensitive | ForEach-Object {
        Assert-WindowsProofPath -Root $Root -Path $_
        [ordered]@{file=[IO.Path]::GetRelativePath($Root,$_).Replace('\','/');sha256=(Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash.ToLowerInvariant()}
    })
    if(!$TogetherRows.Count){throw 'Empty Windows build input inventory'}
    $TogetherBytes=[Text.Encoding]::UTF8.GetBytes((ConvertTo-Json -InputObject $TogetherRows -Depth 4 -Compress))
    $TogetherHasher=[Security.Cryptography.SHA256]::Create()
    try {$TogetherDigest=[Convert]::ToHexString($TogetherHasher.ComputeHash($TogetherBytes)).ToLowerInvariant()}finally{$TogetherHasher.Dispose()}
    [ordered]@{sha256=$TogetherDigest;file_count=$TogetherRows.Count}
}
function Get-WindowsProofSources {
    param([Parameter(Mandatory)][string]$Root)
    $TogetherFiles=@('Cargo.toml','Cargo.lock','apps/windows/package.json','apps/windows/package-lock.json','apps/windows/index.html',
        'apps/windows/app-icon.svg','apps/windows/vite.config.ts','apps/windows/tsconfig.json','apps/windows/svelte.config.js',
        'apps/windows/src-tauri/Cargo.toml','apps/windows/src-tauri/build.rs','apps/windows/src-tauri/tauri.conf.json')
    $TogetherDirectories=@('apps/windows/src','apps/windows/src-tauri/src','apps/windows/src-tauri/capabilities','apps/windows/src-tauri/icons')
    foreach($TogetherCrate in @('sync-core','player-mpv','session-server','source-resolver')) {
        $TogetherFiles+=('crates/'+$TogetherCrate+'/Cargo.toml')
        $TogetherDirectories+=('crates/'+$TogetherCrate+'/src')
    }
    Get-WindowsProofInventory -Root $Root -Files $TogetherFiles -Directories $TogetherDirectories
}
function Get-WindowsProofFrontend {
    param([Parameter(Mandatory)][string]$Root)
    Get-WindowsProofInventory -Root $Root -Directories @('apps/windows/dist')
}
function Get-WindowsProofArtifacts {
    param([Parameter(Mandatory)][string]$Root)
    @(foreach($TogetherName in @('target/debug/together2-windows.exe','native/vendor/windows/libmpv-2.dll')) {
        $TogetherPath=Join-Path $Root $TogetherName
        Assert-WindowsProofPath -Root $Root -Path $TogetherPath
        if(!(Test-Path -LiteralPath $TogetherPath -PathType Leaf)){throw 'Windows proof artifact unavailable'}
        [ordered]@{file=$TogetherName;sha256=(Get-FileHash -LiteralPath $TogetherPath -Algorithm SHA256).Hash.ToLowerInvariant()}
    })
}
function Write-WindowsProofBuild {
    param([Parameter(Mandatory)][string]$Root,[Parameter(Mandatory)]$BeforeSources,[Parameter(Mandatory)]$BeforeFrontend)
    $TogetherSources=Get-WindowsProofSources -Root $Root
    $TogetherFrontend=Get-WindowsProofFrontend -Root $Root
    if($BeforeSources.sha256 -cne $TogetherSources.sha256 -or $BeforeFrontend.sha256 -cne $TogetherFrontend.sha256){throw 'Windows inputs changed during build; build again before testing'}
    $TogetherReceipt=[ordered]@{version=1;sources=$TogetherSources;frontend=$TogetherFrontend;artifacts=@(Get-WindowsProofArtifacts -Root $Root);recorded_utc=[datetime]::UtcNow.ToString('o');authenticated=$false;startup_attestation=$false}
    $TogetherWork=Join-Path $Root 'work'
    Assert-WindowsProofPath -Root $Root -Path $TogetherWork
    $TogetherReceiptPath=Join-Path $TogetherWork 'windows-proof-build.json'
    if(Test-Path -LiteralPath $TogetherReceiptPath){Assert-WindowsProofPath -Root $Root -Path $TogetherReceiptPath}
    $TogetherTemporary=Join-Path $TogetherWork ('windows-build-'+[guid]::NewGuid()+'.tmp')
    try {
        [IO.File]::WriteAllText($TogetherTemporary,(ConvertTo-Json -InputObject $TogetherReceipt -Depth 5),[Text.UTF8Encoding]::new($false))
        [IO.File]::Move($TogetherTemporary,$TogetherReceiptPath,$true)
    }finally{if(Test-Path -LiteralPath $TogetherTemporary){Remove-Item -LiteralPath $TogetherTemporary}}
}
function Assert-WindowsProofBuild {
    param([Parameter(Mandatory)][string]$Root)
    try {
        $TogetherPath=Join-Path $Root 'work/windows-proof-build.json'
        Assert-WindowsProofPath -Root $Root -Path $TogetherPath
        $TogetherFile=[IO.File]::OpenRead($TogetherPath)
        try {
            $TogetherBytes=[byte[]]::new(65537);$TogetherCount=0
            while($TogetherCount -lt $TogetherBytes.Length) {
                $TogetherRead=$TogetherFile.Read($TogetherBytes,$TogetherCount,$TogetherBytes.Length-$TogetherCount)
                if(!$TogetherRead){break};$TogetherCount+=$TogetherRead
            }
        }finally{$TogetherFile.Dispose()}
        if($TogetherCount -lt 1 -or $TogetherCount -gt 65536){throw 'Invalid receipt'}
        $TogetherReceipt=[Text.Encoding]::UTF8.GetString($TogetherBytes,0,$TogetherCount) | ConvertFrom-Json
        if(($TogetherReceipt.version -isnot [int] -and $TogetherReceipt.version -isnot [long]) -or $TogetherReceipt.version -ne 1 -or
            $TogetherReceipt.authenticated -isnot [bool] -or $TogetherReceipt.authenticated -or
            $TogetherReceipt.startup_attestation -isnot [bool] -or $TogetherReceipt.startup_attestation){throw 'Invalid receipt'}
        foreach($TogetherKind in @('sources','frontend')) {
            $TogetherActual=if($TogetherKind -eq 'sources'){Get-WindowsProofSources -Root $Root}else{Get-WindowsProofFrontend -Root $Root}
            if($TogetherKind -eq 'sources'){$TogetherSources=$TogetherActual}else{$TogetherFrontend=$TogetherActual}
            if(($TogetherReceipt.$TogetherKind.file_count -isnot [int] -and $TogetherReceipt.$TogetherKind.file_count -isnot [long]) -or
                $TogetherReceipt.$TogetherKind.sha256 -isnot [string] -or $TogetherReceipt.$TogetherKind.sha256 -cne $TogetherActual.sha256 -or $TogetherReceipt.$TogetherKind.file_count -ne $TogetherActual.file_count){throw 'Stale input receipt'}
        }
        $TogetherArtifacts=@(Get-WindowsProofArtifacts -Root $Root)
        if(@($TogetherReceipt.artifacts).Count -ne $TogetherArtifacts.Count){throw 'Invalid receipt'}
        for($TogetherIndex=0;$TogetherIndex -lt $TogetherArtifacts.Count;$TogetherIndex++) {
            if($TogetherReceipt.artifacts[$TogetherIndex].file -cne $TogetherArtifacts[$TogetherIndex].file -or $TogetherReceipt.artifacts[$TogetherIndex].sha256 -cne $TogetherArtifacts[$TogetherIndex].sha256){throw 'Stale artifact receipt'}
        }
        $TogetherRecorded=$TogetherReceipt.recorded_utc
        if($TogetherRecorded -is [string]) {
            if($TogetherRecorded -notmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{1,7})?(Z|\+00:00)$'){throw 'Invalid recorded time'}
        }elseif($TogetherRecorded -is [datetime]) {
            if($TogetherRecorded.Kind -ne [DateTimeKind]::Utc){throw 'Invalid recorded time'}
        }elseif($TogetherRecorded -is [datetimeoffset]) {
            if($TogetherRecorded.Offset -ne [timespan]::Zero){throw 'Invalid recorded time'}
        }else{throw 'Invalid recorded time'}
        # Export only the checked schema, never unknown local metadata fields.
        return [ordered]@{version=1;sources=$TogetherSources;frontend=$TogetherFrontend;artifacts=$TogetherArtifacts;
            recorded_utc=([datetimeoffset]$TogetherRecorded).ToUniversalTime().ToString('o');authenticated=$false;startup_attestation=$false}
    }catch{throw 'Windows proof artifacts are missing or stale. Build the current sources before using SkipBuild.'}
}
