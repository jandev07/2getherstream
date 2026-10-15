param([string]$RunnerPath=(Join-Path $PSScriptRoot 'Test-AlphaUsability.ps1'))
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'AndroidProofReady.ps1')
$tokens=$null;$errors=$null
$tree=[Management.Automation.Language.Parser]::ParseFile($RunnerPath,[ref]$tokens,[ref]$errors)
if($errors.Count){throw 'Runner parse failed'}
$tries=@($tree.FindAll({param($n) $n -is [Management.Automation.Language.TryStatementAst] -and $null -ne $n.Finally},$true)|Sort-Object {$_.Extent.Text.Length} -Descending)
$body=$tries[0].Finally.Extent.Text
$cleanup=[scriptblock]::Create($body.Substring(1,$body.Length-2))
$original=Get-Item Function:Test-AndroidProofEmulatorAbsent -ErrorAction SilentlyContinue
$command=(Get-Item Function:Invoke-AndroidProofCommand).ScriptBlock
$wait=(Get-Item Function:Wait-AndroidProofEmulatorAbsent).ScriptBlock
function Invoke-AndroidProofCommand {param($Adb,$Arguments);if(($Arguments -join ',') -cne '-s,emulator-5580,emu,kill'){throw 'Cleanup issued an unexpected command'};return 'Fixture stop requested'}
function Test-AndroidProofEmulatorAbsent {param($Adb,$Serial);return $script:cleanupAbsent}
function Wait-AndroidProofEmulatorAbsent {param($Adb,$Serial);Test-AndroidProofEmulatorAbsent -Adb $Adb -Serial $Serial}
$testRoot=Join-Path ([IO.Path]::GetTempPath()) ('together-alpha-cleanup-'+[guid]::NewGuid())
[IO.Directory]::CreateDirectory((Join-Path $testRoot 'work'))|Out-Null
try {
 foreach($case in @(@{absent=$false;alive=$false},@{absent=$true;alive=$false},@{absent=$true;alive=$true})) {
  $absent=$case.absent
  $script:cleanupAbsent=$absent
  $TogetherRoot=$testRoot;$output=Join-Path $testRoot 'work';$marker=Join-Path $output 'native-proof-running.json';$run='fixture-owned-run'
  $receipt=[ordered]@{status='passed';run_id=$run};$reverseOwned=$false;$hostProcess=$null;$ready=$false;$testInstalled=$false;$oldAvd=$env:ANDROID_AVD_HOME
  $emulatorProcess=[pscustomobject]@{HasExited=!$case.alive};$emulatorProcess|Add-Member ScriptMethod WaitForExit {param($milliseconds);if($milliseconds -notin @(15000,5000)){throw 'Unbounded emulator wait'};return $false};$emulatorProcess|Add-Member ScriptMethod Kill {param($tree);if($tree -ne $true){throw 'Unexpected kill scope'}};$emulatorStopped=$true
  $proofEnvironment=@{};$lease=[IO.MemoryStream]::new();$serial='emulator-5580';$adb='unused'
  @{run_id=$run;runner_pid=$PID;runner_started_utc=(Get-Process -Id $PID).StartTime.ToUniversalTime().ToString('o')}|ConvertTo-Json|Set-Content $marker
  & $cleanup
  if($absent -and !$case.alive) {
   if(Test-Path $marker){throw 'Confirmed absent device retained ownership'}
   if($receipt.status -ne 'passed' -or !$receipt.emulator_stopped){throw 'Confirmed cleanup marked uncertain'}
  }else{
   if(!(Test-Path $marker)){throw 'Cleanup dropped ownership while emulator absence was unverified'}
   $owner=Get-Content $marker -Raw|ConvertFrom-Json
   if($receipt.status -ne 'failed' -or !$receipt.cleanup_uncertain -or $receipt.emulator_stopped -or $owner.resource_owner -ne 'unverified_after_owned_emulator_stop'){throw 'Unknown cleanup did not retain fixed failure owner'}
   $observation=& (Join-Path $PSScriptRoot 'Get-NativeProofOwnership.ps1') -WorkDirectory $output
   if(!$observation.resources_must_be_preserved -or $observation.runner_identity -ne 'unverified'){throw 'Retained marker did not reserve resources'}
  }
 }
 'Actual runner cleanup fragment passed: uncertain device/live process preserved; confirmed exited+absent releases owner.'
}finally{
 Set-Item Function:Invoke-AndroidProofCommand $command
 Set-Item Function:Wait-AndroidProofEmulatorAbsent $wait
 if($original){Set-Item Function:Test-AndroidProofEmulatorAbsent $original.ScriptBlock}else{Remove-Item Function:Test-AndroidProofEmulatorAbsent -ErrorAction SilentlyContinue}
 foreach($path in @('native-proof-running.json','receipt.json','alpha-usability-last-run.json','update-archive-last-run.json','emulator-stop.log')) { $file=Join-Path $testRoot ('work/'+$path);if(Test-Path $file){Remove-Item -LiteralPath $file} }
 Remove-Item -LiteralPath (Join-Path $testRoot 'work');Remove-Item -LiteralPath $testRoot
}
