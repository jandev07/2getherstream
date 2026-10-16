# Android proof build association

A native proof must identify the APKs it installs. An installed candidate can
differ from the standard `app/build` outputs, and a successful historical run
must not be assigned to a newly built or installed APK automatically.

`scripts/Build-Android.ps1` holds the shared native ownership lock throughout
compilation, receipt publication and copying the development APK. It builds the
debug application and instrumentation APK and runs the JVM tests. The session
and HTTPS runners also hold this lock throughout their build/test lifetime.
Fixture preparation uses the same exact-process-identity entry guard. A matching
or unverifiable running marker blocks entry even without an OS-held lock.

## Receipt and freshness

The supported build paths record `work/android-proof-build.json` only after the
build succeeds and the selected inputs still match their before-build digest.
The receipt contains the source digest/count and both standard APK SHA256 hashes.
It is associated with the following current repository inputs:

- Every regular file under `apps/android/app/src`, including main assets,
  debug inputs, JVM tests and instrumentation tests.
- Root/application Gradle build files, settings and properties.
- Windows Gradle launcher and wrapper JAR/properties.

The inventory includes relative file names in the digest, so additions, removal
and renaming change it even when other file contents remain identical. Redirected
input paths fail closed. Generated build outputs, local toolchain configuration,
private application data, pairing vaults and credentials are not source inputs.
Future dependencies outside these roots require updating the inventory policy.

Before installing APKs, both proof runners recompute this inventory and the
standard APK hashes. Missing, unreadable, oversized, malformed or stale receipts
produce a fixed error requiring a current build. `-SkipBuild` does not bypass
these checks. A refused session preflight does not replace the previous session
receipt, start its host or install APKs. HTTPS preflight checks skipped builds
before its Windows playback test too.

Use `Build-Android.ps1` for an Android-only current build, or run a proof without
`-SkipBuild` to build through that runner. A raw Gradle or isolated output build
does not publish the standard receipt. Do not fabricate or edit a receipt to
permit skipped builds; use the supported build path. Isolated candidates retain
their separate scope and need explicit validation before assigning results.

Session/HTTPS receipts retain the checked Android build record. Session receipts
use sufficient JSON depth to retain the structured hashes rather than truncated
object strings. Archive receipts with exact raw instrumentation status, selected
test identities and the observations that belong to that run.

## Limits and checks

This is local build association, not authentication, source-to-bytecode proof,
startup attestation, installed-package attestation or a release signing check.
It does not attest the compiler, SDK, dependency cache or Windows binary. Windows
inputs and media still need their separate recorded artifact boundaries. A
matching receipt is not permission to launch a duplicate test and does not
satisfy the physical-phone two-hour or safe TorBox delivery gates.

`Test-AndroidProofBuild.ps1` exercises missing/stale metadata, changed main/test/
configuration inputs, additions/renaming, APK replacements, bounded malformed
metadata and preservation of existing receipts on rejection. Its APKs are checksum
fixtures, not executable Android applications. `Test-AndroidBuildOwnership.ps1`
uses an isolated child build entry to verify live/uncertain marker refusal before
compilation and without changing the marker, creating a lock or building artifacts.
Neither fixture script changes the actual emulator or repository APKs.

## Post-exit observation logs

Host shutdown may interrupt the final append to its PC observation log. Preserve
that raw file. For an explicitly inspected incomplete tail, post-exit analysis
supports `-IgnoreUnterminatedPcTail`; it reports `ignored_unterminated_pc_tail`.
Only malformed JSON in the final line without a newline may be omitted. Default
parsing stays strict, complete malformed lines still reject, and valid final lines
(including without a newline) retain their normal run-identity checks. This option
does not upgrade a failed/running receipt or waive native assertions or duration.

## Windows proof build association

`scripts/Build-WindowsProof.ps1` holds the same native ownership lock through
frontend checks/build, Rust compilation and receipt publication. It captures
source inputs before frontend generation, captures generated frontend inputs
before Rust compilation, and checks both again after compilation. Selected source
inputs include root Cargo manifests/lockfile, the four current crate manifests/src
roots, Windows UI source/config/package lockfile, and Tauri source/config/build
script/capabilities/icons. Current inventory:95source files and three generated
frontend files. Private configuration, credentials, node_modules, dependency
caches and unrelated runtime data are excluded. New dependency/input roots require
updating this explicit policy. Local toolchains and dependency cache are not attested.

The receipt `work/windows-proof-build.json` records both inventories and SHA256
for the standard debug executable and native DLL. Reparse paths fail closed;
source, frontend or runtime changes invalidate the receipt. Checked output uses
only known public metadata fields; malformed/authentication-claiming metadata,
noninteger versions/counts and invalid UTC timestamps reject with a fixed error.
The receipt is a local association, not authentication, startup, compiler or
installed-package attestation. Namespace checks reject existing redirects, without
claiming protection against concurrent privileged namespace replacement.

