# Native engines

Windows: pinned mpv-dev-x86_64-20261007-git-eb0ee10315.7z from
https://github.com/shinchiro/mpv-winbuild-cmake/releases/tag/20261007 . Archive
SHA256 3fd93055d437310ad094d3f80c9b04f9a56e8b138bcb7d9c5cf01870abbfe5a5.
libmpv-2.dll SHA256 872827614ed0adfca11e68def5273bcfcaea6acf38bbf1950c35980b59f43a5f.
Runtime reports mpv v0.41.0-1104-geb0ee1031. Rust loads only its trusted absolute
repository DLL path, constructs the engine on its dedicated worker and submits
argument-array commands. Tauri supplies a native Win32 child window with D3D11;
there is no HTML video playback.

Android: io.github.yuroyami:libmpvkt-view:0.3.0. Bundled engine supports arm64-v8a
and emulator x86_64. Reviewed wrapper source pin:
b001dd97538e7d3e700df80029fd72be99ceba3f. Preserve copied LICENSE.txt, NOTICE.txt
and depinfo.sh under third-party/libmpvKt. The wrapper is Apache-2.0; its native
engine includes GPL-covered dependencies. Complete the corresponding-source
and native build/version manifest before distribution.

Together uses libmpvKt's core/options, with an original worker-owned native
TextureView adapter. The first wrapper SurfaceView proof stalled on foreground
reattachment. Merely moving calls off Main kept UI responsive but let stale
player state appear ready. The working adapter retains SurfaceTexture ownership
until queued native detachment completes, so Android cannot destroy a surface
under a pending native command. A 100 ms native probe provides an independent
freshness timestamp; followers refuse readiness after 1 s without a native probe.
Generation fences reject queued player effects from earlier barriers.

Native proof tests verify actual decoding, position, pause, seek, rotation,
nonblack rendered video and reconnect. Neither emulator GPU performance nor the
short Windows/Android fixture proof validates phone 4K hardware decoding.

Historical HTTPS checkpoint: TorBox media was not connected at that stage.
Current private Alpha direct TorBox PC/emulator playback is recorded separately
in docs/MILESTONE-1.md and docs/DEVELOPMENT.md, including the explicitly labelled
account-bearing private delivery exception. Physical-phone/WAN acceptance remains
open. Android mpv has tls-verify=yes
and a PEM bundle exported from Android system roots. Native HTTPS load/play/pause/
exact-seek proofs pass on Windows and the emulator against a pinned public AndroidX
test asset. These tests do not establish TorBox URL expiry or device 4K decoding.
Do not disable certificate verification to make future TorBox playback work.

Downloaded native binaries and generated media remain ignored. Reproduction:
scripts/Prepare-PlayerProof.ps1 and scripts/Test-SessionProof.ps1.


## Native source inventory (15 October 2026)

SOURCE-INVENTORY.json records public upstream commits, Android BUILD-INFO values,
verified APK native-library hashes and published mpv/FFmpeg source checksums.
All20 libmpvKt native objects in candidate20261014-195818-f829095c match both its
Maven AAR and upstream0.3.0 release ZIPs. The remaining two graphics-path objects
match androidx.graphics:graphics-path:1.0.1. Hashes identify this exact candidate;
a new package must be checked rather than inheriting the old verification.

Windows mpv and FFmpeg exact commits were confirmed against the upstream x86_64
build job that generated the20261007 archive. Build repository source is pinned
as well. This partial inventory is not a complete corresponding-source bundle:
Windows static dependency commit resolution and Android NDK/runtime
source assembly remain release preparation work. Android primary dependency and
build-submodule source archives/notices have now been collected locally. Do not equate an
upstream link or checksum match with a completed distribution requirement.


