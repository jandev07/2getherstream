# Syncplay / Synkplay source reuse

The referenced Access Work Chat was reread on 8 October 2026. It explicitly
permits appropriate open-source code reuse with notices, and calls for adapting
Syncplay synchronization logic to Together's HTTPS/WSS transport. The user
confirmed that Syncplay/Synkplay code reuse is part of the project.

## Official Syncplay

- Repository: https://github.com/Syncplay/syncplay
- Pinned commit: `2af7ace347f437d44cd809a65c049e82bb4027fb`
- Original files: `syncplay/client.py`, `syncplay/constants.py`
- License: Apache License 2.0; original text in
  `third-party/syncplay/LICENSE.txt`.
- Attribution: Syncplay contributors. Upstream README credits Uriziel, Et0h and
  Tomasz Kowalczyk (Fluxid), among other contributors.
- Unmodified copies of the reviewed source files are in `third-party/syncplay/`.

Ported behaviour: `_slowDownToCoverTimeDifference` entry/reset hysteresis and
the drift correction ladder in `_changePlayerStateAccordingToGlobalState`:
4-second ahead rewind, 1.5-second slowdown at 0.95x, 0.1-second slowdown reset,
and lag detection below -1.75 seconds with a 3.25-second observation period and
5-second behind correction threshold.

Changes: Rust/Kotlin implementations, milliseconds at API boundaries, injected
monotonic timing, generation resets, explicit steady/fresh-clock guards. Hard
corrections request a coordinated Together seek barrier at the authoritative
position; they do not directly seek or add Syncplay's 0.25-second extra forward
jump. Together's PC timeline replaces upstream room-member authority selection.
The original Python/Twisted TCP runtime is not used.

## Synkplay (Syncplay Mobile)

- Repository: https://github.com/yuroyami/syncplay-mobile
- Pinned commit: `a7770b10a0d8769cce01e0b144b5957ba57d68c2`
- Original file:
  `shared/src/commonMain/kotlin/app/protocol/sync/SyncDecision.kt`
- License: GNU Affero General Public License version 3; original text in
  `third-party/synkplay/LICENSE.txt`.
- Attribution: Synkplay / Syncplay Mobile contributors.
- Unmodified reviewed source: `third-party/synkplay/SyncDecision.kt`.

Adapted code: `nudgeRate`, including the low-pass filter, separate entry/exit
thresholds and rate changes only when the correction level changes. Originally retained
constants: 0.005 rate adjustment, 0.15-second start, 0.03-second stop and 0.3
smoothing. Both Together players may catch up to the authoritative PC timeline.

Modified files:
- `crates/sync-core/src/drift.rs`: Rust port plus Syncplay ladder.
- `apps/android/app/src/main/java/app/together/v2/sync/DriftController.kt`:
  Kotlin adaptation plus Syncplay ladder.

Together modification, 8 October 2026: normalize the nudge smoothing weight by
elapsed monotonic observation time (upstream weight 0.3 at a 100 ms reference
interval), ignore duplicate/reordered observations, and reset observation timing
with existing generation/barrier resets. That modification retained the original
start/stop thresholds, nudge rates and Syncplay large-drift ladder. This time normalization is a
Together modification, not a claim about upstream Syncplay/Synkplay behavior.

Together experiment, 8 October 2026: reduce nudge entry from 150 to 75 ms and
exit from 30 to 20 ms on both platforms. Opposing errors inside the original
entry band contributed to the remaining cross-device quality gap in emulator
run 4d821313. Keep 0.005 rate adjustment, time-normalized smoothing and large-
drift ladder. These tighter thresholds are a Together modification under
validation, not upstream defaults or a claim of physical-device acceptance.

These derivative modules retain AGPL-3.0 licensing. Preserve the source,
attribution and license texts with builds. For combined distribution and any
applicable network-source obligations, provide corresponding source under the
applicable license terms; include an accessible source/license entry before
shipping playback releases. Private-project status does not remove notices.
This notice does not grant a separate license for unrelated original Together
code or claim the whole Syncplay/Synkplay application has been integrated.

