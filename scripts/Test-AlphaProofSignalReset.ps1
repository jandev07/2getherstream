$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'WindowsProofBuild.ps1')
$tokens=$null;$errors=$null
$tree=[Management.Automation.Language.Parser]::ParseFile((Join-Path $PSScriptRoot 'Test-AlphaUsability.ps1'),[ref]$tokens,[ref]$errors)
if($errors.Count){throw 'Runner parse failed'}
$nodes=@($tree.FindAll({param($node)
 $text=$node.Extent.Text
 ($node -is [Management.Automation.Language.AssignmentStatementAst] -and $text.StartsWith('$candidateExe=')) -or
 ($node -is [Management.Automation.Language.ForEachStatementAst] -and ($text.StartsWith('foreach($existing in') -or $text.StartsWith('foreach($signal in')))
},$true))
if($nodes.Count -ne 3){throw 'Startup signal fragment changed; review test extraction'}
$reset=[scriptblock]::Create(($nodes|Sort-Object {$_.Extent.StartOffset}|ForEach-Object {$_.Extent.Text}) -join "`n")
$testRoot=Join-Path ([IO.Path]::GetTempPath()) ('together-proof-signals-'+[guid]::NewGuid())
$appRoot=$testRoot;[IO.Directory]::CreateDirectory((Join-Path $testRoot 'work'))|Out-Null
function Get-Process {param($Name,$ErrorAction);if($Name -cne '2gether'){throw 'Unexpected process query'};return $script:fixtureHosts}
try {
 foreach($case in @('absent','same','unrelated','unknown')) {
  foreach($name in @('startup-proof-stop.txt','pairing-code-proof.txt','preserved.fixture')){Set-Content (Join-Path $testRoot ('work/'+$name)) 'Owned fixture'}
  $script:fixtureHosts=switch($case) {
   'absent' {@()}
   'same' {@([pscustomobject]@{Path=(Join-Path $appRoot '2gether.exe')})}
   'unrelated' {@([pscustomobject]@{Path=(Join-Path $testRoot 'elsewhere/2gether.exe')})}
   'unknown' {@([pscustomobject]@{Path=$null})}
  }
  $blocked=$false
  try {& $reset} catch {if($_.Exception.Message -cne 'Candidate app is live or ownership uncertain; preserve proof signals'){throw};$blocked=$true}
  $expected=$case -in @('same','unknown')
  if($blocked -ne $expected){throw 'Candidate process guard misclassified ownership'}
  foreach($name in @('startup-proof-stop.txt','pairing-code-proof.txt')) {
   if((Test-Path -LiteralPath (Join-Path $testRoot ('work/'+$name))) -ne $expected){throw 'Proof signal cleanup crossed ownership boundary'}
  }
  if(!(Test-Path -LiteralPath (Join-Path $testRoot 'work/preserved.fixture'))){throw 'Cleanup removed unrelated file'}
 }
 'Four actual startup-fragment cases passed: absent/unrelated reset only proof signals; live/unknown preserve them.'
}finally{
 Remove-Item Function:Get-Process
 foreach($name in @('startup-proof-stop.txt','pairing-code-proof.txt','preserved.fixture')){$path=Join-Path $testRoot ('work/'+$name);if(Test-Path -LiteralPath $path){Remove-Item -LiteralPath $path}}
 Remove-Item -LiteralPath (Join-Path $testRoot 'work');Remove-Item -LiteralPath $testRoot
}
