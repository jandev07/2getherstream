# OS-held ownership covers builds as well as the running native test. An empty
# lock file may remain after exit; ownership is the open exclusive handle.
function Enter-NativeProofLock {
    param([Parameter(Mandatory)][string]$Path)
    $TogetherLockDirectory=Split-Path -Parent ([IO.Path]::GetFullPath($Path))
    $TogetherInventory=Join-Path $PSScriptRoot 'Get-NativeProofOwnership.ps1'
    $TogetherHandle=$null
    try {
        # Also protect against legacy/external runners with a marker but no OS
        # lock. Uncertain identity is not permission to rebuild or reinstall.
        $TogetherBefore=& $TogetherInventory -WorkDirectory $TogetherLockDirectory
        if($TogetherBefore.resources_must_be_preserved){throw 'Ownership not available'}
        $TogetherHandle=[IO.FileStream]::new($Path,[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::None)
        # The marker can change while acquiring the handle. Our own lock is now
        # unavailable to the inventory; only its runner identity is relevant.
        $TogetherAfter=& $TogetherInventory -WorkDirectory $TogetherLockDirectory
        if($TogetherAfter.runner_identity -notin @('absent','exited','different_process')) {throw 'Ownership not available'}
        return $TogetherHandle
    } catch {
        if($TogetherHandle){$TogetherHandle.Dispose()}
        throw 'Native proof ownership unavailable; another runner may be building or testing. No resources were changed.'
    }
}