## Integration state

The adapted controllers compile, have automated behaviour tests and now control the native players in the local PC/emulator proof. Existing WSS authority,
prepare/ready/commit, snapshots and clock estimation remain Together code.
No stock Syncplay TCP compatibility or public Syncplay-room compatibility is
claimed. The native libmpvKt engine selected by Synkplay is integrated, with an original Together surface/worker adapter; see native/README.md.

Together clock update, 10 October 2026: the fresh-window offset-interval
intersection and explicit minimum-RTT fallback in Rust/Kotlin are independently
written Together logic. They use conventional four-timestamp offset/delay
equations but are not an upstream Syncplay/Synkplay algorithm. Bounds assume
constant relative clock offset and nonnegative transit delays; they do not
measure playback alignment. See docs/CLOCK-CALIBRATION.md for assumptions,
diagnostics and validation boundaries. Drift-controller provenance above is
unchanged.

## Native player checkpoint
Android now uses libmpvKt 0.3.0, the wrapper selected by Synkplay. Its copied license, NOTICE and dependency source script are in third-party/libmpvKt. Native libraries are packaged by this dependency. See docs/NATIVE-CHECKPOINT.md; the older integration-state paragraph above predates this local player proof. Shared playback remains unintegrated.

## Connected native proof update
The adapted Rust and Kotlin drift controllers now drive the respective native player workers under Together authority. libmpvKt core/options are used with an original Together TextureView lifecycle adapter; no upstream source was edited. Short PC/emulator session tests passed, preserving the above license and provenance notices. Earlier unintegrated-state notes describe historical checkpoints only.

## QR Code generator

- Project Nayuki qrcodegen 1.8.0, pinned Rust dependency, MIT License.
- Upstream: https://www.nayuki.io/page/qr-code-generator-library
- Unmodified library used to generate the fixed Android download QR symbol.
- Copyright and permission notice: third-party/qrcodegen/LICENSE.txt.

## Cloudflare cloudflared private Alpha connector

- Upstream: https://github.com/cloudflare/cloudflared
- Bundled existing binary:2026.9.3, built2026-09-24, copied read-only from the
  existing authorized Together toolchain. No legacy files modified.
- SHA256:f096265ec2fcbe9bb6e2d64268db167ced3fcbb83d894bdb9e2fcdb26f2ea7e2.
- Apache License2.0 (third-party/cloudflared/LICENSE.txt).
- Unmodified binary; Cloudflare contributors. Used only for HTTPS/WSS control,
  not video proxying, VPN adapters, system DNS or system routing changes.


## Native component source and notices inventory

Unmodified upstream notices for Android native mpv, FFmpeg, libass, libplacebo,
dav1d, Mbed TLS, HarfBuzz, FreeType, FriBidi, libunibreak and Lua are retained
under third-party/native-components. PROVENANCE.json maps each copied file to
its original source-archive member and SHA256. Lua's full license is in
lua/readme.html; its top-level README alone only links to that document.

native/SOURCE-INVENTORY.json records the exact libmpvKt BUILD-INFO source pins,
Android native object correspondence and Windows build/mpv/FFmpeg pins. AndroidX
graphics-path1.0.1 supplies the two additional native path libraries. This is a
partial attribution/source inventory, not a declaration that corresponding-source
assembly for every static dependency, NDK/runtime and native build is complete.
The unmodified upstream licenses keep their individual terms. Source archives
remain local release-preparation materials until publication is allowed.


Windows build preparation additionally retains original libiconv COPYING and
COPYING.LIB, libopenmpt LICENSE, LZO COPYING and Xvid LICENSE from the four
checksum-verified archive inputs. Their exact members/hashes are in the same
PROVENANCE.json. native/WINDOWS-BUILD-INPUTS.json records declared configuration
references; unresolved cached Git revisions are not replaced with guessed commits.
These source inputs and notices do not complete the Windows dependency inventory.