WINDOWS-BUILD-INPUTS.json retains hashes and declared source references for all108
package configuration files in the exact Windows build repository archive. This
is a configuration inventory, not a resolved binary dependency graph: cached Git
sources and moving branches require actual revision evidence. Four archive inputs
mentioned in the x86_64 job (libiconv, libopenmpt, LZO and Xvid) were collected and
checked against the configuration's upstream checksums. LZO uses its configured
SHA1 as well as a local SHA256 integrity hash. The initial Opus model archive download
was interrupted without receiving bytes. A later bounded download succeeded and
matched the exact configured SHA256; its31 members are retained locally. All six
generated checkpoint SHA1 annotations match included model content. Two refer to
nolace_small.pth, whose content is included as nolace_160_v2.pth; original generated
files are preserved. The model archive contains no license/readme notice, and the
Opus Git source revision/binary linkage remain unresolved. Original notices
for the four verified inputs are retained under third-party/native-components;
configuration inclusion or job mention alone does not prove every input is linked
into the distributed DLL. No complete-source/distribution claim is made.


Graphics-path1.0.1 source was resolved from the official AndroidX release notes
commit range endpoint8a05a22af450d589ef911d772a001a49dcb05b71. All four Maven
Kotlin source files match that tree byte-exact. The retained module archive also
contains12 native C++/header/CMake/map files and the Gradle compiler configuration;
all10 quoted local include edges resolve inside the archive. Original root
AndroidX Apache license is retained. This identifies release source and closes
the previously missing C++ source collection; no binary rebuild or exact NDK/
toolchain correspondence is asserted. SOURCE-INVENTORY.json records the URLs,
members and local integrity hashes. Android NDK/runtime material and Windows
cached Git dependency revision evidence remain incomplete.


NDK runtime association: official ndk-r29 selects clang-r563880c. Its published
clang_source_info.md identifies LLVM base386af4a5c64ab75eaee2448dc38f2e34a40bfed0
and Android build/patch revision1dab3288f660d43a6cb2479107e2b54b3ab0a2a1.
Collected libcxx/libcxxabi/libunwind subtrees and Android build sources, retaining
all46 listed patch inputs and original runtime licenses. All27 arm64 and26
x86_64 allocated ELF sections match the attributed APK libc++ runtime in section
names/types/flags/addresses/sizes/content. Whole-file hashes differ; no exact
whole NDK package or source rebuild is claimed. The full pinned LLVM tree and NDK-r29 build source are now collected, including
shared CMake/runtime/builtins build entrypoints. Toolchain/build reproduction and
unresolved Windows Git revisions remain open release work. These
new notices/sources are later preparation records, not retroactively inserted
into an already hashed candidate package.


An internal Android source-preparation bundle now assembles24 native source
archives with the exact candidate's project source ZIP and later source/notices
records. All69 members were checked during streaming copy and again after ZIP
reopening. The two original ABI BUILD-INFO files are retained as separately hashed
companions beside that immutable ZIP. See the DEVELOPMENT receipt pointer for
exact scope. It contains no APK, signer, private credentials or diagnostics. The
bundle remains private preparation material; unresolved Windows revisions and
unexecuted toolchain reproduction are not silently marked complete. Additional
source materials do not change the previously validated candidate binary/package.


Four additional Windows dependency revisions are now identified from Git version
strings embedded in the exact DLL (AOM, libvpx, dav1d, SVT-AV1). Original version
bytes/offsets, upstream full commit resolution, source version API members and
collected archive hashes are in WINDOWS-BUILD-INPUTS.json. These are binary-reported
source associations, not attestations of an unmodified source tree or complete
static linkage. Existing unresolved source-commit fields remain null. Four pinned
source archives are retained locally; their SHA256 values establish local integrity,
not an independently published upstream checksum. Nine original license/patent
provenance records use eight new notice files and the byte-identical existing
dav1d COPYING. Notices inside the full archives are preserved as well. These later
preparation records do not alter already hashed candidates or Android source bundles.
The Windows dependency source graph and toolchain reproduction remain incomplete.


