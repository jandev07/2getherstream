$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Assert-NativeProofResult.ps1')
Assert-NativeProofResult -Lines @('app.together.v2.PairingStoreTest:...','','OK (6 tests)') -ExpectedTests 6
foreach($lines in @(
    ,@('OK (5 tests)'), ,@('OK (7 tests)'), ,@(), ,@('OK (6 tests)','OK (6 tests)'),
    ,@('OK (6 tests)','FAILURES!!!'), ,@('OK (6 tests)','INSTRUMENTATION_FAILED'),
    ,@('OK (6 tests)','INSTRUMENTATION_RESULT: shortMsg=Process crashed'), ,@('prefix OK (6 tests)')
)) {
    $rejected=$false
    try {Assert-NativeProofResult -Lines $lines -ExpectedTests 6}
    catch {if($_.Exception.Message -ne 'native_instrumentation_result_invalid') {throw};$rejected=$true}
    if(-not $rejected) {throw 'instrumentation_result_regression_failed'}
}
Write-Output 'instrumentation_result_regressions_passed'