AndroidX graphics-path1.0.1 release sources, including its native C++ module,
were collected at the official release-notes revision. The four published Maven
Kotlin source files match byte-exact. Its original root Apache license is retained
under third-party/native-components/androidx-graphics-path/LICENSE.txt; original
source files retain AOSP and embedded math code copyright headers. Source pin,
module members and correspondence boundaries are in native/SOURCE-INVENTORY.json.


Original LLVM libcxx, libcxxabi and libunwind LICENSE.TXT files are retained under
third-party/native-components/ndk-*. The runtime source association and Android
patch pins are recorded in native/SOURCE-INVENTORY.json. These licenses retain
their original terms, including upstream exceptions; the project's root license
does not replace them. Source collection and allocated-section correspondence do
not assert a complete or reproducible NDK distribution.


Additional Windows source preparation retains original AOM and libvpx LICENSE/
PATENTS, dav1d COPYING/PATENTS, and SVT-AV1 LICENSE.md/LICENSE-BSD2.md/PATENTS.md.
The dav1d COPYING bytes match the existing retained notice and share that file.
Their exact archive members and hashes are in PROVENANCE.json; upstream notices
remain unchanged. Native version/source association evidence is recorded in
native/WINDOWS-BUILD-INPUTS.json. Embedded version attribution and source collection
do not establish the complete Windows build tree or corresponding-source graph.


Later Windows source-base records retain original libass COPYING, Opus COPYING
and LICENSE_PLEASE_READ.txt, SDL2 LICENSE.txt and libplacebo LICENSE. Identical
libass/libplacebo notices reuse the existing retained files. PROVENANCE.json
records each archive association independently. The libplacebo binary reports a
modified source tree; its upstream base archive is explicitly not the full build
tree. No original notice is replaced by the project's root license.


Windows subrandr source preparation retains its original Mozilla Public License
Version2.0 text. Its committed-lock registry archives retain their original
notices; source-revision supplements for notices omitted from published crates
and the original r-efi AUTHORS grant are retained under
third-party/native-components/windows-subrandr. PROVENANCE.json records original
bytes and source URLs/commits or archive members. These records cover the whole
upstream lock, including non-Windows/dev/build packages, and do not claim all are
linked. See native/SUBRANDR-LOCKED-SOURCES.json for scope, missing standalone
notice material and unverified actual build dependency selection. No upstream
license is replaced by 2gether's root license.

Windows build-log source preparation also retains the original FreeType license
overview/FTL/GPLv2, HarfBuzz COPYING, and libjpeg-turbo LICENSE.md/README.ijg.
Identical notice bytes reuse existing retained files, with separate archive
associations in PROVENANCE.json. These sources are fetched-branch candidates,
not verified final cached/modified build trees; see WINDOWS-BUILD-INPUTS.json.
This software is based in part on the work of the FreeType Team and the
Independent JPEG Group. All original notices retain their individual terms.

Windows glad source preparation retains original MIT and Khronos terms under
third-party/native-components/glad/LICENSE. Pinned recipe configuration specifies
branch candidate glad2; build logs record force-update without fetch output,
leaving the exact commit unrecorded. Upstream terms are preserved without asserting
binary reproduction.

Windows compiler and runtime toolchain inputs (LLVM Clang, compiler-rt, libc++,
libunwind, mingw-w64 CRT, and winpthreads) were consumed from prebuilt GitHub
Actions cache artifacts (clang_root). Applicable upstream runtime licenses (LLVM
Apache 2.0 with LLVM Exception and mingw-w64 permissive/public domain/ZPL terms)
govern any statically linked runtime routines. Toolchain recipes and hashes are
recorded in native/WINDOWS-BUILD-INPUTS.json; full toolchain reproduction and
complete binary correspondence remain open release preparation work.
