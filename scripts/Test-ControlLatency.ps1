param([string]$EmulatorSerial='emulator-5580',[switch]$SkipBuild)
$ErrorActionPreference='Stop'
$TogetherCases=@(@{rtt=100;jitter=20},@{rtt=300;jitter=50},@{rtt=800;jitter=100})
$TogetherIndex=0
foreach($case in $TogetherCases) {
    & (Join-Path $PSScriptRoot 'Test-SessionProof.ps1') -EmulatorSerial $EmulatorSerial -SkipBuild:($SkipBuild -or $TogetherIndex -gt 0) -ControlRttMs $case.rtt -JitterMs $case.jitter
    $TogetherIndex++
}
