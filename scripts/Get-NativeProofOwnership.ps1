param([string]$WorkDirectory=(Join-Path (Split-Path -Parent $PSScriptRoot) 'work'))
$ErrorActionPreference='Stop'
# Read-only inventory. Never removes a marker, creates a lock, kills a process,
# or authorizes a duplicate run. A runner must still acquire its exclusive lock.
$TogetherMarkerPath=Join-Path $WorkDirectory 'native-proof-running.json'
$TogetherLockPath=Join-Path $WorkDirectory 'native-proof-owner.lock'
$TogetherLockState='absent'
if(Test-Path -LiteralPath $TogetherLockPath) {
    try {
        $TogetherProbe=[IO.FileStream]::new($TogetherLockPath,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::None)
        $TogetherProbe.Dispose();$TogetherLockState='free'
    } catch [IO.FileNotFoundException] {$TogetherLockState='absent'}
    catch {$TogetherLockState='unavailable'}
}
$TogetherIdentity='absent'
$TogetherRunnerId=$null
if(Test-Path -LiteralPath $TogetherMarkerPath) {
    $TogetherIdentity='unverified'
    try {
        $TogetherFile=[IO.FileStream]::new($TogetherMarkerPath,[IO.FileMode]::Open,[IO.FileAccess]::Read,[IO.FileShare]::ReadWrite)
        try {
            # Metadata is bounded on the opened handle, including concurrent growth.
            $TogetherBytes=[byte[]]::new(4097)
            $TogetherRead=0
            while($TogetherRead -lt $TogetherBytes.Length) {
                $TogetherCount=$TogetherFile.Read($TogetherBytes,$TogetherRead,$TogetherBytes.Length-$TogetherRead)
                if(!$TogetherCount){break}
                $TogetherRead+=$TogetherCount
            }
        } finally {$TogetherFile.Dispose()}
        if($TogetherRead -lt 1 -or $TogetherRead -gt 4096){throw 'Invalid ownership metadata'}
        $TogetherMetadata=[Text.Encoding]::UTF8.GetString($TogetherBytes,0,$TogetherRead).TrimStart([char]0xFEFF)
        # Inspect the original timezone-qualified field before PowerShell can
        # coerce an ISO-looking JSON string into a DateTime. Reject ambiguous,
        # escaped or duplicate date fields rather than guessing their meaning.
        $TogetherStarts=[regex]::Matches($TogetherMetadata,'(?<!\\)"runner_started_utc"\s*:\s*"(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{1,7})?(Z|[+-]\d{2}:\d{2}))"')
        if($TogetherStarts.Count -ne 1){throw 'Invalid runner start'}
        $TogetherMarker=$TogetherMetadata | ConvertFrom-Json
        $TogetherId=$TogetherMarker.runner_pid
        if($TogetherId -is [bool] -or $TogetherId -is [string] -or $TogetherId -isnot [ValueType] -or
            $TogetherId -lt 1 -or $TogetherId -gt [int]::MaxValue -or $TogetherId -ne [math]::Floor([double]$TogetherId)) {throw 'Invalid runner identity'}
        $TogetherStart=$TogetherMarker.runner_started_utc
        if($TogetherStart -is [string]) {
            if($TogetherStart -notmatch '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{1,7})?(Z|[+-]\d{2}:\d{2})$') {throw 'Invalid runner start'}
        } elseif($TogetherStart -isnot [datetime] -and $TogetherStart -isnot [datetimeoffset]) {throw 'Invalid runner start'}
        $TogetherSaved=[datetimeoffset]$TogetherStart
        # Compare the instant, not formatting (Z vs +00:00), local timezone, or
        # a two-second tolerance that could identify a reused PID as the owner.
        $TogetherRunnerId=[int]$TogetherId
        $TogetherProcess=$null
        try {$TogetherProcess=Get-Process -Id $TogetherRunnerId -ErrorAction Stop}
        catch {
            if($_.FullyQualifiedErrorId -notlike 'NoProcessFoundForGivenId,*') {throw 'Runner query unavailable'}
        }
        if(!$TogetherProcess) {$TogetherIdentity='exited'}
        elseif($TogetherSaved.UtcTicks -eq $TogetherProcess.StartTime.ToUniversalTime().Ticks) {$TogetherIdentity='matching'}
        else {$TogetherIdentity='different_process'}
    } catch {$TogetherIdentity='unverified';$TogetherRunnerId=$null}
}
[pscustomobject]@{
    owner_lock=$TogetherLockState
    runner_identity=$TogetherIdentity
    runner_pid=$TogetherRunnerId
    resources_must_be_preserved=($TogetherLockState -eq 'unavailable' -or $TogetherIdentity -in @('matching','unverified'))
    duplicate_run_authorized=$false
}
