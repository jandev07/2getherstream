# Governing product specification — 8 October 2026

The full user prompt in the development chat is the governing specification.
Canonical repository: `C:/Users/astro/Documents/Together2`. Earlier chat outputs
are historical checkpoints. Keep `Documents/tgt/Together` read-only.

Primary hardware: Ryzen 5 7600X, RX 7800 XT, 32 GB RAM, 1440p Windows;
partner Galaxy S23 Ultra; developer OnePlus 13. Use an isolated emulator for
initial Android validation at the user's request; real-device acceptance follows.

Use Tauri 2/Rust/Svelte on Windows and Kotlin/Compose/native libmpv on Android.
Use traced Syncplay/Synkplay code adaptations and preserve required licenses.
The PC is authority behind CGNAT, with automatic cloudflared and HTTPS/WSS only.
No media relay, WebRTC screen-streaming primary player or Android networking helper.
Both devices decode the same original TorBox file using separate generated URLs.
TorBox credentials stay only on PC; URLs are not the canonical media identity.

Daily use must be automatic after first-time pairing. No copied URLs, IP entry,
codes, manual tunnel launch or VPN steps for the partner. Either device may
control playback. Explicit readiness, buffering, monotonic calibration, scheduled
starts, generational seeks and reconnect snapshots are mandatory.

Priority: investigation -> skeleton -> native player proof -> networking proof
-> synchronization proof on one PC/phone/video. Only then restore the existing
premium OLED/glass Windows UI, native Material Android layouts, in-app chat,
Stremio browsing/source selection, tracks/subtitles, episodes, discovery and
Pocket compatibility. Pocket remains a separate lightweight app.

Milestone acceptance requires movie-length physical playback, latency/jitter,
rapid seeks, loading imbalance, buffering on either player, disconnect/reconnect,
Android lifecycle/rotation, long pause, URL refresh and supported 1080p/4K HEVC.
Use hardware decode when appropriate; no unnecessary transcoding. Make failures
observable through redacted structured session logs and exportable diagnostics.

The local generated-video player proof UI is a developer fixture, not the final
watching experience. Do not mistake it for completion of TorBox or shared playback.

## 9 October 2026 — 2gether identity and release order

The rebuild's product name is **2gether**. Keep the Together2 repository path and
internal package/protocol identifiers stable. Before full release, deliver a fresh
visual identity including typography, Windows/Android app icons, branding, layouts,
controls and transitions. Preserve the requested OLED/glass Windows direction and
native Material Android experience, with readable text, correct scaling and touch
targets. A display-name update is only the first branding step.

The user reaffirmed that dependable streaming for their partner comes first.
Pass the physical-phone playback gate before migrating extra features or undertaking
the full visual redesign. Validate movie-length playback and failure recovery;
do not promise universal zero failures or count emulator results as phone acceptance.

Together must work without the vgate0/Rust Wintun adapter or a system-wide VPN.
No enable/disable routine should be required for games or development. Establish
the adapter's owning software before scoped removal requested for the end of
development; do not uninstall unrelated networking components based on its name.

## Android release updates

User requires an in-app updater before full release. Provide update availability,
version/release information, download progress and an installation action within
2gether's normal UI. Use Android's supported installation/permission flow rather
than assuming silent install privileges. Authenticate release metadata, verify
APK integrity and expected package/signing identity before handing off installation.
Keep credentials out of release URLs/logs and recover clearly from interrupted
downloads or unavailable update services.

Together 1.0 does not need an in-place APK upgrade to 2gether. The user accepts
uninstalling the old app and freshly installing the rebuild. No legacy data or
signing-key migration is required or authorized now. Future 2gether releases
should update in place, preserving pairing and settings with consistent package
and signing identity. Updater implementation follows the physical playback gate
alongside full-release preparation; it is not part of the current core proof.

## First distributed release: 2gether 2.0 Beta

The user requires the first distributed 2.0 release to be a beta representing the
complete rebuilt Windows and Android applications. The minimal native playback
proof is an internal milestone, not that beta. The final experience should exceed
1.0 in reliability, usability and polish; this is an acceptance goal, not a claim
about today's incomplete build. Preserve legacy apps as references.

Order remains: pass physical-phone core playback acceptance; rebuild browsing,
source selection, episodes, chat, subtitles/audio and settings around the new core;
complete fresh typography/icons/native layouts and Android updater; validate the
whole daily-use experience and then distribute 2gether 2.0 Beta. Automatic paired
connection, recovery, direct safe media, diagnostics and operation without the
Rust network adapter are required. Pocket remains the separate existing companion,
with API adaptation later if necessary, rather than a third wholesale rebuild.

## Visual direction and platform performance — 9 October 2026

User requests round buttons and icon treatments instead of rounded-square shapes:
use circular icon-button containers and capsule text buttons. Create an iOS-inspired
liquid-glass interface for both apps, with translucency/blur, carefully chosen
typography, coherent icons and smooth transitions. Retain 2gether's own identity.
This supersedes default Material visual styling, not the native Kotlin/Compose
Android stack. Respect platform launcher icon masks and accessible touch targets.

The user permits richer Windows effects and higher resource use for visual quality.
Android must keep performance and battery cost low: bound blur area/layers, avoid
idle continuous animations and unnecessary recomposition, stop hidden effects,
support reduced motion, and offer lighter effects if needed. Measure playback
smoothness, thermal behavior and battery impact on physical phones before beta;
emulator rendering is insufficient. Visual work must not interfere with native
playback or synchronization. Full design implementation remains after phone gate.
