$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'AndroidProofReady.ps1')
$original=(Get-Item Function:Invoke-AndroidProofCommand).ScriptBlock
$python=(Get-Command python -CommandType Application|Select-Object -First 1).Source
$script:fixtureCode='';$script:inventoryCalls=0
try {
 function Invoke-AndroidProofCommand {
  param($Adb,$Arguments)
  if(($Arguments -join ',') -cne 'devices'){throw 'Inventory used a mutating adb command'}
  $script:inventoryCalls++
  & $original -Adb $python -Arguments @('-c',$script:fixtureCode)
 }
 $cases=@(
  @{text="List of devices attached`r`n";exit=0;absent=$true},
  @{text="List of devices attached`r`nemulator-5580`tdevice`r`n";exit=0;absent=$false},
  @{text="List of devices attached`nemulator-5580`toffline`n";exit=0;absent=$false},
  @{text="List of devices attached`nemulator-5590`tdevice`n";exit=0;absent=$true},
  @{text='';exit=0;absent=$true},
  @{text="List of devices attached`n";exit=1;absent=$false},
  @{text='';exit=1;absent=$false},
  @{text="List of devices attached`nemulator-55800`tdevice`n";exit=0;absent=$true}
 )
 foreach($case in $cases){
  $encoded=[Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($case.text))
  $script:fixtureCode="import base64,sys;sys.stdout.buffer.write(base64.b64decode('$encoded'));sys.exit($($case.exit))"
  $actual=Test-AndroidProofEmulatorAbsent -Adb unused -Serial emulator-5580
  if($actual -isnot [bool] -or $actual -ne $case.absent){throw 'Device presence was not classified as a strict safe boolean'}
 }
 $before=$script:inventoryCalls
 try{Test-AndroidProofEmulatorAbsent -Adb unused -Serial 'physical-device';throw 'Physical serial accepted'}
 catch{if($_.Exception.Message -ne 'Device absence accepts an isolated emulator only'){throw}}
 if($script:inventoryCalls -ne $before){throw 'Invalid serial reached device command'}
 'Eight actual owned child stdout/exit cases + physical serial boundary passed; no adb/emulator/device used.'
}finally{Set-Item Function:Invoke-AndroidProofCommand $original}
