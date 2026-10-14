# Native-player checkpoint — 8 October 2026

Canonical repository: C:/Users/astro/Documents/Together2. Together 1.0 and Pocket remain unchanged.

Windows native DLL decoder proof passed: open, duration, audio/video track count, play, pause, exact seek and resume on a generated 60-second fixture. This test uses null video/audio outputs and does not prove visible rendering or hardware decoding. Tauri native child-window surface and local controls compile; the application launches. Visual surface checks remain pending.

Android libmpvKt 0.3.0 native Compose build and three JVM tests passed. Includes arm64-v8a and x86_64. Android playback runtime has not passed: emulator-5580 remains offline after two software-only boot attempts. No physical device has been tested.

Rust: 11 core tests and 3 actual WebSocket tests passed. Svelte: zero errors/warnings; frontend and Tauri builds passed. Syncplay/Synkplay derived drift controllers still require native/session integration. No synchronized device playback or real two-hour test has passed.

Local CPU query: AMD Ryzen 5 7600X; VirtualizationFirmwareEnabled=True; SecondLevelAddressTranslationExtensions=True; VMMonitorModeExtensions=True. Windows HypervisorPresent=False. Emulator acceleration check reports no installed Android Emulator hypervisor driver. Windows optional-feature query requires administrator rights, so HypervisorPlatform checkbox state could not be read.

Recommended environment step: enable Windows Hypervisor Platform in Windows Features, then restart. Official reference: https://developer.android.com/studio/run/emulator-acceleration . BIOS SVM is already reported enabled. Do not change unrelated BIOS/security options.

Resume: check emulator acceleration after reboot; start Together2Test with acceleration auto; install the native APK and verify load/play/pause/seek and lifecycle. Then connect player adapters to authoritative session barriers and clock scheduling. Pairing, TorBox resolver, separate signed URLs, Cloudflare deployment and WAN/physical-device acceptance remain outstanding. TorBox API key has not been supplied.

Windows library pinned archive: mpv-dev-x86_64-20261007-git-eb0ee10315.7z, SHA256 3fd93055d437310ad094d3f80c9b04f9a56e8b138bcb7d9c5cf01870abbfe5a5. Runtime reports mpv v0.41.0-1104-geb0ee1031. Source https://github.com/shinchiro/mpv-winbuild-cmake/releases/tag/20261007 . Android wrapper source pin b001dd97538e7d3e700df80029fd72be99ceba3f; preserve third-party/libmpvKt notices (wrapper Apache-2.0, native dependencies GPL notices). Complete native corresponding-source/build manifest before distribution.

## Acceleration follow-up — 8 October 2026
After user enabled Windows features and restarted: HypervisorPresent=True; firmware virtualization=True; emulator acceleration check exit 0, WHPX(10.0.26200) installed and usable. Accelerated Together2Test emulator-5580 boot completed (sys.boot_completed=1). Native APK installed successfully and MainActivity launch accepted. Playback interactions and synchronization acceptance remain pending.

## Connected native checkpoint — 8 October 2026
Both native followers are now integrated for the local synthetic fixture. Repeatable runner passes NativePlayerTest and SessionPlaybackTest (no skips): rendering, play/pause/exact seek, rotation, scheduled two-player start, rapid latest-target seeks, background/reconnect snapshots, and native-health fault/recovery with explicit user pause preserved. Latest 36.276 s run: 161 matching playing samples, estimated median 17.5 ms, p95 61 ms, max 95.5 ms. Real TorBox, WAN faults, CDN buffering, physical 4K/HEVC and two-hour acceptance remain pending. docs/VALIDATION.md contains current evidence.
