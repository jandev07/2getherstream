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
Windows static dependency sources/configuration and Android non-mpv dependency
source/notice collection remain release preparation work. Do not equate an
upstream link or checksum match with a completed distribution requirement.
