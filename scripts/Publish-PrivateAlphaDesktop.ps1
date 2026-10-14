param([Parameter(Mandatory)][string]$CandidateId,[datetimeoffset]$NotBeforeUtc=[datetimeoffset]::MinValue,[switch]$ValidateOnly)
$ErrorActionPreference='Stop'
$root=Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'AlphaDeliveryWindow.ps1')
$NotBeforeUtc=Get-AlphaDeliveryNotBefore -Root $root -Requested $NotBeforeUtc
if(!$ValidateOnly -and [datetimeoffset]::UtcNow -lt $NotBeforeUtc) {Write-Output 'Alpha delivery is prepared; scheduled time has not arrived';return}
if($CandidateId -notmatch '^\d{8}-\d{6}-[a-f0-9]{8}$') {throw 'Invalid candidate ID'}
if(Test-Path -LiteralPath (Join-Path $root 'work/alpha-usability-hold.json')) {Write-Output 'Alpha delivery held for application usability validation';return}
. (Join-Path $PSScriptRoot 'WindowsProofBuild.ps1')
$candidate=Join-Path $root "work/private-alpha-candidates/$CandidateId"
Assert-WindowsProofPath -Root $root -Path $candidate
$receipt=Get-Content -LiteralPath (Join-Path $candidate 'candidate-build.json') -Raw | ConvertFrom-Json
$validation=Get-Content -LiteralPath (Join-Path $candidate 'replacement-validation.json') -Raw | ConvertFrom-Json
if($receipt.candidate_id -cne $CandidateId -or $validation.candidate_id -cne $CandidateId -or !$validation.embedded_assets_verified -or !$validation.frontend_path_regression_passed -or $validation.android_jvm_failures -ne 0) {throw 'Replacement validation is incomplete'}
# Compilation and an embedded index do not establish a usable application.
$usabilityPath=Join-Path $candidate $validation.usability_receipt
Assert-WindowsProofPath -Root $candidate -Path $usabilityPath
$usability=Get-Content -LiteralPath $usabilityPath -Raw | ConvertFrom-Json
$expectedTests=@(Get-Content (Join-Path $PSScriptRoot 'native-proof-tests.json') -Raw | ConvertFrom-Json).Count
if($usability.candidate_id -cne $CandidateId -or $usability.status -cne 'passed' -or !$usability.rendered_frontend_verified -or !$usability.instrumentation.all_selected_tests_executed -or !$usability.instrumentation.expected_test_identities_verified -or $usability.instrumentation.completed_tests -ne $expectedTests -or $usability.forced_owned_host_cleanup){throw 'Actual application usability checks are incomplete'}
if(!$usability.normal_pairing_ui_verified -or !$usability.normal_dpapi_host_verified -or !$usability.normal_pairing_instrumentation.expected_test_identities_verified){throw 'Normal application pairing and persistence acceptance missing'}
$version=$receipt.android_identity.version_name
if($version -notmatch '^([0-9]+\.[0-9]+\.[0-9]+)-private-alpha$') {throw 'Invalid Alpha version'}
$destination=Join-Path ([Environment]::GetFolderPath('Desktop')) "2gether Alpha $($Matches[1])"
$package=Join-Path $candidate 'package'
$mapping=@(foreach($item in $receipt.files) {
    if([IO.Path]::IsPathRooted($item.file) -or $item.file.Contains(':') -or $item.file.Split('/') -contains '..') {throw 'Invalid package path'}
    $source=Join-Path $package $item.file
    Assert-WindowsProofPath -Root $root -Path $source
    if((Get-Item -LiteralPath $source).Length -ne $item.bytes -or (Get-FileHash -LiteralPath $source).Hash.ToLowerInvariant() -cne $item.sha256) {throw 'Package integrity mismatch'}
    $name=$item.file -replace '^Windows/',''
    if($name -ceq 'READ-ME.txt') {$name='Original-package-instructions.txt'}
    [ordered]@{source=$item.file;file=$name;sha256=$item.sha256;bytes=$item.bytes}
})
if(@($mapping.file | Sort-Object -Unique).Count -ne $mapping.Count) {throw 'Ambiguous Desktop package mapping'}
if($ValidateOnly) {
    [ordered]@{candidate_id=$CandidateId;destination=$destination;payloads_verified=$mapping.Count;files=$mapping;desktop_modified=$false} | ConvertTo-Json -Depth 6
    return
}
$null=New-Item -ItemType Directory -Path $destination -Force
if((Get-Item -LiteralPath $destination).Attributes -band [IO.FileAttributes]::ReparsePoint) {throw 'Redirected Desktop delivery folder'}
foreach($item in $mapping) {
    $target=Join-Path $destination $item.file
    if(Test-Path -LiteralPath $target) {
        Assert-WindowsProofPath -Root ([Environment]::GetFolderPath('Desktop')) -Path $target
        if((Get-Item -LiteralPath $target).Length -ne $item.bytes -or (Get-FileHash -LiteralPath $target).Hash.ToLowerInvariant() -cne $item.sha256) {throw 'Existing Desktop file differs; preserve it for review'}
    }else {
        $null=New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force
        Assert-WindowsProofPath -Root ([Environment]::GetFolderPath('Desktop')) -Path (Split-Path -Parent $target)
        Copy-Item -LiteralPath (Join-Path $package $item.source) -Destination $target
    }
    if((Get-FileHash -LiteralPath $target).Hash.ToLowerInvariant() -cne $item.sha256) {throw 'Desktop copy verification failed'}
}
$proof=Start-Process -FilePath (Join-Path $destination '2gether.exe') -ArgumentList '--verify-alpha-assets' -WindowStyle Hidden -Wait -PassThru -RedirectStandardOutput (Join-Path $candidate 'desktop-proof.stdout') -RedirectStandardError (Join-Path $candidate 'desktop-proof.stderr')
if($proof.ExitCode -ne 0 -or (Get-Content -LiteralPath (Join-Path $candidate 'desktop-proof.stdout') -Raw).Trim() -cne 'ALPHA_EMBEDDED_FRONTEND_VERIFIED') {throw 'Moved EXE asset verification failed'}
$guide=@'
2gether private Alpha replacement

