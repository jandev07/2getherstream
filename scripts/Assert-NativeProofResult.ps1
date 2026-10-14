# Pure result validation; never runs instrumentation or reads device credentials.
function Assert-NativeProofResult {
    param([AllowEmptyCollection()][string[]]$Lines,[ValidateRange(1,100)][int]$ExpectedTests)
    $success=@($Lines | Where-Object {$_ -cmatch '^OK \([0-9]+ tests\)$'})
    $failure=@($Lines | Where-Object {$_ -match 'FAILURES!!!|INSTRUMENTATION_FAILED|INSTRUMENTATION_ABORTED|shortMsg=|Process crashed'})
    if($success.Count -ne 1 -or $success[0] -cne "OK ($ExpectedTests tests)" -or $failure.Count -ne 0) {
        throw 'native_instrumentation_result_invalid'
    }
}