Windows version attribution now also covers libass, Opus, SDL2 and libplacebo.
The exact DLL reports a dirty libplacebo source tree. Its collected archive is an
upstream base, not the modified build tree. The pinned build recipe replaces glad
and fast_float subdirectories with separately configured sources; those operations
are retained as evidence, without assuming they explain every modification or
resolve those separate inputs. The Opus recipe copies generated model C/header
files from the already checksum-verified opus-dnn archive. Source base/version
members, recipe hashes and unresolved overlay boundaries are in the detailed
inventory. None of these associations closes the complete Windows source graph.


Windows subrandr is associated with the binary's1.4.0 reveaf47d9 marker and full
upstream commiteaf47d947e754565fe60e88ffe9665a2a32b18d5. Its source and committed
Cargo.lock are retained. All255 registry archives in that lock were collected
and verified against lock checksums, including platform/dev/build entries that
are not asserted linked into Windows. native/SUBRANDR-LOCKED-SOURCES.json records
the source archives, notices and package identities. The pinned outer/inner build
commands do not enforce --locked; actual build dependency selection is unverified.
Native FreeType/HarfBuzz and compiler/runtime correspondence remain separate.

Original MPL2 subrandr license, omitted crate notice supplements from exact
published VCS revisions, and r-efi's original AUTHORS grant are retained under
third-party/native-components/windows-subrandr with byte hashes. The old dispatch
crate declares MIT but has no standalone license/notice file in its published
archive or inspected repository root; that limitation remains explicit. Full
archives retain their original source/notice content. These are partial source
preparation records, not a complete licensing/source-distribution certification.
No upstream build script was executed and no new app dependency was added.

Three further Windows source candidates (FreeType, HarfBuzz and libjpeg-turbo)
are collected from the exact x86_64 job's fetched branch destinations. Their
full upstream revisions, archive integrity hashes, original notices and pinned
recipe/helper hashes are recorded separately as build_log_source_candidate in
WINDOWS-BUILD-INPUTS.json. They are not added to the binary-reported revision
list: the actual cached HEAD files and modified final build trees remain
unverified. In particular, a fetch destination plus a reset recipe is not proof
that the corresponding clean source tree produced the DLL.

Run `python scripts/Verify-NativeNotices.py` from the repository for retained
notice integrity. Add `--archives work/alpha-source-material` to compare every
archive-associated notice with its original local tar member, and `--staged`
after staging to compare retained notice bytes with the Git index. Identical
archive copies are accepted only after whole-file SHA256 equality; conflicting
copies fail. URL-only associations are reported as unchecked, without network
requests. This verifies notice bytes against the recorded provenance, not
upstream authenticity, binary correspondence or distribution completeness.


Native media transport rejects redirects, including HTTPS-to-HTTPS: Windows sets
curl-max-redirects=0 and stream-lavf-o=max_redirects=0; Android sets the lavf limit.
TLS verification remains enabled. This matches the resolver's direct-range URL
policy; a later redirect fails through existing native fault/recovery handling.
It does not sandbox every demuxer/subresource request. Actual owned loopback TLS
fixtures verified Windows curl/lavf and Android JNI transport separately; these
are not provider, full paired-session, frame timing or physical acceptance.

Explicit Android NativeRedirectTest requires nativeTlsOrigin=https://127.0.0.1:PORT
and an owned CA staged as cache/isolated-native-redirect-ca.pem in the debug app.
The fixture serves the local clock-test.mp4 directly at /direct, /secure returns
302 to the same HTTPS /direct, and /downgrade returns302 to a separately observed
loopback HTTP listener. Own both reverse mappings and inspect both redirect hits
and zero plaintext requests. The test verifies untrusted TLS rejection before
using its per-player CA, then direct readiness and both redirect failures. It
removes its CA and awaits each player cleanup. Run only with an isolated owned
fixture and existing muted instrumentation runner; never change system trust.
Exact successful run and failed setup/build history are in DEVELOPMENT.