Close the older 2gether window before opening this replacement: only one Windows
host can run at a time. Open 2gether.exe in this folder and keep its native and
work folders beside it. Delivery does not close your current app automatically.
Install 2gether-PrivateAlpha.apk on the phone as an update; do not uninstall first.
The APK uses the same signing identity as the previous private Alphas.

Use Connection on Windows first. For the Desktop Android emulator, open its
shortcut and choose Use this PC emulator on Android (http://127.0.0.1:47820).
Only one Android device connects at a time. Close the emulator before connecting
your physical phone. Generate a pairing code on Windows and enter it in Android. Normal code-entry,
encrypted save and automatic restored connection were checked on the emulator.

For a physical phone on another network, Connection > Start phone connection
creates a temporary HTTPS address. Enter that address on Android and a fresh
Windows pairing code. Keep Windows open while watching. The address changes
when the temporary connector restarts; use Connect to a different PC on Android
if you need to enter a new address. A temporary address is for Alpha testing;
a stable named control tunnel remains the longer-term route.

Temporary connector creation/cleanup was tested locally; its actual public
HTTPS/WSS validation was blocked by automatic approval review and remains
UNVERIFIED. Do not treat emulator success as remote-phone or two-hour acceptance.

Discover provides Stremio catalog/search, movies/series, episodes and source
selection. Ready TorBox videos and copied Stremio sources remain available.
Watch contains shared play/pause/seek and recovery controls. Both devices must
be ready before playback starts. Text covers are placeholders for this Alpha.

The private TorBox account-bearing playback fallback remains explicitly enabled
for this private test only. Keep the pairing private. Credentials are not bundled.
Movie bytes travel directly from the CDN. The physical-phone two-hour gate is
still outstanding. Earlier failed tests and Alpha packages are preserved.

Phone: Send logs to PC after a problem. Normal PC diagnostic folder:
C:/Users/astro/AppData/Local/app.together.v2.windows/diagnostics

Earlier Alpha folders and original packages are preserved. This app was not
opened interactively, and no phone installation was performed by delivery.
'@
if(!(Test-Path (Join-Path $destination 'START-HERE.txt'))) {[IO.File]::WriteAllText((Join-Path $destination 'START-HERE.txt'),$guide,[Text.UTF8Encoding]::new($false))}
$delivery=[ordered]@{candidate_id=$CandidateId;source_commit=$receipt.source_commit;android_identity=$receipt.android_identity;files=$mapping;recorded_utc=[datetime]::UtcNow.ToString('o');originals_preserved=$true;embedded_assets_verified=$true;application_usability_verified=$true;normal_pairing_ui_verified=$true;public_control_tunnel_verified=$false;real_provider_android_verified=[bool]$validation.real_torbox_android_handoff_verified;usability_run_id=$usability.run_id;interactive_app_launched=$false;installed=$false;physical_gate_passed=$false}
$delivery | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $destination 'desktop-delivery.json') -Encoding utf8
Write-Output "Alpha delivered and verified: $destination"
