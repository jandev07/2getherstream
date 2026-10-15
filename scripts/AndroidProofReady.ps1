# Bounded proof commands and read-only readiness/inventory checks. Mutating
# requests require the caller's existing native ownership. Transport is not APK readiness.
function Invoke-AndroidProofCommand {
    param([string]$Adb,[string[]]$Arguments)
    $TogetherInfo=[Diagnostics.ProcessStartInfo]::new()
    $TogetherInfo.FileName=$Adb
    $TogetherInfo.UseShellExecute=$false
    $TogetherInfo.CreateNoWindow=$true
    $TogetherInfo.RedirectStandardOutput=$true
    $TogetherInfo.RedirectStandardError=$true
    foreach($TogetherArgument in $Arguments){$TogetherInfo.ArgumentList.Add($TogetherArgument)}
    $TogetherChild=[Diagnostics.Process]::new()
    $TogetherChild.StartInfo=$TogetherInfo
    try {
        if(!$TogetherChild.Start()){return $null}
        $TogetherOutput=$TogetherChild.StandardOutput.ReadToEndAsync()
        $TogetherError=$TogetherChild.StandardError.ReadToEndAsync()
        if(!$TogetherChild.WaitForExit(3000)){
            # Only the adb client created here; never its server or emulator.
            $TogetherChild.Kill();$TogetherChild.WaitForExit();return $null
        }
        if($TogetherChild.ExitCode -ne 0){return $null}
        return $TogetherOutput.GetAwaiter().GetResult().Trim()
    }catch{return $null}finally{$TogetherChild.Dispose()}
}
function Invoke-AndroidProofReadyCommand {
    param([string]$Adb,[string]$Serial,[string[]]$Arguments)
    Invoke-AndroidProofCommand -Adb $Adb -Arguments (@('-s',$Serial,'shell')+$Arguments)
}
function Test-AndroidProofEmulatorAbsent {
    param([Parameter(Mandatory)][string]$Adb,[Parameter(Mandatory)][string]$Serial)
    if($Serial -notmatch '^emulator-\d+$'){throw 'Device absence accepts an isolated emulator only'}
    $TogetherDevices=Invoke-AndroidProofCommand -Adb $Adb -Arguments @('devices')
    if($null -eq $TogetherDevices){return $false}
    # Failed/timed-out inventory and an offline owned device both reserve resources.
    return -not [bool](($TogetherDevices -split "`r?`n") -match ('^'+[regex]::Escape($Serial)+'\s'))
}
function Wait-AndroidProofReady {
    param([Parameter(Mandatory)][string]$Adb,[Parameter(Mandatory)][string]$Serial,
        [ValidateRange(1,180)][int]$TimeoutSeconds=120)
    if($Serial -notmatch '^emulator-\d+$'){throw 'The isolated readiness check accepts an emulator only'}
    $TogetherTimer=[Diagnostics.Stopwatch]::StartNew()
    do {
        $TogetherBoot=Invoke-AndroidProofReadyCommand -Adb $Adb -Serial $Serial -Arguments @('getprop','sys.boot_completed')
        if($TogetherBoot -ceq '1'){
            $TogetherPackage=Invoke-AndroidProofReadyCommand -Adb $Adb -Serial $Serial -Arguments @('service','check','package')
            if($TogetherPackage -ceq 'Service package: found'){return}
        }
        if($TogetherTimer.Elapsed.TotalSeconds -ge $TimeoutSeconds){break}
        Start-Sleep -Milliseconds 500
    }while($TogetherTimer.Elapsed.TotalSeconds -lt $TimeoutSeconds)
    throw 'Emulator is not ready for proof installation. No APK was installed.'
}