`Test-WindowsProofBuild.ps1` uses nonexecutable checksum fixtures to check changes
to Rust/UI/config/manifests/lockfiles/frontend/executable/DLL, source addition,
removal/rename, mid-build source/frontend changes, missing/malformed/oversized
metadata, redirected source/frontend directories, preservation of receipts/targets,
excluded private/cache files and exclusion of unknown metadata fields.
`Test-WindowsBuildOwnership.ps1` uses an isolated child entry to verify live and
uncertain marker rejection before compilation, without changing metadata or
creating a lock or artifacts. These checks do not rebuild a live soak.

The session proof runner validates both Windows and Android associations before
starting the host or replacing the last-run receipt, including SkipBuild. Its full
build path captures Windows inputs and publishes a fresh receipt after compilation;
the final native receipt includes the checked windows_build and android_build.
Actual missing and stale Windows receipt preflights reject without replacing the
previous cancelled receipt. The standalone Windows build and frontend checks passed.


Session and HTTPS runners wait for sys.boot_completed=1 and the package service
being found before APK installation. Probes use owned adb client processes with
three-second deadlines and a bounded polling window; neither adb server nor the
emulator is killed on a probe timeout. Fixed timeout errors contain no raw device
output. Tests cover offline/cold boot, package delay, timeout and physical-device
rejection. A failed cold-start attempt f9b7ec86 is preserved separately.
Archive verification also rejects root/ancestor/nested reparse paths before
traversal, caps entry counts, rejects ambiguous names and checks exact inventory.
Six regressions include real Windows junctions whose targets remain unchanged.
These namespace checks do not authenticate evidence or protect against privileged
concurrent replacement; original historical archives remain unmodified.



## Repeatable private provider media selection

The source-built torbox-native-proof and torbox-paired-proof examples accept
TOGETHER_PROOF_TORRENT_ID and TOGETHER_PROOF_FILE_ID together in their process
environment. Both are exact unsigned decimal IDs; torrent must be positive.
Missing, malformed or unavailable explicit selection fails with a fixed category;
there is no fallback to another video. Without either variable, legacy first-ready
selection remains and is labelled first_ready_legacy. Numeric identity/mode are
recorded without media names, URLs or credentials. These controls affect proof
examples only, not product browsing, account writes or normal source selection.
Record exact selected identity and runtime codec/dimensions/decoder profile before
comparing runs. In particular, a software-decoded4K emulator result and a lighter
or hardware-decoded result are distinct evidence, not interchangeable passes.

## Headless local session host

The Windows `session-fixture-proof` example runs the production PC native follower
and control Host with local synthetic media, without opening a Tauri window. It
uses null video/audio output and player mute. This permits background session
integration while the desktop is in use; it does not validate rendered Windows
video, presentation timing, provider delivery or the packaged app.

Build the example from the current source under the shared native/control proof
ownership, using the existing toolchain. Its single argument is an owned runtime
directory containing `work/media/clock-test.mp4` and
`native/vendor/windows/libmpv-2.dll`. Record their hashes, the example source hash,
the normal Windows source inventory, and the built executable hash. The normal
Windows inventory excludes examples, so its digest alone does not identify this
host. An isolated example build does not publish or replace the standard Windows
proof build receipt. Use a fresh per-run directory with no previous ready/stop
files or observations; retain prior run directories as evidence.

Supply a fresh non-nil `TOGETHER_PROOF_RUN_ID`, a generated synthetic
`TOGETHER_PHONE_TOKEN`, and `TOGETHER_SESSION_PROOF=1` through the owned process
environment. Never print the token or reuse an account credential. The host binds
only loopback port47822 and refuses an occupied listener. Its ready file is
`work/fixture-host-ready.json`; verify both run ID and port before attaching the
owned emulator. A debug fixture connection can use this temporary token without
running pairing-storage tests. That connection is not evidence of normal pairing,
encrypted save/restore or Keystore behavior.

The existing selected `SessionPlaybackTest` method exercises real Android
controller/socket/native playback against this PC follower. Record exact selected
identities, both APK hashes, the Android source inventory and requested steady
duration. Capture its raw result and run-bound phone/PC observation logs. Do not
add unrelated test classes or treat a skipped instrumentation method as success.

After instrumentation, create the owned runtime's `work/fixture-host-stop` file.
The host requests Host shutdown, joins the native owner and control listener, then
exports its private journal and `work/fixture-host-receipt.json`. Require successful
process exit, matching run ID, stopped native/listener fields and zero journal
errors/drops. Its internal deadline is65minutes; a runner must retain ownership
until cleanup is established, preserve any failure and remove only its own
emulator connection/configuration. The host's `requested_flow_completed` means it
received the stop request before its deadline; the Android raw result separately
establishes whether the requested session assertions passed.

Analyze both all-eligible and explicit-steady native position samples with the
same expected run ID. Record the provisional timing target separately from native
assertion completion. A successful local fixture session is neither a physical
phone/two-hour/WAN gate nor a substitute for the unresolved4K delivery evidence.
