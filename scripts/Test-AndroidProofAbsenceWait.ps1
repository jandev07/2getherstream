$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'AndroidProofReady.ps1')
$original=(Get-Item Function:Test-AndroidProofEmulatorAbsent).ScriptBlock
try {
 function Test-AndroidProofEmulatorAbsent {
  param($Adb,$Serial)
  if($Serial -cne 'emulator-5580'){throw 'Unexpected serial'}
  $script:probeCount++
  return $script:probeCount -ge $script:absentAfter
 }
 foreach($case in @(@{after=1;budget=0;result=$true},@{after=3;budget=1000;result=$true},@{after=999;budget=30;result=$false},@{after=2;budget=0;result=$false})) {
  $script:probeCount=0;$script:absentAfter=$case.after
  $actual=Wait-AndroidProofEmulatorAbsent -Adb unused -Serial emulator-5580 -TimeoutMilliseconds $case.budget
  if($actual -isnot [bool] -or $actual -ne $case.result){throw 'Absence wait misclassified inventory'}
  if($case.result -and $script:probeCount -ne $case.after){throw 'Confirmed absence was not checked freshly'}
  if($case.budget -eq 0 -and $script:probeCount -ne 1){throw 'Zero wait performed extra inventory work'}
 }
 $before=$script:probeCount
 try{Wait-AndroidProofEmulatorAbsent -Adb unused -Serial physical;throw 'Physical serial accepted'}
 catch{if($_.Exception.Message -ne 'Device absence accepts an isolated emulator only'){throw}}
 if($script:probeCount -ne $before){throw 'Invalid serial reached inventory'}
 'Four bounded inventory-wait fixtures + physical serial rejection passed; no adb/emulator used.'
}finally{Set-Item Function:Test-AndroidProofEmulatorAbsent $original}
